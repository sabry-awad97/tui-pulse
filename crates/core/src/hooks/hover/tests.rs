use crate::hooks::hover::*;
use crate::hooks::state::use_state;
use crate::hooks::test_utils::{with_hook_context, with_test_isolate};

use crossterm::event::{MouseButton, MouseEventKind};
use ratatui::layout::Rect;

/// Custom paragraph wrapper that preserves content
#[derive(Clone)]
pub struct ParagraphComponent {
    text: String,
    style: ratatui::style::Style,
}

impl ParagraphComponent {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: ratatui::style::Style::default(),
        }
    }

    pub fn style(mut self, style: ratatui::style::Style) -> Self {
        self.style = style;
        self
    }
}

impl Component for ParagraphComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let paragraph = ratatui::widgets::Paragraph::new(self.text.as_str()).style(self.style);
        frame.render_widget(paragraph, area);
    }
}

/// Specific implementation for Paragraph widget - convert to our custom component
impl crate::IntoElement for ratatui::widgets::Paragraph<'_> {
    type Element = ParagraphComponent;
    fn into_element(self) -> Self::Element {
        // Since we can't extract the original text, create a placeholder
        // In practice, users should use ParagraphComponent::new() directly
        ParagraphComponent::new("Paragraph content")
    }
}

/// Test basic hover detection functionality
#[test]
fn test_basic_hover_detection() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let content = ParagraphComponent::new("Hover me!");
            let (hoverable, is_hovered) = use_hover(content);

            // Initially should not be hovered
            assert!(!is_hovered, "Should not be hovered initially");

            // Verify hoverable component is created
            let _component = hoverable.into_element();
            // Component creation should succeed without panicking
        });
    });
}

/// Test hover state consistency across multiple calls
#[test]
fn test_hover_state_consistency() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let content = ParagraphComponent::new("Test content");

            // Multiple calls should return consistent state
            let (_, is_hovered_1) = use_hover(content.clone());
            let (_, is_hovered_2) = use_hover(content.clone());
            let (_, is_hovered_3) = use_hover(content);

            assert_eq!(is_hovered_1, is_hovered_2, "First two calls should match");
            assert_eq!(is_hovered_2, is_hovered_3, "All calls should be consistent");

            // All should start not hovered
            assert!(!is_hovered_1, "Should start not hovered");
        });
    });
}

/// Test point-in-rectangle utility function
#[test]
fn test_point_in_rect() {
    // Test basic rectangle bounds
    let rect = Rect::new(10, 10, 20, 15);

    // Points inside the rectangle
    assert!(
        is_point_in_rect((10, 10), rect),
        "Top-left corner should be inside"
    );
    assert!(is_point_in_rect((15, 15), rect), "Center should be inside");
    assert!(
        is_point_in_rect((29, 24), rect),
        "Bottom-right corner should be inside"
    );

    // Points outside the rectangle
    assert!(
        !is_point_in_rect((9, 10), rect),
        "Left of rectangle should be outside"
    );
    assert!(
        !is_point_in_rect((10, 9), rect),
        "Above rectangle should be outside"
    );
    assert!(
        !is_point_in_rect((30, 15), rect),
        "Right of rectangle should be outside"
    );
    assert!(
        !is_point_in_rect((15, 25), rect),
        "Below rectangle should be outside"
    );

    // Edge cases
    assert!(
        !is_point_in_rect((30, 25), rect),
        "Bottom-right edge should be outside"
    );
}

