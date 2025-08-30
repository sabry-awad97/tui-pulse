mod component;
pub use component::Component;

pub mod exit;
pub mod hooks;
pub mod frame_ext;
pub use frame_ext::FrameExt;

mod vdom;
pub use vdom::{Element, IntoElement};

// Re-export commonly used items
pub use exit::{exit_guard, request_exit, reset_exit, should_exit};
