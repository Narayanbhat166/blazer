/// The bottom bar is used to show network activity
/// After every user event, the bottom bar is updated with the response of the network activity
use tui_realm_stdlib::{Container, Label};
use tuirealm::{
    props::{Color, Layout},
    tui::layout::Constraint,
    Component, MockComponent,
};

use crate::app::client::network::types::UserEvent;

use super::Msg;

enum MessageType {
    Success,
    Info,
    Error,
}

impl MessageType {
    fn get_color(&self) -> Color {
        match self {
            MessageType::Info => Color::Gray,
            MessageType::Error => Color::Red,
            MessageType::Success => Color::Green,
        }
    }
}

#[derive(MockComponent)]
pub struct BottomBar {
    component: tui_realm_stdlib::Container,
}

impl Default for BottomBar {
    fn default() -> Self {
        let container = Container::default()
            .title("Network Logs", tuirealm::props::Alignment::Left)
            .layout(
                Layout::default()
                    .constraints(&[Constraint::Percentage(100)])
                    .direction(tuirealm::tui::layout::Direction::Horizontal)
                    .margin(1),
            )
            .children(vec![Box::<Label>::default()]);

        Self {
            component: container,
        }
    }
}

impl BottomBar {
    fn set_text(&mut self, text: String, message_type: MessageType) {
        let text_field = Box::new(
            Label::default()
                .text(text)
                .foreground(message_type.get_color()),
        );
        self.component.children[0] = text_field;
    }
}

impl Component<Msg, UserEvent> for BottomBar {
    fn on(&mut self, event: tuirealm::Event<UserEvent>) -> Option<Msg> {
        if let tuirealm::Event::User(user_event) = event {
            match user_event {
                UserEvent::InfoMessage(info_message) => {
                    self.set_text(info_message, MessageType::Success);
                }
                UserEvent::NetworkError(network_error) => {
                    self.set_text(network_error, MessageType::Error);
                }
                UserEvent::RoomCreated { room_id, .. } => {
                    let text_message =
                        format!("Joined room with id {room_id}. Waiting for other players to join");

                    self.set_text(text_message, MessageType::Success);
                }
                UserEvent::GameStart { .. } => {
                    self.set_text(
                        "All users have joined, game will start now".to_string(),
                        MessageType::Info,
                    );
                }
                UserEvent::UserJoined { users } => {
                    let text = format!(
                        "New user has joined the party, the number of users are {}",
                        users.len()
                    );
                    self.set_text(text, MessageType::Info)
                }
            }
        };
        Some(Msg::BottomBarUpdate)
    }
}
