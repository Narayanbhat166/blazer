use std::sync::mpsc;
use std::time::Duration;

use tuirealm::terminal::TerminalBridge;
use tuirealm::{Application, EventListenerCfg, Update};

use crate::components::menu::{self};
use crate::components::room_details::Details;
use crate::components::{help, menu::Menu, Id, Msg};
use crate::{
    app::{
        layout,
        network::{NetworkClient, UserEvent},
    },
    components::bottom_bar::BottomBar,
};

use super::network;
use super::types::ClientConfig;

#[derive(Debug, PartialEq, Clone)]
pub struct UserDetails {
    pub user_id: String,
    pub user_name: String,
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
                // let game_data = GameState {
                //     room_id: todo!(),
                //     game_id: todo!(),
                //     has_started: todo!(),
                // };

                self
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
    /// In order to safely close any open connections
    pub network_join_handler: Option<std::thread::JoinHandle<()>>,
}

impl Model {
    pub fn new(config: ClientConfig) -> Self {
        let (grpc_sender, grpc_receiver) = mpsc::channel::<network::Request>();
        // start the network client

        let mut network_client = NetworkClient::default();
        let cloned_network_client = network_client.clone();

        let join_handler =
            std::thread::spawn(move || network_client.start_network_client(grpc_receiver, config));

        Self {
            app: Self::init_app(cloned_network_client),
            grpc_channel: grpc_sender,
            quit: false,
            redraw: true,
            terminal: TerminalBridge::new().expect("Cannot initialize terminal"),
            state: AppState::default(),
            network_join_handler: Some(join_handler),
        }
    }
}

impl Model {
    pub fn view(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let custom_layout = layout::CustomLayout::new(f.size());

                self.app.view(&Id::RoomDetails, f, custom_layout.details);

                self.app.view(&Id::Help, f, custom_layout.navigation);

                self.app.view(&Id::Menu, f, custom_layout.menu);
                self.app.view(&Id::BottomBar, f, custom_layout.bottom_bar);
            })
            .unwrap();
    }

    fn init_app(network_client: NetworkClient) -> Application<Id, Msg, UserEvent> {
        let mut app: Application<Id, Msg, UserEvent> = Application::init(
            EventListenerCfg::default()
                .default_input_listener(Duration::from_millis(20))
                .port(Box::new(network_client), Duration::from_millis(10))
                .poll_timeout(Duration::from_millis(10))
                .tick_interval(Duration::from_secs(1)),
        );

        app.mount(Id::Menu, Box::<Menu>::default(), Vec::default())
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

        // Activate the menu
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
                    self.grpc_channel.send(network::Request::Quit).unwrap();
                    if let Some(network_join_handler) = self.network_join_handler.take() {
                        network_join_handler.join().unwrap();
                    }
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
                    match &state_update {
                        AppStateUpdate::UserIdUpdate { .. } => {}
                        AppStateUpdate::RoomUpdate { .. } => {
                            // This message should be received only once
                            self.app.active(&Id::RoomDetails).unwrap();
                        }
                        AppStateUpdate::UserRoomJoin { .. } => {}
                        AppStateUpdate::GameStart => todo!(),
                    }
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
