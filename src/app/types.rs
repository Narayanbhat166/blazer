#[derive(serde::Deserialize)]
pub struct ServerConfig {
    pub server: Option<Server>,
    pub redis: Option<RedisConfig>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Server {
    pub host: String,
    pub port: String,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: "6969".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct RedisConfig {
    pub username: Option<String>,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

/// Deault impl to connect to redis running locally
impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            username: None,
            host: "127.0.0.1".to_string(),
            port: 6379,
            password: None,
        }
    }
}

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

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Init),
            2 => Some(Self::UserJoined),
            3 => Some(Self::GameStart),
            _ => None,
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
