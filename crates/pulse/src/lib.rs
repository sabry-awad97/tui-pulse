pub use crossterm;
pub use pulse_core::{
    Component, Element, IntoElement,
    exit::request_exit,
    hooks::{
        effect::{
            EffectDependencies, use_async_effect, use_async_effect_always, use_async_effect_once,
            use_effect, use_effect_always, use_effect_once,
        },
        event::{global_events::on_global_event, use_event},
        interval::{use_async_interval, use_interval},
        signal::{GlobalSignal, Signal, use_global_signal},
        state::{StateHandle, StateSetter, use_state},
    },
};
pub use pulse_runtime::*;

pub mod prelude {
    pub use super::*;
    pub use ratatui::{self, Frame, layout::Rect};
}
