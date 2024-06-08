pub enum RoomServiceRequestType {
    CreateRoom = 1,
    JoinRoom = 2,
}

pub enum RoomServiceResponseType {
    Init = 1,
    UserJoined = 2,
    GameStart = 3,
}

impl RoomServiceResponseType {
    pub fn to_u8(&self) -> u8 {
        match self {
            RoomServiceResponseType::Init => 1,
            RoomServiceResponseType::UserJoined => 2,
            RoomServiceResponseType::GameStart => 3,
        }
    }
}

impl RoomServiceRequestType {
    pub fn from_u8(request_type: u8) -> Option<Self> {
        match request_type {
            1 => Some(Self::CreateRoom),
            2 => Some(Self::JoinRoom),
            _ => None,
        }
    }
}