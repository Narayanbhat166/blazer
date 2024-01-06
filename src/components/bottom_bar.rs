use tui_realm_stdlib::Span;
use tuirealm::{props::TextSpan, Component, NoUserEvent};

use super::Msg;

impl Component<Msg, NoUserEvent> for Span {
    fn on(&mut self, _: tuirealm::Event<NoUserEvent>) -> Option<Msg> {
        Some(Msg::StateUpdate)
    }
}
