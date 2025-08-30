pub trait IntoElement {
    type Element: crate::Component;
    fn into_element(self) -> Self::Element;
}

pub type Element = VNode;
pub enum VNode {}