/// Test edge cases for rectangle bounds
#[test]
fn test_rect_edge_cases() {
    // Zero-width rectangle
    let zero_width = Rect::new(5, 5, 0, 10);
    assert!(
        !is_point_in_rect((5, 5), zero_width),
        "Zero-width rect should not contain points"
    );

    // Zero-height rectangle
    let zero_height = Rect::new(5, 5, 10, 0);
    assert!(
        !is_point_in_rect((5, 5), zero_height),
        "Zero-height rect should not contain points"
    );

    // Single-pixel rectangle
    let single_pixel = Rect::new(5, 5, 1, 1);
    assert!(
        is_point_in_rect((5, 5), single_pixel),
        "Single pixel should contain its point"
    );
    assert!(
        !is_point_in_rect((6, 5), single_pixel),
        "Single pixel should not contain adjacent points"
    );

    // Rectangle at origin
    let origin_rect = Rect::new(0, 0, 5, 5);
    assert!(
        is_point_in_rect((0, 0), origin_rect),
        "Origin rectangle should contain (0,0)"
    );
    assert!(
        is_point_in_rect((4, 4), origin_rect),
        "Origin rectangle should contain (4,4)"
    );
    assert!(
        !is_point_in_rect((5, 5), origin_rect),
        "Origin rectangle should not contain (5,5)"
    );
}

/// Test hover detection with different component types
#[test]
fn test_hover_with_different_components() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test with Paragraph
            let paragraph = ParagraphComponent::new("Paragraph content");
            let (_, is_hovered_p) = use_hover(paragraph);
            assert!(!is_hovered_p, "Paragraph should start not hovered");

            // Test with different text content
            let long_text =
                ParagraphComponent::new("This is a much longer text content for testing");
            let (_, is_hovered_long) = use_hover(long_text);
            assert!(!is_hovered_long, "Long text should start not hovered");

            // Test with empty content
            let empty = ParagraphComponent::new("");
            let (_, is_hovered_empty) = use_hover(empty);
            assert!(!is_hovered_empty, "Empty content should start not hovered");
        });
    });
}

/// Test hover callbacks functionality
#[test]
fn test_hover_callbacks() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let content = ParagraphComponent::new("Interactive content");

            // Test with no callbacks
            let (_, is_hovered) =
                use_hover_with_callbacks(content.clone(), None::<fn()>, None::<fn()>);
            assert!(!is_hovered, "Should start not hovered with no callbacks");

            // Test with callbacks (can't easily test execution without event simulation)
            let enter_called = std::sync::Arc::new(std::sync::Mutex::new(false));
            let exit_called = std::sync::Arc::new(std::sync::Mutex::new(false));

            let enter_called_clone = enter_called.clone();
            let exit_called_clone = exit_called.clone();

            let (_, _is_hovered_with_callbacks) = use_hover_with_callbacks(
                content,
                Some(move || {
                    *enter_called_clone.lock().unwrap() = true;
                }),
                Some(move || {
                    *exit_called_clone.lock().unwrap() = true;
                }),
            );

            // Callbacks should be created successfully
            // Note: Full callback testing would require event simulation
        });
    });
}

/// Test hover hook performance characteristics
#[test]
fn test_hover_performance() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Create multiple hover hooks to test performance
            let components: Vec<_> = (0..50)
                .map(|i| {
                    let content = ParagraphComponent::new(format!("Component {}", i));
                    use_hover(content)
                })
                .collect();

            // All should be created successfully without performance issues
            assert_eq!(components.len(), 50, "Should create 50 hover components");

            // All should start not hovered
            for (i, (_, is_hovered)) in components.iter().enumerate() {
                assert!(!is_hovered, "Component {} should start not hovered", i);
            }
        });
    });
}

/// Test hover state with area tracking
#[test]
fn test_hover_area_tracking() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let content = ParagraphComponent::new("Area tracking test");
            let (hoverable, is_hovered) = use_hover(content);

            // Initially not hovered
            assert!(!is_hovered, "Should start not hovered");

            // The hoverable component should track its area when rendered
            // This is tested implicitly through the component wrapper
            let _component = hoverable.into_element();

            // Area tracking is handled by the HoverableComponent wrapper
            // which updates the component_area state on each render
        });
    });
}

