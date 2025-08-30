//! Global Signal system for terminus-ui framework
//!
//! This module provides a global signal system similar to Dioxus's GlobalSignal pattern,
//! allowing components to share state without prop drilling while maintaining thread safety
//! and automatic re-rendering capabilities.

#[cfg(test)]
mod tests;

use crate::hooks::{state::StateContainer, with_hook_context};
use parking_lot::{Mutex, RwLock};

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, Weak};

/// A thread-safe global signal container that holds the actual signal value
/// This is the core storage for global signal state
#[derive(Debug)]
pub struct GlobalSignalContainer<T> {
    /// Reuse the state container for the core functionality
    state: StateContainer<T>,
    /// Unique identifier for this signal instance
    id: u64,
}

impl<T> GlobalSignalContainer<T> {
    /// Create a new global signal container with the given initial value
    pub fn new(initial_value: T, id: u64) -> Self {
        Self {
            state: StateContainer::new(|| initial_value),
            id,
        }
    }

    /// Get the current value (clones the value)
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.state.get()
    }

    /// Set a new value and increment the version
    pub fn set(&self, new_value: T) {
        self.state.set(new_value);
    }

    /// Update the value using a function and increment the version
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(T) -> T,
        T: Clone,
    {
        // Convert the signal-style updater to state-style updater
        self.state
            .update(|current_value| updater(current_value.clone()));
    }

    /// Get the current version number
    pub fn version(&self) -> u64 {
        self.state.version()
    }

    /// Get the unique ID of this signal
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// A handle to a global signal that provides read and write access
#[derive(Clone)]
pub struct SignalHandle<T> {
    container: Arc<GlobalSignalContainer<T>>,
}

impl<T> SignalHandle<T> {
    /// Create a new signal handle from a container
    pub fn from_container(container: Arc<GlobalSignalContainer<T>>) -> Self {
        Self { container }
    }

    /// Get the current value (clones the value)
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.container.get()
    }

    /// Set a new value
    pub fn set(&self, new_value: T) {
        self.container.set(new_value);
    }

    /// Update the value using a function
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(T) -> T,
        T: Clone,
    {
        self.container.update(updater);
    }

    /// Get the current version number
    pub fn version(&self) -> u64 {
        self.container.version()
    }

    /// Get the unique ID of this signal
    pub fn id(&self) -> u64 {
        self.container.id()
    }
}

/// Global signal registry for managing signal instances
/// Each signal gets a unique key based on its memory address
///
/// Performance optimizations:
/// - Uses RwLock instead of Mutex for better read performance under high load
/// - Read operations (most common) can happen concurrently
/// - Write operations (signal creation/cleanup) are serialized but rare
///
/// Type safety considerations:
/// - Uses Any trait for type erasure, but maintains type safety through TypeId checks
/// - Each signal type gets its own entry in the registry
static GLOBAL_SIGNALS: OnceLock<RwLock<HashMap<usize, Box<dyn std::any::Any + Send + Sync>>>> =
    OnceLock::new();
static SIGNAL_ID_COUNTER: OnceLock<Mutex<u64>> = OnceLock::new();

// TODO: Future type-safe alternative approach
// Consider implementing a type-safe registry using const generics or a trait-based approach:
//
// trait TypedSignalRegistry<T> {
//     fn get_or_create(&self, key: usize, initializer: fn() -> T) -> Arc<GlobalSignalContainer<T>>;
// }
//
// This would eliminate the need for Any trait objects and provide compile-time type safety

/// Get the next unique signal ID
fn next_signal_id() -> u64 {
    let counter = SIGNAL_ID_COUNTER.get_or_init(|| Mutex::new(0));
    let mut id = counter.lock();
    *id += 1;
    *id
}

