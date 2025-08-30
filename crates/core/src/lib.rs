mod component;
pub use component::Component;

pub mod exit;
pub mod hooks;

mod vdom;
pub use vdom::{Element, IntoElement};

// Re-export commonly used items
pub use exit::{exit_guard, request_exit, reset_exit, should_exit};
pub use hooks::event::global_events::on_global_event;
