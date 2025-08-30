use ratatui::Frame;
use ratatui::layout::Rect;

pub trait Component {
    fn on_mount(&self) {}
    fn render(&self, area: Rect, frame: &mut Frame);
}

impl<T: Component> crate::IntoElement for T {
    type Element = T;
    fn into_element(self) -> Self::Element {
        self
    }
}

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
