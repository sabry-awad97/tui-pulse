pub mod component;
pub use component::Component;

pub mod exit;
pub mod hooks;

mod vdom;
pub use vdom::{Element, IntoElement};

pub mod panic_handler;

// Re-export commonly used items
pub use exit::{exit_guard, request_exit, reset_exit, should_exit};
pub use hooks::effect::{
    use_async_effect_always, use_async_effect_once, use_effect, use_effect_always, use_effect_once,
};
pub use hooks::event::global_events::on_global_event;
