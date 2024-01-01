use std::time::{Duration};

use tuirealm::event::NoUserEvent;

use tuirealm::terminal::TerminalBridge;
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::{
    Application, EventListenerCfg, Update,
};

use crate::components::{
    counter::{DigitCounter, LetterCounter},
    Id, Msg,
};

pub struct Model {
    /// Application
    pub app: Application<Id, Msg, NoUserEvent>,
    /// Indicates that the application must quit
    pub quit: bool,
    /// Tells whether to redraw interface
    pub redraw: bool,
    /// Used to draw to terminal
    pub terminal: TerminalBridge,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            app: Self::init_app(),
            quit: false,
            redraw: true,
            terminal: TerminalBridge::new().expect("Cannot initialize terminal"),
        }
    }
}

impl Model {
    pub fn view(&mut self) {
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3), // Clock
                            Constraint::Length(3), // Letter Counter
                            Constraint::Length(3), // Digit Counter
                            Constraint::Length(1), // Label
                        ]
                        .as_ref(),
                    )
                    .split(f.size());
                self.app.view(&Id::LetterCounter, f, chunks[1]);
                self.app.view(&Id::DigitCounter, f, chunks[2]);
            })
            .is_ok());
    }

    fn init_app() -> Application<Id, Msg, NoUserEvent> {
        // Setup application
        // NOTE: NoUserEvent is a shorthand to tell tui-realm we're not going to use any custom user event
        // NOTE: the event listener is configured to use the default crossterm input listener and to raise a Tick event each second
        // which we will use to update the clock

        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default()
                .default_input_listener(Duration::from_millis(20))
                .poll_timeout(Duration::from_millis(10))
                .tick_interval(Duration::from_secs(1)),
        );

        // Mount counters
        assert!(app
            .mount(
                Id::LetterCounter,
                Box::new(LetterCounter::new(0)),
                Vec::new()
            )
            .is_ok());
        assert!(app
            .mount(
                Id::DigitCounter,
                Box::new(DigitCounter::new(5)),
                Vec::default()
            )
            .is_ok());
        // Active letter counter
        assert!(app.active(&Id::LetterCounter).is_ok());
        app
    }
}

// Let's implement Update for model

impl Update<Msg> for Model {
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            // Set redraw
            self.redraw = true;
            // Match message
            match msg {
                Msg::AppClose => {
                    self.quit = true; // Terminate
                    None
                }
                Msg::Clock => None,
                Msg::DigitCounterBlur => {
                    // Give focus to letter counter
                    assert!(self.app.active(&Id::LetterCounter).is_ok());
                    None
                }
                Msg::DigitCounterChanged(_v) => {
                    // Update label
                    None
                }
                Msg::LetterCounterBlur => {
                    // Give focus to digit counter
                    assert!(self.app.active(&Id::DigitCounter).is_ok());
                    None
                }
                Msg::LetterCounterChanged(_v) => {
                    // Update label
                    None
                }
            }
        } else {
            None
        }
    }
}
