#[derive(Debug, PartialEq, Eq, Clone, PartialOrd)]
pub enum UserEvent {
    InfoMessage(String),
    NetworkError(String),
    RoomCreated {
        room_id: String,
        users: Vec<UserDetails>,
    },
    UserJoined {
        users: Vec<UserDetails>,
    },
    GameStart {
        room_id: String,
        users: Vec<UserDetails>,
    },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct UserDetails {
    pub user_id: String,
    pub user_name: String,
    pub games_played: u32,
    pub rank: u32,
}

pub enum NewRequestEntity {
    JoinRoom { room_id: String },
    CreateRoom,
    NewGame,
}

pub enum Request {
    New(NewRequestEntity),
    Quit,
}
