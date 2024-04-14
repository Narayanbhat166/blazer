use tui_realm_stdlib::{Input, Paragraph, Radio};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent, KeyModifiers},
    props::{BorderType, TextSpan},
    tui::layout as tui_layout,
    Component, Event, MockComponent, StateValue,
};

use super::{Msg, UserEvent};

use crate::app::client::types::{MenuMessage, MenuSelection};

#[derive(Default, Debug, PartialEq, Eq)]
#[repr(u8)]
enum Menus {
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

    fn get_helper_text(&self) -> &str {
        match self {
            Menus::NewGame => "Create a game with random players who are online",
            Menus::CreateRoom => "Create a private room, invite your friends",
            Menus::JoinRoom => "Join a private room",
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
    is_input_field_active: bool,
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

        if self.is_input_field_active {
            self.input_field.view(frame, chunks[1]);
        } else {
            self.helper_label.view(frame, chunks[1])
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

impl Default for Menu {
    fn default() -> Self {
        let choices = [Menus::NewGame, Menus::CreateRoom, Menus::JoinRoom]
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>();
        let component = Radio::default()
            .choices(&choices)
            .borders(
                tuirealm::props::Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(tuirealm::props::Color::Green),
            )
            .title("Menu - [ M ]", tuirealm::props::Alignment::Left);

        let input_field = Input::default()
            .title("Enter room id", tui_layout::Alignment::Left)
            .borders(
                tuirealm::props::Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(tuirealm::props::Color::Green),
            )
            .input_type(tuirealm::props::InputType::Number);

        let helper_label = Paragraph::default()
            .text(&[TextSpan::from(
                "Create a game with random players who are online",
            )])
            .borders(tuirealm::props::Borders::default().modifiers(BorderType::Rounded));

        Self {
            component,
            input_field,
            helper_label,
            is_input_field_active: false,
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

        // If the input field is active, then forward the command to input field
        if self.is_input_field_active {
            let cmd_result = self.input_field.perform(cmd);

            match cmd_result {
                CmdResult::Changed(_) => Some(Msg::Menu(MenuMessage::MenuDataChange)),
                CmdResult::Submit(submit_state) => {
                    let input_state = submit_state.unwrap_one();
                    if let StateValue::String(room_id) = input_state {
                        Some(Msg::Menu(MenuMessage::MenuSelect(
                            MenuSelection::JoinRoom { room_id },
                        )))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            match self.perform(cmd) {
                tuirealm::command::CmdResult::Changed(_) => {
                    let menu_state = Menus::from_u8(self.component.states.choice as u8);
                    let menu_text = menu_state.get_helper_text();
                    let helper_text = Paragraph::default()
                        .text(&[TextSpan::from(menu_text)])
                        .borders(
                            tuirealm::props::Borders::default().modifiers(BorderType::Rounded),
                        );
                    self.helper_label = helper_text;
                    Some(Msg::Menu(MenuMessage::MenuChange))
                }
                tuirealm::command::CmdResult::Submit(_) => {
                    let menu_state = Menus::from_u8(self.component.states.choice as u8);

                    let menu_update = match menu_state {
                        Menus::NewGame => MenuMessage::MenuSelect(MenuSelection::NewGame),
                        Menus::CreateRoom => MenuMessage::MenuSelect(MenuSelection::CreateRoom),
                        Menus::JoinRoom => {
                            self.is_input_field_active = true;
                            self.input_field
                                .attr(tuirealm::Attribute::Focus, tuirealm::AttrValue::Flag(true));

                            self.component
                                .attr(tuirealm::Attribute::Focus, tuirealm::AttrValue::Flag(false));
                            MenuMessage::MenuChange
                        }
                    };
                    Some(Msg::Menu(menu_update))
                }
                _ => None,
            }
        }
    }
}
