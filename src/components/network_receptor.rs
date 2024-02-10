use tui_realm_stdlib::{Container, Label, List, Phantom, Table};
use tuirealm::{
    props::{BorderType, Borders, Layout, Style, TextSpan},
    tui::layout::Constraint,
    Component, MockComponent,
};

use crate::app::network::UserEvent;

use super::Msg;

/// This component acts upon the state updates because of network events
#[derive(MockComponent)]
pub struct NetworkReceptor {
    component: Phantom,
}

impl Default for NetworkReceptor {
    fn default() -> Self {
        Self {
            component: Phantom::default(),
        }
    }
}

impl Component<Msg, UserEvent> for NetworkReceptor {
    fn on(&mut self, event: tuirealm::Event<UserEvent>) -> Option<Msg> {
        match event {
            tuirealm::Event::User(user_event) => match user_event {
                UserEvent::RoomCreated { room_id, users } => todo!(),
                UserEvent::GameStart => todo!(),
            },
            _ => None,
        }
        Some(Msg::NetworkUpdate)
    }
}
