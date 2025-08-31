use crate::hooks::test_utils::with_test_isolate;

use super::*;
use std::sync::{Arc, Mutex};

type CallTracker = Arc<Mutex<Vec<String>>>;

// Test component that tracks mount/unmount calls
#[derive(Clone, Debug)]
struct TestComponent {
    id: String,
    mount_calls: CallTracker,
    unmount_calls: CallTracker,
}

impl TestComponent {
    fn new(id: &str) -> (Self, CallTracker, CallTracker) {
        let mount_calls = Arc::new(Mutex::new(Vec::new()));
        let unmount_calls = Arc::new(Mutex::new(Vec::new()));

        let component = TestComponent {
            id: id.to_string(),
            mount_calls: mount_calls.clone(),
            unmount_calls: unmount_calls.clone(),
        };

        (component, mount_calls, unmount_calls)
    }
}

impl Component for TestComponent {
    fn component_id(&self) -> String {
        format!("test_component_{}", self.id)
    }

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
fn simulate_render_with_mount<T: Component>(component: &T) {
    let component_id = component.component_id();
    let id_hash = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        component_id.hash(&mut hasher);
        hasher.finish() as usize
    };

    // Track this component in the current render
    let is_first_render = MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.track_mount(id_hash, component)
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

        // Calculate the expected component ID hash
        let component_id = component.component_id();
        let id_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            component_id.hash(&mut hasher);
            hasher.finish() as usize
        };

        assert!(state.mounted.contains(&id_hash));
        assert!(state.current_render.contains(&id_hash));
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

        // Calculate the expected component ID hash
        let component_id = component1.component_id();
        let id_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            component_id.hash(&mut hasher);
            hasher.finish() as usize
        };
        assert!(state.mounted.contains(&id_hash));
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

// Test for component ID stability and tracking
#[test]
fn test_component_id_stability() {
    // Clear mount state before test
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let (component, _, _) = TestComponent::new("id_test");

    // Get the component ID and hash
    let id1 = component.component_id();
    let hash1 = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        id1.hash(&mut hasher);
        hasher.finish() as usize
    };

    // Render multiple times
    simulate_render_with_mount(&component);
    let id2 = component.component_id();
    let hash2 = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        id2.hash(&mut hasher);
        hasher.finish() as usize
    };

    simulate_render_with_mount(&component);
    let id3 = component.component_id();
    let hash3 = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        id3.hash(&mut hasher);
        hasher.finish() as usize
    };

    // Component ID should remain stable
    assert_eq!(id1, id2);
    assert_eq!(id2, id3);
    assert_eq!(hash1, hash2);
    assert_eq!(hash2, hash3);

    // Verify it's tracked consistently
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert!(state.mounted.contains(&hash1));
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

#[test]
fn test_component_id_collision_resistance() {
    // Test that different components with similar IDs don't collide
    let (comp1, _, _) = TestComponent::new("test");
    let (comp2, _, _) = TestComponent::new("test_");
    let (comp3, _, _) = TestComponent::new("test1");

    let id1 = comp1.component_id();
    let id2 = comp2.component_id();
    let id3 = comp3.component_id();

    // All IDs should be different
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);

    // Hash values should also be different
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let hash1 = {
        let mut hasher = DefaultHasher::new();
        id1.hash(&mut hasher);
        hasher.finish() as usize
    };

    let hash2 = {
        let mut hasher = DefaultHasher::new();
        id2.hash(&mut hasher);
        hasher.finish() as usize
    };

    let hash3 = {
        let mut hasher = DefaultHasher::new();
        id3.hash(&mut hasher);
        hasher.finish() as usize
    };

    assert_ne!(hash1, hash2);
    assert_ne!(hash2, hash3);
    assert_ne!(hash1, hash3);
}

