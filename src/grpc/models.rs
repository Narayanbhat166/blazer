#[derive(serde::Deserialize, serde::Serialize)]
pub struct User {
    pub user_id: String,
    pub user_name: Option<String>,
    pub games_played: usize,
    pub player_rank: usize,
}

impl User {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            user_name: None,
            games_played: 0,
            player_rank: 0,
        }
    }
}
