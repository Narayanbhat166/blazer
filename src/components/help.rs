use tui_realm_stdlib::Table;
use tuirealm::{props::TextSpan, Component, MockComponent};

use crate::app::network::UserEvent;

use super::Msg;

#[derive(MockComponent)]
pub struct Help {
    component: Table,
}

impl Default for Help {
    fn default() -> Self {
        let component = Table::default()
            .title("Navigation", tuirealm::props::Alignment::Center)
            .table(vec![
                vec![TextSpan::from("Arrow keys"), TextSpan::from("Navigate")],
                vec![TextSpan::from("Return / Enter"), TextSpan::from("Select")],
                vec![TextSpan::from("M / m"), TextSpan::from("Menu")],
            ]);

        Self { component }
    }
}

impl Component<Msg, UserEvent> for Help {
    fn on(&mut self, _event: tuirealm::Event<UserEvent>) -> Option<Msg> {
        // Change navigation information based on the user action
        None
    }
}
