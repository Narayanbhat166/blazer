#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct User {
    pub user_id: String,
    pub user_name: String,
    pub games_played: usize,
    pub player_rank: usize,
    pub room_id: Option<String>,
    pub game_id: Option<String>,
}

impl User {
    pub fn new(user_id: String, user_name: String) -> Self {
        Self {
            user_id,
            user_name,
            games_played: 0,
            player_rank: 0,
            room_id: None,
            game_id: None,
        }
    }

    pub fn assign_room_id(&mut self, room_id: String) {
        self.room_id = Some(room_id)
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

    pub fn remove_user(&mut self, user_id_to_be_removed: String) -> usize {
        let position = self
            .users
            .iter()
            .position(|user_id| user_id == &user_id_to_be_removed);

        if let Some(index) = position {
            self.users.remove(index);
        }

        self.users.len()
    }
}