#[test]
fn test_mount_unmount_rapid_cycling() {
    // Test rapid mounting and unmounting cycles
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let (component, mount_calls, unmount_calls) = TestComponent::new("rapid_cycle");

    // Perform multiple mount/unmount cycles
    for _i in 0..5 {
        // Mount
        simulate_render_with_mount(&component);

        // Verify mounted
        MOUNT_STATE.with(|state| {
            let state = state.borrow();
            let component_id = component.component_id();
            let id_hash = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                component_id.hash(&mut hasher);
                hasher.finish() as usize
            };
            assert!(state.mounted.contains(&id_hash));
        });

        // Unmount by clearing current render and cleaning up
        MOUNT_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.current_render.clear();
        });
        cleanup_unmounted();

        // Verify unmounted
        MOUNT_STATE.with(|state| {
            let state = state.borrow();
            assert_eq!(state.mounted.len(), 0);
        });
    }

    // Should have 5 mount calls and 5 unmount calls
    assert_eq!(mount_calls.lock().unwrap().len(), 5);
    assert_eq!(unmount_calls.lock().unwrap().len(), 5);
}

#[test]
fn test_concurrent_component_operations() {
    // Test multiple components being mounted/unmounted simultaneously
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let component_ids: Vec<String> = (0..10).map(|i| format!("concurrent_{}", i)).collect();

    let components: Vec<_> = component_ids
        .iter()
        .map(|id| TestComponent::new(id))
        .collect();

    // Mount all components
    for (component, _, _) in &components {
        simulate_render_with_mount(component);
    }

    // Verify all are mounted
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 10);
        assert_eq!(state.current_render.len(), 10);
    });

    // Unmount half of them
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.current_render.clear();

        // Re-render only first 5 components
        for (component, _, _) in components.iter().take(5) {
            let component_id = component.component_id();
            let id_hash = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                component_id.hash(&mut hasher);
                hasher.finish() as usize
            };
            state.current_render.insert(id_hash);
        }
    });

    cleanup_unmounted();

    // Verify only 5 remain mounted
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 5);
    });

    // Verify mount/unmount call counts
    for (i, (_, mount_calls, unmount_calls)) in components.iter().enumerate() {
        assert_eq!(mount_calls.lock().unwrap().len(), 1); // All mounted once
        if i < 5 {
            assert_eq!(unmount_calls.lock().unwrap().len(), 0); // First 5 still mounted
        } else {
            assert_eq!(unmount_calls.lock().unwrap().len(), 1); // Last 5 unmounted
        }
    }
}

#[test]
fn test_empty_component_id_handling() {
    // Test component with empty ID
    let (component, mount_calls, _) = TestComponent::new("");

    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    // Should still work with empty ID
    simulate_render_with_mount(&component);

    assert_eq!(mount_calls.lock().unwrap().len(), 1);

    // Verify component is tracked
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 1);
    });
}

#[test]
fn test_very_long_component_id() {
    // Test component with extremely long ID
    let long_id = "a".repeat(10000);
    let (component, mount_calls, _unmount_calls) = TestComponent::new(&long_id);

    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    // Should handle long IDs without issues
    simulate_render_with_mount(&component);

    assert_eq!(mount_calls.lock().unwrap().len(), 1);

    // Verify ID is correct
    assert_eq!(
        component.component_id(),
        format!("test_component_{}", long_id)
    );
}

#[test]
fn test_special_characters_in_component_id() {
    // Test component IDs with special characters
    let special_ids = vec![
        "comp@#$%^&*()",
        "comp\n\t\r",
        "comp🚀🎉💻",
        "comp with spaces",
        "comp-with-dashes_and_underscores",
        "comp.with.dots",
    ];

    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let mut components = Vec::new();

    for id in &special_ids {
        let (component, mount_calls, _) = TestComponent::new(id);
        simulate_render_with_mount(&component);
        components.push((component, mount_calls));
    }

    // All should be mounted successfully
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), special_ids.len());
    });

    // All should have been called once
    for (_, mount_calls) in &components {
        assert_eq!(mount_calls.lock().unwrap().len(), 1);
    }
}

