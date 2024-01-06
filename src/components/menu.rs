use tui_realm_stdlib::Radio;
use tuirealm::{
    command::Cmd,
    event::{Key, KeyEvent, KeyModifiers},
    props::BorderType,
    Component, Event, MockComponent,
};

use crate::{app::network::UserEvent, components::Msg};

#[derive(MockComponent)]
pub struct Menu {
    component: Radio,
}

impl Menu {
    pub fn new<S: AsRef<str>>(choices: &[S]) -> Self {
        let component = Radio::default()
            .choices(choices)
            .borders(tuirealm::props::Borders::default().modifiers(BorderType::Rounded))
            .title("Menu", tuirealm::props::Alignment::Left);

        Self { component }
    }
}

impl Component<Msg, UserEvent> for Menu {
    fn on(&mut self, event: tuirealm::Event<UserEvent>) -> Option<Msg> {
        let cmd = match event {
            Event::Keyboard(KeyEvent {
                code: Key::Left,
                modifiers: KeyModifiers::NONE,
            }) => Cmd::Move(tuirealm::command::Direction::Left),

            Event::Keyboard(KeyEvent {
                code: Key::Right,
                modifiers: KeyModifiers::NONE,
            }) => Cmd::Move(tuirealm::command::Direction::Right),

            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => Cmd::Submit,

            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::AppClose),

            _ => Cmd::None,
        };

        match self.perform(cmd) {
            tuirealm::command::CmdResult::Changed(_) => Some(Msg::StateUpdate),
            tuirealm::command::CmdResult::Submit(_) => Some(Msg::SelectMenu),
            tuirealm::command::CmdResult::None => None,
            _ => None,
        }
    }
}
