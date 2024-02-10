use tui_realm_stdlib::{Container, Label, List, Phantom, Table};
use tuirealm::{
    props::{BorderType, Borders, Layout, Style, TextSpan},
    tui::layout::Constraint,
    Component, MockComponent,
};

use crate::app::network::UserEvent;

use super::Msg;

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
        Some(Msg::NetworkUpdate)
    }
}
