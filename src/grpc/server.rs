use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use blazer_grpc::{
    grpc_client, grpc_server, PingRequest, PingResponse, RoomServiceRequest, RoomServiceResponse,
    UserDetails,
};

use rand::Rng;

use tokio::sync::mpsc;

use crate::{
    app::errors::{self, ApiError, ResultExtApp},
    grpc::{redis_client::RedisClient, storage::models},
};

mod blazer_grpc {
    // The string specified here must match the proto package name
    tonic::include_proto!("server");
}

impl From<models::User> for UserDetails {
    fn from(db_model: models::User) -> Self {
        Self {
            user_id: db_model.user_id,
            user_name: db_model.user_name,
            games_played: db_model.games_played as u32,
            rank: db_model.player_rank as u32,
        }
    }
}

pub enum Message {
    RoomCreated {
        room_id: String,
        users: Vec<models::User>,
    },
    RoomJoined {
        room_id: String,
        users: Vec<models::User>,
    },
    GameStart {
        room_id: String,
        users: Vec<models::User>,
    },
    UserJoined {
        room_id: String,
        users: Vec<models::User>,
    },
}

pub struct MyGrpc {
    redis_client: RedisClient,
    connected_users: Arc<Mutex<HashMap<String, mpsc::Sender<Message>>>>,
}

const COMMON_ROOM: &str = "COMMON_ROOM_KEY";
const COMMON_ROOM_SIZE: u8 = 5;

