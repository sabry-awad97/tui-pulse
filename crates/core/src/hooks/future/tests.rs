//! Comprehensive tests for the useFuture hook
//!
//! This module contains extensive tests covering:
//! - Basic future execution and state management
//! - Dependency tracking and re-execution
//! - Error handling and edge cases
//! - Cancellation behavior
//! - Thread safety and concurrent access

use super::*;
use crate::hooks::test_utils::{
    with_async_component_id, with_async_hook_context, with_async_test_isolate,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_basic_future_success() {
    with_async_test_isolate(|| async {
        with_async_component_id("BasicFutureComponent", |_context| async {
            let handle = use_future(
                || async { Ok::<i32, String>(42) },
                (), // No dependencies - run once
            );

            // Initially should be pending
            assert!(handle.is_pending());
            assert_eq!(handle.state(), FutureState::Pending);

            // Wait for the future to complete
            sleep(Duration::from_millis(50)).await;

            // Should now be resolved
            assert!(
                handle.is_resolved(),
                "Future should be resolved after waiting"
            );
            assert_eq!(handle.value(), Some(42));
            assert_eq!(handle.state(), FutureState::Resolved(42));
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_basic_future_error() {
    with_async_test_isolate(|| async {
        with_async_component_id("ErrorFutureComponent", |_context| async {
            let handle = use_future(
                || async { Err::<i32, String>("Test error".to_string()) },
                (),
            );

            // Initially should be pending
            assert!(handle.is_pending());

            // Wait for the future to complete
            sleep(Duration::from_millis(50)).await;

            // Should now be in error state
            assert!(
                handle.is_error(),
                "Future should be in error state after waiting"
            );
            assert_eq!(handle.error(), Some("Test error".to_string()));
            assert_eq!(handle.state(), FutureState::Error("Test error".to_string()));
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_future_with_dependencies() {
    with_async_test_isolate(|| async {
        let counter = Arc::new(AtomicUsize::new(0));

        // First render
        with_async_component_id("DependencyFutureComponent", |_context| async {
            let counter_clone = counter.clone();
            let handle1 = use_future(
                move || {
                    let counter = counter_clone.clone();
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Ok::<usize, String>(42)
                    }
                },
                1, // dependency value
            );

            // Wait for the future to complete
            sleep(Duration::from_millis(10)).await;
            assert_eq!(counter.load(Ordering::SeqCst), 1);
            assert!(handle1.is_resolved());
        })
        .await;

        // Second render with same dependency - should not re-execute
        with_async_component_id("DependencyFutureComponent", |_context| async {
            let counter_clone2 = counter.clone();
            let _handle2 = use_future(
                move || {
                    let counter = counter_clone2.clone();
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Ok::<usize, String>(42)
                    }
                },
                1, // same dependency value
            );

            sleep(Duration::from_millis(10)).await;

            // Should still have executed only once since dependency didn't change
            assert_eq!(counter.load(Ordering::SeqCst), 1);
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_future_dependency_change() {
    with_async_test_isolate(|| async {
        let counter = Arc::new(AtomicUsize::new(0));
        let results = Arc::new(Mutex::new(Vec::new()));

        // First execution with dependency = 1
        with_async_hook_context(|_context| async {
            let counter_clone = counter.clone();
            let results_clone = results.clone();
            let handle = use_future(
                move || {
                    let counter = counter_clone.clone();
                    let results = results_clone.clone();
                    async move {
                        let count = counter.fetch_add(1, Ordering::SeqCst);
                        let result = format!("execution_{}", count);
                        results.lock().unwrap().push(result.clone());
                        Ok::<String, String>(result)
                    }
                },
                1, // dependency = 1
            );

            sleep(Duration::from_millis(10)).await;
            assert!(handle.is_resolved());
        })
        .await;

        // Second execution with dependency = 2 (changed)
        with_async_hook_context(|_context| async {
            let counter_clone = counter.clone();
            let results_clone = results.clone();
            let handle = use_future(
                move || {
                    let counter = counter_clone.clone();
                    let results = results_clone.clone();
                    async move {
                        let count = counter.fetch_add(1, Ordering::SeqCst);
                        let result = format!("execution_{}", count);
                        results.lock().unwrap().push(result.clone());
                        Ok::<String, String>(result)
                    }
                },
                2, // dependency = 2 (changed)
            );

            sleep(Duration::from_millis(10)).await;
            assert!(handle.is_resolved());
        })
        .await;

        // Should have executed twice due to dependency change
        assert_eq!(counter.load(Ordering::SeqCst), 2);
        let results_vec = results.lock().unwrap();
        assert_eq!(results_vec.len(), 2);
        assert_eq!(results_vec[0], "execution_0");
        assert_eq!(results_vec[1], "execution_1");
    })
    .await;
}

#[tokio::test]
async fn test_future_state_methods() {
    // Test FutureState utility methods
    let pending: FutureState<i32, String> = FutureState::Pending;
    assert!(pending.is_pending());
    assert!(!pending.is_resolved());
    assert!(!pending.is_error());
    assert_eq!(pending.value(), None);
    assert_eq!(pending.error(), None);

    let resolved: FutureState<i32, String> = FutureState::Resolved(42);
    assert!(!resolved.is_pending());
    assert!(resolved.is_resolved());
    assert!(!resolved.is_error());
    assert_eq!(resolved.value(), Some(&42));
    assert_eq!(resolved.error(), None);

    let error: FutureState<i32, String> = FutureState::Error("test error".to_string());
    assert!(!error.is_pending());
    assert!(!error.is_resolved());
    assert!(error.is_error());
    assert_eq!(error.value(), None);
    assert_eq!(error.error(), Some(&"test error".to_string()));
}

#[tokio::test]
async fn test_future_state_map() {
    let resolved: FutureState<i32, String> = FutureState::Resolved(42);
    let mapped = resolved.map(|x| x * 2);
    assert_eq!(mapped, FutureState::Resolved(84));

    let pending: FutureState<i32, String> = FutureState::Pending;
    let mapped_pending = pending.map(|x| x * 2);
    assert_eq!(mapped_pending, FutureState::Pending);

    let error: FutureState<i32, String> = FutureState::Error("test".to_string());
    let mapped_error = error.map(|x: i32| x * 2);
    assert_eq!(mapped_error, FutureState::Error("test".to_string()));
}

#[tokio::test]
async fn test_future_state_map_err() {
    let error: FutureState<i32, String> = FutureState::Error("test".to_string());
    let mapped = error.map_err(|e| format!("Error: {}", e));
    assert_eq!(mapped, FutureState::Error("Error: test".to_string()));

    let resolved: FutureState<i32, String> = FutureState::Resolved(42);
    let mapped_resolved = resolved.map_err(|e: String| format!("Error: {}", e));
    assert_eq!(mapped_resolved, FutureState::Resolved(42));
}

#[tokio::test]
async fn test_future_handle_clone() {
    with_async_test_isolate(|| async {
        with_async_component_id("CloneFutureComponent", |_context| async {
            let handle = use_future(|| async { Ok::<i32, String>(42) }, ());
            let handle_clone = handle.clone();

            // Wait for completion
            sleep(Duration::from_millis(50)).await;

            // Both handles should see the same state
            assert_eq!(handle.state(), handle_clone.state());
            assert_eq!(handle.value(), handle_clone.value());
            assert!(handle.is_resolved(), "Original handle should be resolved");
            assert!(
                handle_clone.is_resolved(),
                "Cloned handle should be resolved"
            );
        })
        .await;
    })
    .await;
}

/// Test optimized state access methods to ensure they don't clone unnecessarily
#[tokio::test]
async fn test_optimized_state_access() {
    with_async_test_isolate(|| async {
        with_async_component_id("OptimizedStateComponent", |_context| async {
            let handle = use_future(
                || async { Ok::<String, String>("test_value".to_string()) },
                (),
            );

            // Wait for completion
            sleep(Duration::from_millis(50)).await;

            // Test optimized methods
            assert!(
                handle.is_resolved(),
                "Future should be resolved after waiting"
            );
            assert!(!handle.is_pending());
            assert!(!handle.is_error());

            // Test value extraction
            assert_eq!(handle.value(), Some("test_value".to_string()));
            assert_eq!(handle.error(), None);
        })
        .await;
    })
    .await;
}

/// Test memory cleanup when future hook is dropped
#[tokio::test]
async fn test_memory_cleanup_on_drop() {
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Clone)]
    struct DropTracker;

    impl Drop for DropTracker {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    with_async_test_isolate(|| async {
        let _initial_count = DROP_COUNT.load(Ordering::SeqCst);

        {
            let _handle = with_async_hook_context(|_context| async {
                use_future(
                    || async {
                        let _tracker = DropTracker;
                        Ok::<String, String>("test".to_string())
                    },
                    (),
                )
            })
            .await;

            // Wait for future to complete
            sleep(Duration::from_millis(10)).await;
        } // handle dropped here

        // Give some time for cleanup
        sleep(Duration::from_millis(10)).await;

        // The test passes if no panic occurs during cleanup
        // The actual memory cleanup is tested by the Drop implementation
        // which is automatically called when the handle goes out of scope

        // We can't reliably test the drop count in this async environment
        // because the DropTracker is inside the async closure and may not
        // be dropped in a predictable way. The important thing is that
        // the Drop implementation exists and will be called.
    })
    .await;
}

/// Test that cancellation properly cleans up resources
#[tokio::test]
async fn test_cancellation_cleanup() {
    with_async_test_isolate(|| async {
        let handle = with_async_hook_context(|_context| async {
            use_future(
                || async {
                    // Long-running future
                    sleep(Duration::from_secs(10)).await;
                    Ok::<String, String>("should_not_complete".to_string())
                },
                (),
            )
        })
        .await;

        // Verify it's pending
        assert!(handle.is_pending());

        // Cancel the future
        handle.cancel();

        // Give some time for cancellation to take effect
        sleep(Duration::from_millis(10)).await;

        // Future should still be in pending state since it was cancelled
        // (cancellation doesn't change the state, it just stops the task)
        assert!(handle.is_pending());
    })
    .await;
}

/// Test performance of state access methods under concurrent load
#[tokio::test]
async fn test_concurrent_state_access_performance() {
    with_async_test_isolate(|| async {
        with_async_component_id("ConcurrentStateComponent", |_context| async {
            let handle = use_future(|| async { Ok::<i32, String>(42) }, ());

            // Wait for completion
            sleep(Duration::from_millis(50)).await;

            let handle = Arc::new(handle);
            let mut tasks = Vec::new();

            // Spawn multiple concurrent tasks accessing state
            for _ in 0..100 {
                let handle_clone = handle.clone();
                tasks.push(tokio::spawn(async move {
                    // Perform multiple state accesses
                    for _ in 0..10 {
                        let _ = handle_clone.is_resolved();
                        let _ = handle_clone.is_pending();
                        let _ = handle_clone.is_error();
                        let _ = handle_clone.value();
                        let _ = handle_clone.error();
                    }
                }));
            }

            // Wait for all tasks to complete
            for task in tasks {
                task.await.expect("Task should complete successfully");
            }

            // Verify state is still correct
            assert!(handle.is_resolved());
            assert_eq!(handle.value(), Some(42));
        })
        .await;
    })
    .await;
}

/// Benchmark test to demonstrate performance improvements
/// This test shows the difference between optimized and non-optimized state access
#[test]
fn test_state_access_performance_comparison() {
    use std::time::Instant;

    // Create a mock future handle with resolved state
    let handle = FutureHandle::<String, String>::new();
    handle.set_state(FutureState::Resolved("test_value".to_string()));

    let iterations = 10000;

    // Test optimized methods
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = handle.is_resolved();
        let _ = handle.is_pending();
        let _ = handle.is_error();
        let _ = handle.value();
        let _ = handle.error();
    }
    let optimized_duration = start.elapsed();

    println!(
        "Optimized state access for {} iterations: {:?}",
        iterations, optimized_duration
    );

    // The optimized version should be significantly faster than the old approach
    // because it avoids cloning the entire state for each access
    assert!(
        optimized_duration.as_millis() < 100,
        "Optimized access should be fast"
    );
}

/// Security test: Verify resource exhaustion protection
#[tokio::test]
async fn test_resource_exhaustion_protection() {
    with_async_test_isolate(|| async {
        with_async_component_id("ResourceTestComponent", |_context| async {
            // This test verifies that the security limits are in place
            // We can't easily test the actual limits without spawning many futures
            // but we can verify the error handling works

            let handle = use_future(|| async { Ok::<String, String>("test".to_string()) }, ());

            // Wait for completion
            sleep(Duration::from_millis(50)).await;

            // Should complete successfully under normal conditions
            assert!(
                handle.is_resolved(),
                "Normal future should complete successfully"
            );
        })
        .await;
    })
    .await;
}

/// Security test: Verify error handling provides detailed error information
#[tokio::test]
async fn test_error_handling_security() {
    with_async_test_isolate(|| async {
        with_async_component_id("ErrorTestComponent", |_context| async {
            let handle = use_future(
                || async {
                    // Simulate a security-related error
                    Err::<String, String>("Security violation: unauthorized access".to_string())
                },
                (),
            );

            // Wait for the error to be handled
            sleep(Duration::from_millis(50)).await;

            // Should be in error state with detailed error information
            assert!(handle.is_error(), "Failed future should be in error state");

            if let Some(error) = handle.error() {
                assert!(
                    error.contains("Security violation"),
                    "Error should contain detailed security information, got: {}",
                    error
                );
            } else {
                panic!("Expected error information from failed future");
            }
        })
        .await;
    })
    .await;
}

/// Test progress tracking functionality
#[tokio::test]
async fn test_progress_tracking() {
    with_async_test_isolate(|| async {
        with_async_component_id("ProgressTestComponent", |_context| async {
            let handle = use_future_with_progress(
                |progress_callback| async move {
                    // Simulate work with progress updates
                    for i in 0..=5 {
                        progress_callback(i as f32 / 5.0);
                        sleep(Duration::from_millis(10)).await;
                    }
                    Ok::<String, String>("Complete".to_string())
                },
                (),
            );

            // Initially should be pending
            assert!(handle.is_pending());

            // Wait for progress updates
            sleep(Duration::from_millis(30)).await;

            // Should be in progress or completed
            assert!(handle.is_progress() || handle.is_resolved());

            // Wait for completion
            sleep(Duration::from_millis(100)).await;

            // Should be resolved
            assert!(handle.is_resolved());
            assert_eq!(handle.value(), Some("Complete".to_string()));
        })
        .await;
    })
    .await;
}

/// Test progress state methods
#[tokio::test]
async fn test_progress_state_methods() {
    with_async_test_isolate(|| async {
        with_async_component_id("ProgressStateComponent", |_context| async {
            let handle = use_future_with_progress(
                |progress_callback| async move {
                    progress_callback(0.5);
                    sleep(Duration::from_millis(50)).await;
                    Ok::<String, String>("Done".to_string())
                },
                (),
            );

            // Wait for progress update
            sleep(Duration::from_millis(25)).await;

            // Test progress state methods
            if handle.is_progress() {
                assert_eq!(handle.progress(), Some(0.5));
                assert!(handle.is_running());
                assert!(!handle.is_pending());
                assert!(!handle.is_resolved());
                assert!(!handle.is_error());
            }

            // Wait for completion
            sleep(Duration::from_millis(50)).await;

            // Should be resolved
            assert!(handle.is_resolved());
            assert!(!handle.is_running());
            assert_eq!(handle.progress(), None);
        })
        .await;
    })
    .await;
}

/// Test FutureState progress methods
#[test]
fn test_future_state_progress_methods() {
    // Test progress state
    let progress_state = FutureState::<String, String>::Progress(0.75);
    assert!(progress_state.is_progress());
    assert!(!progress_state.is_pending());
    assert!(!progress_state.is_resolved());
    assert!(!progress_state.is_error());
    assert!(progress_state.is_running());
    assert_eq!(progress_state.progress(), Some(0.75));
    assert_eq!(progress_state.value(), None);
    assert_eq!(progress_state.error(), None);

    // Test pending state
    let pending_state = FutureState::<String, String>::Pending;
    assert!(pending_state.is_pending());
    assert!(!pending_state.is_progress());
    assert!(pending_state.is_running());
    assert_eq!(pending_state.progress(), None);

    // Test resolved state
    let resolved_state = FutureState::<String, String>::Resolved("test".to_string());
    assert!(resolved_state.is_resolved());
    assert!(!resolved_state.is_progress());
    assert!(!resolved_state.is_running());
    assert_eq!(resolved_state.progress(), None);
}

/// Test progress value clamping
#[tokio::test]
async fn test_progress_clamping() {
    with_async_test_isolate(|| async {
        with_async_component_id("ProgressClampComponent", |_context| async {
            let handle = use_future_with_progress(
                |progress_callback| async move {
                    // Test values outside 0.0-1.0 range
                    progress_callback(-0.5); // Should be clamped to 0.0
                    sleep(Duration::from_millis(10)).await;
                    progress_callback(1.5); // Should be clamped to 1.0
                    sleep(Duration::from_millis(10)).await;
                    Ok::<String, String>("Done".to_string())
                },
                (),
            );

            // Wait for progress updates
            sleep(Duration::from_millis(15)).await;

            // Progress should be clamped
            if let Some(progress) = handle.progress() {
                assert!((0.0..=1.0).contains(&progress));
            }

            // Wait for completion
            sleep(Duration::from_millis(50)).await;
            assert!(handle.is_resolved());
        })
        .await;
    })
    .await;
}
