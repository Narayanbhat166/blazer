use std::sync::{mpsc, Arc, Mutex};

use crate::grpc::server::{grpc_client, PingRequest};

use tuirealm::listener::Poll;

use super::{
    types::{self, ClientConfig},
    utils,
};

#[derive(PartialEq, Eq, PartialOrd, Clone)]
pub enum UserEvent {
    Pong,
    InfoMessage(String),
    NetworkError(String),
}

pub enum Request {
    Ping,
}

#[derive(Clone)]
pub struct NetworkClient {
    messsages: Arc<Mutex<Vec<UserEvent>>>,
    client_id: Option<String>,
}

impl NetworkClient {
    #[tokio::main(flavor = "current_thread")]
    pub async fn start_network_client(
        &mut self,
        message_receiver: mpsc::Receiver<Request>,
        config: ClientConfig,
    ) {
        let mut client = match grpc_client::GrpcClient::connect(config.server_url.clone()).await {
            Ok(grpc_client) => {
                let message = format!(
                    "Successfully connected to server at address {}",
                    config.server_url
                );
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

        let local_storage = utils::read_local_storage::<types::LocalStorage>("~/.local/state/");

        let ping_request = PingRequest {
            client_id: local_storage.and_then(|user_details| user_details.client_id),
        };

        let pong = client.ping(ping_request).await.unwrap();
        self.client_id = Some(pong.get_ref().client_id.clone());

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
            client_id: None,
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
