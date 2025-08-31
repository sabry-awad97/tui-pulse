use ratatui::Frame;
use ratatui::layout::Rect;
use std::collections::HashMap;

thread_local! {
    // Track mounted component instances and their mount states
    static MOUNT_STATE: std::cell::RefCell<MountState> = Default::default();
}

// Component wrapper that can be stored and called for unmounting
struct ComponentWrapper {
    unmount_fn: Box<dyn Fn()>,
}

impl ComponentWrapper {
    fn new<T: Component + Clone + 'static>(component: T) -> Self {
        Self {
            unmount_fn: Box::new(move || component.on_unmount()),
        }
    }

    fn call_unmount(&self) {
        (self.unmount_fn)();
    }
}

#[derive(Default)]
struct MountState {
    // Tracks all currently mounted components by their ID hash
    mounted: std::collections::HashSet<usize>,
    // Components that were mounted in the last render
    current_render: std::collections::HashSet<usize>,
    // Store component wrappers for unmount callbacks
    component_refs: HashMap<usize, ComponentWrapper>,
}

impl MountState {
    fn track_mount<T: Component + Clone + 'static>(
        &mut self,
        id_hash: usize,
        component: &T,
    ) -> bool {
        self.current_render.insert(id_hash);

        // Returns true if this is the first time mounting (newly inserted)
        let is_new = self.mounted.insert(id_hash);

        if is_new {
            let wrapper = ComponentWrapper::new(component.clone());
            self.component_refs.insert(id_hash, wrapper);
        }

        is_new
    }

    fn cleanup_unmounted(&mut self) {
        // Find components that were mounted before but not in current render
        let unmounted: Vec<_> = self
            .mounted
            .difference(&self.current_render)
            .cloned()
            .collect();

        // Call on_unmount for each unmounted component
        for &id_hash in &unmounted {
            if let Some(wrapper) = self.component_refs.remove(&id_hash) {
                wrapper.call_unmount();
            }
            self.mounted.remove(&id_hash);
        }

        // Prepare for next render
        self.current_render.clear();
    }
}

pub trait Component: Clone + 'static {
    /// Called once when the component is first mounted
    fn on_mount(&self) {}

    /// Called when the component is about to be unmounted
    fn on_unmount(&self) {}

    /// Called on every render
    fn render(&self, area: Rect, frame: &mut Frame);

    /// Gets a unique identifier for this component instance
    fn component_id(&self) -> String {
        // Default implementation uses the type name
        std::any::type_name::<Self>().to_string()
    }

    /// Renders the component with mount/unmount lifecycle tracking
    fn render_with_mount(&self, area: Rect, frame: &mut Frame) {
        let component_id = self.component_id();
        let id_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            component_id.hash(&mut hasher);
            hasher.finish() as usize
        };

        // Track this component in the current render
        let is_first_render = MOUNT_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.track_mount(id_hash, self)
        });

        // Call on_mount on first render
        if is_first_render {
            self.on_mount();
        }

        // Call the actual render method
        self.render(area, frame);
    }
}

/// Cleans up any components that were unmounted in the last render cycle
/// This should be called after each render cycle
pub fn cleanup_unmounted() {
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.cleanup_unmounted();
    });
}

impl<T: Component> crate::IntoElement for T {
    type Element = T;
    fn into_element(self) -> Self::Element {
        self
    }
}

#[cfg(test)]
mod tests;

#[allow(dead_code)]
pub trait ComponentHooks {
    /// Gets the component ID for hooks
    fn get_component_id(&self) -> u64 {
        // Generate a unique ID for this component instance based on its memory address
        let ptr = std::ptr::addr_of!(self) as usize;
        ptr as u64
    }

    fn render(&self, _frame: &mut Frame) {
        // Get the component ID
        let _id = self.get_component_id();
    }
}
