use crossterm::event::Event;
use ratatui::Frame;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

static CURRENT_EVENT: AtomicPtr<Event> = AtomicPtr::new(ptr::null_mut());

/// Extension trait for `ratatui::Frame` to provide additional functionality
pub trait FrameExt {
    /// Get the current event if available
    ///
    /// # Returns
    /// `Option<&Event>` - The current event if available, `None` otherwise
    ///
    /// # Safety
    /// This method is safe to call from any thread, but the returned reference
    /// should not outlive the current frame.
    ///
    /// # Example
    /// ```rust
    /// use ratatui::Frame;
    /// use pulse_core::frame_ext::FrameExt;
    ///
    /// fn handle_event(frame: &mut Frame) {
    ///     if let Some(event) = frame.event() {
    ///         match event {
    ///             Event::Key(key) => { /* handle key event */ }
    ///             Event::Mouse(mouse) => { /* handle mouse event */ }
    ///             _ => {}
    ///         }
    ///     }
    /// }
    /// ```
    fn event(&self) -> Option<&Event>;
}

impl FrameExt for Frame<'_> {
    fn event(&self) -> Option<&Event> {
        // Safety: We're only reading the pointer, not modifying it
        let ptr = CURRENT_EVENT.load(Ordering::Acquire);
        if ptr.is_null() {
            None
        } else {
            // Safety: The pointer is only set to a valid Event and cleared when dropped
            unsafe { Some(&*ptr) }
        }
    }
}

/// Set the current event to be available through FrameExt
///
/// # Safety
/// The caller must ensure the pointer remains valid until it's no longer needed
#[allow(clippy::missing_safety_doc)]
pub unsafe fn set_current_event(event: Option<&Event>) {
    if let Some(e) = event {
        let ptr = Box::into_raw(Box::new(e.clone()));
        let old_ptr = CURRENT_EVENT.swap(ptr, Ordering::Release);

        // Free the old pointer if it exists
        if !old_ptr.is_null() {
            unsafe {
                drop(Box::from_raw(old_ptr));
            }
        }
    } else {
        let old_ptr = CURRENT_EVENT.swap(ptr::null_mut(), Ordering::Release);
        if !old_ptr.is_null() {
            unsafe {
                drop(Box::from_raw(old_ptr));
            }
        }
    }
}