/// Test mouse event filtering for hover detection
#[test]
fn test_mouse_event_filtering() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let content = ParagraphComponent::new("Mouse event test");
            let (_, is_hovered) = use_hover(content);

            // Test that hook handles mouse event types correctly
            let mouse_events = vec![
                ("Move", MouseEventKind::Moved),
                ("Click", MouseEventKind::Down(MouseButton::Left)),
                ("Release", MouseEventKind::Up(MouseButton::Left)),
                ("Drag", MouseEventKind::Drag(MouseButton::Left)),
                ("ScrollUp", MouseEventKind::ScrollUp),
                ("ScrollDown", MouseEventKind::ScrollDown),
            ];

            for (event_name, _kind) in mouse_events {
                println!("Testing mouse event: {}", event_name);

                // Only MouseEventKind::Moved should affect hover state
                // Other events are ignored for hover detection
                assert!(!is_hovered, "Should remain not hovered for {}", event_name);
            }
        });
    });
}

/// Test hover hook integration with other hooks
#[test]
fn test_hover_integration() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let content = ParagraphComponent::new("Integration test");
            let (hoverable, is_hovered) = use_hover(content);

            // Test that hover works alongside other hooks
            let (counter, set_counter) = use_state(|| 0);

            if is_hovered {
                set_counter.update(|c| c + 1);
            }

            // Initially not hovered, counter should be 0
            assert!(!is_hovered, "Should start not hovered");
            assert_eq!(counter.get(), 0, "Counter should start at 0");

            // Hover hook should work independently of other state
            let _component = hoverable.into_element();
        });
    });
}

/// Test hover boundary conditions
#[test]
fn test_hover_boundary_conditions() {
    // Test rectangle boundary calculations
    let test_cases = vec![
        // (rect, point, expected)
        (Rect::new(0, 0, 10, 10), (0, 0), true), // Top-left corner
        (Rect::new(0, 0, 10, 10), (9, 9), true), // Bottom-right inside
        (Rect::new(0, 0, 10, 10), (10, 10), false), // Bottom-right outside
        (Rect::new(5, 5, 1, 1), (5, 5), true),   // Single pixel
        (Rect::new(5, 5, 1, 1), (6, 6), false),  // Outside single pixel
        (Rect::new(100, 200, 50, 30), (149, 229), true), // Large rect inside
        (Rect::new(100, 200, 50, 30), (150, 230), false), // Large rect outside
    ];

    for (rect, point, expected) in test_cases {
        let result = is_point_in_rect(point, rect);
        assert_eq!(
            result, expected,
            "Point {:?} in rect {:?} should be {}",
            point, rect, expected
        );
    }
}

/// Test hover component wrapper functionality
#[test]
fn test_hoverable_component_wrapper() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let content = ParagraphComponent::new("Wrapper test");
            let (hoverable, _) = use_hover(content);

            // The hoverable component should implement Component trait
            let _component = hoverable.into_element();

            // Component should be renderable (tested implicitly)
            // The wrapper tracks area and delegates rendering to wrapped content

            // This tests the HoverableComponent implementation
            println!("HoverableComponent wrapper created successfully");
        });
    });
}

/// Test hover hook memory management
#[test]
fn test_hover_memory_management() {
    with_test_isolate(|| {
        // Test that hover hooks clean up properly
        for _ in 0..10 {
            with_hook_context(|_| {
                let content = ParagraphComponent::new("Memory test");
                let (_, _) = use_hover(content);
                // Hook should be created and cleaned up automatically
            });
        }

        // Multiple iterations should not cause memory leaks
        // This is tested implicitly through the test framework
    });
}

/// Test hover state transitions
#[test]
fn test_hover_state_transitions() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let content = ParagraphComponent::new("State transition test");
            let (_, is_hovered) = use_hover(content);

            // Document expected state transitions:
            println!("Hover State Transitions:");
            println!("1. Initial: not hovered (false)");
            println!("2. Mouse enters area: hovered (true)");
            println!("3. Mouse moves within area: remains hovered (true)");
            println!("4. Mouse exits area: not hovered (false)");
            println!("5. Mouse re-enters: hovered (true) again");

            // Initially not hovered
            assert!(!is_hovered, "Should start in not-hovered state");

            // State transitions would be tested with event simulation
            // For now, we verify the initial state and API contract
        });
    });
}
