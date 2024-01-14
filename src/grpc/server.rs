pub use blazer_grpc::{grpc_client, grpc_server, PingRequest, PingResponse};

use crate::{
    app::errors,
    grpc::{models, redis_client::RedisClient},
};

mod blazer_grpc {
    // The string specified here must match the proto package name
    tonic::include_proto!("server");
}

pub struct MyGrpc {
    redis_client: RedisClient,
}

impl MyGrpc {
    pub fn new(redis_client: RedisClient) -> Self {
        Self { redis_client }
    }
}

#[tonic::async_trait]
impl grpc_server::Grpc for MyGrpc {
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
}
