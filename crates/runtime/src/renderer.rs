use crate::terminal::setup_terminal;
use pulse_core::{Component, IntoElement};

/// Renders a component-based TUI application
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
    // Initialize terminal backend
    let mut terminal = setup_terminal()?;

    // Create the element instance and convert it
    let element = initializer().into_element();

    // Render the component once
    terminal.draw(|frame| {
        element.render(frame.area(), frame);
    })?;

    // Keep the terminal open briefly to see the result
    std::thread::sleep(std::time::Duration::from_millis(2000));

    Ok(())
}

/// Renders a component-based TUI application asynchronously
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
    // Initialize terminal backend
    let mut terminal = setup_terminal()?;

    // Create the element instance and convert it
    let element = app_fn().await.into_element();

    // Get terminal size for rendering
    let size = terminal.size()?;

    // Render the component once
    terminal.draw(|frame| {
        element.render(size, frame);
    })?;

    // Keep the terminal open briefly to see the result
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    Ok(())
}