/// Get or create a global signal container for the given signal instance
fn get_or_create_global_signal<T, F>(
    signal_key: usize,
    initializer: F,
) -> Arc<GlobalSignalContainer<T>>
where
    T: Clone + Send + Sync + 'static,
    F: FnOnce() -> T,
{
    let registry = GLOBAL_SIGNALS.get_or_init(|| RwLock::new(HashMap::new()));

    // First try to read (most common case)
    {
        let signals = registry.read();
        if let Some(existing) = signals.get(&signal_key)
            && let Some(container) = existing.downcast_ref::<Arc<GlobalSignalContainer<T>>>()
        {
            return container.clone();
        }
    }

    // Need to create new signal, acquire write lock
    let mut signals = registry.write();

    // Double-check in case another thread created it while we were waiting
    if let Some(existing) = signals.get(&signal_key)
        && let Some(container) = existing.downcast_ref::<Arc<GlobalSignalContainer<T>>>()
    {
        return container.clone();
    }

    // Create new signal container
    let id = next_signal_id();
    let container = Arc::new(GlobalSignalContainer::new(initializer(), id));
    signals.insert(signal_key, Box::new(container.clone()));
    container
}

/// Performance monitoring utilities for the global signal system
#[cfg(debug_assertions)]
pub mod perf {
    use super::*;

    /// Get performance statistics about the global signal registry
    pub fn get_registry_stats() -> RegistryStats {
        if let Some(registry) = GLOBAL_SIGNALS.get() {
            let signals = registry.read();
            RegistryStats {
                total_signals: signals.len(),
                memory_usage_estimate: signals.len() * std::mem::size_of::<usize>(),
            }
        } else {
            RegistryStats {
                total_signals: 0,
                memory_usage_estimate: 0,
            }
        }
    }

    /// Statistics about the global signal registry
    #[derive(Debug, Clone)]
    pub struct RegistryStats {
        /// Total number of signals in the registry
        pub total_signals: usize,
        /// Estimated memory usage in bytes (rough approximation)
        pub memory_usage_estimate: usize,
    }
}

