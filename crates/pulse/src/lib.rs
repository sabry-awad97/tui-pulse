pub use crossterm;
pub use pulse_core::{
    Component, Element, IntoElement,
    exit::request_exit,
    hooks::{
        event::{global_events::on_global_event, use_event},
        signal::{GlobalSignal, Signal, use_global_signal},
        state::{StateHandle, StateSetter, use_state},
    },
};
pub use pulse_runtime::*;

pub mod prelude {
    pub use super::*;
    pub use ratatui::{self, Frame, layout::Rect};
}
