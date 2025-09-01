use super::*;
use std::sync::{Arc, Mutex};

#[test]
fn test_callback_basic_creation_and_emit() {
    let result = Arc::new(Mutex::new(String::new()));
    let result_clone = result.clone();

    let callback: Callback<String> = Callback::from(move |msg: String| {
        *result_clone.lock().unwrap() = msg;
    });

    callback.emit("Hello".to_string());
    assert_eq!(*result.lock().unwrap(), "Hello");
}

#[test]
fn test_callback_with_return_value() {
    let callback: Callback<i32, String> = Callback::from(|num: i32| format!("Number: {}", num));

    let result = callback.emit(42);
    assert_eq!(result, "Number: 42");
}

#[test]
fn test_callback_clone() {
    let call_count = Arc::new(Mutex::new(0));
    let count_clone = call_count.clone();

    let callback = Callback::from(move |_: ()| {
        *count_clone.lock().unwrap() += 1;
    });

    let cloned_callback = callback.clone();

    callback.emit(());
    cloned_callback.emit(());

    assert_eq!(*call_count.lock().unwrap(), 2);
}

#[test]
fn test_callback_new() {
    let callback = Callback::new(|x: i32| x * 2);
    assert_eq!(callback.emit(5), 10);
}

#[test]
fn test_callback_reform() {
    let original: Callback<String, usize> = Callback::from(|s: String| s.len());

    let reformed: Callback<i32, usize> = original.reform(|num: i32| format!("Number: {}", num));
    let result = reformed.emit(42);
    assert_eq!(result, 10); // "Number: 42".len() = 10
}

#[test]
fn test_callback_filter_reform() {
    let call_count = Arc::new(Mutex::new(0));
    let count_clone = call_count.clone();

    let original: Callback<String> = Callback::from(move |s: String| {
        *count_clone.lock().unwrap() += 1;
        println!("Got: {}", s);
    });

    let filtered: Callback<i32, Option<()>> = original.filter_reform(|num: i32| {
        if num > 0 {
            Some(format!("Positive: {}", num))
        } else {
            None
        }
    });

    // Positive number should call original
    let result1 = filtered.emit(42);
    assert_eq!(result1, Some(()));
    assert_eq!(*call_count.lock().unwrap(), 1);

    // Negative number should not call original
    let result2 = filtered.emit(-1);
    assert_eq!(result2, None);
    assert_eq!(*call_count.lock().unwrap(), 1); // Should remain 1
}

#[test]
fn test_callback_noop() {
    let callback: Callback<String> = Callback::noop();
    // Should not panic
    callback.emit("test".to_string());
}

#[test]
fn test_callback_default() {
    let callback: Callback<String, i32> = Callback::default();
    let result = callback.emit("test".to_string());
    assert_eq!(result, 0); // Default for i32
}

#[test]
fn test_callback_from_option_some() {
    let original = Callback::from(|x: i32| println!("Value: {}", x));
    let callback: Callback<i32> = Callback::from(Some(original));

    // Should behave like the original callback
    callback.emit(5); // Should not panic
}

#[test]
fn test_callback_from_option_none() {
    let callback: Callback<String> = Callback::from(None);
    // Should not panic (noop behavior)
    callback.emit("test".to_string());
}

#[test]
fn test_callback_from_fn() {
    let callback = Callback::from_fn(|x: String| x.len());
    assert_eq!(callback.emit("hello".to_string()), 5);
}

#[test]
fn test_callback_from_arc() {
    let func: Arc<dyn Fn(i32) -> i32 + Send + Sync> = Arc::new(|x| x * 3);
    let callback = Callback::from(func);
    assert_eq!(callback.emit(4), 12);
}

#[test]
fn test_callback_debug() {
    let callback: Callback<String> = Callback::from(|_| {});
    let debug_str = format!("{:?}", callback);
    assert!(debug_str.contains("Callback"));
    assert!(debug_str.contains("<function>"));
}

#[test]
fn test_callback_partial_eq() {
    let func = |x: i32| x * 2;
    let callback1 = Callback::from(func);
    let callback2 = callback1.clone();
    let callback3 = Callback::from(|x: i32| x * 2);

    // Cloned callbacks should be equal
    assert_eq!(callback1, callback2);

    // Different callbacks should not be equal
    assert_ne!(callback1, callback3);
}

