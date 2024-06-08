/// This file contains the application model
use std::sync::mpsc;
use std::time::Duration;

use tuirealm::terminal::TerminalBridge;
use tuirealm::{Application, EventListenerCfg, Update};

use crate::app::client::{
    components,
    types::{self, Id, Msg},
};

use super::{
    layout,
    network::{types::UserEvent, NetworkClient},
};

use super::network;
use super::types::ClientConfig;

pub struct Model {
    /// Application
    pub app: Application<Id, Msg, UserEvent>,
    /// Indicates that the application must quit
    pub grpc_channel: mpsc::Sender<network::types::Request>,
    pub quit: bool,
    /// Tells whether to redraw interface
    pub redraw: bool,
    /// Used to draw to terminal
    pub terminal: TerminalBridge,
    /// State of the application
    pub state: types::AppState,
    /// In order to safely close any open connections
    pub network_join_handler: Option<std::thread::JoinHandle<()>>,
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ClientArgs {
    /// Whether new users should be created
    /// If not passed, then use the existing user key in ./local/state/blazerapp.toml
    #[arg(short, long, default_value_t = false)]
    pub create_guest: bool,
}

impl Model {
    pub fn new(config: ClientConfig, args: ClientArgs) -> Self {
        let (grpc_sender, grpc_receiver) = mpsc::channel::<network::types::Request>();
        // start the network client

        let mut network_client = NetworkClient::default();
        let cloned_network_client = network_client.clone();

        let join_handler = std::thread::spawn(move || {
            network_client.start_network_client(grpc_receiver, config, args)
        });

        Self {
            app: Self::init_app(cloned_network_client),
            grpc_channel: grpc_sender,
            quit: false,
            redraw: true,
            terminal: TerminalBridge::new().expect("Cannot initialize terminal"),
            state: types::AppState::default(),
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

        app.mount(
            Id::Menu,
            Box::<components::menu::Menu>::default(),
            Vec::default(),
        )
        .unwrap();

        app.mount(
            Id::BottomBar,
            Box::<components::bottom_bar::BottomBar>::default(),
            vec![tuirealm::Sub::new(
                tuirealm::SubEventClause::Any,
                tuirealm::SubClause::Always,
            )],
        )
        .unwrap();

        app.mount(
            Id::RoomDetails,
            Box::<components::room_details::Details>::default(),
            vec![tuirealm::Sub::new(
                tuirealm::SubEventClause::Any,
                tuirealm::SubClause::Always,
            )],
        )
        .unwrap();

        app.mount(
            Id::Help,
            Box::<components::help::Help>::default(),
            Vec::default(),
        )
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
                    self.grpc_channel
                        .send(network::types::Request::Quit)
                        .unwrap();
                    if let Some(network_join_handler) = self.network_join_handler.take() {
                        network_join_handler.join().unwrap();
                    }
                    None
                }
                Msg::BottomBarUpdate | Msg::ReDraw => None,
                Msg::Menu(menu_message) => {
                    let network_request = match menu_message {
                        types::MenuMessage::MenuChange | types::MenuMessage::MenuDataChange => None,
                        types::MenuMessage::MenuSelect(menu_selection) => {
                            Some(network::types::NewRequestEntity::from(menu_selection))
                        }
                    };
                    if let Some(network_request) = network_request {
                        self.grpc_channel
                            .send(network::types::Request::New(network_request))
                            .unwrap();
                    }

                    None
                }
                Msg::StateUpdate(state_update) => {
                    match &state_update {
                        types::AppStateUpdate::UserIdUpdate { .. } => {}
                        types::AppStateUpdate::RoomUpdate { .. } => {
                            // This message should be received only once
                            self.app.active(&Id::RoomDetails).unwrap();
                        }
                        types::AppStateUpdate::UserRoomJoin { .. } => {}
                        types::AppStateUpdate::GameStart => todo!(),
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
