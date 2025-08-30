use crate::terminal::setup_terminal;
use pulse_core::{Component, IntoElement, hooks::HookContext};
use std::rc::Rc;

/// Renders a component-based TUI application with hooks support
///
/// This function sets up a hook context and manages the component lifecycle
/// including state persistence between renders.
///
/// # Arguments
/// * `app_fn` - A closure that returns anything that can be converted into an element
///
/// # Example
/// ```no_run
/// use pulse_runtime::render_with_hooks;
/// use pulse_core::{hooks::state::use_state, Component, IntoElement};
/// use ratatui::{Frame, layout::Rect, text::Text};
///
/// struct Counter;
///
/// impl Component for Counter {
///     fn render(&self, area: Rect, frame: &mut Frame) {
///         let (count, _) = use_state(|| 0);
///         frame.render_widget(Text::from(format!("Count: {}", count.get())), area);
///     }
/// }
///
/// render_with_hooks(|| Counter).unwrap();
/// ```
pub(crate) fn render_with_hooks<F, T>(initializer: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> T,
    T: IntoElement,
{
    // Initialize terminal backend
    let mut terminal = setup_terminal()?;

    // Create a new hook context for this component tree
    let hook_context = Rc::new(HookContext::new());

    // Set the hook context for this thread
    pulse_core::hooks::set_hook_context(hook_context.clone());

    // Create the element instance and convert it
    let element = initializer().into_element();

    // Reset hook index before each render
    hook_context.reset_hook_index();

    // Render the component
    terminal.draw(|frame| {
        element.render(frame.area(), frame);
    })?;

    // Keep the terminal open briefly to see the result
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Clean up the hook context
    pulse_core::hooks::clear_hook_context();

    Ok(())
}

/// Renders a component-based TUI application
///
/// This is a convenience wrapper around `render_with_hooks` for components that don't use hooks.
///
/// # Arguments
/// * `app_fn` - A closure that returns anything that can be converted into an element
///
/// # Example
/// ```no_run
/// use pulse_runtime::render;
/// use pulse_core::{Component, IntoElement};
/// use ratatui::{Frame, layout::Rect};
///
/// struct MyComponent;
///
/// impl Component for MyComponent {
///     fn render(&self, _area: Rect, _frame: &mut Frame) {}
/// }
///
/// render(|| MyComponent).unwrap();
/// ```
pub fn render<F, T>(initializer: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> T,
    T: IntoElement,
{
    render_with_hooks(initializer)
}

/// Renders a component-based TUI application asynchronously with hooks support
///
/// This function sets up a hook context and manages the component lifecycle
/// including state persistence between renders in an async context.
///
/// # Arguments
/// * `app_fn` - A closure that returns a future that resolves to anything that can be converted into an element
///
/// # Example
/// ```no_run
/// use pulse_runtime::render_async_with_hooks;
/// use pulse_core::{hooks::state::use_state, Component, IntoElement};
/// use ratatui::{Frame, layout::Rect, text::Text};
///
/// struct AsyncCounter;
///
/// impl Component for AsyncCounter {
///     fn render(&self, area: Rect, frame: &mut Frame) {
///         let (count, _set_count) = use_state(|| 0);
///         frame.render_widget(Text::from(format!("Async Count: {}", count.get())), area);
///     }
/// }
///
/// # async fn example() {
/// render_async_with_hooks(|| async { AsyncCounter }).await.unwrap();
/// # }
/// ```
pub(crate) async fn render_async_with_hooks<F, Fut, T>(app_fn: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = T>,
    T: IntoElement,
{
    // Initialize terminal backend
    let mut terminal = setup_terminal()?;

    // Create a new hook context for this component tree
    let hook_context = Rc::new(HookContext::new());

    // Set the hook context for this thread
    pulse_core::hooks::set_hook_context(hook_context.clone());

    // Create the element instance and convert it
    let element = app_fn().await.into_element();

    // Reset hook index before render
    hook_context.reset_hook_index();

    // Get terminal size for rendering
    let size = terminal.size()?;

    // Render the component
    terminal.draw(|frame| {
        element.render(size, frame);
    })?;

    // Keep the terminal open briefly to see the result
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    // Clean up the hook context
    pulse_core::hooks::clear_hook_context();

    Ok(())
}

/// Renders a component-based TUI application asynchronously
///
/// This is a convenience wrapper around `render_async_with_hooks` for components that don't use hooks.
///
/// # Arguments
/// * `app_fn` - A closure that returns a future that resolves to anything that can be converted into an element
///
/// # Example
/// ```no_run
/// use pulse_runtime::render_async;
/// use pulse_core::{Component, IntoElement};
/// use ratatui::{Frame, layout::Rect};
///
/// struct MyComponent;
///
/// impl Component for MyComponent {
///     fn render(&self, _area: Rect, _frame: &mut Frame) {}
/// }
///
/// # async fn example() {
/// render_async(|| async { MyComponent }).await.unwrap();
/// # }
/// ```
pub async fn render_async<F, Fut, T>(app_fn: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = T>,
    T: IntoElement,
{
    render_async_with_hooks(app_fn).await
}
