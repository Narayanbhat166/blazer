use std::sync::{mpsc, Arc, Mutex};

use crate::grpc::server::{grpc_client, PingRequest, RoomServiceRequest};

use tokio_stream::StreamExt;
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
    RoomCreated { room_id: Option<String> },
    GameStart,
}

pub enum NewRequestEntity {
    Room,
    Game,
}

pub enum Request {
    New(NewRequestEntity),
}

#[derive(Clone)]
pub struct NetworkClient {
    messsages: Arc<Mutex<Vec<UserEvent>>>,
    client_id: Option<String>,
}

pub trait DisplayNetworkError {
    type Item;
    fn error_handler(self, network_client: &NetworkClient) -> Option<Self::Item>;
}

impl<U> DisplayNetworkError for Result<tonic::Response<U>, tonic::Status> {
    type Item = U;
    fn error_handler(self, network_client: &NetworkClient) -> Option<Self::Item> {
        match self {
            Ok(res) => Some(res.into_inner()),
            Err(tonic_status) => {
                let stringified_error = tonic_status.message();
                network_client
                    .messsages
                    .lock()
                    .unwrap()
                    .push(UserEvent::NetworkError(stringified_error.to_string()));
                None
            }
        }
    }
}

impl Default for NetworkClient {
    fn default() -> Self {
        Self {
            messsages: Arc::new(Mutex::new(vec![])),
            client_id: None,
        }
    }
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

                // Add the retry logic for exponential retry
                return;
            }
        };

        // Read the client details from ~/.local/state/blazerapp.toml for a returning user
        let local_storage =
            utils::read_local_storage::<types::LocalStorage>("~/.local/state/blazerapp.toml");

        let ping_request = PingRequest {
            client_id: local_storage.and_then(|user_details| user_details.client_id),
        };

        let ping_result = client.ping(ping_request).await;

        if let Some(ping_response) = ping_result.error_handler(self) {
            let client_id = ping_response.client_id;

            // Write the client_id / user_id to localstorage data to persist session
            let local_storage_data = types::LocalStorage::new(client_id.clone());
            utils::write_local_storage("~/.local/state/blazerapp.toml", local_storage_data);

            self.client_id = Some(client_id);
        }

        while let Ok(message) = message_receiver.recv() {
            match message {
                Request::New(request_type) => {
                    let request_type = match request_type {
                        NewRequestEntity::Room => 0,
                        NewRequestEntity::Game => 1,
                    };

                    let room_request = RoomServiceRequest {
                        client_id: self.client_id.clone().unwrap(),
                        room_id: None,
                        request_type,
                    };

                    let room_stream = client.room_service(room_request).await.error_handler(self);

                    let cloned_self = self.clone();

                    match room_stream {
                        Some(mut stream) => {
                            while let Some(stream_message) = stream.next().await {
                                let inner_message = stream_message;

                                match inner_message {
                                    Ok(message) => {
                                        if message.start_game {
                                            cloned_self.push_user_event(UserEvent::GameStart);
                                            // Break the loop and disconnect the client as this stream is no longer needed
                                            break;
                                        } else {
                                            // Since the game has not yet begun, wait for next message and do not break the loop
                                            cloned_self.push_user_event(UserEvent::RoomCreated {
                                                room_id: message.room_id,
                                            })
                                        }
                                    }
                                    Err(error) => {
                                        let stringified_error = error.message();
                                        self.messsages.lock().unwrap().push(
                                            UserEvent::NetworkError(stringified_error.to_string()),
                                        );
                                    }
                                }
                            }
                        }
                        None => {
                            // This can happen in case the server panics
                        }
                    }
                }
            }
        }
    }

    fn push_user_event(&self, event: UserEvent) {
        self.messsages.lock().unwrap().push(event)
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
