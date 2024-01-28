use std::sync::mpsc;
use std::time::Duration;

use tuirealm::terminal::TerminalBridge;
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::{Application, EventListenerCfg, Update};

use crate::components::{menu::Menu, Id, Msg};
use crate::{
    app::network::{NetworkClient, UserEvent},
    components::bottom_bar::BottomBar,
};

use super::network;
use super::types::ClientConfig;

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
                self.app.view(&Id::Menu, f, chunks[0]);
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

        // Active the menu
        assert!(app.active(&Id::Menu).is_ok());
        app
    }
}

impl Update<Msg> for Model {
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            // Set redraw
            self.redraw = true;
            // Match message
            match msg {
                Msg::AppClose => {
                    self.quit = true; // Terminate
                    None
                }
                Msg::Clock => None,
                Msg::StateUpdate => None,
                Msg::PingServer => None,
                Msg::SelectMenu(menu_state) => {
                    let network_request = match menu_state {
                        crate::components::menu::Menus::NewGame => {
                            network::Request::New(network::NewRequestEntity::Game)
                        }
                        crate::components::menu::Menus::CreateRoom => {
                            network::Request::New(network::NewRequestEntity::Room)
                        }
                        crate::components::menu::Menus::JoinRoom => todo!(),
                    };

                    self.grpc_channel.send(network_request).unwrap();
                    None
                }
            }
        } else {
            None
        }
    }
}
