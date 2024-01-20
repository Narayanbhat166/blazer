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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Room {
    pub room_id: String,
    pub room_size: u8,
    pub users: Vec<String>,
}

impl Room {
    pub fn new(room_id: String, room_size: u8) -> Self {
        Self {
            room_id,
            room_size,
            users: vec![],
        }
    }

    pub fn add_user(&mut self, user_id: String) -> usize {
        self.users.push(user_id);
        self.users.len()
    }
}
