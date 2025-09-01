use std::fmt;
use std::sync::Arc;

use crate::hooks::effect::EffectDependencies;
use crate::hooks::with_hook_context;

/// Universal callback wrapper inspired by Yew's Callback system.
///
/// This provides a type-safe way to pass function callbacks between components,
/// similar to React's event handlers and Yew's callback system.
///
/// # Examples
///
/// ```rust
///
/// // Define a callback that takes a string and returns nothing
/// let on_click: Callback<String> = Callback::from(|msg: String| {
///     println!("Clicked: {}", msg);
/// });
///
/// // Emit the callback
/// on_click.emit("Hello".to_string());
///
/// // Define a callback with return value
/// let transform: Callback<i32, String> = Callback::from(|num: i32| {
///     format!("Number: {}", num)
/// });
///
/// let result = transform.emit(42);
/// assert_eq!(result, "Number: 42");
/// ```
pub struct Callback<IN, OUT = ()> {
    callback: Arc<dyn Fn(IN) -> OUT + Send + Sync>,
}

impl<IN, OUT> Clone for Callback<IN, OUT> {
    fn clone(&self) -> Self {
        Self {
            callback: self.callback.clone(),
        }
    }
}

impl<IN, OUT> Callback<IN, OUT> {
    /// Create a new callback from a function
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(IN) -> OUT + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(func),
        }
    }

    /// Emit the callback with the given input value
    pub fn emit(&self, input: IN) -> OUT {
        (self.callback)(input)
    }

    /// Create a reformed callback that transforms the input before calling this callback
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminus_ui::prelude::*;
    ///
    /// let original: Callback<String> = Callback::from(|s: String| {
    ///     println!("Got: {}", s);
    /// });
    ///
    /// let reformed: Callback<i32> = original.reform(|num: i32| format!("Number: {}", num));
    /// reformed.emit(42); // Prints "Got: Number: 42"
    /// ```
    pub fn reform<F, T>(&self, func: F) -> Callback<T, OUT>
    where
        F: Fn(T) -> IN + Send + Sync + 'static,
        IN: 'static,
        OUT: 'static,
    {
        let callback = self.callback.clone();
        Callback::new(move |input: T| {
            let transformed = func(input);
            callback(transformed)
        })
    }

    /// Create a reformed callback that optionally transforms the input
    /// Only calls the original callback if the transform function returns Some
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminus_ui::prelude::*;
    ///
    /// let original: Callback<String> = Callback::from(|s: String| {
    ///     println!("Got: {}", s);
    /// });
    ///
    /// let filtered: Callback<i32, Option<()>> = original.filter_reform(|num: i32| {
    ///     if num > 0 {
    ///         Some(format!("Positive: {}", num))
    ///     } else {
    ///         None
    ///     }
    /// });
    ///
    /// filtered.emit(42);  // Prints "Got: Positive: 42", returns Some(())
    /// filtered.emit(-1);  // Does nothing, returns None
    /// ```
    pub fn filter_reform<F, T>(&self, func: F) -> Callback<T, Option<OUT>>
    where
        F: Fn(T) -> Option<IN> + Send + Sync + 'static,
        IN: 'static,
        OUT: 'static,
    {
        let callback = self.callback.clone();
        Callback::new(move |input: T| func(input).map(|v| callback(v)))
    }
}

impl<IN> Callback<IN> {
    /// Create a no-op callback that does nothing when called
    /// Useful for optional callbacks or default values
    pub fn noop() -> Self {
        Self::new(|_| {})
    }
}

impl<IN, OUT> Default for Callback<IN, OUT>
where
    OUT: Default,
{
    fn default() -> Self {
        Self::new(|_| OUT::default())
    }
}

impl<IN, OUT, F> From<F> for Callback<IN, OUT>
where
    F: Fn(IN) -> OUT + Send + Sync + 'static,
{
    fn from(func: F) -> Self {
        Self::new(func)
    }
}

// Additional From implementations for common patterns

/// Convert from Arc<dyn Fn> for shared callbacks
impl<IN, OUT> From<Arc<dyn Fn(IN) -> OUT + Send + Sync>> for Callback<IN, OUT> {
    fn from(func: Arc<dyn Fn(IN) -> OUT + Send + Sync>) -> Self {
        Self { callback: func }
    }
}

