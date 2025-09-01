//! Beautiful Once Hook - Execute code exactly once per component lifecycle
//!
//! This module provides the `use_once` hook for one-time initialization,
//! perfect for setup tasks, logging, API calls, or any operation that should
//! only happen once during a component's lifetime.

#[cfg(test)]
mod tests;

/// Execute a function exactly once per component lifecycle
///
/// This hook ensures that the provided function is executed only once during
/// the component's lifetime, regardless of how many times the component re-renders.
/// It's perfect for initialization tasks, logging, or any side effects that should
/// only happen once.
///
/// # Arguments
///
/// * `init_fn` - The function to execute once. Can return a value or be void.
///
/// # Examples
///
/// ## Basic Initialization
/// ```rust,no_run
/// use pulse_core::hooks::once::use_once;
///
/// // This will only print once, even if the component re-renders
/// use_once(|| {
///     println!("Component initialized!");
/// });
/// ```
///
/// ## With Return Value
/// ```rust,no_run
/// use pulse_core::hooks::once::use_once;
///
/// // Initialize expensive resources once
/// let config = use_once(|| {
///     // This expensive operation only happens once
///     load_configuration_from_file()
/// });
///
/// if let Some(cfg) = config {
///     // Use the configuration
/// }
///
/// fn load_configuration_from_file() -> String {
///     "config data".to_string()
/// }
/// ```
///
/// ## Logging and Analytics
/// ```rust,no_run
/// use pulse_core::hooks::once::use_once;
///
/// // Log component mount only once
/// use_once(|| {
///     log::info!("UserProfile component mounted");
///     analytics::track_event("component_mounted", "UserProfile");
/// });
/// ```
pub fn use_once<F>(init_fn: F)
where
    F: FnOnce() + 'static,
{
    use crate::hooks::effect::use_effect_once;

    use_effect_once(move || {
        init_fn();
        || {} // No cleanup
    });
}
