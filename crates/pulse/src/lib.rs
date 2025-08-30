pub use pulse_core::{Component, Element, IntoElement};
pub use pulse_runtime::*;

pub mod prelude {
    pub use super::*;
    pub use ratatui::{self, Frame, layout::Rect};
}
