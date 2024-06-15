use rand::Rng;
use tokio::sync::mpsc;

use crate::app::{
    server::{
        errors::{self, ResultExtApp},
        grpc::storage::interface::{
            room::RoomInterface, session::SessionInterface, user::UserInterface,
        },
    },
    types::{RoomServiceRequestType, RoomServiceResponseType},
};

use crate::app::server::grpc::{
    server::{grpc_server, MyGrpc, RoomServiceRequest, RoomServiceResponse},
    storage::models,
    types,
};

pub async fn room_service(
    state: &MyGrpc,
    user: models::User,
    request: RoomServiceRequest,
) -> Result<tonic::Response<<MyGrpc as grpc_server::Grpc>::RoomServiceStream>, errors::ApiError> {
    let request_type = RoomServiceRequestType::from_u8(request.request_type as u8).ok_or(
        errors::ApiError::BadRequest {
            message: "Received invalid request type".to_string(),
        },
    )?;

    let current_user_id = user.user_id.clone();

    let (response_sender, response_receiver) = mpsc::channel(128);
    let (tx, mut rx) = mpsc::channel::<types::Message>(10);

    // Insert the user into channel so that async communication can take place
    state
        .store
        .insert_channel(&current_user_id, tx.clone())
        .await
        .to_internal_api_error()?;

    // Authenticate user
    let mut user_from_db = user;

    match request_type {
        RoomServiceRequestType::CreateRoom => {
            let room_id = request.room_id.unwrap_or_else(|| {
                let mut rng = rand::thread_rng();
                rng.gen_range(100000..1000000).to_string()
            });

            // If the room already exists then the client must retry with a different room_id
            match state.store.find_room(&room_id).await {
                Ok(_) => Err(errors::ApiError::RoomAlreadyExists { room_id })?,
                Err(error) => {
                    if error.is_not_found() {
                        let room = models::Room::new(room_id, 2);
                        let db_room = state
                            .store
                            .insert_room(room)
                            .await
                            .to_internal_api_error()?;

                        tx.send(types::Message::RoomCreated {
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
            let room_id = request.room_id.unwrap_or(types::COMMON_ROOM.to_string());

            // The room should already exist, or else return an error
            let mut room = state.store.find_room(&room_id).await.to_not_found(
                errors::ApiError::RoomNotFound {
                    room_id: room_id.clone(),
                },
            )?;

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
            user_from_db = state
                .store
                .insert_user(user_from_db.clone())
                .await
                .to_internal_api_error()?;

            let room_max_capacity = room.room_size;

            // Get details about all users in the room, send them update
            let users_in_the_room = room.users.clone();

            // Update the room in database
            state
                .store
                .insert_room(room)
                .await
                .to_internal_api_error()?;

            let all_users_in_room = state
                .store
                .get_multiple_users(users_in_the_room.clone())
                .await
                .to_internal_api_error()?;

            if room_size == room_max_capacity as usize {
                // The game can be started, inform all the connected users of this room
                for user_id in users_in_the_room {
                    state
                        .store
                        .get_channel(&user_id)
                        .await
                        .to_internal_api_error()?
                        .send(types::Message::GameStart {
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
                    state
                        .store
                        .get_channel(&user_id)
                        .await
                        .to_internal_api_error()?
                        .send(types::Message::UserJoined {
                            room_id: room_id.clone(),
                            users: all_users_in_room.clone(),
                        })
                        .await
                        .unwrap();
                }

                // Send the message to current user
                tx.send(types::Message::RoomCreated {
                    room_id: room_id.clone(),
                    users: all_users_in_room.clone(),
                })
                .await
                .unwrap();
            }
        }
    };

    let cloned_store = state.store.clone();

    // This spawns a tokio task which reads from the channel and writes to the server stream
    tokio::spawn(async move {
        while let Some(item) = rx.recv().await {
            let (response, should_close_stream) = match item {
                types::Message::GameStart { room_id, users } => (
                    Some(RoomServiceResponse {
                        room_id: Some(room_id),
                        message_type: RoomServiceResponseType::GameStart.to_u8().into(),
                        user_details: users.into_iter().map(Into::into).collect::<Vec<_>>(),
                    }),
                    true,
                ),
                types::Message::RoomCreated { room_id, users } => (
                    Some(RoomServiceResponse {
                        room_id: Some(room_id),
                        message_type: RoomServiceResponseType::Init.to_u8().into(),
                        user_details: users.into_iter().map(Into::into).collect::<Vec<_>>(),
                    }),
                    false,
                ),
                types::Message::RoomJoined { room_id, users } => (
                    Some(RoomServiceResponse {
                        room_id: Some(room_id),
                        message_type: RoomServiceResponseType::Init.to_u8().into(),
                        user_details: users.into_iter().map(Into::into).collect::<Vec<_>>(),
                    }),
                    false,
                ),
                types::Message::UserJoined { room_id, users } => (
                    Some(RoomServiceResponse {
                        room_id: Some(room_id),
                        message_type: RoomServiceResponseType::UserJoined.to_u8().into(),
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
            let room = cloned_store.find_room(&room_id).await.to_not_found(
                errors::ApiError::RoomNotFound {
                    room_id: room_id.clone(),
                },
            );

            match room {
                Ok(mut room) => {
                    room.remove_user(user_from_db.user_id.clone());
                    cloned_store.insert_room(room).await.unwrap();
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
        Box::pin(output_stream) as <MyGrpc as grpc_server::Grpc>::RoomServiceStream
    ))
}