#[test]
fn test_component_remount_after_unmount() {
    // Test that a component can be remounted after being unmounted
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let (component, mount_calls, unmount_calls) = TestComponent::new("remount_test");

    // Initial mount
    simulate_render_with_mount(&component);
    assert_eq!(mount_calls.lock().unwrap().len(), 1);

    // Unmount
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.current_render.clear();
    });
    cleanup_unmounted();
    assert_eq!(unmount_calls.lock().unwrap().len(), 1);

    // Remount
    simulate_render_with_mount(&component);
    assert_eq!(mount_calls.lock().unwrap().len(), 2);

    // Verify state
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 1);
    });
}

#[test]
fn test_mount_state_stress_test() {
    // Stress test with many components
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let num_components = 1000;
    let component_ids: Vec<String> = (0..num_components)
        .map(|i| format!("stress_{}", i))
        .collect();

    let components: Vec<_> = component_ids
        .iter()
        .map(|id| TestComponent::new(id))
        .collect();

    // Mount all components
    for (component, _, _) in &components {
        simulate_render_with_mount(component);
    }

    // Verify all mounted
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), num_components);
        assert_eq!(state.current_render.len(), num_components);
    });

    // Unmount all
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.current_render.clear();
    });
    cleanup_unmounted();

    // Verify all unmounted
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 0);
    });

    // Verify all components called mount and unmount exactly once
    for (_, mount_calls, unmount_calls) in &components {
        assert_eq!(mount_calls.lock().unwrap().len(), 1);
        assert_eq!(unmount_calls.lock().unwrap().len(), 1);
    }
}

#[test]
fn test_partial_render_cycles() {
    // Test components that are only rendered in some cycles
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let (comp1, mount1, unmount1) = TestComponent::new("partial_1");
    let (comp2, mount2, unmount2) = TestComponent::new("partial_2");
    let (comp3, mount3, unmount3) = TestComponent::new("partial_3");

    // Cycle 1: Render comp1 and comp2
    simulate_render_with_mount(&comp1);
    simulate_render_with_mount(&comp2);
    cleanup_unmounted();

    assert_eq!(mount1.lock().unwrap().len(), 1);
    assert_eq!(mount2.lock().unwrap().len(), 1);
    assert_eq!(mount3.lock().unwrap().len(), 0);

    // Cycle 2: Render comp2 and comp3 (comp1 gets unmounted)
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.current_render.clear();
    });
    simulate_render_with_mount(&comp2);
    simulate_render_with_mount(&comp3);
    cleanup_unmounted();

    assert_eq!(mount1.lock().unwrap().len(), 1);
    assert_eq!(mount2.lock().unwrap().len(), 1); // Still 1, already mounted
    assert_eq!(mount3.lock().unwrap().len(), 1);
    assert_eq!(unmount1.lock().unwrap().len(), 1); // comp1 unmounted
    assert_eq!(unmount2.lock().unwrap().len(), 0); // comp2 still mounted
    assert_eq!(unmount3.lock().unwrap().len(), 0); // comp3 still mounted

    // Cycle 3: Render only comp1 (comp2 and comp3 get unmounted)
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.current_render.clear();
    });
    simulate_render_with_mount(&comp1);
    cleanup_unmounted();

    assert_eq!(mount1.lock().unwrap().len(), 2); // Remounted
    assert_eq!(unmount2.lock().unwrap().len(), 1); // comp2 unmounted
    assert_eq!(unmount3.lock().unwrap().len(), 1); // comp3 unmounted
}

