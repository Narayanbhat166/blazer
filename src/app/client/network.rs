use std::{
    collections::VecDeque,
    sync::{mpsc, Arc, Mutex},
};

pub mod types;

use crate::{
    app::utils,
    grpc::server::{grpc_client, PingRequest, RoomServiceRequest, RoomServiceResponse},
};

use tokio_stream::StreamExt;
use tuirealm::listener::Poll;

use super::network::types::UserEvent;
use super::types::{ClientConfig, LocalStorage};

const NETWORK_MESSAGE_QUEUE_CAPACITY: usize = 10;

#[derive(Clone)]
pub struct NetworkClient {
    messages: Arc<Mutex<VecDeque<UserEvent>>>,
    user_id: Option<String>,
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
                    .push_user_event(UserEvent::NetworkError(stringified_error.to_string()));
                None
            }
        }
    }
}

impl Default for NetworkClient {
    fn default() -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::with_capacity(
                NETWORK_MESSAGE_QUEUE_CAPACITY,
            ))),
            user_id: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RoomMessageType {
    Init,
    UserJoined,
    GameStart,
}

impl RoomMessageType {
    pub fn from_u8(message_type_in_u8: u8) -> Self {
        match message_type_in_u8 {
            0 => Self::Init,
            1 => Self::UserJoined,
            2 => Self::GameStart,
            _ => panic!("Unexpected value received for message type"),
        }
    }
}

fn handle_room_service_message(message: RoomServiceResponse, network_client: NetworkClient) {
    let message_type = RoomMessageType::from_u8(message.message_type as u8);

    match message_type {
        RoomMessageType::Init => {
            let room_id = message
                .room_id
                .expect("Required room id, but did not find in init message");

            let users = message
                .user_details
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>();

            let room_created_event = UserEvent::RoomCreated { room_id, users };

            network_client.push_user_event(room_created_event)
        }
        RoomMessageType::UserJoined => {
            let users = message
                .user_details
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>();

            let user_joined_event = UserEvent::UserJoined { users };

            network_client.push_user_event(user_joined_event);
        }
        RoomMessageType::GameStart => network_client.push_user_event(UserEvent::GameStart),
    }
}

async fn handle_room_service_stream(
    mut network_stream: tonic::Streaming<RoomServiceResponse>,
    network_client: NetworkClient,
    mut quit_signal_receiver: tokio::sync::watch::Receiver<bool>,
) {
    let handle_stream_message = |stream_message: Result<_, tonic::Status>,
                                 network_client: NetworkClient| {
        match stream_message {
            Ok(message) => handle_room_service_message(message, network_client),
            Err(error) => {
                let stringified_error = error.message();
                network_client
                    .push_user_event(UserEvent::NetworkError(stringified_error.to_string()));
            }
        }
    };

    loop {
        tokio::select! {
            Some(stream_message) = network_stream.next() => handle_stream_message(stream_message, network_client.clone()),
            _ = quit_signal_receiver.changed() => {
                drop(network_stream);
                return;
            },
            else => return,
        }
    }
}

impl NetworkClient {
    #[tokio::main]
    pub async fn start_network_client(
        &mut self,
        message_receiver: mpsc::Receiver<types::Request>,
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

                self.push_user_event(UserEvent::NetworkError(error));

                // Add the retry logic for exponential retry
                return;
            }
        };

        // Read the client details from ~/.local/state/blazerapp.toml for a returning user
        let local_storage =
            utils::read_local_storage::<LocalStorage>("~/.local/state/blazerapp.toml").await;

        let ping_request = PingRequest {
            user_id: local_storage.and_then(|user_details| user_details.client_id),
        };

        let ping_result = client.ping(ping_request).await;

        if let Some(ping_response) = ping_result.error_handler(self) {
            let client_id = ping_response.user_id;

            // Write the client_id / user_id to localstorage data to persist session
            let local_storage_data = LocalStorage::new(client_id.clone());
            utils::write_local_storage("~/.local/state/blazerapp.toml", local_storage_data).await;

            self.user_id = Some(client_id);
        }

        let (quit_signal_sender, quit_signal_receiver) = tokio::sync::watch::channel(false);

        let mut join_handlers = Vec::<tokio::task::JoinHandle<()>>::new();

        while let Ok(message) = message_receiver.recv() {
            match message {
                types::Request::Quit => {
                    // Inform all the join handles to finish their task
                    quit_signal_sender.send(true).unwrap();
                    for handle in join_handlers {
                        // wait for all tasks to finish
                        handle.await.unwrap();
                    }

                    return;
                }
                types::Request::New(request_type) => {
                    let (request_type, room_id) = match request_type {
                        types::NewRequestEntity::JoinRoom { room_id } => (1, Some(room_id)),
                        types::NewRequestEntity::NewGame => (1, None),
                        types::NewRequestEntity::CreateRoom => (0, None),
                    };

                    let room_request = RoomServiceRequest {
                        client_id: self.user_id.clone().unwrap(),
                        room_id,
                        request_type,
                    };

                    let room_stream = client.room_service(room_request).await.error_handler(self);

                    let cloned_self = self.clone();

                    match room_stream {
                        Some(stream) => {
                            let join_handler = tokio::spawn(handle_room_service_stream(
                                stream,
                                cloned_self,
                                quit_signal_receiver.clone(),
                            ));

                            join_handlers.push(join_handler);
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
        tracing::info!(push_user_event=?event);
        self.messages.lock().unwrap().push_back(event)
    }
}

impl Poll<UserEvent> for NetworkClient {
    fn poll(&mut self) -> tuirealm::listener::ListenerResult<Option<tuirealm::Event<UserEvent>>> {
        Ok(self
            .messages
            .lock()
            .unwrap()
            .pop_front()
            .map(tuirealm::Event::User))
    }
}
