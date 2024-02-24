use tui_realm_stdlib::Phantom;
use tuirealm::{Component, MockComponent};

use crate::app::{model::AppStateUpdate, network::UserEvent};

use super::Msg;

/// This component acts upon the state updates because of network events
#[derive(MockComponent, Default)]
pub struct NetworkReceptor {
    component: Phantom,
}

impl Component<Msg, UserEvent> for NetworkReceptor {
    fn on(&mut self, event: tuirealm::Event<UserEvent>) -> Option<Msg> {
        match event {
            tuirealm::Event::User(user_event) => match user_event {
                UserEvent::RoomCreated { room_id, users } => {
                    let users = users.into_iter().map(Into::into).collect::<Vec<_>>();

                    let app_state_update = AppStateUpdate::RoomUpdate { room_id, users };
                    Some(Msg::StateUpdate(app_state_update))
                }

                UserEvent::GameStart => None,
                UserEvent::InfoMessage(_) => todo!(),
                UserEvent::NetworkError(_) => todo!(),
                UserEvent::UserJoined { users } => {
                    let users = users.into_iter().map(Into::into).collect::<Vec<_>>();

                    let app_state_update = AppStateUpdate::UserRoomJoin { users };
                    Some(Msg::StateUpdate(app_state_update))
                }
            },
            _ => None,
        }
    }
}
