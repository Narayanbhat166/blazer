use std::sync::mpsc;
use std::time::Duration;

use tuirealm::terminal::TerminalBridge;
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::{Application, EventListenerCfg, Update};

use crate::components::menu::{self, MenuSelection};
use crate::components::room_details::Details;
use crate::components::{help, menu::Menu, Id, Msg};
use crate::{
    app::network::{NetworkClient, UserEvent},
    components::bottom_bar::BottomBar,
};

use super::network;
use super::types::ClientConfig;

#[derive(Debug, PartialEq, Clone)]
pub struct UserDetails {
    pub user_id: String,
    pub user_name: Option<String>,
    pub games_played: u32,
    pub rank: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RoomState {
    room_id: String,
    room_users: Vec<UserDetails>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GameState {
    room_id: String,
    game_id: String,
    has_started: bool,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct AppState {
    user_id: Option<String>,
    current_user: Option<UserDetails>,
    room_details: Option<RoomState>,
    game_details: Option<GameState>,
}

#[derive(Debug, PartialEq)]
pub enum AppStateUpdate {
    UserIdUpdate {
        user_id: String,
    },
    RoomUpdate {
        room_id: String,
        users: Vec<UserDetails>,
    },
    UserRoomJoin {
        users: Vec<UserDetails>,
    },
    GameStart,
}

impl AppState {
    fn apply_update(self, update: AppStateUpdate) -> Self {
        match update {
            AppStateUpdate::UserIdUpdate { user_id } => Self {
                user_id: Some(user_id),
                ..self
            },
            AppStateUpdate::RoomUpdate { room_id, users } => {
                let room_state = RoomState {
                    room_id,
                    room_users: users,
                };

                Self {
                    room_details: Some(room_state),
                    ..self
                }
            }
            AppStateUpdate::UserRoomJoin { users } => {
                let previous_room_state = self.room_details.expect(
                    "Message ordering is invalid. Expected room details before user room join",
                );

                let new_room_state = RoomState {
                    room_users: users,
                    ..previous_room_state
                };

                Self {
                    room_details: Some(new_room_state),
                    ..self
                }
            }
            AppStateUpdate::GameStart => {
                let game_data = GameState {
                    room_id: todo!(),
                    game_id: todo!(),
                    has_started: todo!(),
                };

                Self {
                    game_details: Some(game_data),
                    ..self
                }
            }
        }
    }
}

pub struct Model {
    /// Application
    pub app: Application<Id, Msg, UserEvent>,
    /// Indicates that the application must quit
    pub grpc_channel: mpsc::Sender<network::Request>,
    pub quit: bool,
    /// Tells whether to redraw interface
    pub redraw: bool,
    /// Used to draw to terminal
    pub terminal: TerminalBridge,
    /// State of the application
    pub state: AppState,
}

impl Model {
    pub fn new(config: ClientConfig) -> Self {
        let (grpc_sender, grpc_receiver) = mpsc::channel::<network::Request>();
        // start the network client

        let mut network_client = NetworkClient::default();
        let cloned_network_client = network_client.clone();

        std::thread::spawn(move || network_client.start_network_client(grpc_receiver, config));

        Self {
            app: Self::init_app(cloned_network_client),
            grpc_channel: grpc_sender,
            quit: false,
            redraw: true,
            terminal: TerminalBridge::new().expect("Cannot initialize terminal"),
            state: AppState::default(),
        }
    }
}

impl Model {
    pub fn view(&mut self) {
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3), // Menu
                            Constraint::Min(10),   // Action area
                            Constraint::Length(3), // Bottom bar
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let middle_chunk = chunks[1];
                let middle_parts = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
                    .split(middle_chunk);

                let middle_half_split = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(middle_parts[1]);

                self.app
                    .view(&Id::RoomDetails, f, *middle_half_split.first().unwrap());

                self.app
                    .view(&Id::Help, f, *middle_half_split.last().unwrap());

                self.app.view(&Id::Menu, f, *chunks.first().unwrap());
                self.app.view(&Id::BottomBar, f, *chunks.last().unwrap());
            })
            .is_ok());
    }

    fn init_app(network_client: NetworkClient) -> Application<Id, Msg, UserEvent> {
        let mut app: Application<Id, Msg, UserEvent> = Application::init(
            EventListenerCfg::default()
                .default_input_listener(Duration::from_millis(20))
                .port(Box::new(network_client), Duration::from_millis(10))
                .poll_timeout(Duration::from_millis(10))
                .tick_interval(Duration::from_secs(1)),
        );

        app.mount(Id::Menu, Box::new(Menu::default()), Vec::default())
            .unwrap();

        app.mount(
            Id::BottomBar,
            Box::<BottomBar>::default(),
            vec![tuirealm::Sub::new(
                tuirealm::SubEventClause::Any,
                tuirealm::SubClause::Always,
            )],
        )
        .unwrap();

        app.mount(
            Id::RoomDetails,
            Box::<Details>::default(),
            vec![tuirealm::Sub::new(
                tuirealm::SubEventClause::Any,
                tuirealm::SubClause::Always,
            )],
        )
        .unwrap();

        app.mount(Id::Help, Box::<help::Help>::default(), Vec::default())
            .unwrap();

        // Active the menu
        assert!(app.active(&Id::Menu).is_ok());
        app
    }
}

impl Update<Msg> for Model {
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            self.redraw = true;
            match msg {
                Msg::AppClose => {
                    self.quit = true;
                    None
                }
                Msg::NetworkUpdate | Msg::ReDraw => None,
                Msg::Menu(menu_message) => {
                    let network_request = match menu_message {
                        menu::MenuMessage::MenuChange | menu::MenuMessage::MenuDataChange => None,
                        menu::MenuMessage::MenuSelect(menu_selection) => {
                            Some(network::NewRequestEntity::from(menu_selection))
                        }
                    };
                    if let Some(network_request) = network_request {
                        self.grpc_channel
                            .send(network::Request::New(network_request))
                            .unwrap();
                    }

                    None
                }
                Msg::StateUpdate(state_update) => {
                    let new_state = self.state.clone().apply_update(state_update);
                    self.state = new_state;
                    None
                }
            }
        } else {
            None
        }
    }
}
