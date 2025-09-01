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

#[cfg(feature = "sqlite")]
use tempfile::NamedTempFile;
#[cfg(feature = "sqlite")]
use tokio;

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

// SQLite Backend Tests
#[cfg(feature = "sqlite")]
mod sqlite_tests {
    use super::*;

    /// Helper function to create a temporary SQLite database for testing
    async fn create_test_sqlite_backend() -> LocalStorageResult<SqliteStorageBackend> {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        let database_url = format!("sqlite:{}", db_path);

        SqliteStorageBackend::new(&database_url).await
    }

    #[tokio::test]
    async fn test_sqlite_backend_creation() {
        let backend = create_test_sqlite_backend().await.unwrap();
        assert!(backend.is_available());
        assert_eq!(backend.table_name(), "local_storage");
    }

    #[tokio::test]
    async fn test_sqlite_backend_with_custom_table() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        let database_url = format!("sqlite:{}", db_path);

        let backend = SqliteStorageBackend::new_with_table(&database_url, "custom_table")
            .await
            .unwrap();
        assert!(backend.is_available());
        assert_eq!(backend.table_name(), "custom_table");
    }

    #[tokio::test]
    async fn test_sqlite_basic_operations() {
        let backend = create_test_sqlite_backend().await.unwrap();

        // Test write and read
        backend.write_async("test_key", "test_value").await.unwrap();
        let result = backend.read_async("test_key").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));

        // Test read non-existent key
        let result = backend.read_async("non_existent").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_sqlite_update_operation() {
        let backend = create_test_sqlite_backend().await.unwrap();

        // Initial write
        backend
            .write_async("update_key", "initial_value")
            .await
            .unwrap();
        let result = backend.read_async("update_key").await.unwrap();
        assert_eq!(result, Some("initial_value".to_string()));

        // Update the value
        backend
            .write_async("update_key", "updated_value")
            .await
            .unwrap();
        let result = backend.read_async("update_key").await.unwrap();
        assert_eq!(result, Some("updated_value".to_string()));
    }

    #[tokio::test]
    async fn test_sqlite_remove_operation() {
        let backend = create_test_sqlite_backend().await.unwrap();

        // Write a value
        backend
            .write_async("remove_key", "remove_value")
            .await
            .unwrap();
        let result = backend.read_async("remove_key").await.unwrap();
        assert_eq!(result, Some("remove_value".to_string()));

        // Remove the value
        backend.remove_async("remove_key").await.unwrap();
        let result = backend.read_async("remove_key").await.unwrap();
        assert_eq!(result, None);

        // Remove non-existent key (should not error)
        backend.remove_async("non_existent").await.unwrap();
    }

    #[tokio::test]
    async fn test_sqlite_complex_json_data() {
        let backend = create_test_sqlite_backend().await.unwrap();

        let test_data = TestData::default();
        let json_value = serde_json::to_string(&test_data).unwrap();

        // Store complex JSON data
        backend
            .write_async("complex_data", &json_value)
            .await
            .unwrap();
        let result = backend.read_async("complex_data").await.unwrap();
        assert_eq!(result, Some(json_value.clone()));

        // Verify we can deserialize it back
        let retrieved_data: TestData = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(retrieved_data, test_data);
    }

    #[tokio::test]
    async fn test_sqlite_multiple_keys() {
        let backend = create_test_sqlite_backend().await.unwrap();

        // Store multiple key-value pairs
        backend.write_async("key1", "value1").await.unwrap();
        backend.write_async("key2", "value2").await.unwrap();
        backend.write_async("key3", "value3").await.unwrap();

        // Verify all values can be retrieved
        assert_eq!(
            backend.read_async("key1").await.unwrap(),
            Some("value1".to_string())
        );
        assert_eq!(
            backend.read_async("key2").await.unwrap(),
            Some("value2".to_string())
        );
        assert_eq!(
            backend.read_async("key3").await.unwrap(),
            Some("value3".to_string())
        );

        // Remove one key and verify others remain
        backend.remove_async("key2").await.unwrap();
        assert_eq!(
            backend.read_async("key1").await.unwrap(),
            Some("value1".to_string())
        );
        assert_eq!(backend.read_async("key2").await.unwrap(), None);
        assert_eq!(
            backend.read_async("key3").await.unwrap(),
            Some("value3".to_string())
        );
    }

    #[tokio::test]
    async fn test_sqlite_unicode_and_special_characters() {
        let backend = create_test_sqlite_backend().await.unwrap();

        let unicode_key = "🔑_key_测试";
        let unicode_value = "🎯 Value with émojis and 中文 characters! @#$%^&*()";

        backend
            .write_async(unicode_key, unicode_value)
            .await
            .unwrap();
        let result = backend.read_async(unicode_key).await.unwrap();
        assert_eq!(result, Some(unicode_value.to_string()));
    }

    #[tokio::test]
    async fn test_sqlite_concurrent_operations() {
        let backend = Arc::new(create_test_sqlite_backend().await.unwrap());
        let num_tasks = 10;
        let mut handles = Vec::new();

        // Spawn multiple concurrent write operations
        for i in 0..num_tasks {
            let backend_clone = backend.clone();
            let handle = tokio::spawn(async move {
                let key = format!("concurrent_key_{}", i);
                let value = format!("concurrent_value_{}", i);
                backend_clone.write_async(&key, &value).await.unwrap();

                // Verify the write
                let result = backend_clone.read_async(&key).await.unwrap();
                assert_eq!(result, Some(value));
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all keys exist
        for i in 0..num_tasks {
            let key = format!("concurrent_key_{}", i);
            let expected_value = format!("concurrent_value_{}", i);
            let result = backend.read_async(&key).await.unwrap();
            assert_eq!(result, Some(expected_value));
        }
    }

    #[tokio::test]
    async fn test_sqlite_error_handling() {
        // Test with invalid database URL
        let result = SqliteStorageBackend::new("invalid://database/url").await;
        assert!(result.is_err());

        if let Err(LocalStorageError::ReadError(msg)) = result {
            assert!(msg.contains("Failed to connect to SQLite database"));
        } else {
            panic!("Expected ReadError for invalid database URL");
        }
    }

    #[tokio::test]
    async fn test_sqlite_persistence_across_connections() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        let database_url = format!("sqlite:{}", db_path);

        // Create first backend and store data
        {
            let backend1 = SqliteStorageBackend::new(&database_url).await.unwrap();
            backend1
                .write_async("persistent_key", "persistent_value")
                .await
                .unwrap();
        }

        // Create second backend with same database and verify data persists
        {
            let backend2 = SqliteStorageBackend::new(&database_url).await.unwrap();
            let result = backend2.read_async("persistent_key").await.unwrap();
            assert_eq!(result, Some("persistent_value".to_string()));
        }
    }
}