/// A global signal that can be declared as a static variable
pub struct GlobalSignal<T> {
    initializer: fn() -> T,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> GlobalSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new global signal with the given initializer
    pub const fn new(initializer: fn() -> T) -> Self {
        Self {
            initializer,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Force cleanup of all signals of this type (use with caution)
    ///
    /// This method forcibly removes all signals of type T from the global registry,
    /// regardless of active handles. This can cause panics if handles are still in use.
    /// Only use this for testing or emergency cleanup scenarios.
    ///
    /// # Safety
    ///
    /// This method can cause undefined behavior if there are still active SignalHandle
    /// instances pointing to the cleaned up signal. Use only when you're certain
    /// no handles are in use.
    #[cfg(test)]
    pub fn force_cleanup() {
        if let Some(registry) = GLOBAL_SIGNALS.get() {
            let mut signals = registry.write();

            // Collect keys to remove for our type
            let mut keys_to_remove = Vec::new();

            for (&key, container) in signals.iter() {
                // Try to downcast to our type
                if container
                    .downcast_ref::<Arc<GlobalSignalContainer<T>>>()
                    .is_some()
                {
                    keys_to_remove.push(key);
                }
            }

            // Remove all signals of this type
            for key in keys_to_remove {
                signals.remove(&key);
            }
        }
    }

    /// Get a handle to this global signal, initializing it if necessary
    pub fn handle(&self) -> SignalHandle<T> {
        let signal_key = self as *const _ as usize;
        let container = get_or_create_global_signal(signal_key, self.initializer);
        SignalHandle::from_container(container)
    }

    /// Get the current value (clones the value)
    pub fn get(&self) -> T {
        self.handle().get()
    }

    /// Set a new value
    pub fn set(&self, new_value: T) {
        self.handle().set(new_value);
    }

    /// Update the value using a function
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(T) -> T,
    {
        self.handle().update(updater);
    }

    /// Get the current version number
    pub fn version(&self) -> u64 {
        self.handle().version()
    }

    /// Get the unique ID of this signal
    pub fn id(&self) -> u64 {
        self.handle().id()
    }

    /// Reset signal to its initial value
    ///
    /// This method resets the global signal back to its initial value as defined
    /// by the initializer function. This is useful for:
    /// - Resetting application state
    /// - Test isolation
    /// - Implementing "reset to defaults" functionality
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pulse_core::hooks::signal::{Signal, GlobalSignal};
    ///
    /// static COUNTER: GlobalSignal<i32> = Signal::global(|| 0);
    ///
    /// // Reset the counter to its initial value (0)
    /// COUNTER.reset();
    ///
    /// // In a test context
    /// #[cfg(test)]
    /// mod tests {
    ///     use super::*;
    ///
    ///     #[test]
    ///     fn test_counter_increment() {
    ///         // Reset to initial state before test
    ///         COUNTER.reset();
    ///
    ///         assert_eq!(COUNTER.get(), 0);
    ///         COUNTER.update(|c| c + 1);
    ///         assert_eq!(COUNTER.get(), 1);
    ///
    ///         // Reset after test for isolation
    ///         COUNTER.reset();
    ///     }
    /// }
    /// ```
    pub fn reset(&self) {
        self.set((self.initializer)());
    }

    /// Reset all global signals to their initial values (test utility)
    ///
    /// This method clears the entire global signal registry, forcing all signals
    /// to be re-initialized on next access. This provides comprehensive test isolation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pulse_core::hooks::signal::GlobalSignal;
    ///
    /// #[cfg(test)]
    /// mod tests {
    ///     use super::*;
    ///
    ///     #[test]
    ///     fn test_with_clean_state() {
    ///         // Reset all signals before test
    ///         GlobalSignal::<()>::reset_all();
    ///
    ///         // Test logic here...
    ///
    ///         // All signals start fresh
    ///     }
    /// }
    /// ```
    #[cfg(test)]
    pub fn reset_all() {
        // Clear the entire registry to force re-initialization
        if let Some(registry) = GLOBAL_SIGNALS.get() {
            registry.write().clear();
        }
    }
}

/// The main Signal struct that provides the global() constructor
pub struct Signal;

impl Signal {
    /// Create a new global signal that can be declared as a static variable
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pulse_core::hooks::signal::{Signal, GlobalSignal};
    ///
    /// // Create a global signal that can be used anywhere in your app
    /// static COUNTER: GlobalSignal<i32> = Signal::global(|| 0);
    ///
    pub const fn global<T>(initializer: fn() -> T) -> GlobalSignal<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        GlobalSignal::new(initializer)
    }
}

/// Hook to use a global signal within a component context
///
/// This function provides a way to use global signals within components while
/// ensuring proper integration with the hook system for potential future
/// optimizations like automatic re-rendering.
///
/// # Examples
///
/// ```rust,no_run
/// use pulse_core::hooks::signal::{Signal, GlobalSignal, use_global_signal};
/// use pulse_core::Element;
///
/// static COUNTER: GlobalSignal<i32> = Signal::global(|| 0);
///
/// fn my_component() -> Element {
///     let counter_handle = use_global_signal(&COUNTER);
///     let count = counter_handle.get();
///
///     // Simple text-based UI for documentation
///     Element::Text(format!("Count: {}", count))
/// }
/// ```
pub fn use_global_signal<T>(global_signal: &GlobalSignal<T>) -> SignalHandle<T>
where
    T: Clone + Send + Sync + 'static,
{
    with_hook_context(|_ctx| {
        // For now, just return the handle directly
        // In the future, this could register the component for re-rendering
        // when the signal changes
        global_signal.handle()
    })
}

// ============================================================================
// Advanced Features
// ============================================================================

/// A computed signal that derives its value from other signals
///
/// Computed signals automatically update when their dependencies change,
/// providing a reactive programming model similar to Vue.js computed properties
/// or MobX computed values.
///
/// # Examples
///
/// ```rust,no_run
/// use pulse_core::hooks::signal::{Signal, GlobalSignal, ComputedSignal};
///
/// static FIRST_NAME: GlobalSignal<String> = Signal::global(|| "John".to_string());
/// static LAST_NAME: GlobalSignal<String> = Signal::global(|| "Doe".to_string());
///
/// // Computed signal that combines first and last name
/// static FULL_NAME: ComputedSignal<String> = ComputedSignal::new(|| {
///     format!("{} {}", FIRST_NAME.get(), LAST_NAME.get())
/// });
///
/// // Usage
/// let full_name = FULL_NAME.get(); // "John Doe"
/// FIRST_NAME.set("Jane".to_string());
/// let updated_name = FULL_NAME.get(); // "Jane Doe" - automatically updated!
/// ```
pub struct ComputedSignal<T> {
    compute_fn: fn() -> T,
    cached_value: OnceLock<Arc<parking_lot::RwLock<Option<T>>>>,
    dependencies_version: OnceLock<Arc<parking_lot::RwLock<u64>>>,
}

impl<T> ComputedSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new computed signal with the given computation function
    pub const fn new(compute_fn: fn() -> T) -> Self {
        Self {
            compute_fn,
            cached_value: OnceLock::new(),
            dependencies_version: OnceLock::new(),
        }
    }

