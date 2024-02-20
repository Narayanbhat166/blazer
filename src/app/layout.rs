/// Divide the screen real estate into various chunks, each with one specific purpose
// ┌───────────────────────────────────────────────────────────────────────────────────────────────────┐
// │                                                                                                   │
// │ ┌───────────────────────────────────────────────────────────────────────────────────────────────┐ │
// │ │                                          Menu                                                 │ │
// │ └───────────────────────────────────────────────────────────────────────────────────────────────┘ │
// │                                                                                                   │
// │  ┌────────────────────────┐ ┌───────────────────────────────────────────────────────────────────┐ │
// │  │                        │ │                                                                   │ │
// │  │                        │ │                                                                   │ │
// │  │                        │ │                                                                   │ │
// │  │                        │ │                                                                   │ │
// │  │       Details          │ │                                                                   │ │
// │  │                        │ │                                                                   │ │
// │  │                        │ │                                                                   │ │
// │  └────────────────────────┘ │                                                                   │ │
// │                             │                                                                   │ │
// │  ┌────────────────────────┐ │                            Action┼area                            │ │
// │  │                        │ │                                                                   │ │
// │  │                        │ │                                                                   │ │
// │  │                        │ │                                                                   │ │
// │  │       Navigation       │ │                                                                   │ │
// │  │                        │ │                                                                   │ │
// │  │                        │ │                                                                   │ │
// │  │                        │ │                                                                   │ │
// │  └────────────────────────┘ └───────────────────────────────────────────────────────────────────┘ │
// │                                                                                                   │
// │ ┌───────────────────────────────────────────────────────────────────────────────────────────────┐ │
// │ │                                         Bottom bar                                            │ │
// │ └───────────────────────────────────────────────────────────────────────────────────────────────┘ │
// │                                                                                                   │
// └───────────────────────────────────────────────────────────────────────────────────────────────────┘
use tuirealm::tui::layout::{Constraint, Direction, Layout, Rect};

pub struct CustomLayout {
    pub menu: Rect,
    pub action_area: Rect,
    pub details: Rect,
    pub navigation: Rect,
    pub bottom_bar: Rect,
}

impl CustomLayout {
    pub fn new(main_screen_area: Rect) -> Self {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3), // Menu
                    Constraint::Min(10),   // Action area
                    Constraint::Length(3), // Bottom bar
                ]
                .as_ref(),
            )
            .split(main_screen_area);

        let middle_chunk = main_chunks[1];
        let middle_parts = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(middle_chunk);

        let middle_first_half_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(middle_parts[0]);

        Self {
            menu: main_chunks[0],
            action_area: middle_parts[1],
            details: middle_first_half_split[0],
            navigation: middle_first_half_split[1],
            bottom_bar: main_chunks[2],
        }
    }
}