pub fn generate_name() -> String {
    let random_name_generator = rnglib::RNG::from(&rnglib::Language::Fantasy);

    format!(
        "{} {}",
        random_name_generator.generate_name(),
        random_name_generator.generate_name()
    )
}

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
            tracing::info!("Created a common room");
        } else {
            tracing::info!("Common room already exists");
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
    Box<dyn tokio_stream::Stream<Item = Result<RoomServiceResponse, tonic::Status>> + Send>,
>;

enum RoomServiceRequestType {
    CreateRoom = 0,
    JoinRoom = 1,
}

impl RoomServiceRequestType {
    pub fn from_u8(request_type: u8) -> Option<Self> {
        match request_type {
            0 => Some(Self::CreateRoom),
            1 => Some(Self::JoinRoom),
            _ => None,
        }
    }
}

#[tonic::async_trait]
impl grpc_server::Grpc for MyGrpc {
    type RoomServiceStream = CreateRoomStream;

    async fn ping(
        &self,
        request: tonic::Request<PingRequest>,
    ) -> Result<tonic::Response<PingResponse>, tonic::Status> {
        let ping_request = request.into_inner();
        tracing::info!(?ping_request);
        let optional_user_id = ping_request.user_id;

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
                    user_id: db_user.user_id,
                    user_name: db_user.user_name,
                }
            }
            None => {
                // Create new user
                let user_uuid = uuid::Uuid::new_v4().as_simple().to_string();
                let new_user_name = generate_name();
                let new_user = models::User::new(user_uuid.clone(), new_user_name);

                let db_user = self
                    .redis_client
                    .insert_user(new_user)
                    .await
                    .to_duplicate(errors::ApiError::UserAlreadyExists { user_id: user_uuid })?;

                PingResponse {
                    user_id: db_user.user_id,
                    user_name: db_user.user_name,
                }
            }
        };

        tracing::info!(?ping_response);

        Ok(tonic::Response::new(ping_response))
    }

    async fn room_service(
        &self,
        request: tonic::Request<RoomServiceRequest>,
    ) -> Result<tonic::Response<Self::RoomServiceStream>, tonic::Status> {
        let request = request.into_inner();
        tracing::info!(room_service_request=?request);

        let request_type = RoomServiceRequestType::from_u8(request.request_type as u8).ok_or(
            ApiError::BadRequest {
                message: "Received invalid request type".to_string(),
            },
        )?;

        let current_user_id = request.client_id;

        let (response_sender, response_receiver) = mpsc::channel(128);
        let (tx, mut rx) = mpsc::channel::<Message>(10);

        // Insert the user into channel so that async communication can take place
        self.insert_user_channel(current_user_id.clone(), tx.clone());

        // Authenticate user
        let mut user_from_db = self
            .redis_client
            .get_user(current_user_id.clone())
            .await
            .to_not_found(errors::ApiError::UserNotFound {
                user_id: current_user_id.clone(),
            })?;

        match request_type {
            RoomServiceRequestType::CreateRoom => {
                let room_id = request.room_id.unwrap_or_else(|| {
                    let mut rng = rand::thread_rng();
                    rng.gen_range(100000..1000000).to_string()
                });

                // If the room already exists then the client must retry with a different room_id
                match self.redis_client.get_room(room_id.clone()).await {
                    Ok(_) => Err(ApiError::RoomAlreadyExists { room_id })?,
                    Err(error) => {
                        if error.is_not_found() {
                            let room = models::Room::new(room_id, 2);
                            let db_room = self
                                .redis_client
                                .insert_room(room)
                                .await
                                .to_internal_api_error()?;

                            tx.send(Message::RoomCreated {
                                room_id: db_room.room_id,
                                users: vec![user_from_db.clone()],
                            })
                            .await
                            .unwrap();
                        } else {
                            Err(error).to_internal_api_error()?
                        }
                    }
                }
            }
            RoomServiceRequestType::JoinRoom => {
                // This can also be used to join the common room by not passing the `room_id`
                let room_id = request.room_id.unwrap_or(COMMON_ROOM.to_string());

                // The room should already exist, or else return an error
                let mut room = self
                    .redis_client
                    .get_room(room_id.clone())
                    .await
                    .to_not_found(errors::ApiError::RoomNotFound {
                        room_id: room_id.clone(),
                    })?;

                // Check if user is not in the current room
                if room.users.contains(&current_user_id) {
                    Err(errors::ApiError::BadRequest {
                        message: "User trying to join the same room".to_string(),
                    })?
                }

                // Add the current user to the room
                let room_size = room.add_user(current_user_id.clone());

                // Update the user that he has been assigned to a room
                user_from_db.assign_room_id(room_id.clone());

                // Update the user in database
                user_from_db = self
                    .redis_client
                    .insert_user(user_from_db.clone())
                    .await
                    .to_internal_api_error()?;

                let room_max_capacity = room.room_size;

                // Get details about all users in the room, send them update
                let users_in_the_room = room.users.clone();

                // Update the room in database
                self.redis_client
                    .insert_room(room)
                    .await
                    .to_internal_api_error()?;

                let all_users_in_room = self
                    .redis_client
                    .get_multiple_users(users_in_the_room.clone())
                    .await
                    .to_internal_api_error()?;

                if room_size == room_max_capacity as usize {
                    // The game can be started, inform all the connected users of this room
                    for user_id in users_in_the_room {
                        self.get_user_channel(user_id)
                            .await
                            .send(Message::GameStart {
                                room_id: room_id.clone(),
                                users: all_users_in_room.clone(),
                            })
                            .await
                            .unwrap();
                    }
                } else {
                    // The current user has joined this room
                    // Inform all other users, except current user, that this person has joined the room

                    let users_in_room_except_self = users_in_the_room
                        .into_iter()
                        .filter(|user_id| user_id != &current_user_id)
                        .collect::<Vec<_>>();

                    for user_id in users_in_room_except_self {
                        self.get_user_channel(user_id)
                            .await
                            .send(Message::UserJoined {
                                room_id: room_id.clone(),
                                users: all_users_in_room.clone(),
                            })
                            .await
                            .unwrap();
                    }

                    // Send the message to current user
                    tx.send(Message::RoomCreated {
                        room_id: room_id.clone(),
                        users: all_users_in_room.clone(),
                    })
                    .await
                    .unwrap();
                }
            }
        };

        let cloned_redis_client = self.redis_client.clone();

        // This spawns a tokio task which reads from the channel and writes to the server stream
        tokio::spawn(async move {
            while let Some(item) = rx.recv().await {
                let (response, should_close_stream) = match item {
                    Message::GameStart { room_id, users } => (
                        Some(RoomServiceResponse {
                            room_id: Some(room_id),
                            message_type: 2,
                            user_details: users.into_iter().map(Into::into).collect::<Vec<_>>(),
                        }),
                        true,
                    ),
                    Message::RoomCreated { room_id, users } => (
                        Some(RoomServiceResponse {
                            room_id: Some(room_id),
                            message_type: 0,
                            user_details: users.into_iter().map(Into::into).collect::<Vec<_>>(),
                        }),
                        false,
                    ),
                    Message::RoomJoined { room_id, users } => (
                        Some(RoomServiceResponse {
                            room_id: Some(room_id),
                            message_type: 0,
                            user_details: users.into_iter().map(Into::into).collect::<Vec<_>>(),
                        }),
                        false,
                    ),
                    Message::UserJoined { room_id, users } => (
                        Some(RoomServiceResponse {
                            room_id: Some(room_id),
                            message_type: 1,
                            user_details: users.into_iter().map(Into::into).collect::<Vec<_>>(),
                        }),
                        false,
                    ),
                };

                if let Some(response) = response {
                    tracing::info!(message=?response, to=?current_user_id);
                    match response_sender
                        .send(Result::<_, tonic::Status>::Ok(response))
                        .await
                    {
                        Ok(()) => {
                            if should_close_stream {
                                // Break the loop and close this stream, drop the receiver and sender
                                break;
                            }
                        }
                        Err(_item) => {
                            // output_stream was built from rx and both are dropped
                            break;
                        }
                    }
                }
            }

            println!("Client disconnected");
            // Remove the user from room if he is in any
            let room_id = user_from_db.room_id.clone();
            if let Some(room_id) = room_id {
                let room = cloned_redis_client
                    .get_room(room_id.clone())
                    .await
                    .to_not_found(ApiError::RoomNotFound {
                        room_id: room_id.clone(),
                    });

                match room {
                    Ok(mut room) => {
                        room.remove_user(user_from_db.user_id.clone());
                        cloned_redis_client.insert_room(room).await.unwrap();
                        log::info!(
                            "Removed user {} from the room {}",
                            user_from_db.user_id,
                            room_id
                        );
                    }
                    Err(_) => todo!(),
                };
            };
        });

        let output_stream = tokio_stream::wrappers::ReceiverStream::new(response_receiver);
        Ok(tonic::Response::new(
            Box::pin(output_stream) as Self::RoomServiceStream
        ))
    }
}
