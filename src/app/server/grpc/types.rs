use super::storage::models;

pub const COMMON_ROOM: &str = "COMMON_ROOM_KEY";
pub const COMMON_ROOM_SIZE: u8 = 5;

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