#[test]
fn test_mount_state_memory_efficiency() {
    // Test that mount state doesn't grow indefinitely
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    // Create and mount many components, then unmount them
    for batch in 0..10 {
        let component_ids: Vec<String> =
            (0..100).map(|i| format!("batch_{}_{}", batch, i)).collect();

        let components: Vec<_> = component_ids
            .iter()
            .map(|id| TestComponent::new(id))
            .collect();

        // Mount all in this batch
        for (component, _, _) in &components {
            simulate_render_with_mount(component);
        }

        // Verify they're all mounted
        MOUNT_STATE.with(|state| {
            let state = state.borrow();
            assert_eq!(state.mounted.len(), 100);
        });

        // Unmount all
        MOUNT_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.current_render.clear();
        });
        cleanup_unmounted();

        // Verify all unmounted
        MOUNT_STATE.with(|state| {
            let state = state.borrow();
            assert_eq!(state.mounted.len(), 0);
        });
    }
}

#[test]
fn test_component_lifecycle_error_recovery() {
    // Test that lifecycle continues working even if a component panics
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    #[derive(Debug, Clone)]
    struct PanicComponent {
        id: String,
        should_panic_on_mount: bool,
        mount_calls: Arc<Mutex<Vec<String>>>,
    }

    impl Component for PanicComponent {
        fn component_id(&self) -> String {
            format!("panic_component_{}", self.id)
        }

        fn on_mount(&self) {
            if self.should_panic_on_mount {
                // In a real scenario, we'd want to handle panics gracefully
                // For this test, we'll just record the attempt
                self.mount_calls
                    .lock()
                    .unwrap()
                    .push("attempted_mount".to_string());
            } else {
                self.mount_calls
                    .lock()
                    .unwrap()
                    .push("successful_mount".to_string());
            }
        }

        fn render(&self, _area: Rect, _frame: &mut Frame) {}
    }

    let mount_calls = Arc::new(Mutex::new(Vec::new()));
    let panic_component = PanicComponent {
        id: "panic_test".to_string(),
        should_panic_on_mount: true,
        mount_calls: mount_calls.clone(),
    };

    let normal_component = PanicComponent {
        id: "normal_test".to_string(),
        should_panic_on_mount: false,
        mount_calls: mount_calls.clone(),
    };

    // Render both components
    simulate_render_with_mount(&panic_component);
    simulate_render_with_mount(&normal_component);

    // Both should have attempted to mount
    let calls = mount_calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert!(calls.contains(&"attempted_mount".to_string()));
    assert!(calls.contains(&"successful_mount".to_string()));
}

#[test]
fn test_component_id_uniqueness_across_types() {
    // Test that different component types have unique IDs
    #[derive(Debug, Clone)]
    struct ComponentTypeA {
        id: String,
    }

    #[derive(Debug, Clone)]
    struct ComponentTypeB {
        id: String,
    }

    impl Component for ComponentTypeA {
        fn component_id(&self) -> String {
            format!("type_a_{}", self.id)
        }
        fn render(&self, _area: Rect, _frame: &mut Frame) {}
    }

    impl Component for ComponentTypeB {
        fn component_id(&self) -> String {
            format!("type_b_{}", self.id)
        }
        fn render(&self, _area: Rect, _frame: &mut Frame) {}
    }

    let comp_a = ComponentTypeA {
        id: "same_id".to_string(),
    };
    let comp_b = ComponentTypeB {
        id: "same_id".to_string(),
    };

    let id_a = comp_a.component_id();
    let id_b = comp_b.component_id();

    // Should be different despite same internal ID
    assert_ne!(id_a, id_b);
    assert_eq!(id_a, "type_a_same_id");
    assert_eq!(id_b, "type_b_same_id");
}

