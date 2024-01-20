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
    app::errors,
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

impl MyGrpc {
    pub fn new(redis_client: RedisClient) -> Self {
        Self {
            redis_client,
            connected_users: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn insert_user(&self, user_id: String, sender_channel: mpsc::Sender<Message>) {
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
            Some(user_id) => self
                .redis_client
                .get_user_optional(user_id)
                .await?
                .map(|user| PingResponse {
                    client_id: user.user_id,
                    client_name: user.user_name,
                })
                .ok_or(errors::DbError::NotFound)?,
            None => {
                // Create new user
                let user_uuid = uuid::Uuid::new_v4().as_simple().to_string();
                let new_user = models::User::new(user_uuid);
                let db_user = self.redis_client.insert_user(new_user).await?;

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

                // If the room already exists then the client must retry
                if self
                    .redis_client
                    .get_room_optional(room_id.clone())
                    .await?
                    .is_some()
                {
                    Err(tonic::Status::new(
                        tonic::Code::AlreadyExists,
                        format!("The room with the id {room_id} already exists"),
                    ))?
                }

                let room = models::Room::new(room_id, 2);
                let db_room = self.redis_client.insert_room(room).await?;

                tx.send(Message::RoomCreated(db_room.room_id))
                    .await
                    .unwrap();

                self.insert_user(user_id, tx);
            }
            1 => {
                // Join the room
                // This can also be used to join the common room by not passing the `room_id`
                let room_id = request.room_id.unwrap_or(COMMON_ROOM.to_string());

                let mut room = self
                    .redis_client
                    .get_room_optional(room_id.clone())
                    .await?
                    .ok_or(errors::DbError::NotFound)?;

                let room_size = room.add_user(user_id);
                let room_max_capacity = room.room_size;
                let user_ids = room.users.clone();

                // Update the room
                self.redis_client.insert_room(room).await?;

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
                }
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