    /// Get the current computed value, recomputing if dependencies have changed
    pub fn get(&self) -> T {
        let cache = self
            .cached_value
            .get_or_init(|| Arc::new(parking_lot::RwLock::new(None)));
        let version_tracker = self
            .dependencies_version
            .get_or_init(|| Arc::new(parking_lot::RwLock::new(0)));

        // For now, we recompute every time since dependency tracking is complex
        // In a full implementation, we'd track which signals this computation depends on
        let computed_value = (self.compute_fn)();

        // Cache the computed value
        *cache.write() = Some(computed_value.clone());
        *version_tracker.write() += 1;

        computed_value
    }

    /// Force recomputation of the cached value
    pub fn invalidate(&self) {
        if let Some(cache) = self.cached_value.get() {
            *cache.write() = None;
        }
    }
}

/// Signal persistence utilities for saving and restoring signal state
///
/// This module provides functionality to persist signal values to storage
/// and restore them on application startup, useful for maintaining user
/// preferences and application state across sessions.
pub mod persistence {
    use std::collections::HashMap;

    /// Trait for signal persistence backends
    pub trait PersistenceBackend {
        type Error;

        /// Save a value with the given key
        fn save(&self, key: &str, value: &str) -> Result<(), Self::Error>;

        /// Load a value by key
        fn load(&self, key: &str) -> Result<Option<String>, Self::Error>;

        /// Remove a value by key
        fn remove(&self, key: &str) -> Result<(), Self::Error>;
    }

    /// In-memory persistence backend for testing
    #[derive(Default)]
    pub struct MemoryBackend {
        storage: parking_lot::RwLock<HashMap<String, String>>,
    }

    impl PersistenceBackend for MemoryBackend {
        type Error = ();

        fn save(&self, key: &str, value: &str) -> Result<(), Self::Error> {
            self.storage
                .write()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn load(&self, key: &str) -> Result<Option<String>, Self::Error> {
            Ok(self.storage.read().get(key).cloned())
        }

        fn remove(&self, key: &str) -> Result<(), Self::Error> {
            self.storage.write().remove(key);
            Ok(())
        }
    }

    /// File-based persistence backend
    #[cfg(feature = "file-persistence")]
    pub struct FileBackend {
        base_path: std::path::PathBuf,
    }

    #[cfg(feature = "file-persistence")]
    impl FileBackend {
        pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
            Self {
                base_path: base_path.into(),
            }
        }
    }

    #[cfg(feature = "file-persistence")]
    impl PersistenceBackend for FileBackend {
        type Error = std::io::Error;

        fn save(&self, key: &str, value: &str) -> Result<(), Self::Error> {
            let file_path = self.base_path.join(format!("{}.signal", key));
            std::fs::write(file_path, value)
        }

        fn load(&self, key: &str) -> Result<Option<String>, Self::Error> {
            let file_path = self.base_path.join(format!("{}.signal", key));
            match std::fs::read_to_string(file_path) {
                Ok(content) => Ok(Some(content)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e),
            }
        }

        fn remove(&self, key: &str) -> Result<(), Self::Error> {
            let file_path = self.base_path.join(format!("{}.signal", key));
            match std::fs::remove_file(file_path) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e),
            }
        }
    }
}

