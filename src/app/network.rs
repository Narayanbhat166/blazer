use std::sync::{mpsc, Arc, Mutex};

use crate::grpc::server::{grpc_client, PingRequest};

use tuirealm::listener::Poll;

#[derive(PartialEq, Eq, PartialOrd, Clone)]
pub enum UserEvent {
    Pong,
    InfoMessage(String),
    NetworkError(String),
}

pub enum Request {
    Ping,
}

const SERVER_ADDRESS: &str = "http://127.0.0.1:6969";

#[derive(Clone)]
pub struct NetworkClient {
    messsages: Arc<Mutex<Vec<UserEvent>>>,
}

impl NetworkClient {
    #[tokio::main(flavor = "current_thread")]
    pub async fn start_network_client(self, message_receiver: mpsc::Receiver<Request>) {
        let mut client = match grpc_client::GrpcClient::connect(SERVER_ADDRESS).await {
            Ok(grpc_client) => {
                let message =
                    format!("Successfully connected to server at address {SERVER_ADDRESS}");
                self.push_user_event(UserEvent::InfoMessage(message));
                grpc_client
            }
            Err(network_error) => {
                let error = format!("Connection to server failed {network_error:?}");
                self.messsages
                    .lock()
                    .unwrap()
                    .push(UserEvent::NetworkError(error));
                return ();
            }
        };

        while let Ok(message) = message_receiver.recv() {
            match message {
                Request::Ping => {
                    let ping_request = PingRequest { client_id: None };
                    let _response = client.ping(ping_request).await;
                    self.push_user_event(UserEvent::Pong)
                }
            }
        }
    }

    fn push_user_event(&self, event: UserEvent) {
        self.messsages.lock().unwrap().push(event)
    }

    pub fn new() -> Self {
        Self {
            messsages: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl Poll<UserEvent> for NetworkClient {
    fn poll(&mut self) -> tuirealm::listener::ListenerResult<Option<tuirealm::Event<UserEvent>>> {
        Ok(self
            .messsages
            .lock()
            .unwrap()
            .pop()
            .map(tuirealm::Event::User))
    }
}
