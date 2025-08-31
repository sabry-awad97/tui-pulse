use ratatui::Frame;
use ratatui::layout::Rect;

thread_local! {
    // Track mounted component instances and their mount states
    static MOUNT_STATE: std::cell::RefCell<MountState> = Default::default();
}

#[derive(Default)]
struct MountState {
    // Tracks all currently mounted components by their memory address
    mounted: std::collections::HashSet<usize>,
    // Components that were mounted in the last render
    current_render: std::collections::HashSet<usize>,
}

impl MountState {
    fn track_mount(&mut self, ptr: usize) -> bool {
        self.current_render.insert(ptr);
        // Returns true if this is the first time mounting (newly inserted)
        self.mounted.insert(ptr)
    }

    fn cleanup_unmounted(&mut self) {
        // Find components that were mounted before but not in current render
        let unmounted: Vec<_> = self
            .mounted
            .difference(&self.current_render)
            .cloned()
            .collect();

        // Remove unmounted components from tracking
        for &ptr in &unmounted {
            self.mounted.remove(&ptr);
        }

        // Prepare for next render
        self.current_render.clear();
    }
}

pub trait Component: 'static {
    /// Called once when the component is first mounted
    fn on_mount(&self) {}

    /// Called when the component is about to be unmounted
    fn on_unmount(&self) {}

    /// Called on every render
    fn render(&self, area: Rect, frame: &mut Frame);

    /// Internal method to handle mounting logic
    fn render_with_mount(&self, area: Rect, frame: &mut Frame) {
        let ptr = std::ptr::addr_of!(*self) as *const () as usize;

        // Track this component in the current render
        let is_first_render = MOUNT_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.track_mount(ptr)
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