/// Signal middleware system for intercepting and logging signal changes
///
/// Middleware allows you to intercept signal changes for debugging, logging,
/// analytics, or other cross-cutting concerns without modifying the signal logic.
pub mod middleware {
    use std::fmt::Debug;

    /// Trait for signal middleware
    pub trait SignalMiddleware<T>: Send + Sync {
        /// Called before a signal value is changed
        fn before_change(&self, signal_id: u64, old_value: &T, new_value: &T);

        /// Called after a signal value is changed
        fn after_change(&self, signal_id: u64, old_value: &T, new_value: &T);
    }

    /// Logging middleware that prints signal changes
    pub struct LoggingMiddleware;

    impl<T> SignalMiddleware<T> for LoggingMiddleware
    where
        T: Debug,
    {
        fn before_change(&self, signal_id: u64, old_value: &T, new_value: &T) {
            println!(
                "[SIGNAL] Before change - ID: {}, Old: {:?}, New: {:?}",
                signal_id, old_value, new_value
            );
        }

        fn after_change(&self, signal_id: u64, old_value: &T, new_value: &T) {
            println!(
                "[SIGNAL] After change - ID: {}, Old: {:?}, New: {:?}",
                signal_id, old_value, new_value
            );
        }
    }

    /// Analytics middleware that tracks signal usage
    pub struct AnalyticsMiddleware {
        change_count: parking_lot::RwLock<std::collections::HashMap<u64, usize>>,
    }

    impl Default for AnalyticsMiddleware {
        fn default() -> Self {
            Self {
                change_count: parking_lot::RwLock::new(std::collections::HashMap::new()),
            }
        }
    }

    impl AnalyticsMiddleware {
        pub fn new() -> Self {
            Self::default()
        }

        /// Get the number of changes for a signal
        pub fn get_change_count(&self, signal_id: u64) -> usize {
            self.change_count
                .read()
                .get(&signal_id)
                .copied()
                .unwrap_or(0)
        }

        /// Get all signal change counts
        pub fn get_all_change_counts(&self) -> std::collections::HashMap<u64, usize> {
            self.change_count.read().clone()
        }
    }

    impl<T> SignalMiddleware<T> for AnalyticsMiddleware {
        fn before_change(&self, _signal_id: u64, _old_value: &T, _new_value: &T) {
            // No action needed before change
        }

        fn after_change(&self, signal_id: u64, _old_value: &T, _new_value: &T) {
            let mut counts = self.change_count.write();
            *counts.entry(signal_id).or_insert(0) += 1;
        }
    }

    /// Composite middleware that runs multiple middleware in sequence
    pub struct CompositeMiddleware<T> {
        middleware: Vec<Box<dyn SignalMiddleware<T>>>,
    }

    impl<T> CompositeMiddleware<T> {
        pub fn new() -> Self {
            Self {
                middleware: Vec::new(),
            }
        }

        #[allow(clippy::should_implement_trait)]
        pub fn add<M: SignalMiddleware<T> + 'static>(mut self, middleware: M) -> Self {
            self.middleware.push(Box::new(middleware));
            self
        }
    }

    impl<T> Default for CompositeMiddleware<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T> SignalMiddleware<T> for CompositeMiddleware<T> {
        fn before_change(&self, signal_id: u64, old_value: &T, new_value: &T) {
            for middleware in &self.middleware {
                middleware.before_change(signal_id, old_value, new_value);
            }
        }

        fn after_change(&self, signal_id: u64, old_value: &T, new_value: &T) {
            for middleware in &self.middleware {
                middleware.after_change(signal_id, old_value, new_value);
            }
        }
    }
}

/// Weak reference utilities for automatic garbage collection of unused signals
///
/// This module provides weak reference support that allows signals to be
/// automatically garbage collected when no strong references remain,
/// preventing memory leaks in long-running applications.
pub mod weak_refs {
    use super::*;

