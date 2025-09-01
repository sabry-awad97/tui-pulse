use crossterm::event::{Event, MouseEventKind};
use ratatui::{Frame, layout::Rect};

use crate::{
    Component, IntoElement,
    hooks::{event::use_event, state::use_state},
};

/// A hook for detecting hover events on a component
///
/// This hook wraps a component with hover detection capabilities, tracking mouse
/// position and determining when the mouse is hovering over the component's area.
///
/// # Arguments
///
/// * `content` - The component to be wrapped with hover detection
///
/// # Returns
///
/// * `(impl IntoElement, bool)` - A tuple containing:
///   - The hoverable component wrapped with hover detection
///   - A boolean indicating whether the component is currently being hovered
///
/// # Examples
///
/// ## Basic Usage
/// ```rust,no_run
/// # use pulse_core::hooks::hover::use_hover;
/// # use pulse_core::{Component, IntoElement};
/// # use ratatui::{Frame, layout::Rect};
/// #[derive(Clone)]
/// struct MyButton;
/// impl Component for MyButton {
///     fn render(&self, _area: Rect, _frame: &mut Frame) {}
/// }
/// let button = MyButton;
/// let (hoverable_button, is_hovered) = use_hover(button);
///
/// if is_hovered {
///     // Apply hover styling or effects
/// }
/// ```
///
/// ## With Conditional Styling
/// ```rust,no_run
/// # use pulse_core::hooks::hover::use_hover;
/// # use pulse_core::{Component, IntoElement};
/// # use ratatui::{Frame, layout::Rect, style::{Color, Style}};
/// #[derive(Clone)]
/// struct StyledButton { style: Style }
/// impl Component for StyledButton {
///     fn render(&self, _area: Rect, _frame: &mut Frame) {}
/// }
/// let base_style = Style::default().fg(Color::White);
/// let hover_style = Style::default().fg(Color::Yellow);
/// let button = StyledButton { style: base_style };
/// let (hoverable_button, is_hovered) = use_hover(button);
/// ```
///
/// # Performance Notes
///
/// - Hover detection is based on mouse movement events
/// - Area calculation is cached and only updated when component area changes
/// - Minimal overhead when mouse is not moving
/// - Thread-safe and can be used in async contexts
/// - Only processes mouse events, ignoring other input types
pub fn use_hover(content: impl IntoElement) -> (impl IntoElement, bool) {
    // State to track current hover status
    let (is_hovered, set_is_hovered) = use_state(|| false);

    // State to track the component's rendered area
    let (component_area, set_component_area) = use_state(Rect::default);

    // Monitor mouse events for hover detection
    if let Some(event) = use_event()
        && let Event::Mouse(mouse_event) = event
        && mouse_event.kind == MouseEventKind::Moved
    {
        let mouse_pos = (mouse_event.column, mouse_event.row);
        let area = component_area.get();
        let is_inside = is_point_in_rect(mouse_pos, area);

        // Only update state if hover status changed
        if is_inside != is_hovered.get() {
            set_is_hovered.set(is_inside);
        }
    }

    let hoverable_component = HoverableComponent {
        content: content.into_element(),
        set_component_area,
    };

    (hoverable_component, is_hovered.get())
}

/// Advanced hover hook with enter/exit callbacks
///
/// This enhanced version of `use_hover` accepts optional callbacks that are
/// triggered when the mouse enters or exits the component area.
///
/// # Arguments
///
/// * `content` - The component to be wrapped with hover detection
/// * `on_enter` - Optional callback triggered when mouse enters the area
/// * `on_exit` - Optional callback triggered when mouse exits the area
///
/// # Returns
///
/// * `(impl IntoElement, bool)` - Hoverable component and current hover state
///
/// # Examples
///
/// ```rust,no_run
/// # use pulse_core::hooks::hover::use_hover_with_callbacks;
/// # use pulse_core::{Component, IntoElement};
/// # use ratatui::{Frame, layout::Rect};
/// #[derive(Clone)]
/// struct InteractiveButton;
/// impl Component for InteractiveButton {
///     fn render(&self, _area: Rect, _frame: &mut Frame) {}
/// }
/// let button = InteractiveButton;
/// let (hoverable_button, is_hovered) = use_hover_with_callbacks(
///     button,
///     Some(|| println!("Mouse entered button area")),
///     Some(|| println!("Mouse left button area")),
/// );
/// ```
pub fn use_hover_with_callbacks<F1, F2>(
    content: impl IntoElement,
    on_enter: Option<F1>,
    on_exit: Option<F2>,
) -> (impl IntoElement, bool)
where
    F1: Fn() + 'static,
    F2: Fn() + 'static,
{
    let (is_hovered, set_is_hovered) = use_state(|| false);
    let (component_area, set_component_area) = use_state(Rect::default);
    let (previous_hover, set_previous_hover) = use_state(|| false);

    if let Some(event) = use_event()
        && let Event::Mouse(mouse_event) = event
        && let MouseEventKind::Moved = mouse_event.kind
    {
        let mouse_pos = (mouse_event.column, mouse_event.row);
        let area = component_area.get();
        let is_inside = is_point_in_rect(mouse_pos, area);
        let was_hovered = previous_hover.get();

        if is_inside != was_hovered {
            set_is_hovered.set(is_inside);
            set_previous_hover.set(is_inside);

            // Trigger callbacks on state change
            if is_inside && !was_hovered {
                if let Some(callback) = &on_enter {
                    callback();
                }
            } else if !is_inside
                && was_hovered
                && let Some(callback) = &on_exit
            {
                callback();
            }
        }
    }
    let hoverable_component = HoverableComponent {
        content: content.into_element(),
        set_component_area,
    };

    (hoverable_component, is_hovered.get())
}

/// Utility function to check if a point is within a rectangle
fn is_point_in_rect(point: (u16, u16), rect: Rect) -> bool {
    let (x, y) = point;
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// Component wrapper that tracks its rendered area for hover detection
#[derive(Clone)]
struct HoverableComponent<T: Component> {
    content: T,
    set_component_area: crate::hooks::state::StateSetter<Rect>,
}

impl<T: Component> Component for HoverableComponent<T> {
    fn render(&self, area: Rect, frame: &mut Frame) {
        // Update the tracked area whenever the component is rendered
        self.set_component_area.set(area);

        // Render the wrapped content
        self.content.render(area, frame);
    }
}

#[cfg(test)]
mod tests;
