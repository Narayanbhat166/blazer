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

        app.mount(Id::RoomDetails, Box::<Details>::default(), Vec::default())
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
                Msg::NetworkUpdate => None,
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
            }
        } else {
            None
        }
    }
}