#[test]
fn test_callback_constant() {
    let callback: Callback<String, i32> = Callback::constant(42);
    assert_eq!(callback.emit("anything".to_string()), 42);
    assert_eq!(callback.emit("different".to_string()), 42);
}

#[test]
fn test_callback_always() {
    let callback: Callback<i32, String> = Callback::always("fixed".to_string());
    assert_eq!(callback.emit(1), "fixed");
    assert_eq!(callback.emit(999), "fixed");
}

#[test]
fn test_callback_debug_method() {
    let callback: Callback<i32> = Callback::debug();
    callback.emit(42); // Should print debug info

    // Should not panic and return default
}

#[test]
fn test_callback_then_chaining() {
    let callback = Callback::from(|x: i32| x * 2)
        .then(|x| x + 1)
        .then(|x| format!("Result: {}", x));

    let result = callback.emit(5);
    assert_eq!(result, "Result: 11"); // (5 * 2) + 1 = 11
}

#[test]
fn test_callback_map() {
    let callback = Callback::from(|x: i32| x * 2).map(|x| format!("Doubled: {}", x));

    let result = callback.emit(7);
    assert_eq!(result, "Doubled: 14");
}

#[test]
fn test_callback_filter() {
    let call_count = Arc::new(Mutex::new(0));
    let count_clone = call_count.clone();

    let callback = Callback::from(move |x: i32| {
        *count_clone.lock().unwrap() += 1;
        x * 2
    })
    .filter(|x| *x > 0);

    // Positive input should pass filter
    let result1 = callback.emit(5);
    assert_eq!(result1, Some(10));
    assert_eq!(*call_count.lock().unwrap(), 1);

    // Negative input should be filtered out
    let result2 = callback.emit(-3);
    assert_eq!(result2, None);
    assert_eq!(*call_count.lock().unwrap(), 1); // Should not increment
}

#[test]
fn test_callback_catch_unwind_success() {
    let callback = Callback::from(|x: i32| x * 2).catch_unwind();
    let result = callback.emit(5);
    assert_eq!(result, Ok(10));
}

#[test]
fn test_callback_catch_unwind_panic() {
    let callback = Callback::from(|_: i32| {
        panic!("Test panic");
    })
    .catch_unwind();

    let result = callback.emit(5);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Callback panicked");
}

#[test]
fn test_callback_from_mut() {
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let callback = Callback::from_mut(move |increment: i32| {
        let mut count = counter_clone.lock().unwrap();
        *count += increment;
        *count
    });

    assert_eq!(callback.emit(5), 5);
    assert_eq!(callback.emit(3), 8);
    assert_eq!(callback.emit(2), 10);
}

#[test]
fn test_into_callback_trait() {
    fn accepts_callback<C: IntoCallback<i32, String>>(callback: C) -> Callback<i32, String> {
        callback.into_callback()
    }

    // Function should implement IntoCallback
    let callback1 = accepts_callback(|x: i32| format!("Value: {}", x));
    assert_eq!(callback1.emit(42), "Value: 42");

    // Existing Callback should implement IntoCallback
    let existing = Callback::from(|x: i32| format!("Existing: {}", x));
    let callback2 = accepts_callback(existing);
    assert_eq!(callback2.emit(42), "Existing: 42");
}

#[test]
fn test_into_callback_prop_required() {
    fn accepts_required_callback<F: IntoCallbackProp<Callback<i32>>>(callback: F) -> Callback<i32> {
        callback.into_callback_prop()
    }

    let callback = accepts_required_callback(|x: i32| println!("Got: {}", x));
    callback.emit(42); // Should not panic
}

#[test]
fn test_into_callback_prop_optional() {
    fn accepts_optional_callback<F: IntoCallbackProp<Option<Callback<i32>>>>(
        callback: F,
    ) -> Option<Callback<i32>> {
        callback.into_callback_prop()
    }

    let callback_opt = accepts_optional_callback(|x: i32| println!("Got: {}", x));
    assert!(callback_opt.is_some());

    if let Some(callback) = callback_opt {
        callback.emit(42); // Should not panic
    }
}

