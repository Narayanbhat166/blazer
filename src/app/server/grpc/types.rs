use crate::app::{server::grpc::server::RoomServiceResponse, types::RoomServiceResponseType};

use super::storage::models;

pub const COMMON_ROOM_KEY: &str = "COMMON_ROOM";
pub const COMMON_ROOM_SIZE: u8 = 2;

/// Message that can be sent between the client session channels
pub enum RoomMessage {
    RoomCreated {
        room_id: String,
        users: Vec<models::User>,
    },
    RoomJoined {
        room_id: String,
        users: Vec<models::User>,
    },
    AllUsersJoined {
        room_id: String,
        users: Vec<models::User>,
    },
    UserJoined {
        room_id: String,
        users: Vec<models::User>,
    },
}

impl From<RoomMessage> for RoomServiceResponse {
    fn from(value: RoomMessage) -> Self {
        match value {
            RoomMessage::AllUsersJoined { room_id, users } => RoomServiceResponse {
                room_id,
                message_type: RoomServiceResponseType::GameStart.to_u8().into(),
                user_details: users.into_iter().map(From::from).collect::<Vec<_>>(),
            },
            RoomMessage::RoomCreated { room_id, users } => RoomServiceResponse {
                room_id,
                message_type: RoomServiceResponseType::Init.to_u8().into(),
                user_details: users.into_iter().map(From::from).collect::<Vec<_>>(),
            },
            RoomMessage::RoomJoined { room_id, users } => RoomServiceResponse {
                room_id,
                message_type: RoomServiceResponseType::Init.to_u8().into(),
                user_details: users.into_iter().map(From::from).collect::<Vec<_>>(),
            },
            RoomMessage::UserJoined { room_id, users } => RoomServiceResponse {
                room_id,
                message_type: RoomServiceResponseType::UserJoined.to_u8().into(),
                user_details: users.into_iter().map(From::from).collect::<Vec<_>>(),
            },
        }
    }
}
