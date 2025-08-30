//! Event handling hooks for TUI applications

use crossterm::event::Event;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

pub mod global_events;

static CURRENT_EVENT: AtomicPtr<Event> = AtomicPtr::new(ptr::null_mut());

use std::cell::Cell;
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
use std::thread_local;

thread_local! {
    // Track if we've already consumed the event in this render cycle
    static EVENT_CONSUMED: Cell<bool> = const { Cell::new(false) };
}

pub fn use_event() -> Option<&'static Event> {
    // If we've already consumed the event in this render cycle, return None
    if EVENT_CONSUMED.with(|consumed| consumed.get()) {
        return None;
    }

    // Mark the event as consumed
    EVENT_CONSUMED.with(|consumed| consumed.set(true));

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
    // Reset the event consumption flag at the start of each render cycle
    EVENT_CONSUMED.with(|consumed| consumed.set(false));

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
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_use_event() {
        // Test with no event set
        unsafe { set_current_event(None) };
        assert!(use_event().is_none());
    }

    #[test]
    fn test_event_consumed_once_per_render() {
        // Set up a test event
        let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));

        // First render cycle
        unsafe { set_current_event(Some(&event)) };

        // First call should return the event
        assert!(use_event().is_some());

        // Second call in the same render cycle should return None
        assert!(use_event().is_none());

        // New render cycle
        let event2 = Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        unsafe { set_current_event(Some(&event2)) };

        // Should be able to get the new event
        assert!(use_event().is_some());
    }

    #[test]
    fn test_multiple_components_consume_same_event() {
        // Set up a test event
        let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));

        // Simulate render cycle start
        unsafe { set_current_event(Some(&event)) };

        // First component consumes the event
        let first_consumption = use_event();
        assert!(first_consumption.is_some());

        // Second component tries to consume the same event
        let second_consumption = use_event();
        assert!(
            second_consumption.is_none(),
            "Second component should not be able to consume the same event in the same render cycle"
        );
    }

    #[test]
    fn test_event_reset_between_renders() {
        // First render cycle
        let event1 = Event::Key(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE));
        unsafe { set_current_event(Some(&event1)) };

        // Consume in first render
        assert!(use_event().is_some());

        // Second render cycle with new event
        let event2 = Event::Key(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE));
        unsafe { set_current_event(Some(&event2)) };

        // Should be able to consume the new event
        assert!(use_event().is_some());
    }
}
