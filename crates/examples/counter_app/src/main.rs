use pulse::prelude::*;

struct Counter {}

impl Component for Counter {
    fn render(&self, _area: Rect, _frame: &mut Frame) {
        // TODO: Implement rendering logic
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pulse::render(|| Counter {})
}
