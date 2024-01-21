use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use blazer_grpc::{
    grpc_client, grpc_server, CreateRoomRequest, CreateRoomResponse, PingRequest, PingResponse,
};

use rand::Rng;

use tokio::sync::mpsc;

use crate::{
    app::errors::{self, ResultExtApp},
    grpc::{models, redis_client::RedisClient},
};

mod blazer_grpc {
    // The string specified here must match the proto package name
    tonic::include_proto!("server");
}

pub enum Message {
    RoomCreated(String),
    GameStart(String),
}

pub struct MyGrpc {
    redis_client: RedisClient,
    connected_users: Arc<Mutex<HashMap<String, mpsc::Sender<Message>>>>,
}

const COMMON_ROOM: &str = "COMMON_ROOM_KEY";
const COMMON_ROOM_SIZE: u8 = 2;

impl MyGrpc {
    pub async fn new(redis_client: RedisClient) -> Self {
        // Create the common room if not exists

        let common_room = redis_client
            .get_room_optional(COMMON_ROOM.to_owned())
            .await
            .unwrap();

        if common_room.is_none() {
            let common_room = models::Room::new(COMMON_ROOM.to_string(), COMMON_ROOM_SIZE);
            redis_client.insert_room(common_room).await.unwrap();
            log::info!("Created a common room");
        } else {
            log::info!("Common room already exists");
        }

        Self {
            redis_client,
            connected_users: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn insert_user_channel(&self, user_id: String, sender_channel: mpsc::Sender<Message>) {
        let mut connected_users = self.connected_users.lock().unwrap();
        connected_users.insert(user_id, sender_channel);
    }

    pub async fn get_user_channel(&self, user_id: String) -> mpsc::Sender<Message> {
        let connected_users = self.connected_users.lock().unwrap();
        let user_channel = connected_users.get(&user_id).unwrap().clone();
        user_channel
    }
}

type CreateRoomStream = std::pin::Pin<
    Box<dyn tokio_stream::Stream<Item = Result<CreateRoomResponse, tonic::Status>> + Send>,
>;

#[tonic::async_trait]
impl grpc_server::Grpc for MyGrpc {
    type CreateRoomStream = CreateRoomStream;

    async fn ping(
        &self,
        request: tonic::Request<PingRequest>,
    ) -> Result<tonic::Response<PingResponse>, tonic::Status> {
        log::info!("{request:?}");
        let request = request.into_inner();

        let optional_user_id = request.client_id;

        let ping_response = match optional_user_id {
            Some(user_id) => {
                let db_user = self
                    .redis_client
                    .get_user(user_id.clone())
                    .await
                    .to_not_found(errors::ApiError::UserNotFound {
                        user_id: user_id.clone(),
                    })?;

                PingResponse {
                    client_id: db_user.user_id,
                    client_name: db_user.user_name,
                }
            }
            None => {
                // Create new user
                let user_uuid = uuid::Uuid::new_v4().as_simple().to_string();
                let new_user = models::User::new(user_uuid.clone());
                let db_user = self
                    .redis_client
                    .insert_user(new_user)
                    .await
                    .to_duplicate(errors::ApiError::UserAlreadyExists { user_id: user_uuid })?;

                PingResponse {
                    client_id: db_user.user_id,
                    client_name: db_user.user_name,
                }
            }
        };

        Ok(tonic::Response::new(ping_response))
    }

    async fn create_room(
        &self,
        request: tonic::Request<CreateRoomRequest>,
    ) -> Result<tonic::Response<Self::CreateRoomStream>, tonic::Status> {
        log::info!("{request:?}");
        let request = request.into_inner();

        let request_type = request.request_type;
        let user_id = request.client_id;

        let (response_sender, response_receiver) = mpsc::channel(128);
        let (tx, mut rx) = mpsc::channel::<Message>(10);

        match request_type {
            0 => {
                // Create Room
                let room_id = request.room_id.unwrap_or_else(|| {
                    let mut rng = rand::thread_rng();
                    rng.gen_range(100000..1000000).to_string()
                });

                // If the room already exists then the client must retry with a different room_id
                self.redis_client
                    .get_room(room_id.clone())
                    .await
                    .to_duplicate(errors::ApiError::RoomAlreadyExists {
                        room_id: room_id.clone(),
                    })?;

                let room = models::Room::new(room_id, 2);
                let db_room = self
                    .redis_client
                    .insert_room(room)
                    .await
                    .to_internal_api_error()?;

                tx.send(Message::RoomCreated(db_room.room_id))
                    .await
                    .unwrap();

                self.insert_user_channel(user_id, tx);
            }
            1 => {
                // Join the room
                // This can also be used to join the common room by not passing the `room_id`
                let room_id = request.room_id.unwrap_or(COMMON_ROOM.to_string());

                let mut room = self
                    .redis_client
                    .get_room(room_id.clone())
                    .await
                    .to_not_found(errors::ApiError::RoomNotFound {
                        room_id: room_id.clone(),
                    })?;

                let room_size = room.add_user(user_id.clone());
                let room_max_capacity = room.room_size;
                let user_ids = room.users.clone();

                // Update the room
                self.redis_client
                    .insert_room(room)
                    .await
                    .to_internal_api_error()?;

                if room_size == room_max_capacity as usize {
                    // The game can be started
                    let user_ids = user_ids;
                    for user_id in user_ids {
                        self.get_user_channel(user_id)
                            .await
                            .send(Message::GameStart(room_id.clone()))
                            .await
                            .unwrap();
                    }
                } else {
                    tx.send(Message::RoomCreated(COMMON_ROOM.to_string()))
                        .await
                        .unwrap();
                }

                self.insert_user_channel(user_id, tx);
            }
            _ => {
                // Invalid request
            }
        };

        tokio::spawn(async move {
            while let Some(item) = rx.recv().await {
                let response = match item {
                    Message::GameStart(room_id) => Some(CreateRoomResponse {
                        room_id: Some(room_id),
                        start_game: true,
                    }),
                    Message::RoomCreated(room_id) => Some(CreateRoomResponse {
                        room_id: Some(room_id),
                        start_game: false,
                    }),
                };

                if let Some(response) = response {
                    match response_sender
                        .send(Result::<_, tonic::Status>::Ok(response))
                        .await
                    {
                        Ok(_) => {
                            // item (server response) was queued to be send to client
                        }
                        Err(_item) => {
                            // output_stream was build from rx and both are dropped
                            break;
                        }
                    }
                }
            }
            println!("Client disconnected");
        });

        let output_stream = tokio_stream::wrappers::ReceiverStream::new(response_receiver);
        Ok(tonic::Response::new(
            Box::pin(output_stream) as Self::CreateRoomStream
        ))
    }
}
