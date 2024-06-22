use crate::app::server::{
    errors::{self, ResultExtApp},
    grpc::storage::interface::user::UserInterface,
};

use crate::app::server::grpc::{
    server::{MyGrpc, PingRequest, PingResponse},
    storage::models,
};

pub async fn ping(
    state: &MyGrpc,
    request: tonic::Request<PingRequest>,
) -> Result<tonic::Response<PingResponse>, tonic::Status> {
    let ping_request = request.into_inner();
    tracing::info!(?ping_request);
    let optional_user_id = ping_request.user_id;

    let ping_response = match optional_user_id {
        Some(user_id) => {
            let db_user = state.store.find_user(&user_id).await.to_not_found(
                errors::ApiError::UserNotFound {
                    user_id: user_id.clone(),
                },
            )?;

            PingResponse {
                user_id: db_user.user_id,
                user_name: db_user.user_name,
            }
        }
        None => {
            // Create new user
            let new_user = models::User::new();
            let user_id = new_user.user_id.clone();

            let db_user = state
                .store
                .insert_user(new_user)
                .await
                .to_duplicate(errors::ApiError::UserAlreadyExists { user_id })?;

            PingResponse {
                user_id: db_user.user_id,
                user_name: db_user.user_name,
            }
        }
    };

    tracing::info!(?ping_response);

    Ok(tonic::Response::new(ping_response))
}
