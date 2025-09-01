pub use crossterm;
pub use pulse_core::{
    Component, Element, IntoElement,
    exit::request_exit,
    hooks::{
        callback::{Callback, CallbackFactory, use_callback, use_callback_once},
        context::{Context, use_context, use_context_provider, use_context_with_default},
        effect::{
            EffectDependencies, use_async_effect, use_async_effect_always, use_async_effect_once,
            use_effect, use_effect_always, use_effect_once,
        },
        event::{global_events::on_global_event, use_event},
        future::{FutureError, FutureHandle, FutureState, use_future, use_future_with_progress},
        hover::{use_hover, use_hover_with_callbacks},
        idle::{use_idle, use_idle_timing, use_idle_with_callback},
        interval::{use_async_interval, use_interval},
        reducer::{DispatchFn, ReducerStateHandle, use_reducer},
        signal::{GlobalSignal, Signal, use_global_signal},
        state::{StateHandle, StateSetter, use_state},
        storage::{LocalStorageConfig, set_storage_config, use_local_storage},
    },
};

#[cfg(feature = "sqlite")]
pub use pulse_core::hooks::storage::{AsyncStorageBackend, SqliteStorageBackend};

pub use pulse_runtime::*;

pub mod prelude {
    pub use super::*;
    pub use ratatui::{self, Frame, layout::Rect};
}
