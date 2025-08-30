pub trait IntoElement {
    type Element: crate::Component;
    fn into_element(self) -> Self::Element;
}

pub type Element = VNode;
pub enum VNode {
    Text(String),
    // Element(Element),
    // Component(Component),
    // Fragment(Vec<VNode>),
}

// pub enum Component {
//     Function(ComponentFn),
//     Struct(ComponentStruct),
// }

// pub struct ComponentFn {
//     pub fn create(props: Props) -> Element,
//     pub fn update(prev_props: Props, props: Props) -> Element,
// }

// pub struct ComponentStruct {
//     pub props: Props,
//     pub children: Vec<Element>,
// }

// pub struct Props {
//     pub id: u64,
//     pub props: HashMap<String, String>,
// }