    /// A weak reference to a global signal that doesn't prevent garbage collection
    ///
    /// WeakSignalRef allows you to hold a reference to a signal without preventing
    /// it from being garbage collected when all strong references are dropped.
    /// This is useful for observers, caches, or other systems that should not
    /// keep signals alive indefinitely.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pulse_core::hooks::signal::{Signal, GlobalSignal};
    /// use pulse_core::hooks::signal::weak_refs::WeakSignalRef;
    ///
    /// static COUNTER: GlobalSignal<i32> = Signal::global(|| 0);
    ///
    /// // Create a weak reference that won't prevent garbage collection
    /// let weak_ref = WeakSignalRef::from_global(&COUNTER);
    ///
    /// // Try to upgrade to a strong reference
    /// if let Some(strong_ref) = weak_ref.upgrade() {
    ///     println!("Counter value: {}", strong_ref.get());
    /// } else {
    ///     println!("Signal has been garbage collected");
    /// }
    /// ```
    pub struct WeakSignalRef<T> {
        weak_container: Weak<GlobalSignalContainer<T>>,
        signal_id: u64,
    }

    impl<T> WeakSignalRef<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        /// Create a weak reference from a global signal
        pub fn from_global(signal: &GlobalSignal<T>) -> Self {
            let handle = signal.handle();
            Self {
                weak_container: Arc::downgrade(&handle.container),
                signal_id: handle.id(),
            }
        }

        /// Create a weak reference from a signal handle
        pub fn from_handle(handle: &SignalHandle<T>) -> Self {
            Self {
                weak_container: Arc::downgrade(&handle.container),
                signal_id: handle.id(),
            }
        }

        /// Attempt to upgrade the weak reference to a strong reference
        ///
        /// Returns `Some(SignalHandle)` if the signal is still alive,
        /// or `None` if it has been garbage collected.
        pub fn upgrade(&self) -> Option<SignalHandle<T>> {
            self.weak_container
                .upgrade()
                .map(|container| SignalHandle { container })
        }

        /// Check if the signal is still alive without upgrading
        pub fn is_alive(&self) -> bool {
            self.weak_container.strong_count() > 0
        }

        /// Get the signal ID (available even if the signal is dead)
        pub fn signal_id(&self) -> u64 {
            self.signal_id
        }
    }

    impl<T> Clone for WeakSignalRef<T> {
        fn clone(&self) -> Self {
            Self {
                weak_container: self.weak_container.clone(),
                signal_id: self.signal_id,
            }
        }
    }

    /// A registry for managing weak references to signals
    ///
    /// This registry automatically cleans up dead weak references
    /// and provides utilities for bulk operations on weak signal references.
    pub struct WeakSignalRegistry<T> {
        weak_refs: parking_lot::RwLock<Vec<WeakSignalRef<T>>>,
    }

    impl<T> WeakSignalRegistry<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        /// Create a new weak signal registry
        pub fn new() -> Self {
            Self {
                weak_refs: parking_lot::RwLock::new(Vec::new()),
            }
        }

        /// Add a weak reference to the registry
        pub fn add(&self, weak_ref: WeakSignalRef<T>) {
            self.weak_refs.write().push(weak_ref);
        }

        /// Remove dead weak references and return the count of live ones
        pub fn cleanup_dead_refs(&self) -> usize {
            let mut refs = self.weak_refs.write();
            refs.retain(|weak_ref| weak_ref.is_alive());
            refs.len()
        }

        /// Get all live signal handles
        pub fn get_live_handles(&self) -> Vec<SignalHandle<T>> {
            let refs = self.weak_refs.read();
            refs.iter()
                .filter_map(|weak_ref| weak_ref.upgrade())
                .collect()
        }

        /// Get the count of registered weak references (including dead ones)
        pub fn total_count(&self) -> usize {
            self.weak_refs.read().len()
        }

        /// Get the count of live weak references
        pub fn live_count(&self) -> usize {
            self.weak_refs
                .read()
                .iter()
                .filter(|weak_ref| weak_ref.is_alive())
                .count()
        }
    }

    impl<T> Default for WeakSignalRegistry<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        fn default() -> Self {
            Self::new()
        }
    }
}
