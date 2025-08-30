use super::*;

use crate::hooks::{signal::middleware::SignalMiddleware, test_utils::with_component_id};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

// Test mutex to ensure global signal tests run sequentially
// This prevents state pollution between tests since global signals are truly global
static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_global_signal_basic_functionality() {
    let _guard = TEST_MUTEX.lock();
    static TEST_SIGNAL: GlobalSignal<i32> = Signal::global(|| 42);

    // Reset to ensure clean state
    TEST_SIGNAL.reset();

    // Test initial value
    assert_eq!(TEST_SIGNAL.get(), 42);

    // Test setting value
    TEST_SIGNAL.set(100);
    assert_eq!(TEST_SIGNAL.get(), 100);

    // Test updating value
    TEST_SIGNAL.update(|x| x * 2);
    assert_eq!(TEST_SIGNAL.get(), 200);

    // Clean up after test
    TEST_SIGNAL.reset();
}

#[test]
fn test_global_signal_version_tracking() {
    let _guard = TEST_MUTEX.lock();
    static VERSION_SIGNAL: GlobalSignal<String> = Signal::global(|| "initial".to_string());

    let initial_version = VERSION_SIGNAL.version();

    // Version should increment on set
    VERSION_SIGNAL.set("updated".to_string());
    assert_eq!(VERSION_SIGNAL.version(), initial_version + 1);

    // Version should increment on update
    VERSION_SIGNAL.update(|s| format!("{}_modified", s));
    assert_eq!(VERSION_SIGNAL.version(), initial_version + 2);

    assert_eq!(VERSION_SIGNAL.get(), "updated_modified");
}

#[test]
fn test_global_signal_unique_ids() {
    let _guard = TEST_MUTEX.lock();
    static SIGNAL_A: GlobalSignal<i32> = Signal::global(|| 1);
    static SIGNAL_B: GlobalSignal<i32> = Signal::global(|| 2);

    // Reset to ensure clean state
    SIGNAL_A.reset();
    SIGNAL_B.reset();

    let id_a = SIGNAL_A.id();
    let id_b = SIGNAL_B.id();

    // Each signal should have a unique ID
    assert_ne!(id_a, id_b);

    // IDs should be consistent across calls
    assert_eq!(SIGNAL_A.id(), id_a);
    assert_eq!(SIGNAL_B.id(), id_b);

    // Clean up after test
    SIGNAL_A.reset();
    SIGNAL_B.reset();
}

