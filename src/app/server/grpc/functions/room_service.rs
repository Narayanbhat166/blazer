use rand::Rng;
use tokio::sync::mpsc::{self};

use crate::app::{
    server::{
        errors::{self, ResultExtApp},
        grpc::storage::interface::{
            game::GameInterface, room::RoomInterface, session::SessionInterface,
            user::UserInterface,
        },
    },
    types::RoomServiceRequestType,
};

use crate::app::server::grpc::{
    server::{grpc_server, MyGrpc, RoomServiceRequest},
    storage::models,
    types,
};

/// Establish a streaming connection with the client
/// This helps in notifying about the current users in the room
///
/// Once all the users join the room, game can be started
/// A new stream has to be established with the client for the game
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

    let (response_sender, response_receiver) = mpsc::channel::<Result<_, _>>(128);

    // Insert the user into channel so that async communication can take place
    state
        .store
        .insert_channel(&current_user_id, response_sender.clone())
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

                        state
                            .store
                            .send_message_to_user(
                                &user_from_db.user_id,
                                types::RoomMessage::RoomCreated {
                                    room_id: db_room.room_id,
                                    users: vec![user_from_db.clone()],
                                },
                            )
                            .await
                            .to_internal_api_error()?;
                    } else {
                        Err(error).to_internal_api_error()?
                    }
                }
            }
        }
        RoomServiceRequestType::JoinRoom => {
            // This can also be used to join the common room by not passing the `room_id`
            let room_id = request
                .room_id
                .unwrap_or(types::COMMON_ROOM_KEY.to_string());

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

            // Check if the length of the room is max
            // This can happen in cases when there is a slight delay in starting the game when all users are already in the room
            if room.users.len() == usize::from(room.room_size) {
                Err(errors::ApiError::BadRequest {
                    message: "Maximum capacity has been reached for the room".to_string(),
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
            tracing::info!("users in room {users_in_the_room:?}");

            let all_users_in_room = state
                .store
                .get_multiple_users(users_in_the_room.clone())
                .await
                .to_internal_api_error()?;

            tracing::info!("users in room {all_users_in_room:?}");

            // If the room has reached its maximum capacity, start the game
            if room_size == room_max_capacity as usize {
                // Create the game
                let test_prompt = "This is a sample prompt for the game".to_string();
                let game = models::Game::new(&all_users_in_room, test_prompt);
                state
                    .store
                    .insert_game(game)
                    .await
                    .to_internal_api_error()?;

                // The game can be started, inform all the connected users of this room
                for user_id in users_in_the_room {
                    state
                        .store
                        .send_message_to_user(
                            &user_id,
                            types::RoomMessage::AllUsersJoined {
                                room_id: room_id.clone(),
                                users: all_users_in_room.clone(),
                            },
                        )
                        .await
                        .unwrap();
                }

                if room_id == types::COMMON_ROOM_KEY {
                    room.users.clear();
                    state
                        .store
                        .insert_room(room)
                        .await
                        .to_internal_api_error()?;
                } else {
                    state
                        .store
                        .delete_room(&room_id)
                        .await
                        .to_internal_api_error()?;
                }
            } else {
                // Update the room in database with the new user
                state
                    .store
                    .insert_room(room)
                    .await
                    .to_internal_api_error()?;

                // The current user has joined this room
                // Inform all other users, except current user, that this person has joined the room
                let users_in_room_except_self = users_in_the_room
                    .into_iter()
                    .filter(|user_id| user_id != &current_user_id)
                    .collect::<Vec<_>>();

                for user_id in users_in_room_except_self {
                    state
                        .store
                        .send_message_to_user(
                            &user_id,
                            types::RoomMessage::UserJoined {
                                room_id: room_id.clone(),
                                users: all_users_in_room.clone(),
                            },
                        )
                        .await
                        .unwrap();
                }

                state
                    .store
                    .send_message_to_user(
                        &user_from_db.user_id,
                        types::RoomMessage::RoomCreated {
                            room_id: room_id.clone(),
                            users: all_users_in_room.clone(),
                        },
                    )
                    .await
                    .to_internal_api_error()?;
            }
        }
    };

    let cloned_store = state.store.clone();
    let cloned_response_sender = response_sender.clone();

    // Spawn a tokio task to remove the user session from the session store
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
        interval.tick().await;

        loop {
            interval.tick().await;

            if cloned_response_sender.is_closed() {
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

                            cloned_store
                                .remove_channel(&user_from_db.user_id)
                                .to_internal_api_error()
                                .unwrap();
                        }
                        Err(error) => {
                            tracing::error!(?error);
                        }
                    };
                };

                break;
            }
        }
    });

    let output_stream = tokio_stream::wrappers::ReceiverStream::new(response_receiver);
    Ok(tonic::Response::new(
        Box::pin(output_stream) as <MyGrpc as grpc_server::Grpc>::RoomServiceStream
    ))
}
