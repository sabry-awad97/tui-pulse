use crate::hooks::test_utils::with_test_isolate;

use super::*;
use std::sync::{Arc, Mutex};

type CallTracker = Arc<Mutex<Vec<String>>>;

// Test component that tracks mount/unmount calls
#[derive(Clone)]
struct TestComponent {
    id: &'static str,
    mount_calls: CallTracker,
    unmount_calls: CallTracker,
}

impl TestComponent {
    fn new(id: &'static str) -> (Self, CallTracker, CallTracker) {
        let mount_calls = Arc::new(Mutex::new(Vec::new()));
        let unmount_calls = Arc::new(Mutex::new(Vec::new()));

        let component = TestComponent {
            id,
            mount_calls: mount_calls.clone(),
            unmount_calls: unmount_calls.clone(),
        };

        (component, mount_calls, unmount_calls)
    }
}

impl Component for TestComponent {
    fn on_mount(&self) {
        self.mount_calls
            .lock()
            .unwrap()
            .push(format!("{}_mounted", self.id));
    }

    fn on_unmount(&self) {
        self.unmount_calls
            .lock()
            .unwrap()
            .push(format!("{}_unmounted", self.id));
    }

    fn render(&self, _area: Rect, _frame: &mut Frame) {
        // Minimal render implementation for testing
    }
}

// Helper to simulate render with mount tracking
fn simulate_render_with_mount(component: &TestComponent) {
    let ptr = component as *const TestComponent as usize;

    // Track this component in the current render
    let is_first_render = MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.track_mount(ptr)
    });

    // Call on_mount on first render
    if is_first_render {
        component.on_mount();
    }
}

#[test]
fn test_component_mount_called_on_first_render() {
    // Clear mount state before test
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let (component, mount_calls, _unmount_calls) = TestComponent::new("test1");

    // First render should call on_mount
    simulate_render_with_mount(&component);

    let calls = mount_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], "test1_mounted");
}

#[test]
fn test_component_mount_called_only_once() {
    // Clear mount state before test
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let (component, mount_calls, _unmount_calls) = TestComponent::new("test2");

    // Multiple renders should only call on_mount once
    for _ in 0..3 {
        simulate_render_with_mount(&component);
    }

    let calls = mount_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], "test2_mounted");
}

#[test]
fn test_different_component_instances_mount_separately() {
    // Clear mount state before test
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let (component1, mount_calls1, _) = TestComponent::new("comp1");
    let (component2, mount_calls2, _) = TestComponent::new("comp2");

    // Both components should have their on_mount called
    simulate_render_with_mount(&component1);
    simulate_render_with_mount(&component2);

    let calls1 = mount_calls1.lock().unwrap();
    let calls2 = mount_calls2.lock().unwrap();

    assert_eq!(calls1.len(), 1);
    assert_eq!(calls1[0], "comp1_mounted");

    assert_eq!(calls2.len(), 1);
    assert_eq!(calls2[0], "comp2_mounted");
}

#[test]
fn test_mount_state_tracking() {
    // Clear mount state before test
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let (component, mount_calls, _) = TestComponent::new("track_test");

    // Verify component is tracked after first render
    simulate_render_with_mount(&component);

    // Check that mount state is properly tracked
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        let ptr = &component as *const TestComponent as usize;
        assert!(state.mounted.contains(&ptr));
        assert!(state.current_render.contains(&ptr));
    });

    let calls = mount_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
}

#[test]
fn test_cleanup_unmounted_removes_components() {
    // Clear mount state before test
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let (component1, _, _) = TestComponent::new("cleanup1");
    let (component2, _, _) = TestComponent::new("cleanup2");

    // Render both components
    simulate_render_with_mount(&component1);
    simulate_render_with_mount(&component2);

    // Verify both are tracked
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 2);
        assert_eq!(state.current_render.len(), 2);
    });

    // Call cleanup - this simulates end of render cycle
    cleanup_unmounted();

    // After cleanup, current_render should be cleared but mounted should remain
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 2);
        assert_eq!(state.current_render.len(), 0);
    });

    // Render only component1 in next cycle
    simulate_render_with_mount(&component1);

    // Call cleanup again - component2 should be removed from mounted
    cleanup_unmounted();

    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 1);
        let ptr1 = &component1 as *const TestComponent as usize;
        assert!(state.mounted.contains(&ptr1));
    });
}

#[test]
fn test_mount_state_reset_between_renders() {
    // Clear mount state before test
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let (component, _, _) = TestComponent::new("reset_test");

    // First render cycle
    simulate_render_with_mount(&component);
    cleanup_unmounted();

    // Verify current_render is cleared
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.current_render.len(), 0);
        assert_eq!(state.mounted.len(), 1);
    });

    // Second render cycle
    simulate_render_with_mount(&component);

    // Verify component is tracked in current_render again
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.current_render.len(), 1);
        assert_eq!(state.mounted.len(), 1);
    });
}

#[test]
fn test_component_lifecycle_integration() {
    with_test_isolate(|| {
        // Clear mount state before test
        MOUNT_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.mounted.clear();
            state.current_render.clear();
        });

        let (component1, mount_calls1, _) = TestComponent::new("lifecycle1");
        let (component2, mount_calls2, _) = TestComponent::new("lifecycle2");

        // Render cycle 1: Both components
        simulate_render_with_mount(&component1);
        simulate_render_with_mount(&component2);
        cleanup_unmounted();

        // Both should be mounted
        assert_eq!(mount_calls1.lock().unwrap().len(), 1);
        assert_eq!(mount_calls2.lock().unwrap().len(), 1);

        // Render cycle 2: Only component1
        simulate_render_with_mount(&component1);
        cleanup_unmounted();

        // No additional mount calls (already mounted)
        assert_eq!(mount_calls1.lock().unwrap().len(), 1);
        assert_eq!(mount_calls2.lock().unwrap().len(), 1);

        // Render cycle 3: Add component2 back
        simulate_render_with_mount(&component1);
        simulate_render_with_mount(&component2);
        cleanup_unmounted();

        // component2 should be mounted again (was unmounted in cycle 2)
        assert_eq!(mount_calls1.lock().unwrap().len(), 1);
        assert_eq!(mount_calls2.lock().unwrap().len(), 2); // Re-mounted after being unmounted
    });
}

// Test for memory safety and pointer validity
#[test]
fn test_component_pointer_stability() {
    // Clear mount state before test
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let (component, _, _) = TestComponent::new("pointer_test");

    // Get the pointer value
    let ptr1 = &component as *const TestComponent as usize;

    // Render multiple times
    simulate_render_with_mount(&component);
    let ptr2 = &component as *const TestComponent as usize;

    simulate_render_with_mount(&component);
    let ptr3 = &component as *const TestComponent as usize;

    // Pointer should remain stable
    assert_eq!(ptr1, ptr2);
    assert_eq!(ptr2, ptr3);

    // Verify it's tracked consistently
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert!(state.mounted.contains(&ptr1));
    });
}

#[test]
fn test_mount_state_isolation_between_tests() {
    // This test verifies that mount state doesn't leak between tests
    let (component, mount_calls, _) = TestComponent::new("isolation_test");

    // This should start with clean state
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 0);
        assert_eq!(state.current_render.len(), 0);
    });

    simulate_render_with_mount(&component);

    // Should have one mounted component
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 1);
    });

    assert_eq!(mount_calls.lock().unwrap().len(), 1);
}