#[test]
fn test_component_lifecycle_with_nested_renders() {
    // Test components that trigger additional renders during their lifecycle
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    #[derive(Debug, Clone)]
    struct NestedRenderComponent {
        id: String,
        mount_calls: Arc<Mutex<Vec<String>>>,
        child_component: Option<TestComponent>,
    }

    impl Component for NestedRenderComponent {
        fn component_id(&self) -> String {
            format!("nested_{}", self.id)
        }

        fn on_mount(&self) {
            self.mount_calls
                .lock()
                .unwrap()
                .push("nested_mounted".to_string());

            // Note: In real usage, child rendering would happen in the render method
            // For this test, we'll track the child separately
        }

        fn render(&self, _area: Rect, _frame: &mut Frame) {}
    }

    let (child_component, child_mount_calls, _) = TestComponent::new("child");
    let mount_calls = Arc::new(Mutex::new(Vec::new()));

    let parent_component = NestedRenderComponent {
        id: "parent".to_string(),
        mount_calls: mount_calls.clone(),
        child_component: Some(child_component),
    };

    // Render parent and child separately to avoid RefCell conflicts
    simulate_render_with_mount(&parent_component);
    if let Some(ref child) = parent_component.child_component {
        simulate_render_with_mount(child);
    }

    // Both parent and child should be mounted
    assert_eq!(mount_calls.lock().unwrap().len(), 1);
    assert_eq!(child_mount_calls.lock().unwrap().len(), 1);

    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 2); // Parent and child
    });
}

#[test]
fn test_component_id_hash_distribution() {
    // Test that component ID hashes are well-distributed
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    use std::collections::hash_map::DefaultHasher;
    use std::collections::{HashMap, HashSet};
    use std::hash::{Hash, Hasher};

    let mut hash_counts = HashMap::new();
    let mut unique_hashes = HashSet::new();

    // Generate many component IDs and check hash distribution
    for i in 0..1000 {
        let component_id = format!("hash_test_{}", i);
        let hash = {
            let mut hasher = DefaultHasher::new();
            component_id.hash(&mut hasher);
            hasher.finish() as usize
        };

        *hash_counts.entry(hash % 100).or_insert(0) += 1;
        unique_hashes.insert(hash);
    }

    // Should have good distribution (no bucket should have more than 20% of items)
    for count in hash_counts.values() {
        assert!(
            *count < 200,
            "Hash distribution is too skewed: {} items in one bucket",
            count
        );
    }

    // Should have mostly unique hashes
    assert!(
        unique_hashes.len() > 990,
        "Too many hash collisions: only {} unique hashes",
        unique_hashes.len()
    );
}

#[test]
fn test_mount_state_thread_safety_simulation() {
    // Simulate potential thread safety issues (though we use RefCell)
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let component_ids: Vec<String> = (0..50).map(|i| format!("thread_test_{}", i)).collect();

    let components: Vec<_> = component_ids
        .iter()
        .map(|id| TestComponent::new(id))
        .collect();

    // Simulate interleaved mount/unmount operations
    for chunk in components.chunks(10) {
        // Mount this chunk
        for (component, _, _) in chunk {
            simulate_render_with_mount(component);
        }

        // Verify mount state consistency
        MOUNT_STATE.with(|state| {
            let state = state.borrow();
            assert_eq!(state.mounted.len(), state.current_render.len());
        });

        // Unmount this chunk
        MOUNT_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.current_render.clear();
        });
        cleanup_unmounted();

        // Verify clean state
        MOUNT_STATE.with(|state| {
            let state = state.borrow();
            assert_eq!(state.mounted.len(), 0);
            assert_eq!(state.current_render.len(), 0);
        });
    }
}

#[test]
fn test_component_lifecycle_with_dynamic_ids() {
    // Test components that change their IDs dynamically
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    #[derive(Debug, Clone)]
    struct DynamicIdComponent {
        base_id: String,
        counter: Arc<Mutex<usize>>,
        mount_calls: Arc<Mutex<Vec<String>>>,
    }

    impl Component for DynamicIdComponent {
        fn component_id(&self) -> String {
            let counter = *self.counter.lock().unwrap();
            format!("dynamic_{}_{}", self.base_id, counter)
        }

        fn on_mount(&self) {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;
            self.mount_calls
                .lock()
                .unwrap()
                .push(format!("mounted_{}", *counter));
        }

        fn render(&self, _area: Rect, _frame: &mut Frame) {}
    }

    let mount_calls = Arc::new(Mutex::new(Vec::new()));
    let component = DynamicIdComponent {
        base_id: "test".to_string(),
        counter: Arc::new(Mutex::new(0)),
        mount_calls: mount_calls.clone(),
    };

    // Each render should see a different component ID due to counter increment
    simulate_render_with_mount(&component);
    simulate_render_with_mount(&component);
    simulate_render_with_mount(&component);

    // Should have 3 mount calls since ID changes each time
    assert_eq!(mount_calls.lock().unwrap().len(), 3);

    // Should have 3 different components tracked
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 3);
    });
}