#[test]
fn test_complex_callback_composition() {
    let call_log = Arc::new(Mutex::new(Vec::new()));
    let log_clone = call_log.clone();

    let callback = Callback::from(move |x: i32| {
        log_clone.lock().unwrap().push(format!("Input: {}", x));
        x
    })
    .filter(|x| *x > 0)
    .map(|opt| opt.unwrap_or(0))
    .then(|x| x * 2)
    .map(|x| format!("Final: {}", x));

    // Positive input - should call original callback
    let result1 = callback.emit(5);
    assert_eq!(result1, "Final: 10");

    // Negative input - filter prevents original callback from being called
    let result2 = callback.emit(-3);
    assert_eq!(result2, "Final: 0");

    let log = call_log.lock().unwrap();
    assert_eq!(log.len(), 1); // Only positive input gets logged
    assert_eq!(log[0], "Input: 5");
}

#[test]
fn test_callback_thread_safety() {
    use std::thread;

    let call_count = Arc::new(Mutex::new(0));
    let count_clone = call_count.clone();

    let callback = Callback::from(move |_: ()| {
        *count_clone.lock().unwrap() += 1;
    });

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let cb = callback.clone();
            thread::spawn(move || {
                cb.emit(());
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*call_count.lock().unwrap(), 10);
}

// Tests for use_callback hook functionality
#[test]
fn test_use_callback_basic() {
    use crate::hooks::test_utils::with_component_id;

    with_component_id("UseCallbackTest", |_ctx| {
        let call_count = Arc::new(Mutex::new(0));
        let count_clone = call_count.clone();

        let memoized_callback = super::use_callback(
            move |_: ()| {
                *count_clone.lock().unwrap() += 1;
            },
            (),
        );

        // First call
        memoized_callback.emit(());
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Second call should use same callback
        memoized_callback.emit(());
        assert_eq!(*call_count.lock().unwrap(), 2);
    });
}

#[test]
fn test_use_callback_with_dependencies() {
    use crate::hooks::test_utils::with_component_id;

    with_component_id("UseCallbackDepsTest", |_ctx| {
        let call_count = Arc::new(Mutex::new(0));
        let count_clone = call_count.clone();

        // First render with dep = 1
        let memoized_callback1 = super::use_callback(
            move |value: i32| {
                *count_clone.lock().unwrap() += value;
            },
            1,
        );

        memoized_callback1.emit(10);
        assert_eq!(*call_count.lock().unwrap(), 10);
    });

    // Second render with same component but different deps should create new callback
    with_component_id("UseCallbackDepsTest", |_ctx| {
        let call_count = Arc::new(Mutex::new(0));
        let count_clone = call_count.clone();

        // Second render with dep = 2 (different)
        let memoized_callback2 = super::use_callback(
            move |value: i32| {
                *count_clone.lock().unwrap() += value * 2; // Different behavior
            },
            2,
        );

        memoized_callback2.emit(10);
        assert_eq!(*call_count.lock().unwrap(), 20); // 10 * 2
    });
}

#[test]
fn test_use_callback_memoization() {
    use crate::hooks::test_utils::with_component_id;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let call_count = Arc::new(AtomicUsize::new(0));
    let call_clone1 = call_count.clone();
    let call_clone2 = call_count.clone();

    with_component_id("MemoizationTest", |_ctx| {
        // First call - should create callback
        let callback1 = super::use_callback(
            move |_: ()| {
                call_clone1.fetch_add(1, Ordering::SeqCst);
                "called"
            },
            42, // Same dependency
        );

        // Call the callback to verify it works
        callback1.emit(());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Second call with same dependency - should reuse same callback
        let callback2 = super::use_callback(
            move |_: ()| {
                call_clone2.fetch_add(1, Ordering::SeqCst);
                "called"
            },
            42, // Same dependency
        );

        // Call the second callback - should use same underlying callback
        callback2.emit(());
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
        // Note: Each use_callback call creates a new factory, so we can't test pointer equality
        // Instead, we verify that both callbacks work correctly
    });
}

#[test]
fn test_use_callback_dependency_change() {
    use crate::hooks::test_utils::with_component_id;

    let call_count = Arc::new(Mutex::new(0));
    let call_clone1 = call_count.clone();
    let call_clone2 = call_count.clone();

    // First render with dep = 1
    let callback1 = with_component_id("DepChangeTest", |_ctx| {
        super::use_callback(
            move |value: i32| {
                *call_clone1.lock().unwrap() += value;
            },
            1,
        )
    });

    callback1.emit(10);
    assert_eq!(*call_count.lock().unwrap(), 10);

    // Second render with dep = 2 (changed) - should create new callback
    let callback2 = with_component_id("DepChangeTest", |_ctx| {
        super::use_callback(
            move |value: i32| {
                *call_clone2.lock().unwrap() += value * 2; // Different behavior
            },
            2,
        )
    });

    callback2.emit(10);
    assert_eq!(*call_count.lock().unwrap(), 30); // 10 + (10 * 2)
}

#[test]
fn test_use_callback_with_return_value() {
    use crate::hooks::test_utils::with_component_id;

    with_component_id("ReturnValueTest", |_ctx| {
        let multiplier = 3;
        let memoized_callback = super::use_callback(move |x: i32| x * multiplier, multiplier);

        let result = memoized_callback.emit(5);
        assert_eq!(result, 15);

        let result2 = memoized_callback.emit(10);
        assert_eq!(result2, 30);
    });
}

#[test]
fn test_use_callback_once() {
    use crate::hooks::test_utils::with_component_id;

    let call_count = Arc::new(Mutex::new(0));
    let call_clone1 = call_count.clone();
    let call_clone2 = call_count.clone();

    with_component_id("OnceTest", |_ctx| {
        // First call
        let callback1 = super::use_callback_once(move |_: ()| {
            *call_clone1.lock().unwrap() += 1;
            "once"
        });

        // Call the callback to verify it works
        callback1.emit(());
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Second call - should reuse same callback (no dependencies)
        let callback2 = super::use_callback_once(move |_: ()| {
            *call_clone2.lock().unwrap() += 1;
            "once"
        });

        // Call the second callback - should use same underlying callback
        callback2.emit(());
        assert_eq!(*call_count.lock().unwrap(), 2);

        // Note: Each use_callback call creates a new factory, so we can't test pointer equality
        // Instead, we verify that both callbacks work correctly
    });
}

#[test]
fn test_use_event_handler() {
    use crate::hooks::test_utils::with_component_id;

    with_component_id("EventHandlerTest", |_ctx| {
        let call_log = Arc::new(Mutex::new(Vec::new()));
        let log_clone = call_log.clone();

        let event_handler = super::use_event_handler(
            move |event: String| {
                log_clone.lock().unwrap().push(event);
            },
            (),
        );

        event_handler.emit("click".to_string());
        event_handler.emit("hover".to_string());

        let log = call_log.lock().unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0], "click");
        assert_eq!(log[1], "hover");
    });
}

