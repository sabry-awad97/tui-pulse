use crate::terminal::{restore_terminal, setup_terminal};
use crossterm::event;
use pulse_core::{
    Component, IntoElement,
    exit::should_exit,
    hooks::{
        HookContext,
        event::{global_events::process_global_event, set_current_event},
    },
};
use std::{rc::Rc, time::Duration};

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
/// use pulse_runtime::render;
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
/// render(|| Counter).unwrap();
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

    // Call on_mount for the root component
    element.on_mount();

    // Main render loop
    let mut running = true;
    while running {
        // Reset hook index before each render
        hook_context.reset_hook_index();

        // Handle events with a small timeout to prevent blocking
        if event::poll(Duration::from_millis(16))? {
            if let Ok(event) = event::read() {
                // Process key events
                if let event::Event::Key(key_event) = &event {
                    // First try to process as a global event
                    let processed = process_global_event(key_event);

                    // If not processed as a global event, make it available to components
                    if !processed {
                        set_current_event(Some(event.into()));

                        // Check for exit after component event handling
                        if should_exit() {
                            running = false;
                        }
                    }
                }
            }
        } else {
            // No events, clear the current event
            set_current_event(None);
        }

        // Render the component
        terminal.draw(|frame| {
            element.render(frame.area(), frame);
        })?;
    }

    // Clear the current event
    set_current_event(None);

    // Clean up the hook context
    pulse_core::hooks::clear_hook_context();

    // Restore terminal state
    restore_terminal()?;

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
/// use pulse_runtime::render_async;
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
/// render_async(|| async { AsyncCounter }).await.unwrap();
/// # }
/// ```
pub(crate) async fn render_async_with_hooks<F, Fut, T>(
    app_fn: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = T> + Send + 'static,
    T: IntoElement + 'static,
{
    // Initialize terminal backend
    let mut terminal = setup_terminal()?;

    // Create a new hook context for this component tree
    let hook_context = Rc::new(HookContext::new());

    // Set the hook context for this thread
    pulse_core::hooks::set_hook_context(hook_context.clone());

    // Create the element instance and convert it
    let element = app_fn().await.into_element();

    // Call on_mount for the root component
    element.on_mount();

    // Main render loop
    loop {
        // Reset hook index before each render
        hook_context.reset_hook_index();

        // Get terminal size for rendering
        let size = terminal.size()?;

        // Handle events with a small timeout to prevent blocking
        if event::poll(Duration::from_millis(16))? {
            if let Ok(event) = event::read() {
                // Process key events
                if let event::Event::Key(key_event) = &event {
                    // First try to process as a global event
                    let processed = process_global_event(key_event);

                    // If not processed as a global event, make it available to components
                    if !processed {
                        set_current_event(Some(event.into()));

                        // Check for exit after component event handling
                        if should_exit() {
                            break;
                        }
                    }
                }
            }
        } else {
            // No events, clear the current event
            set_current_event(None);
        }

        // If no events and exit is requested, quit
        if should_exit() {
            break;
        }

        // Render the component
        terminal.draw(|frame| {
            element.render(size, frame);
        })?;

        // Small delay to prevent high CPU usage
        tokio::time::sleep(Duration::from_millis(16)).await; // ~60 FPS
    }

    // Clear the current event
    set_current_event(None);

    // Clean up the hook context
    pulse_core::hooks::clear_hook_context();

    // Restore terminal state
    restore_terminal()?;

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
    Fut: std::future::Future<Output = T> + Send + 'static,
    T: IntoElement + 'static,
{
    render_async_with_hooks(app_fn).await
}
