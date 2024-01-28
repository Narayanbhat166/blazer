use tui_realm_stdlib::{Input, Label, Paragraph, Radio};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent, KeyModifiers},
    props::{BorderType, TextSpan},
    tui::layout as tui_layout,
    Component, Event, MockComponent,
};

use crate::{app::network::UserEvent, components::Msg};

#[derive(Default, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Menus {
    #[default]
    NewGame = 0,
    CreateRoom = 1,
    JoinRoom = 2,
}

impl Menus {
    fn from_u8(int_value: u8) -> Self {
        match int_value {
            0 => Self::NewGame,
            1 => Self::CreateRoom,
            2 => Self::JoinRoom,
            _ => panic!("Unexpected value received when converting u8 to menus"),
        }
    }
}

impl ToString for Menus {
    fn to_string(&self) -> String {
        match self {
            Menus::NewGame => String::from("New Game"),
            Menus::CreateRoom => String::from("Create Room"),
            Menus::JoinRoom => String::from("Join Room"),
        }
    }
}

pub struct Menu {
    component: Radio,
    input_field: Input,
    helper_label: Paragraph,
}

impl MockComponent for Menu {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: tuirealm::tui::prelude::Rect) {
        let chunks = tui_layout::Layout::default()
            .direction(tui_layout::Direction::Horizontal)
            .constraints([
                tui_layout::Constraint::Percentage(60),
                tui_layout::Constraint::Percentage(40),
            ])
            .split(area);

        self.component.view(frame, chunks[0]);

        let menu_option = Menus::from_u8(self.component.states.choice as u8);
        if matches!(menu_option, Menus::CreateRoom | Menus::NewGame) {
            self.helper_label.view(frame, chunks[1])
        } else {
            self.input_field.view(frame, chunks[1])
        }
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        self.component.query(attr)
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        self.component.attr(attr, value)
    }

    fn state(&self) -> tuirealm::State {
        self.component.state()
    }

    fn perform(&mut self, cmd: Cmd) -> tuirealm::command::CmdResult {
        self.component.perform(cmd)
    }
}

impl Menu {
    pub fn default() -> Self {
        let choices = [Menus::NewGame, Menus::CreateRoom, Menus::JoinRoom]
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>();
        let component = Radio::default()
            .choices(&choices)
            .borders(tuirealm::props::Borders::default().modifiers(BorderType::Rounded))
            .title("Menu", tuirealm::props::Alignment::Left);

        let input_field = Input::default().title("Enter room id", tui_layout::Alignment::Left);
        let helper_label = Paragraph::default()
            .text(&[TextSpan::from(
                "Create a game with random players who are online",
            )])
            .borders(tuirealm::props::Borders::default().modifiers(BorderType::Rounded));

        Self {
            component,
            input_field,
            helper_label,
        }
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

            Event::Keyboard(KeyEvent {
                code: Key::Delete,
                modifiers: KeyModifiers::NONE,
            }) => Cmd::Delete,

            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => Cmd::Delete,

            Event::Keyboard(KeyEvent {
                code: Key::Char(character),
                modifiers: KeyModifiers::NONE,
            }) => Cmd::Type(character),

            _ => Cmd::None,
        };

        // If the present state is to join the room, then forward the command to input field
        let menu_state = Menus::from_u8(self.component.states.choice as u8);
        if matches!(menu_state, Menus::JoinRoom) {
            let cmd_result = self.input_field.perform(cmd);

            match cmd_result {
                CmdResult::Changed(_) => Some(Msg::StateUpdate),
                _ => None,
            }
        } else {
            match self.perform(cmd) {
                tuirealm::command::CmdResult::Changed(_) => {
                    let menu_state = Menus::from_u8(self.component.states.choice as u8);
                    match menu_state {
                        Menus::NewGame => {
                            let helper_text = Paragraph::default()
                                .text(&[TextSpan::from(
                                    "Create a game with random players who are online",
                                )])
                                .borders(
                                    tuirealm::props::Borders::default()
                                        .modifiers(BorderType::Rounded),
                                );
                            self.helper_label = helper_text;
                        }
                        Menus::CreateRoom => {
                            let helper_text = Paragraph::default()
                                .text(&[TextSpan::from(
                                    "Create a private room, invite your friends",
                                )])
                                .borders(
                                    tuirealm::props::Borders::default()
                                        .modifiers(BorderType::Rounded),
                                );
                            self.helper_label = helper_text;
                        }
                        Menus::JoinRoom => {}
                    }
                    Some(Msg::StateUpdate)
                }
                tuirealm::command::CmdResult::Submit(_) => {
                    let menu_state = Menus::from_u8(self.component.states.choice as u8);
                    Some(Msg::SelectMenu(menu_state))
                }
                tuirealm::command::CmdResult::None => None,
                _ => None,
            }
        }
    }
}
