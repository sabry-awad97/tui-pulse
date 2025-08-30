//! Event handling hooks for TUI applications

use crossterm::event::Event;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

pub mod global_events;

static CURRENT_EVENT: AtomicPtr<Event> = AtomicPtr::new(ptr::null_mut());

/// A hook that provides access to the current event
///
/// # Returns
/// `Option<&'static Event>` - The current event if available, `None` otherwise
///
/// # Example
/// ```rust
/// use pulse_core::hooks::event::use_event;
/// use crossterm::event::Event;
///
/// fn my_component() {
///     let event = use_event();
///     // Handle the event...
/// }
/// ```
pub fn use_event() -> Option<&'static Event> {
    // Safety: We're only reading the pointer, not modifying it
    let ptr = CURRENT_EVENT.load(Ordering::Acquire);
    if ptr.is_null() {
        None
    } else {
        // Safety: The pointer is only set to a valid Event and cleared when dropped
        unsafe { Some(&*ptr) }
    }
}

/// Set the current event to be available through the use_event hook
///
/// # Safety
/// The caller must ensure the pointer remains valid until it's no longer needed
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_event() {
        // Test that we can set and get an event
        let test_event = Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('a'),
            crossterm::event::KeyModifiers::NONE,
        ));

        // Set the event
        unsafe {
            set_current_event(Some(&test_event));
        }

        // Get the event back
        let event = use_event();
        assert!(event.is_some());
        assert_eq!(event.unwrap(), &test_event);

        // Clear the event
        unsafe {
            set_current_event(None);
        }

        // Verify it's cleared
        assert!(use_event().is_none());
    }
}
