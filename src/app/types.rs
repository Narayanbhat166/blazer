#[derive(serde::Deserialize)]
pub struct ClientConfig {
    pub server_url: String,
}

#[derive(serde::Deserialize)]
pub struct LocalStorage {
    pub client_id: Option<String>,
}