#[test]
fn test_global_signal_thread_safety() {
    let _guard = TEST_MUTEX.lock();
    static THREAD_SIGNAL: GlobalSignal<i32> = Signal::global(|| 0);

    // Reset to ensure clean state
    THREAD_SIGNAL.reset();

    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..100 {
                    THREAD_SIGNAL.update(|counter| counter + 1);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Should have incremented 1000 times total
    assert_eq!(THREAD_SIGNAL.get(), 1000);

    // Clean up after test
    THREAD_SIGNAL.reset();
}

#[test]
fn test_global_signal_multiple_handles() {
    let _guard = TEST_MUTEX.lock();
    static MULTI_SIGNAL: GlobalSignal<Vec<String>> = Signal::global(|| vec!["initial".to_string()]);

    let handle1 = MULTI_SIGNAL.handle();
    let handle2 = MULTI_SIGNAL.handle();

    // Both handles should see the same initial value
    assert_eq!(handle1.get(), vec!["initial"]);
    assert_eq!(handle2.get(), vec!["initial"]);

    // Changes through one handle should be visible through the other
    handle1.update(|mut v| {
        v.push("from_handle1".to_string());
        v
    });

    assert_eq!(handle2.get(), vec!["initial", "from_handle1"]);

    handle2.set(vec!["reset".to_string()]);
    assert_eq!(handle1.get(), vec!["reset"]);
}

#[test]
fn test_use_global_signal_hook() {
    let _guard = TEST_MUTEX.lock();
    static HOOK_SIGNAL: GlobalSignal<f64> = Signal::global(|| std::f64::consts::PI);

    with_component_id("GlobalSignalHookComponent", |_ctx| {
        let handle = use_global_signal(&HOOK_SIGNAL);

        assert_eq!(handle.get(), std::f64::consts::PI);

        handle.set(std::f64::consts::E);
        assert_eq!(handle.get(), std::f64::consts::E);

        // Direct access should also see the change
        assert_eq!(HOOK_SIGNAL.get(), std::f64::consts::E);
    });
}

#[test]
fn test_global_signal_different_types() {
    let _guard = TEST_MUTEX.lock();
    static INT_SIGNAL: GlobalSignal<i32> = Signal::global(|| 42);
    static STRING_SIGNAL: GlobalSignal<String> = Signal::global(|| "hello".to_string());
    static BOOL_SIGNAL: GlobalSignal<bool> = Signal::global(|| true);

    // Each signal should maintain its own state
    assert_eq!(INT_SIGNAL.get(), 42);
    assert_eq!(STRING_SIGNAL.get(), "hello");
    assert!(BOOL_SIGNAL.get());

    // Modifying one shouldn't affect others
    INT_SIGNAL.set(100);
    STRING_SIGNAL.set("world".to_string());
    BOOL_SIGNAL.set(false);

    assert_eq!(INT_SIGNAL.get(), 100);
    assert_eq!(STRING_SIGNAL.get(), "world");
    assert!(!BOOL_SIGNAL.get());
}

#[test]
fn test_global_signal_complex_types() {
    let _guard = TEST_MUTEX.lock();
    #[derive(Clone, Debug, PartialEq)]
    struct User {
        id: u32,
        name: String,
        active: bool,
    }

    static USER_SIGNAL: GlobalSignal<Option<User>> = Signal::global(|| None);

    assert_eq!(USER_SIGNAL.get(), None);

    let user = User {
        id: 1,
        name: "Alice".to_string(),
        active: true,
    };

    USER_SIGNAL.set(Some(user.clone()));
    assert_eq!(USER_SIGNAL.get(), Some(user.clone()));

    USER_SIGNAL.update(|mut user_opt| {
        if let Some(ref mut user) = user_opt {
            user.active = false;
        }
        user_opt
    });

    let expected = User {
        id: 1,
        name: "Alice".to_string(),
        active: false,
    };
    assert_eq!(USER_SIGNAL.get(), Some(expected));
}

#[test]
fn test_signal_handle_consistency() {
    let _guard = TEST_MUTEX.lock();
    static CONSISTENCY_SIGNAL: GlobalSignal<i32> = Signal::global(|| 0);

    let handle1 = CONSISTENCY_SIGNAL.handle();
    let handle2 = CONSISTENCY_SIGNAL.handle();

    // Both handles should have the same ID (pointing to same signal)
    assert_eq!(handle1.id(), handle2.id());
    assert_eq!(handle1.id(), CONSISTENCY_SIGNAL.id());

    // Version should be consistent across handles
    let initial_version = handle1.version();
    assert_eq!(handle2.version(), initial_version);

    handle1.set(42);
    assert_eq!(handle1.version(), initial_version + 1);
    assert_eq!(handle2.version(), initial_version + 1);
}

#[test]
fn test_global_signal_lazy_initialization() {
    let _guard = TEST_MUTEX.lock();
    use std::sync::atomic::{AtomicBool, Ordering};

    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    static LAZY_SIGNAL: GlobalSignal<String> = Signal::global(|| {
        INITIALIZED.store(true, Ordering::SeqCst);
        "initialized".to_string()
    });

    // Signal should not be initialized yet
    assert!(!INITIALIZED.load(Ordering::SeqCst));

    // First access should trigger initialization
    let value = LAZY_SIGNAL.get();
    assert!(INITIALIZED.load(Ordering::SeqCst));
    assert_eq!(value, "initialized");

    // Subsequent accesses should not re-initialize
    INITIALIZED.store(false, Ordering::SeqCst);
    let value2 = LAZY_SIGNAL.get();
    assert!(!INITIALIZED.load(Ordering::SeqCst)); // Should still be false
    assert_eq!(value2, "initialized");
}

// ============================================================================
// Memory Management & Testing Isolation Tests
// ============================================================================

#[test]
fn test_signal_reset() {
    let _guard = TEST_MUTEX.lock();
    static RESET_SIGNAL: GlobalSignal<i32> = Signal::global(|| 100);

    // Modify the signal
    RESET_SIGNAL.set(200);
    assert_eq!(RESET_SIGNAL.get(), 200);

    // Reset should restore initial value
    RESET_SIGNAL.reset();
    assert_eq!(RESET_SIGNAL.get(), 100);
}

#[test]
fn test_signal_reset_isolation() {
    let _guard = TEST_MUTEX.lock();
    static RESET_SIGNAL_A: GlobalSignal<i32> = Signal::global(|| 10);
    static RESET_SIGNAL_B: GlobalSignal<String> = Signal::global(|| "initial".to_string());

    // Modify both signals
    RESET_SIGNAL_A.set(20);
    RESET_SIGNAL_B.set("modified".to_string());

    // Reset only A
    RESET_SIGNAL_A.reset();

    // A should be reset, B should remain modified
    assert_eq!(RESET_SIGNAL_A.get(), 10);
    assert_eq!(RESET_SIGNAL_B.get(), "modified");

    // Clean up for other tests
    RESET_SIGNAL_B.reset();
}

#[test]
fn test_signal_reset_all() {
    let _guard = TEST_MUTEX.lock();
    static RESET_ALL_SIGNAL_C: GlobalSignal<i32> = Signal::global(|| 30);
    static RESET_ALL_SIGNAL_D: GlobalSignal<f64> = Signal::global(|| std::f64::consts::PI);

    // Reset to ensure clean state
    RESET_ALL_SIGNAL_C.reset();
    RESET_ALL_SIGNAL_D.reset();

    // Modify both signals
    RESET_ALL_SIGNAL_C.set(40);
    RESET_ALL_SIGNAL_D.set(std::f64::consts::E);

    // Reset all signals
    GlobalSignal::<()>::reset_all();

    // Both should be reset to initial values on next access
    assert_eq!(RESET_ALL_SIGNAL_C.get(), 30);
    assert_eq!(RESET_ALL_SIGNAL_D.get(), std::f64::consts::PI);
}

#[test]
fn test_force_cleanup() {
    let _guard = TEST_MUTEX.lock();
    static FORCE_CLEANUP_SIGNAL: GlobalSignal<i32> = Signal::global(|| 70);

    // Reset to ensure clean state
    FORCE_CLEANUP_SIGNAL.reset();

    // Use the signal
    FORCE_CLEANUP_SIGNAL.set(80);
    assert_eq!(FORCE_CLEANUP_SIGNAL.get(), 80);

    // Force cleanup removes the signal
    GlobalSignal::<i32>::force_cleanup();

    // Next access should re-initialize with initial value
    assert_eq!(FORCE_CLEANUP_SIGNAL.get(), 70);
}

#[test]
fn test_signal_reset_with_complex_types() {
    let _guard = TEST_MUTEX.lock();
    #[derive(Clone, Debug, PartialEq)]
    struct Config {
        debug: bool,
        max_connections: u32,
        timeout_ms: u64,
    }

    static CONFIG_SIGNAL: GlobalSignal<Config> = Signal::global(|| Config {
        debug: false,
        max_connections: 100,
        timeout_ms: 5000,
    });

    // Modify the config
    CONFIG_SIGNAL.update(|mut config| {
        config.debug = true;
        config.max_connections = 200;
        config.timeout_ms = 10000;
        config
    });

    let modified_config = CONFIG_SIGNAL.get();
    assert!(modified_config.debug);
    assert_eq!(modified_config.max_connections, 200);
    assert_eq!(modified_config.timeout_ms, 10000);

    // Reset should restore initial values
    CONFIG_SIGNAL.reset();
    let reset_config = CONFIG_SIGNAL.get();
    assert!(!reset_config.debug);
    assert_eq!(reset_config.max_connections, 100);
    assert_eq!(reset_config.timeout_ms, 5000);
}

#[test]
fn test_memory_management_with_handles() {
    let _guard = TEST_MUTEX.lock();
    static MEMORY_SIGNAL: GlobalSignal<Vec<String>> =
        Signal::global(|| vec!["initial".to_string()]);

    // Create multiple handles
    let handle1 = MEMORY_SIGNAL.handle();
    let handle2 = MEMORY_SIGNAL.handle();

    // Modify through handles
    handle1.update(|mut v| {
        v.push("handle1".to_string());
        v
    });

    handle2.update(|mut v| {
        v.push("handle2".to_string());
        v
    });

    // Verify state
    let current = MEMORY_SIGNAL.get();
    assert_eq!(current.len(), 3);
    assert!(current.contains(&"initial".to_string()));
    assert!(current.contains(&"handle1".to_string()));
    assert!(current.contains(&"handle2".to_string()));

    // Reset should work even with active handles
    MEMORY_SIGNAL.reset();
    assert_eq!(MEMORY_SIGNAL.get(), vec!["initial".to_string()]);

    // Handles should see the reset value
    assert_eq!(handle1.get(), vec!["initial".to_string()]);
    assert_eq!(handle2.get(), vec!["initial".to_string()]);
}

#[test]
fn test_test_isolation_pattern() {
    let _guard = TEST_MUTEX.lock();
    static ISOLATION_SIGNAL: GlobalSignal<i32> = Signal::global(|| 0);

    // Simulate test 1
    {
        ISOLATION_SIGNAL.reset(); // Reset before test
        assert_eq!(ISOLATION_SIGNAL.get(), 0);

        ISOLATION_SIGNAL.set(42);
        assert_eq!(ISOLATION_SIGNAL.get(), 42);

        ISOLATION_SIGNAL.reset(); // Reset after test
    }

    // Simulate test 2 - should start fresh
    {
        ISOLATION_SIGNAL.reset(); // Reset before test
        assert_eq!(ISOLATION_SIGNAL.get(), 0); // Should be initial value, not 42

        ISOLATION_SIGNAL.update(|x| x + 100);
        assert_eq!(ISOLATION_SIGNAL.get(), 100);

        ISOLATION_SIGNAL.reset(); // Reset after test
    }

    // Final verification
    assert_eq!(ISOLATION_SIGNAL.get(), 0);
}

// ============================================================================
// Advanced Features Tests
// ============================================================================

#[test]
fn test_computed_signal() {
    let _guard = TEST_MUTEX.lock();

    static FIRST_NAME: GlobalSignal<String> = Signal::global(|| "John".to_string());
    static LAST_NAME: GlobalSignal<String> = Signal::global(|| "Doe".to_string());

    // Reset to ensure clean state
    FIRST_NAME.reset();
    LAST_NAME.reset();

    // Create computed signal
    let full_name = ComputedSignal::new(|| format!("{} {}", FIRST_NAME.get(), LAST_NAME.get()));

    // Test initial computation
    assert_eq!(full_name.get(), "John Doe");

    // Change first name and verify recomputation
    FIRST_NAME.set("Jane".to_string());
    assert_eq!(full_name.get(), "Jane Doe");

    // Change last name and verify recomputation
    LAST_NAME.set("Smith".to_string());
    assert_eq!(full_name.get(), "Jane Smith");

    // Test invalidation
    full_name.invalidate();
    assert_eq!(full_name.get(), "Jane Smith"); // Should recompute

    // Clean up
    FIRST_NAME.reset();
    LAST_NAME.reset();
}

#[test]
fn test_signal_persistence() {
    use crate::hooks::signal::persistence::{MemoryBackend, PersistenceBackend};

    let backend = MemoryBackend::default();

    // Test save and load
    backend.save("test_key", "test_value").unwrap();
    let loaded = backend.load("test_key").unwrap();
    assert_eq!(loaded, Some("test_value".to_string()));

    // Test non-existent key
    let missing = backend.load("missing_key").unwrap();
    assert_eq!(missing, None);

    // Test remove
    backend.remove("test_key").unwrap();
    let removed = backend.load("test_key").unwrap();
    assert_eq!(removed, None);
}

#[cfg(feature = "file-persistence")]
#[test]
fn test_file_persistence_backend() {
    use crate::hooks::signal::persistence::{FileBackend, PersistenceBackend};
    use std::fs;

    // Create a temporary directory for testing
    let temp_dir = std::env::temp_dir().join("rink_signal_test");
    let _ = fs::create_dir_all(&temp_dir);

    let backend = FileBackend::new(&temp_dir);

    // Test save and load
    backend.save("test_file_key", "test_file_value").unwrap();
    let loaded = backend.load("test_file_key").unwrap();
    assert_eq!(loaded, Some("test_file_value".to_string()));

    // Verify file exists
    let file_path = temp_dir.join("test_file_key.signal");
    assert!(file_path.exists());

    // Test non-existent key
    let missing = backend.load("missing_file_key").unwrap();
    assert_eq!(missing, None);

    // Test remove
    backend.remove("test_file_key").unwrap();
    let removed = backend.load("test_file_key").unwrap();
    assert_eq!(removed, None);
    assert!(!file_path.exists());

    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_signal_middleware() {
    use crate::hooks::signal::middleware::{
        AnalyticsMiddleware, CompositeMiddleware, LoggingMiddleware,
    };

    let analytics = AnalyticsMiddleware::new();

    // Test analytics middleware
    analytics.after_change(1, &"old", &"new");
    analytics.after_change(1, &"new", &"newer");
    analytics.after_change(2, &"old", &"new");

    assert_eq!(analytics.get_change_count(1), 2);
    assert_eq!(analytics.get_change_count(2), 1);
    assert_eq!(analytics.get_change_count(3), 0);

    let all_counts = analytics.get_all_change_counts();
    assert_eq!(all_counts.len(), 2);
    assert_eq!(all_counts[&1], 2);
    assert_eq!(all_counts[&2], 1);

    // Test composite middleware
    let composite = CompositeMiddleware::new()
        .add(LoggingMiddleware)
        .add(analytics);

    // This should trigger both logging and analytics
    composite.after_change(3, &"old", &"new");
}

#[test]
fn test_weak_signal_references() {
    use crate::hooks::signal::weak_refs::{WeakSignalRef, WeakSignalRegistry};

    let _guard = TEST_MUTEX.lock();

    static WEAK_TEST_SIGNAL: GlobalSignal<i32> = Signal::global(|| 42);
    WEAK_TEST_SIGNAL.reset();

    // Create weak reference
    let weak_ref = WeakSignalRef::from_global(&WEAK_TEST_SIGNAL);

    // Should be able to upgrade initially
    assert!(weak_ref.is_alive());
    let strong_ref = weak_ref.upgrade().unwrap();
    assert_eq!(strong_ref.get(), 42);

    // Test signal ID
    assert_eq!(weak_ref.signal_id(), strong_ref.id());

    // Test weak registry
    let registry = WeakSignalRegistry::new();
    registry.add(weak_ref.clone());

    assert_eq!(registry.total_count(), 1);
    assert_eq!(registry.live_count(), 1);

    let live_handles = registry.get_live_handles();
    assert_eq!(live_handles.len(), 1);
    assert_eq!(live_handles[0].get(), 42);

    // Cleanup should not remove live references
    let live_count = registry.cleanup_dead_refs();
    assert_eq!(live_count, 1);

    // Clean up
    WEAK_TEST_SIGNAL.reset();
}

#[test]
fn test_reset() {
    // Create a counter that increments on each initialization
    static INIT_COUNT: AtomicUsize = AtomicUsize::new(0);

    // Use a static with a unique name for this test
    static TEST_RESET_SIGNAL: GlobalSignal<i32> = Signal::global(|| {
        INIT_COUNT.fetch_add(1, Ordering::SeqCst);
        0
    });
    
    let signal = &TEST_RESET_SIGNAL;

    // Initial state
    assert_eq!(signal.get(), 0);
    assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 1);

    // Change the value
    signal.set(42);
    assert_eq!(signal.get(), 42);
    assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 1);

    // Reset should use the initializer again
    signal.reset();
    assert_eq!(signal.get(), 0);
    assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 2);

    // Multiple resets should keep working
    signal.set(100);
    signal.reset();
    assert_eq!(signal.get(), 0);
    assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 3);
}

#[test]
fn test_reset_with_complex_state() {
    #[derive(Debug, Clone, PartialEq)]
    struct State {
        counter: i32,
        message: String,
    }

    let signal = GlobalSignal::new(|| State {
        counter: 0,
        message: "Initial".to_string(),
    });

    // Initial state
    assert_eq!(signal.get().counter, 0);
    assert_eq!(signal.get().message, "Initial");

    // Modify the state
    signal.update(|mut state| {
        state.counter = 42;
        state.message = "Modified".to_string();
        state
    });

    assert_eq!(signal.get().counter, 42);
    assert_eq!(signal.get().message, "Modified");

    // Reset should restore initial state
    signal.reset();
    assert_eq!(signal.get().counter, 0);
    assert_eq!(signal.get().message, "Initial");
}