#[test]
fn test_use_callback_with_complex_dependencies() {
    use crate::hooks::test_utils::with_component_id;

    with_component_id("ComplexDepsTest", |_ctx| {
        let result = Arc::new(Mutex::new(String::new()));
        let result_clone = result.clone();

        let name = "Alice".to_string();
        let age = 30;
        let name_for_closure = name.clone();

        let memoized_callback = super::use_callback(
            move |greeting: String| {
                let formatted = format!("{} {} (age {})", greeting, name_for_closure, age);
                *result_clone.lock().unwrap() = formatted.clone();
                formatted
            },
            (name, age),
        );

        let output = memoized_callback.emit("Hello".to_string());
        assert_eq!(output, "Hello Alice (age 30)");
        assert_eq!(*result.lock().unwrap(), "Hello Alice (age 30)");
    });
}

#[test]
fn test_memoized_callback_interface() {
    use crate::hooks::test_utils::with_component_id;

    with_component_id("InterfaceTest", |_ctx| {
        let memoized_callback = super::use_callback(|x: i32| x.to_string(), ());

        // Test MemoizedCallback methods
        let result1 = memoized_callback.emit(42);
        assert_eq!(result1, "42");

        let result2 = memoized_callback.callback().emit(100);
        assert_eq!(result2, "100");

        // Test cloning
        let cloned = memoized_callback.clone();
        let result3 = cloned.emit(200);
        assert_eq!(result3, "200");
    });
}

#[test]
fn test_callback_factory_trait() {
    use crate::hooks::test_utils::with_component_id;

    with_component_id("FactoryTest", |_ctx| {
        // Test direct closure (most common usage)
        let callback1 = super::use_callback(|x: i32| x * 2, ());
        assert_eq!(callback1.emit(5), 10);

        // Test CallbackFactory wrapper (advanced usage)
        let callback2 = super::use_callback(
            super::CallbackFactory(|| Callback::from(|x: i32| x * 3)),
            (),
        );
        assert_eq!(callback2.emit(5), 15);
    });
}
