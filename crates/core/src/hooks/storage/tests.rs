//! Comprehensive tests for the use_local_storage hook implementation
//!
//! This module contains extensive tests covering:
//! - Basic functionality with primitive and complex types
//! - Error handling and edge cases
//! - Thread safety and concurrent access
//! - Storage backend integration
//! - Memory management and cleanup
//! - Serialization/deserialization edge cases

use super::*;
use crate::hooks::test_utils::{with_hook_context, with_test_isolate};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Barrier;
use std::thread;
use std::time::Duration;

// Global test mutex to ensure tests run sequentially
static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Test data structure for complex serialization tests
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestData {
    id: u32,
    name: String,
    values: Vec<i32>,
    metadata: TestMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestMetadata {
    created_at: String,
    tags: Vec<String>,
}

impl Default for TestData {
    fn default() -> Self {
        Self {
            id: 1,
            name: "test".to_string(),
            values: vec![1, 2, 3],
            metadata: TestMetadata {
                created_at: "2024-01-01".to_string(),
                tags: vec!["test".to_string(), "data".to_string()],
            },
        }
    }
}

/// Helper function to create a temporary storage backend for testing
fn create_temp_storage_backend() -> Arc<MemoryStorageBackend> {
    Arc::new(MemoryStorageBackend::new())
}

/// Helper function to run tests with proper isolation
fn with_storage_test<F>(test_fn: F)
where
    F: FnOnce(),
{
    let _guard = TEST_MUTEX.lock();
    with_test_isolate(|| {
        clear_storage_state();
        test_fn();
    });
}

/// Test basic functionality with primitive types
#[test]
fn test_use_local_storage_basic_primitives() {
    with_storage_test(|| {
        // Set up temporary storage
        let backend = create_temp_storage_backend();
        set_storage_backend(backend);

        with_hook_context(|_ctx| {
            // Test with integer
            let (handle, setter) = use_local_storage("test_int".to_string(), 42i32);
            assert_eq!(handle.get(), 42);

            // Update value
            setter.set(100);
            assert_eq!(handle.get(), 100);

            // Test with string
            let (str_handle, str_setter) =
                use_local_storage("test_string".to_string(), "hello".to_string());
            assert_eq!(str_handle.get(), "hello");

            str_setter.set("world".to_string());
            assert_eq!(str_handle.get(), "world");

            // Test with boolean
            let (bool_handle, bool_setter) = use_local_storage("test_bool".to_string(), false);
            assert!(!bool_handle.get());

            bool_setter.set(true);
            assert!(bool_handle.get());
        });
    });
}

/// Test complex data structures with serde
#[test]
fn test_use_local_storage_complex_types() {
    with_storage_test(|| {
        let backend = create_temp_storage_backend();
        set_storage_backend(backend);

        with_hook_context(|_ctx| {
            let default_data = TestData::default();
            let (handle, setter) =
                use_local_storage("test_complex".to_string(), default_data.clone());

            assert_eq!(handle.get(), default_data);

            // Update complex data
            let updated_data = TestData {
                id: 2,
                name: "updated".to_string(),
                values: vec![4, 5, 6],
                metadata: TestMetadata {
                    created_at: "2024-01-02".to_string(),
                    tags: vec!["updated".to_string()],
                },
            };

            setter.set(updated_data.clone());
            assert_eq!(handle.get(), updated_data);

            // Test functional update
            setter.update(|prev| TestData {
                id: prev.id + 1,
                ..prev.clone()
            });

            let final_data = handle.get();
            assert_eq!(final_data.id, 3);
            assert_eq!(final_data.name, "updated");
        });
    });
}

/// Test persistence across hook instances
#[test]
fn test_use_local_storage_persistence() {
    with_storage_test(|| {
        let backend = create_temp_storage_backend();
        set_storage_backend(backend);

        // First hook instance
        with_hook_context(|_ctx| {
            let (handle, setter) =
                use_local_storage("persistent_test".to_string(), "initial".to_string());
            assert_eq!(handle.get(), "initial");

            setter.set("persisted_value".to_string());
            assert_eq!(handle.get(), "persisted_value");
        });

        // Second hook instance (should load persisted value)
        with_hook_context(|_ctx| {
            let (handle, _setter) =
                use_local_storage("persistent_test".to_string(), "default".to_string());
            assert_eq!(handle.get(), "persisted_value");
        });
    });
}

/// Test memory storage backend
#[test]
fn test_memory_storage_backend() {
    with_storage_test(|| {
        let memory_backend = create_temp_storage_backend();
        set_storage_backend(memory_backend.clone());

        with_hook_context(|_ctx| {
            let (handle, setter) = use_local_storage("memory_test".to_string(), 123i32);
            assert_eq!(handle.get(), 123);

            setter.set(456);
            assert_eq!(handle.get(), 456);

            // Verify it's stored in memory backend
            assert_eq!(memory_backend.len(), 1);
            assert!(!memory_backend.is_empty());
        });
    });
}

/// Test thread safety with concurrent access
#[test]
fn test_use_local_storage_thread_safety() {
    with_storage_test(|| {
        let memory_backend = create_temp_storage_backend();
        set_storage_backend(memory_backend);

        with_hook_context(|_ctx| {
            let (handle, setter) = use_local_storage("thread_test".to_string(), 0i32);

            let setter1 = setter.clone();
            let setter2 = setter.clone();
            let setter3 = setter.clone();

            let barrier = Arc::new(Barrier::new(4));
            let barrier1 = barrier.clone();
            let barrier2 = barrier.clone();
            let barrier3 = barrier.clone();

            let handle1 = thread::spawn(move || {
                barrier1.wait();
                for _i in 0..50 {
                    setter1.update(|prev| prev + 1);
                    thread::sleep(Duration::from_micros(1));
                }
            });

            let handle2 = thread::spawn(move || {
                barrier2.wait();
                for _i in 0..50 {
                    setter2.update(|prev| prev + 2);
                    thread::sleep(Duration::from_micros(1));
                }
            });

            let handle3 = thread::spawn(move || {
                barrier3.wait();
                for _i in 0..50 {
                    setter3.update(|prev| prev + 3);
                    thread::sleep(Duration::from_micros(1));
                }
            });

            barrier.wait();

            handle1.join().unwrap();
            handle2.join().unwrap();
            handle3.join().unwrap();

            // Final value should be close to 0 + (50*1) + (50*2) + (50*3) = 300
            let final_value = handle.get();
            assert!(
                (280..=320).contains(&final_value),
                "Expected final value to be around 300, got {}",
                final_value
            );
        });
    });
}

/// Test temporary updates (not persisted)
#[test]
fn test_temporary_updates() {
    with_storage_test(|| {
        let memory_backend = create_temp_storage_backend();
        set_storage_backend(memory_backend.clone());

        with_hook_context(|_ctx| {
            let (handle, setter) =
                use_local_storage("temp_test".to_string(), "initial".to_string());

            // Regular update (should be persisted)
            setter.set("persisted".to_string());
            assert_eq!(handle.get(), "persisted");
            assert_eq!(memory_backend.len(), 1);

            // Temporary update (should not be persisted)
            setter.set_temporary("temporary".to_string());
            assert_eq!(handle.get(), "temporary");

            // Storage should still contain the persisted value
            let stored_value = memory_backend.read("temp_test").unwrap().unwrap();
            assert!(stored_value.contains("persisted"));
        });
    });
}

/// Test storage operations and error handling
#[test]
fn test_storage_operations() {
    with_storage_test(|| {
        let memory_backend = create_temp_storage_backend();
        set_storage_backend(memory_backend.clone());

        with_hook_context(|_ctx| {
            let (handle, setter) = use_local_storage("ops_test".to_string(), vec![1, 2, 3]);

            // Test storage availability
            assert!(handle.is_storage_available());

            // Test manual save
            setter.set(vec![4, 5, 6]);
            assert!(setter.save().is_ok());

            // Test remove from storage
            assert!(setter.remove_from_storage().is_ok());
            assert_eq!(memory_backend.len(), 0);

            // Value should still be in memory
            assert_eq!(handle.get(), vec![4, 5, 6]);
        });
    });
}

/// Test error handling with unavailable storage
#[test]
fn test_storage_unavailable() {
    with_storage_test(|| {
        // Create a mock backend that reports as unavailable
        struct UnavailableBackend;
        impl StorageBackend for UnavailableBackend {
            fn read(&self, _key: &str) -> LocalStorageResult<Option<String>> {
                Err(LocalStorageError::StorageUnavailable)
            }
            fn write(&self, _key: &str, _value: &str) -> LocalStorageResult<()> {
                Err(LocalStorageError::StorageUnavailable)
            }
            fn remove(&self, _key: &str) -> LocalStorageResult<()> {
                Err(LocalStorageError::StorageUnavailable)
            }
            fn is_available(&self) -> bool {
                false
            }
        }

        let unavailable_backend = Arc::new(UnavailableBackend);
        set_storage_backend(unavailable_backend);

        with_hook_context(|_ctx| {
            // Should still work with default values when storage is unavailable
            let (handle, setter) =
                use_local_storage("unavailable_test".to_string(), "default".to_string());
            assert_eq!(handle.get(), "default");
            assert!(!handle.is_storage_available());

            // Updates should still work in memory
            setter.set("updated".to_string());
            assert_eq!(handle.get(), "updated");

            // Save operations should return errors
            assert!(setter.save().is_err());
            assert!(setter.remove_from_storage().is_err());
        });
    });
}

/// Test functional updates with complex state
#[test]
fn test_functional_updates() {
    with_storage_test(|| {
        let backend = create_temp_storage_backend();
        set_storage_backend(backend);

        with_hook_context(|_ctx| {
            let initial_data = TestData::default();
            let (handle, setter) = use_local_storage("functional_test".to_string(), initial_data);

            // Test functional update
            setter.update(|prev| TestData {
                id: prev.id * 2,
                name: format!("{}_updated", prev.name),
                values: prev.values.iter().map(|x| x * 2).collect(),
                metadata: TestMetadata {
                    created_at: prev.metadata.created_at.clone(),
                    tags: {
                        let mut tags = prev.metadata.tags.clone();
                        tags.push("functional_update".to_string());
                        tags
                    },
                },
            });

            let updated_data = handle.get();
            assert_eq!(updated_data.id, 2);
            assert_eq!(updated_data.name, "test_updated");
            assert_eq!(updated_data.values, vec![2, 4, 6]);
            assert!(
                updated_data
                    .metadata
                    .tags
                    .contains(&"functional_update".to_string())
            );
        });
    });
}

/// Test storage key uniqueness
#[test]
fn test_storage_key_uniqueness() {
    with_storage_test(|| {
        let backend = create_temp_storage_backend();
        set_storage_backend(backend.clone());

        with_hook_context(|_ctx| {
            // Create multiple storage hooks with different keys
            let (handle1, setter1) = use_local_storage("key1".to_string(), 100i32);
            let (handle2, setter2) = use_local_storage("key2".to_string(), 200i32);
            let (handle3, _setter3) = use_local_storage("key1".to_string(), 300i32); // Same key as first

            // Initial values
            assert_eq!(handle1.get(), 100);
            assert_eq!(handle2.get(), 200);
            assert_eq!(handle3.get(), 100); // Should share state with handle1

            // Update first key
            setter1.set(150);
            assert_eq!(handle1.get(), 150);
            assert_eq!(handle2.get(), 200); // Should be unchanged
            assert_eq!(handle3.get(), 150); // Should share state with handle1

            // Update second key
            setter2.set(250);
            assert_eq!(handle1.get(), 150); // Should be unchanged
            assert_eq!(handle2.get(), 250);
            assert_eq!(handle3.get(), 150); // Should be unchanged

            // Verify storage contains both keys
            assert_eq!(backend.len(), 2);
        });
    });
}
