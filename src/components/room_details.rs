use tui_realm_stdlib::{Container, Label, List, Table};
use tuirealm::{
    command::Cmd,
    props::{BorderType, Borders, Layout, Style, TextSpan},
    tui::layout::Constraint,
    Component, MockComponent,
};

use crate::app::{model, network::UserEvent};

use super::Msg;

#[derive(Default)]
pub struct OwnStates {
    pub users: Vec<model::UserDetails>,
    pub global_details: GlobalDetails,
    pub room_details: RoomDetails,
    pub is_in_waiting_room: bool,
    pub is_in_game: bool,
}

#[derive(Default)]
pub struct RoomDetails {
    room_id: Option<String>,
}

/// Based on the screen that is currently active, show relevant information
pub struct Details {
    component: tui_realm_stdlib::Container,
    state: OwnStates,
}

impl MockComponent for Details {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: tuirealm::tui::prelude::Rect) {
        if self.state.is_in_waiting_room {
            let users_list = get_users_list(self.state.users.clone());
            self.component.children[1] = Box::new(users_list);
        }

        self.component.view(frame, area);
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

pub struct GlobalDetails {
    active_games: u16,
    active_players: u16,
}

impl Default for GlobalDetails {
    fn default() -> Self {
        Self {
            active_games: Default::default(),
            active_players: Default::default(),
        }
    }
}

fn get_users_list(user_details: Vec<model::UserDetails>) -> List {
    let user_details = user_details
        .iter()
        .map(|user_details| {
            vec![TextSpan::new(
                user_details
                    .user_name
                    .clone()
                    .unwrap_or("Anonymous".to_string()),
            )]
        })
        .collect::<Vec<_>>();

    List::default()
        .title("Users in the room", tuirealm::props::Alignment::Left)
        .rows(user_details)
        .borders(tuirealm::props::Borders::default().modifiers(BorderType::Rounded))
        .rewind(true)
        .scroll(true)
        .highlighted_color(tuirealm::props::Color::Gray)
}

fn get_user_information_table(user: model::UserDetails) -> Table {
    // Display all the keys on col 1
    // Display all the values on col 2
    let first_row = vec![
        TextSpan::new("User Name"),
        TextSpan::new(user.user_name.unwrap_or("Anonymous".to_string())),
    ];
    let second_row = vec![
        TextSpan::new("Games Played"),
        TextSpan::new(user.games_played.to_string()),
    ];
    let thrid_row = vec![TextSpan::new("Rank"), TextSpan::new(user.rank.to_string())];
    let row_information = vec![first_row, second_row, thrid_row];

    Table::default()
        .title("User Information", tuirealm::props::Alignment::Left)
        .table(row_information)
}

fn get_global_information(global_details: GlobalDetails) -> Table {
    // Display all the keys on col 1
    // Display all the values on col 2
    let first_row = vec![
        TextSpan::new("Currently active players"),
        TextSpan::new(global_details.active_players.to_string()),
    ];

    let second_row = vec![
        TextSpan::new("Currently active games"),
        TextSpan::new(global_details.active_games.to_string()),
    ];

    let row_information = vec![first_row, second_row];

    Table::default()
        .title("Global Stats", tuirealm::props::Alignment::Left)
        .table(row_information)
}

impl Default for Details {
    fn default() -> Self {
        let users_list = List::default()
            .title("Users in the room", tuirealm::props::Alignment::Left)
            .borders(tuirealm::props::Borders::default().modifiers(BorderType::Rounded))
            .rows(vec![vec![TextSpan::new("Please join a room")]]);

        let users_information =
            Table::default().title("User Information", tuirealm::props::Alignment::Left);

        let global_stats = get_global_information(GlobalDetails::default());

        let container = Container::default()
            .title("Details", tuirealm::props::Alignment::Center)
            .layout(
                Layout::default()
                    .constraints(&[
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(60),
                    ])
                    .margin(1)
                    .direction(tuirealm::tui::layout::Direction::Horizontal),
            )
            .children(vec![Box::new(global_stats), Box::new(users_list)]);

        Self {
            component: container,
            state: OwnStates::default(),
        }
    }
}

impl Component<Msg, UserEvent> for Details {
    fn on(&mut self, event: tuirealm::Event<UserEvent>) -> Option<Msg> {
        match event {
            tuirealm::Event::User(user_event) => match user_event {
                UserEvent::RoomCreated { room_id, users } => {
                    self.state.users = users
                        .into_iter()
                        .map(|user_details| model::UserDetails::from(user_details))
                        .collect::<Vec<_>>();

                    self.state.is_in_waiting_room = true;
                    None
                }
                UserEvent::UserJoined { users } => todo!(),
                _ => None,
            },
            _ => None,
        }
    }
}
