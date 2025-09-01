use crate::hooks::{once::use_once, test_utils::with_component_id};
use std::sync::{Arc, Mutex};

#[test]
fn test_use_once_basic() {
    let call_count = Arc::new(Mutex::new(0));

    with_component_id("OnceTestComponent", |_ctx| {
        let count_clone = call_count.clone();

        // First call should execute
        use_once(move || {
            *count_clone.lock().unwrap() += 1;
            println!("First execution");
        });

        // Give effect time to run
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(*call_count.lock().unwrap(), 1);
    });

    // Second render of same component - should not execute again
    with_component_id("OnceTestComponent", |_ctx| {
        let count_clone2 = call_count.clone();

        use_once(move || {
            *count_clone2.lock().unwrap() += 1;
            println!("Second execution - should not happen");
        });

        // Give effect time to run
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(*call_count.lock().unwrap(), 1); // Should still be 1
    });
}

#[test]
fn test_use_once_different_components() {
    let call_count = Arc::new(Mutex::new(0));

    // First component
    with_component_id("OnceTestComponent1", |_ctx| {
        let count_clone = call_count.clone();
        use_once(move || {
            *count_clone.lock().unwrap() += 1;
            println!("Component 1 execution");
        });
        assert_eq!(*call_count.lock().unwrap(), 1);
    });

    // Second component should execute independently
    with_component_id("OnceTestComponent2", |_ctx| {
        let count_clone = call_count.clone();
        use_once(move || {
            *count_clone.lock().unwrap() += 1;
            println!("Component 2 execution");
        });
        assert_eq!(*call_count.lock().unwrap(), 2);
    });
}

#[test]
fn test_use_once_multiple_calls_same_component() {
    let call_count = Arc::new(Mutex::new(0));

    with_component_id("OnceMultipleCallsComponent", |_ctx| {
        let count_clone1 = call_count.clone();
        let count_clone2 = call_count.clone();
        let count_clone3 = call_count.clone();

        // Multiple different once calls in the same component
        use_once(move || {
            *count_clone1.lock().unwrap() += 1;
            println!("First call site");
        });

        use_once(move || {
            *count_clone2.lock().unwrap() += 1;
            println!("Second call site");
        });

        use_once(move || {
            *count_clone3.lock().unwrap() += 1;
            println!("Third call site");
        });

        // Give effects time to run
        std::thread::sleep(std::time::Duration::from_millis(10));
        // All should execute since they're different call sites
        assert_eq!(*call_count.lock().unwrap(), 3);
    });

    // Second render - should not execute again
    with_component_id("OnceMultipleCallsComponent", |_ctx| {
        let count_clone4 = call_count.clone();

        use_once(move || {
            *count_clone4.lock().unwrap() += 1;
            println!("Fourth call - should not execute");
        });

        // Give effect time to run
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(*call_count.lock().unwrap(), 3); // Should remain 3
    });
}

#[test]
fn test_use_once_with_side_effects() {
    let side_effect_count = Arc::new(Mutex::new(0));

    with_component_id("OnceSideEffectComponent", |_ctx| {
        let count_clone1 = side_effect_count.clone();

        // First call should have side effects
        use_once(move || {
            *count_clone1.lock().unwrap() += 1;
            println!("Side effect executed");
        });

        // Give effect time to run
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(*side_effect_count.lock().unwrap(), 1);
    });

    // Second render - should not have side effects
    with_component_id("OnceSideEffectComponent", |_ctx| {
        let count_clone2 = side_effect_count.clone();

        use_once(move || {
            *count_clone2.lock().unwrap() += 1;
            println!("Side effect executed again");
        });

        // Give effect time to run
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(*side_effect_count.lock().unwrap(), 1); // Should remain 1
    });
}

#[test]
fn test_use_once_logging() {
    // Test that logging works without panicking
    with_component_id("LoggingTestComponent", |_ctx| {
        use_once(|| {
            println!("Component mounted - this should only print once");
        });

        use_once(|| {
            println!("This should not print on subsequent calls");
        });
    });
}
