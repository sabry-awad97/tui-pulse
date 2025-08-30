pub trait IntoElement {
    type Element;
    fn into_element(self) -> Self::Element;
}

pub type Element = VNode;
pub enum VNode {}
