use tui_realm_stdlib::{Container, Label, List, Table};
use tuirealm::{
    props::{BorderType, Borders, Layout, Style, TextSpan},
    tui::layout::Constraint,
    Component, MockComponent,
};

use crate::app::network::UserEvent;

use super::Msg;

#[derive(Default)]
pub struct OwnStates {
    pub users: Vec<UserDetails>,
    pub global_details: GlobalDetails,
    pub room_details: RoomDetails,
}

#[derive(Default)]
pub struct RoomDetails {
    room_id: Option<String>,
}

/// Based on the screen that is currently active, show relevant information
#[derive(MockComponent)]
pub struct Details {
    component: tui_realm_stdlib::Container,
    state: OwnStates,
}

pub struct UserDetails {
    user_name: String,
    games_played: Option<u16>,
    rank: Option<u16>,
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

fn get_users_list(user_details: Vec<UserDetails>) -> List {
    let user_details = user_details
        .iter()
        .map(|user_details| TextSpan::new(user_details.user_name.clone()))
        .collect::<Vec<_>>();

    List::default()
        .title("Users in the room", tuirealm::props::Alignment::Left)
        .rows(vec![user_details])
        .borders(tuirealm::props::Borders::default().modifiers(BorderType::Rounded))
}

fn get_user_information_table(user: UserDetails) -> Table {
    // Display all the keys on col 1
    // Display all the values on col 2
    let first_row = vec![TextSpan::new("User Name"), TextSpan::new(user.user_name)];
    let second_row = vec![
        TextSpan::new("Games Played"),
        TextSpan::new(user.games_played.unwrap_or_default().to_string()),
    ];
    let thrid_row = vec![
        TextSpan::new("Rank"),
        TextSpan::new(user.rank.unwrap_or_default().to_string()),
    ];
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
            .borders(tuirealm::props::Borders::default().modifiers(BorderType::Rounded));

        let users_information =
            Table::default().title("User Information", tuirealm::props::Alignment::Left);

        let global_stats = get_global_information(GlobalDetails::default());

        let container = Container::default()
            .title("Details", tuirealm::props::Alignment::Center)
            .layout(
                Layout::default()
                    .constraints(&[
                        Constraint::Percentage(50),
                        Constraint::Percentage(30),
                        Constraint::Percentage(20),
                    ])
                    .margin(1)
                    .direction(tuirealm::tui::layout::Direction::Horizontal),
            )
            .children(vec![Box::new(global_stats)]);

        Self {
            component: container,
            state: OwnStates::default(),
        }
    }
}

impl Component<Msg, UserEvent> for Details {
    fn on(&mut self, event: tuirealm::Event<UserEvent>) -> Option<Msg> {
        // match event {
        //     // UserEvent::RoomCreated { room_id } => {
        //     //     let
        //     // }
        // }
        Some(Msg::NetworkUpdate)
    }
}
