//! Beautiful Context Provider API for sharing state between components
//!
//! This module provides a more elegant context API that allows components to share state
//! without having to pass props down through many levels, similar to React's Context API.
//! This implementation is designed to be more ergonomic and beautiful to use.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

#[cfg(test)]
mod tests;

use crate::hooks::with_hook_context;

thread_local! {
    static CONTEXT_PROVIDERS: RefCell<HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>> =
        RefCell::new(HashMap::new());
}

/// Clear all context providers (called when hook context is reset)
pub fn clear_context_providers() {
    CONTEXT_PROVIDERS.with(|providers| {
        providers.borrow_mut().clear();
    });
}

/// Provides a context value for a type
///
/// This function creates a context value that will be available to all components
/// rendered within the current component's render function. It's similar to React's
/// Context.Provider component but as a hook.
///
/// # Type Parameters
///
/// * `T` - The type of the context value
///
/// # Arguments
///
/// * `create_value` - A function that creates the context value
///
/// # Returns
///
/// * The context value
///
/// # Examples
///
/// ```rust,no_run
/// use pulse_core::hooks::context::use_context_provider;
///
/// // Define a context type
/// #[derive(Clone)]
/// struct TitleContext(String);
///
/// // In a component function, provide the context value
/// fn my_component() {
///     // Provide the title context
///     let title = use_context_provider(|| TitleContext("My App".to_string()));
///
///     // The title context is now available to all child components
///     // ... render child components
/// }
/// ```
pub fn use_context_provider<T, F>(create_value: F) -> T
where
    T: Clone + Send + Sync + 'static,
    F: FnOnce() -> T,
{
    with_hook_context(|_ctx| {
        let type_id = TypeId::of::<T>();
        let value = create_value();
        let value_clone = value.clone();

        // Store the value in the thread-local provider stack
        CONTEXT_PROVIDERS.with(|providers| {
            let mut providers = providers.borrow_mut();
            let provider_stack = providers.entry(type_id).or_default();
            provider_stack.push(Box::new(value_clone));
        });

        value
    })
}

/// Consumes a context value for a type
///
/// This function retrieves a context value that was provided by a parent component
/// using `use_context_provider`. If no context value is found, it will panic.
///
/// # Type Parameters
///
/// * `T` - The type of the context value
///
/// # Returns
///
/// * The context value
///
/// # Examples
///
/// ```rust,no_run
/// use pulse_core::hooks::context::use_context;
///
/// // Define a context type
/// #[derive(Clone)]
/// struct TitleContext(String);
///
/// // In a child component function, consume the context value
/// fn child_component() {
///     // Get the title from context
///     let title: TitleContext = use_context::<TitleContext>();
///
///     // Use the title
///     let text = format!("Title: {}", title.0);
///     // ... use the text
/// }
/// ```
pub fn use_context<T>() -> T
where
    T: Clone + Send + Sync + 'static,
{
    with_hook_context(|_ctx| {
        let type_id = TypeId::of::<T>();

        // Try to get the value from the thread-local provider stack
        let value = CONTEXT_PROVIDERS.with(|providers| {
            let providers = providers.borrow();
            if let Some(provider_stack) = providers.get(&type_id)
                && let Some(last_provider) = provider_stack.last()
                && let Some(value) = last_provider.downcast_ref::<T>()
            {
                return Some(value.clone());
            }
            None
        });

        // If found in the thread-local stack, return it
        if let Some(value) = value {
            return value;
        }

        // If not found, panic with a helpful error message
        panic!(
            "Context value for type {} not found. Make sure to call use_context_provider in a parent component.",
            std::any::type_name::<T>()
        );
    })
}

/// Creates a context with a default value
///
/// This function creates a context with a default value that can be used with
/// the `use_context_with_default` function. This is useful when you want to provide
/// a default value for a context that might not be provided by a parent component.
///
/// # Type Parameters
///
/// * `T` - The type of the context value
///
/// # Arguments
///
/// * `default_value` - The default value for the context
///
/// # Returns
///
/// * A context with the default value
///
/// # Examples
///
/// ```rust,no_run
/// use pulse_core::hooks::context::{create_context_with_default, Context};
/// use ratatui::style::Color;
/// use once_cell::sync::Lazy;
///
/// // Define a context type
/// #[derive(Clone)]
/// struct ThemeContext {
///     primary_color: Color,
///     secondary_color: Color,
/// }
///
/// // Create a context with a default theme
/// static DEFAULT_THEME: Lazy<Context<ThemeContext>> = Lazy::new(|| {
///     create_context_with_default(ThemeContext {
///         primary_color: Color::Blue,
///         secondary_color: Color::White,
///     })
/// });
/// ```
pub fn create_context_with_default<T>(default_value: T) -> Context<T>
where
    T: Clone + Send + Sync + 'static,
{
    Context {
        default_value: Arc::new(default_value),
        _phantom: PhantomData,
    }
}

/// A context with a default value
///
/// This struct represents a context with a default value that can be used with
/// the `use_context_with_default` function.
#[derive(Debug, Clone)]
pub struct Context<T: Clone + Send + Sync + 'static> {
    /// The default value for this context
    default_value: Arc<T>,
    /// Phantom data to ensure the context is associated with the correct type
    _phantom: PhantomData<T>,
}

/// Consumes a context value with a default fallback
///
/// This function retrieves a context value that was provided by a parent component
/// using `use_context_provider`. If no context value is found, it will return the
/// default value from the provided context.
///
/// # Arguments
///
/// * `context` - The context with the default value
///
/// # Returns
///
/// * The context value, or the default value if no context value is found
///
/// # Examples
///
/// ```rust,no_run
/// use pulse_core::hooks::context::{create_context_with_default, use_context_with_default, Context};
/// use ratatui::style::Color;
/// use once_cell::sync::Lazy;
///
/// // Define a context type
/// #[derive(Clone)]
/// struct ThemeContext {
///     primary_color: Color,
///     secondary_color: Color,
/// }
///
/// // Create a context with a default theme
/// static DEFAULT_THEME: Lazy<Context<ThemeContext>> = Lazy::new(|| {
///     create_context_with_default(ThemeContext {
///         primary_color: Color::Blue,
///         secondary_color: Color::White,
///     })
/// });
///
/// // In a child component function, consume the context value with a default fallback
/// fn child_component() {
///     // Get the theme from context, or use the default theme
///     let theme = use_context_with_default(&DEFAULT_THEME);
///
///     // Use the theme
///     let _primary = theme.primary_color;
/// }
/// ```
pub fn use_context_with_default<T>(context: &Context<T>) -> T
where
    T: Clone + Send + Sync + 'static,
{
    with_hook_context(|_ctx| {
        let type_id = TypeId::of::<T>();

        // Try to get the value from the thread-local provider stack
        let value = CONTEXT_PROVIDERS.with(|providers| {
            let providers = providers.borrow();
            if let Some(provider_stack) = providers.get(&type_id)
                && let Some(last_provider) = provider_stack.last()
                && let Some(value) = last_provider.downcast_ref::<T>()
            {
                return Some(value.clone());
            }
            None
        });

        // If found in the thread-local stack, return it
        if let Some(value) = value {
            return value;
        }

        // Otherwise, return the default value
        context.default_value.as_ref().clone()
    })
}