/// Convert from Option<Callback> - None becomes a noop callback
impl<IN> From<Option<Callback<IN>>> for Callback<IN> {
    fn from(opt: Option<Callback<IN>>) -> Self {
        opt.unwrap_or_else(Self::noop)
    }
}

impl<IN, OUT> fmt::Debug for Callback<IN, OUT> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Callback")
            .field("callback", &"<function>")
            .finish()
    }
}

impl<IN, OUT> PartialEq for Callback<IN, OUT> {
    fn eq(&self, other: &Self) -> bool {
        // Compare by pointer equality since we can't compare function contents
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

/// Trait for types that can be converted into event callbacks
/// This allows for flexible callback prop handling in components
pub trait IntoCallback<IN, OUT = ()> {
    fn into_callback(self) -> Callback<IN, OUT>;
}

impl<IN, OUT, F> IntoCallback<IN, OUT> for F
where
    F: Fn(IN) -> OUT + Send + Sync + 'static,
{
    fn into_callback(self) -> Callback<IN, OUT> {
        Callback::from(self)
    }
}

impl<IN, OUT> IntoCallback<IN, OUT> for Callback<IN, OUT> {
    fn into_callback(self) -> Callback<IN, OUT> {
        self
    }
}

/// Trait for converting callbacks into props that can be either Option<Callback> or Callback
/// This handles the automatic wrapping for optional callback props in the rsx! macro
pub trait IntoCallbackProp<T> {
    fn into_callback_prop(self) -> T;
}

// Implementation for non-optional callback props (Callback<IN, OUT>)
impl<F, IN, OUT> IntoCallbackProp<Callback<IN, OUT>> for F
where
    F: IntoCallback<IN, OUT>,
{
    fn into_callback_prop(self) -> Callback<IN, OUT> {
        self.into_callback()
    }
}

// Implementation for optional callback props (Option<Callback<IN, OUT>>)
impl<F, IN, OUT> IntoCallbackProp<Option<Callback<IN, OUT>>> for F
where
    F: IntoCallback<IN, OUT>,
{
    fn into_callback_prop(self) -> Option<Callback<IN, OUT>> {
        Some(self.into_callback())
    }
}

impl<IN, OUT> Callback<IN, OUT>
where
    IN: 'static,
{
    /// Create a callback from a closure with explicit type annotation
    /// This helps with type inference in complex scenarios
    pub fn from_fn<F>(func: F) -> Self
    where
        F: Fn(IN) -> OUT + Send + Sync + 'static,
    {
        Self::from(func)
    }

    /// Create a callback that ignores its input and returns a constant value
    pub fn constant(value: OUT) -> Self
    where
        OUT: Clone + Send + Sync + 'static,
    {
        Self::from(move |_| value.clone())
    }

    /// Create a callback that just prints its input (useful for debugging)
    pub fn debug() -> Self
    where
        IN: std::fmt::Debug,
        OUT: Default,
    {
        Self::from(|input| {
            println!("Callback called with: {:?}", input);
            OUT::default()
        })
    }

    /// Chain this callback with another callback
    /// The output of this callback becomes the input of the next
    pub fn then<F, NextOut>(self, next: F) -> Callback<IN, NextOut>
    where
        F: Fn(OUT) -> NextOut + Send + Sync + 'static,
        OUT: 'static,
    {
        Callback::from(move |input| {
            let intermediate = self.emit(input);
            next(intermediate)
        })
    }

    /// Map the output of this callback to a different type
    pub fn map<F, NewOut>(self, mapper: F) -> Callback<IN, NewOut>
    where
        F: Fn(OUT) -> NewOut + Send + Sync + 'static,
        OUT: 'static,
    {
        self.then(mapper)
    }

    /// Create a callback that calls this callback only if a condition is met
    pub fn filter<F>(self, predicate: F) -> Callback<IN, Option<OUT>>
    where
        F: Fn(&IN) -> bool + Send + Sync + 'static,
        IN: Clone + 'static,
        OUT: 'static,
    {
        Callback::from(move |input| {
            if predicate(&input) {
                Some(self.emit(input))
            } else {
                None
            }
        })
    }

    /// Create a callback that catches panics and returns a Result
    pub fn catch_unwind(self) -> Callback<IN, Result<OUT, String>>
    where
        IN: 'static,
        OUT: 'static,
    {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        Callback::from(move |input| {
            catch_unwind(AssertUnwindSafe(|| self.emit(input)))
                .map_err(|_| "Callback panicked".to_string())
        })
    }
}

// Additional convenience constructors
impl<IN, OUT> Callback<IN, OUT>
where
    IN: 'static,
{
    /// Create a callback that always returns the same value, ignoring input
    pub fn always(value: OUT) -> Self
    where
        OUT: Clone + Send + Sync + 'static,
    {
        Self::constant(value)
    }

    /// Create a callback from a mutable closure using interior mutability
    pub fn from_mut<F>(func: F) -> Self
    where
        F: FnMut(IN) -> OUT + Send + Sync + 'static,
    {
        use std::sync::Mutex;
        let func = Mutex::new(func);
        Self::from(move |input| {
            let mut guard = func.lock().unwrap();
            guard(input)
        })
    }
}

/// A memoized callback that only updates when dependencies change
#[derive(Clone)]
pub struct MemoizedCallback<IN, OUT = ()> {
    callback: Callback<IN, OUT>,
}

impl<IN, OUT> MemoizedCallback<IN, OUT> {
    /// Create a new memoized callback
    fn new(callback: Callback<IN, OUT>) -> Self {
        Self { callback }
    }

    /// Get the underlying callback
    pub fn callback(&self) -> &Callback<IN, OUT> {
        &self.callback
    }

    /// Emit the callback with the given input
    pub fn emit(&self, input: IN) -> OUT {
        self.callback.emit(input)
    }
}

/// Internal state for tracking memoized callbacks
struct CallbackState<IN, OUT> {
    /// Previous dependencies for comparison
    prev_deps: Option<Box<dyn EffectDependencies>>,
    /// The underlying callback
    callback: Option<Callback<IN, OUT>>,
}

impl<IN, OUT> CallbackState<IN, OUT> {
    fn new() -> Self {
        Self {
            prev_deps: None,
            callback: None,
        }
    }
}

/// Trait for types that can be converted into callback factories
///
/// This allows both direct closures and callback factory functions to be used
/// with use_callback, providing a more ergonomic API.
pub trait IntoCallbackFactory<IN, OUT> {
    /// Convert this into a callback factory function
    fn into_factory(self) -> Box<dyn FnOnce() -> Callback<IN, OUT>>;
}

/// Implementation for direct closures - most ergonomic usage
/// This is the primary use case: use_callback(|x| { ... }, deps)
impl<IN, OUT, F> IntoCallbackFactory<IN, OUT> for F
where
    F: Fn(IN) -> OUT + Send + Sync + Clone + 'static,
    IN: 'static,
    OUT: 'static,
{
    fn into_factory(self) -> Box<dyn FnOnce() -> Callback<IN, OUT>> {
        Box::new(move || Callback::from(self))
    }
}

/// Wrapper type for callback factory functions
/// Use this for advanced cases: use_callback(CallbackFactory(|| Callback::from(...)), deps)
pub struct CallbackFactory<F>(pub F);

/// Implementation for callback factory functions - advanced usage
impl<IN, OUT, F> IntoCallbackFactory<IN, OUT> for CallbackFactory<F>
where
    F: FnOnce() -> Callback<IN, OUT> + 'static,
    IN: 'static,
    OUT: 'static,
{
    fn into_factory(self) -> Box<dyn FnOnce() -> Callback<IN, OUT>> {
        Box::new(self.0)
    }
}

/// Professional use_callback hook with dependency tracking
///
/// This hook memoizes a callback function and only recreates it when the dependencies change.
/// This is useful for optimizing performance by preventing unnecessary re-renders of child
/// components that depend on callback props.
///
/// Uses the same dependency system as effect hooks for consistency.
///
/// # Arguments
///
/// * `factory` - A function that creates the callback (can be a closure or Callback factory)
/// * `deps` - Dependencies that the callback depends on (same as useEffect)
///
/// # Examples
///
/// ```rust
/// use rink_core::hooks::callback::use_callback;
/// use rink_core::hooks::state::use_state;
///
/// // Direct closure usage (ergonomic)
/// let (count, set_count) = use_state(0);
/// let count_value = count.get();
/// let increment = use_callback(
///     move |_| set_count.update(|c| c + 1),
///     count_value
/// );
///
/// // Factory function usage (advanced)
/// let submit = use_callback(
///     move || Callback::from(move |_| {
///         println!("Submitting: {} ({})", name_value, age_value);
///     }),
///     (name_value, age_value)
/// );
/// ```
pub fn use_callback<IN, OUT, F, Deps>(
    factory: F,
    deps: impl Into<Option<Deps>>,
) -> MemoizedCallback<IN, OUT>
where
    F: IntoCallbackFactory<IN, OUT> + 'static,
    Deps: EffectDependencies + Clone + PartialEq + 'static,
    IN: 'static,
    OUT: 'static,
{
    let deps = deps.into();
    with_hook_context(|ctx| {
        let index = ctx.next_hook_index();
        // Use get_or_init_state like use_state does
        let state_ref = ctx.get_or_init_state(index, || CallbackState::<IN, OUT>::new());

        let _should_recreate = {
            let mut state = state_ref.borrow_mut();

            // Determine if callback should be recreated
            let should_recreate = match &deps {
                None => {
                    // No dependencies - only create once (like useCallback with empty deps)
                    state.callback.is_none()
                }
                Some(current_deps) => {
                    // Check if dependencies have changed
                    match &state.prev_deps {
                        None => {
                            // First run - always create
                            true
                        }
                        Some(prev_deps) => {
                            // Compare dependencies using EffectDependencies trait
                            !current_deps.deps_eq(prev_deps.as_ref())
                        }
                    }
                }
            };

            if should_recreate {
                // Create new callback using the factory
                let callback_factory = factory.into_factory();
                let new_callback = callback_factory();
                state.callback = Some(new_callback);

                // Store new dependencies
                if let Some(current_deps) = &deps {
                    state.prev_deps = Some(current_deps.clone_deps());
                } else {
                    state.prev_deps = None;
                }
            }

            should_recreate
        };

        // Get the callback after releasing the mutable borrow and wrap it
        let state = state_ref.borrow();
        let callback = (*state
            .callback
            .as_ref()
            .expect("Callback should be initialized"))
        .clone();

        MemoizedCallback::new(callback)
    })
}

/// Convenience function for use_callback without dependencies
///
/// This creates a callback that never changes, similar to useCallback with an empty
/// dependency array in React.
///
/// # Examples
///
/// ```rust
/// use rink_core::hooks::callback::use_callback_once;
///
/// let log_click = use_callback_once(|msg: String| println!("Clicked: {}", msg));
/// ```
pub fn use_callback_once<IN, OUT, F>(factory: F) -> MemoizedCallback<IN, OUT>
where
    F: IntoCallbackFactory<IN, OUT> + 'static,
    IN: 'static,
    OUT: 'static,
{
    use_callback(factory, ())
}

/// Hook for creating event handlers with automatic memoization
///
/// This is a specialized version of use_callback for event handlers that
/// provides additional type safety and ergonomics.
///
/// # Examples
///
/// ```rust
/// use rink_core::hooks::callback::use_event_handler;
/// use rink_core::hooks::state::use_state;
/// use crossterm::event::KeyCode;
///
/// let (count, set_count) = use_state(0);
/// let count_value = count.get();
/// let on_key_press = use_event_handler(
///     move |key: KeyCode| {
///         match key {
///             KeyCode::Up => set_count.update(|c| c + 1),
///             KeyCode::Down => set_count.update(|c| c - 1),
///             _ => {}
///         }
///     },
///     count_value
/// );
/// ```
pub fn use_event_handler<T, F, Deps>(
    handler: F,
    deps: impl Into<Option<Deps>>,
) -> MemoizedCallback<T>
where
    F: Fn(T) + Clone + Send + Sync + 'static,
    Deps: EffectDependencies + Clone + PartialEq + 'static,
    T: 'static,
{
    use_callback(CallbackFactory(move || Callback::from(handler)), deps)
}

#[cfg(test)]
mod tests;