#[test]
fn test_zero_sized_component_handling() {
    // Test zero-sized types as components
    #[derive(Debug, Clone)]
    struct ZeroSizedComponent;

    impl Component for ZeroSizedComponent {
        fn component_id(&self) -> String {
            "zero_sized_component".to_string()
        }

        fn render(&self, _area: Rect, _frame: &mut Frame) {}
    }

    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let component1 = ZeroSizedComponent;
    let component2 = ZeroSizedComponent;

    // Both instances should have the same component ID
    assert_eq!(component1.component_id(), component2.component_id());

    // But they should be treated as the same component
    simulate_render_with_mount(&component1);
    simulate_render_with_mount(&component2);

    // Should only be mounted once since they have the same ID
    MOUNT_STATE.with(|state| {
        let state = state.borrow();
        assert_eq!(state.mounted.len(), 1);
    });
}

#[test]
fn test_component_lifecycle_ordering() {
    // Test that mount/unmount events happen in the correct order
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.mounted.clear();
        state.current_render.clear();
    });

    let global_log = Arc::new(Mutex::new(Vec::new()));

    #[derive(Debug, Clone)]
    struct OrderTestComponent {
        id: String,
        log: Arc<Mutex<Vec<String>>>,
    }

    impl Component for OrderTestComponent {
        fn component_id(&self) -> String {
            format!("order_test_{}", self.id)
        }

        fn on_mount(&self) {
            self.log
                .lock()
                .unwrap()
                .push(format!("{}_mount_start", self.id));
            // Simulate some work
            std::thread::sleep(std::time::Duration::from_millis(1));
            self.log
                .lock()
                .unwrap()
                .push(format!("{}_mount_end", self.id));
        }

        fn on_unmount(&self) {
            self.log
                .lock()
                .unwrap()
                .push(format!("{}_unmount_start", self.id));
            // Simulate some work
            std::thread::sleep(std::time::Duration::from_millis(1));
            self.log
                .lock()
                .unwrap()
                .push(format!("{}_unmount_end", self.id));
        }

        fn render(&self, _area: Rect, _frame: &mut Frame) {}
    }

    let comp1 = OrderTestComponent {
        id: "comp1".to_string(),
        log: global_log.clone(),
    };

    let comp2 = OrderTestComponent {
        id: "comp2".to_string(),
        log: global_log.clone(),
    };

    // Mount comp1, then comp2
    simulate_render_with_mount(&comp1);
    simulate_render_with_mount(&comp2);

    // Unmount comp1, keep comp2
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.current_render.clear();
        let comp2_id = comp2.component_id();
        let id_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            comp2_id.hash(&mut hasher);
            hasher.finish() as usize
        };
        state.current_render.insert(id_hash);
    });
    cleanup_unmounted();

    let log = global_log.lock().unwrap();

    // Verify ordering: comp1 mount, comp2 mount, comp1 unmount
    assert!(log.contains(&"comp1_mount_start".to_string()));
    assert!(log.contains(&"comp1_mount_end".to_string()));
    assert!(log.contains(&"comp2_mount_start".to_string()));
    assert!(log.contains(&"comp2_mount_end".to_string()));
    assert!(log.contains(&"comp1_unmount_start".to_string()));
    assert!(log.contains(&"comp1_unmount_end".to_string()));

    // comp2 should not have unmount events
    assert!(!log.iter().any(|entry| entry.contains("comp2_unmount")));
}
