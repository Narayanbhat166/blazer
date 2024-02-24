use tui_realm_stdlib::{Container, List, Paragraph, Table};
use tuirealm::{
    command::Cmd,
    props::{BorderType, Layout, TextSpan},
    tui::layout::Constraint,
    Component, MockComponent,
};

use crate::app::{model, network::UserEvent};

use super::Msg;

#[derive(Default)]
pub struct OwnStates {
    pub users: Vec<model::UserDetails>,
    pub global_details: GlobalDetails,
    pub room_details: Option<RoomDetails>,
    pub is_in_waiting_room: bool,
    pub is_in_game: bool,
}

#[derive(Default, Clone)]
pub struct RoomDetails {
    room_id: String,
    max_players: u32,
    pub current_players: usize,
}

/// Based on the screen that is currently active, show relevant information
pub struct Details {
    component: tui_realm_stdlib::Container,
    state: OwnStates,
}

pub struct UiState {
    _users_list: Option<List>,
}

impl MockComponent for Details {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: tuirealm::tui::prelude::Rect) {
        if self.state.is_in_waiting_room {
            let users_list = get_users_list(self.state.users.clone());

            let room_details = get_room_details(self.state.room_details.clone());

            let current_selected_user_index = users_list.states.list_index;
            let current_selected_user = self
                .state
                .users
                .get(current_selected_user_index)
                .expect("Index out of bounds")
                .clone();

            let user_details = get_user_information_table(current_selected_user);

            self.component.children[0] = room_details;

            if self.component.children.get(1).is_some() {
                self.component.children[1] = Box::new(users_list);
            } else {
                self.component.children.push(Box::new(users_list))
            }

            if self.component.children.get(2).is_some() {
                self.component.children[2] = Box::new(user_details);
            } else {
                self.component.children.push(Box::new(user_details))
            }
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

#[derive(Default)]
pub struct GlobalDetails {
    active_games: u16,
    active_players: u16,
}

fn get_room_details(room_details: Option<RoomDetails>) -> Box<dyn MockComponent> {
    if let Some(room_details) = room_details {
        let first_row = vec![
            TextSpan::new("Room id"),
            TextSpan::new(room_details.room_id),
        ];

        let second_row = vec![
            TextSpan::new("Max Players"),
            TextSpan::new(room_details.max_players.to_string()),
        ];
        let thrid_row = vec![
            TextSpan::new("Current Players"),
            TextSpan::new(room_details.current_players.to_string()),
        ];
        let row_information = vec![first_row, second_row, thrid_row];

        Box::new(
            Table::default()
                .title("Room Details", tuirealm::props::Alignment::Left)
                .table(row_information),
        )
    } else {
        Box::new(
            Paragraph::default()
                .text(&[TextSpan::from("Join a room to display room details")])
                .alignment(tuirealm::props::Alignment::Center),
        )
    }
}

fn get_users_list(user_details: Vec<model::UserDetails>) -> List {
    let user_details = user_details
        .iter()
        .map(|user_details| vec![TextSpan::new(user_details.user_name.clone())])
        .collect::<Vec<_>>();

    List::default()
        .title("Users - [U]", tuirealm::props::Alignment::Left)
        .rows(user_details)
        .borders(tuirealm::props::Borders::default().modifiers(BorderType::Rounded))
        .rewind(true)
        .scroll(true)
        .highlighted_color(tuirealm::props::Color::Gray)
        .selected_line(0)
}

#[allow(dead_code)]
fn get_user_information_table(user: model::UserDetails) -> Table {
    // Display all the keys on col 1
    // Display all the values on col 2
    let first_row = vec![TextSpan::new("User Name"), TextSpan::new(user.user_name)];
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

#[allow(dead_code)]
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
        let room_details = get_room_details(None);

        let container = Container::default()
            .title("Room Details", tuirealm::props::Alignment::Center)
            .layout(
                Layout::default()
                    .constraints(&[
                        Constraint::Percentage(25),
                        Constraint::Percentage(50),
                        Constraint::Percentage(25),
                    ])
                    .margin(1)
                    .direction(tuirealm::tui::layout::Direction::Vertical),
            )
            .children(vec![room_details]);

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
                    let current_players = users.len();

                    self.state.users = users
                        .into_iter()
                        .map(model::UserDetails::from)
                        .collect::<Vec<_>>();

                    let room_details = RoomDetails {
                        room_id,
                        max_players: 2,
                        current_players,
                    };

                    self.state.room_details = Some(room_details);

                    self.state.is_in_waiting_room = true;
                    None
                }
                UserEvent::UserJoined { users } => {
                    if let Some(room_details) = self.state.room_details.as_mut() {
                        room_details.current_players = users.len();
                    }

                    self.state.users = users
                        .into_iter()
                        .map(model::UserDetails::from)
                        .collect::<Vec<_>>();

                    None
                }
                _ => None,
            },
            _ => None,
        }
    }
}
