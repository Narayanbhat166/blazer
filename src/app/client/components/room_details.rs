use tui_realm_stdlib::{List, Paragraph, Table};
use tuirealm::{
    command::{Cmd, CmdResult},
    props::{BorderType, Layout, TextSpan},
    tui::layout::{Constraint, Rect},
    Component, MockComponent,
};

use crate::app::client::types::UserDetails;

use super::{Msg, UserEvent};

#[derive(Default)]
pub struct OwnStates {
    pub users: Vec<UserDetails>,
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

pub struct CustomLayout {
    room_details: Rect,
    user_list: Rect,
    user_details: Rect,
}

impl CustomLayout {
    fn new(main_screen: Rect) -> Self {
        let layout = Layout::default()
            .constraints(&[
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .direction(tuirealm::tui::layout::Direction::Vertical)
            .chunks(main_screen);

        Self {
            room_details: layout[0],
            user_list: layout[1],
            user_details: layout[2],
        }
    }
}

/// Based on the screen that is currently active, show relevant information
pub struct Details {
    room_details: Box<dyn MockComponent>,
    user_list: Option<List>,
    user_information: Option<Table>,
    state: OwnStates,
}

impl MockComponent for Details {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: tuirealm::tui::prelude::Rect) {
        let room_details = get_room_details(self.state.room_details.clone());
        self.room_details = room_details;

        if self.state.is_in_waiting_room {
            let layout = CustomLayout::new(area);

            let users_list = get_users_list(self.state.users.clone());

            let current_selected_user_index = users_list.states.list_index;
            let current_selected_user = self
                .state
                .users
                .get(current_selected_user_index)
                .expect("Index out of bounds")
                .clone();

            let user_details = get_user_information_table(current_selected_user);
            self.user_information = Some(user_details);
            self.user_list = Some(users_list);

            if let Some(ref mut user_list) = self.user_list {
                user_list.view(frame, layout.user_list);
            }

            if let Some(ref mut user_information) = self.user_information {
                user_information.view(frame, layout.user_details);
            }

            self.room_details.view(frame, layout.room_details);
        } else {
            self.room_details.view(frame, area);
        }
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        self.room_details.query(attr)
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        self.room_details.attr(attr, value)
    }

    fn state(&self) -> tuirealm::State {
        self.room_details.state()
    }

    fn perform(&mut self, cmd: Cmd) -> tuirealm::command::CmdResult {
        if let Some(ref mut user_list) = self.user_list {
            user_list.perform(cmd)
        } else {
            CmdResult::None
        }
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
                .title("Room Details", tuirealm::props::Alignment::Center)
                .text(&[TextSpan::from("Join a room to display room information")])
                .alignment(tuirealm::props::Alignment::Center),
        )
    }
}

fn get_users_list(user_details: Vec<UserDetails>) -> List {
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
fn get_user_information_table(user: UserDetails) -> Table {
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

        Self {
            room_details,
            user_list: None,
            user_information: None,
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

                    self.state.users = users.into_iter().map(UserDetails::from).collect::<Vec<_>>();

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

                    self.state.users = users.into_iter().map(UserDetails::from).collect::<Vec<_>>();

                    None
                }
                _ => None,
            },
            _ => None,
        }
    }
}
