use tui_realm_stdlib::{Container, Label, List, Phantom, Table};
use tuirealm::{
    props::{BorderType, Borders, Layout, Style, TextSpan},
    tui::layout::Constraint,
    Component, MockComponent,
};

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
    fn on(&mut self, event: tuirealm::Event<UserEvent>) -> Option<Msg> {
        None
    }
}
