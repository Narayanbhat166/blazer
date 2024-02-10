use tui_realm_stdlib::{Container, Label};
use tuirealm::{
    props::{Color, Layout},
    tui::layout::Constraint,
    Component, MockComponent,
};

use crate::app::network::UserEvent;

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
        match event {
            tuirealm::Event::User(user_event) => match user_event {
                UserEvent::InfoMessage(info_message) => {
                    self.set_text(info_message, MessageType::Success);
                }
                UserEvent::NetworkError(network_error) => {
                    self.set_text(network_error, MessageType::Error);
                }
                UserEvent::RoomCreated { .. } => {
                    let text_message = "Waiting for other players to join".to_string();

                    self.set_text(text_message, MessageType::Success);
                }
                UserEvent::GameStart => {
                    self.set_text("Game will start now".to_string(), MessageType::Info);
                }
            },
            _ => {}
        };
        Some(Msg::NetworkUpdate)
    }
}
