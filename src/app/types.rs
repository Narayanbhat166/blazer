#[derive(serde::Deserialize)]
pub struct ClientConfig {
    pub server_url: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LocalStorage {
    pub client_id: Option<String>,
}

impl LocalStorage {
    pub fn new(client_id: String) -> Self {
        Self {
            client_id: Some(client_id),
        }
    }
}

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
