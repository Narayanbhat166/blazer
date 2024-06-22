use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

pub use blazer_grpc::{
    grpc_client, grpc_server, PingRequest, PingResponse, RoomServiceRequest, RoomServiceResponse,
    UserDetails, FILE_DESCRIPTOR_SET,
};

use super::{functions, redis_client::RedisClient, storage::models, types};

use crate::app::server::{
    errors::{self, ResultExtApp},
    grpc::{
        server::blazer_grpc::{GameServiceRequest, GameServiceResponse},
        storage::{
            interface::{room::RoomInterface, user::UserInterface},
            Store,
        },
    },
};

mod blazer_grpc {
    // The string specified here must match the proto package name
    tonic::include_proto!("server");

    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("grpc");
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

pub struct MyGrpc {
    pub store: Store,
}

impl MyGrpc {
    pub async fn new(redis_client: RedisClient) -> Self {
        // Create the common room if not exists at the application startup
        let store = Store {
            redis_client,
            session_state: Arc::new(Mutex::new(std::collections::HashMap::new())),
        };

        let common_room = store.find_room(types::COMMON_ROOM).await;

        match common_room {
            Ok(_) => {
                tracing::info!("Common room already exists, skipping creation");
            }
            Err(db_error) => {
                if db_error.is_not_found() {
                    let common_room =
                        models::Room::new(types::COMMON_ROOM.to_string(), types::COMMON_ROOM_SIZE);
                    store.insert_room(common_room).await.unwrap();
                    tracing::info!("Created a common room");
                } else {
                    panic!("Database Error when creating new room")
                }
            }
        }
        Self { store }
    }
}

type CreateRoomStream = std::pin::Pin<
    Box<dyn tokio_stream::Stream<Item = Result<RoomServiceResponse, tonic::Status>> + Send>,
>;

type GameServiceStream = std::pin::Pin<
    Box<dyn tokio_stream::Stream<Item = Result<GameServiceResponse, tonic::Status>> + Send>,
>;

trait GetAuthData {
    fn get_user_id(&self) -> String;
}

async fn authenticate(state: &MyGrpc, user_id: String) -> Result<models::User, errors::ApiError> {
    state
        .store
        .find_user(&user_id)
        .await
        .to_not_found(errors::ApiError::UserNotFound { user_id })
}

impl GetAuthData for RoomServiceRequest {
    fn get_user_id(&self) -> String {
        self.client_id.clone()
    }
}

/// A generic wrapper for all the server functions
/// Authenticates the user and fetches user data
async fn server_wrap<'a, Req, Res, Fut>(
    state: &'a MyGrpc,
    request: tonic::Request<Req>,
    func: fn(&'a MyGrpc, models::User, Req) -> Fut,
) -> Result<tonic::Response<Res>, tonic::Status>
where
    Req: GetAuthData + Debug,
    Fut: std::future::Future<Output = Result<tonic::Response<Res>, errors::ApiError>>,
{
    tracing::info!(?request);
    let request = request.into_inner();

    let user_id = request.get_user_id();
    let user = authenticate(state, user_id).await?;
    let result = func(state, user, request).await;

    match &result {
        Ok(_) => {}
        Err(error) => tracing::error!(?error),
    }

    Ok(result?)
}

#[tonic::async_trait]
impl grpc_server::Grpc for MyGrpc {
    type RoomServiceStream = CreateRoomStream;
    type GameServiceStream = GameServiceStream;

    async fn ping(
        &self,
        request: tonic::Request<PingRequest>,
    ) -> Result<tonic::Response<PingResponse>, tonic::Status> {
        functions::ping::ping(self, request).await
    }

    async fn room_service(
        &self,
        request: tonic::Request<RoomServiceRequest>,
    ) -> Result<tonic::Response<Self::RoomServiceStream>, tonic::Status> {
        server_wrap(self, request, |state, user, request| async {
            functions::room_service::room_service(state, user, request).await
        })
        .await
    }

    async fn game_service(
        &self,
        _request: tonic::Request<tonic::Streaming<GameServiceRequest>>,
    ) -> Result<tonic::Response<Self::GameServiceStream>, tonic::Status> {
        todo!()
    }
}
