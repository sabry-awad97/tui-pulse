mod renderer;
mod terminal;
pub use renderer::{render, render_async};
pub use terminal::{ManagedTerminal, restore_terminal, setup_terminal};

#[cfg(test)]
mod tests {
    use super::*;
    use pulse_core::{Component, hooks::state::use_state};
    use ratatui::{
        Frame,
        layout::Rect,
        text::Text,
        widgets::{Block, Borders, Paragraph},
    };

    // Simple counter component using hooks
    struct CounterComponent;

    impl Component for CounterComponent {
        fn render(&self, area: Rect, frame: &mut Frame) {
            let (count, _set_count) = use_state(|| 0);
            let text = Text::from(format!("Count: {}", count.get()));
            frame.render_widget(Paragraph::new(text), area);
        }
    }

    // Component with multiple hooks
    struct MultiHookComponent;

    impl Component for MultiHookComponent {
        fn render(&self, area: Rect, frame: &mut Frame) {
            let (count, _set_count) = use_state(|| 0);
            let (name, _set_name) = use_state(|| "User".to_string());

            let text = Text::from(format!("Hello, {}! Count: {}", name.get(), count.get()));
            frame.render_widget(
                Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL).title("MultiHook")),
                area,
            );
        }
    }

    // Component with effect hook
    struct EffectComponent;

    impl Component for EffectComponent {
        fn render(&self, area: Rect, frame: &mut Frame) {
            let (count, _set_count) = use_state(|| 0);

            // This effect will run after each render
            // In a real app, this might update state or perform side effects
            println!("Effect ran with count: {}", count.get());

            let text = Text::from(format!("Effect Count: {}", count.get()));
            frame.render_widget(Paragraph::new(text), area);
        }
    }

    // Test synchronous rendering with hooks
    #[test]
    fn test_render_with_hooks() {
        let result = render(|| CounterComponent);
        assert!(result.is_ok());
    }

    // Test async rendering with hooks
    #[tokio::test]
    async fn test_async_render_with_hooks() {
        let result = render_async(|| async { CounterComponent }).await;
        assert!(result.is_ok());
    }

    // Test component with multiple hooks
    #[test]
    fn test_component_with_multiple_hooks() {
        let result = render(|| MultiHookComponent);
        assert!(result.is_ok());
    }

    // Test component with effect hook
    #[test]
    fn test_component_with_effect() {
        let result = render(|| EffectComponent);
        assert!(result.is_ok());
    }

    // Test multiple renders with the same component
    #[test]
    fn test_multiple_renders() {
        // First render
        let result1 = render(|| CounterComponent);
        assert!(result1.is_ok());

        // Second render should work independently
        let result2 = render(|| CounterComponent);
        assert!(result2.is_ok());
    }
}
