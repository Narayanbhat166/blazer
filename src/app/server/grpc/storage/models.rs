use crate::app::utils;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct User {
    pub user_id: String,
    pub user_name: String,
    pub games_played: usize,
    pub player_rank: usize,
    pub room_id: Option<String>,
    pub game_id: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Copy, Clone)]
pub enum GameStatus {
    Init,
    InProgress,
    End,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Game {
    pub game_id: String,
    pub users_in_game: Vec<String>,
    pub game_status: GameStatus,
    pub prompt: String,
}

impl Game {
    pub fn new(users: &[User], prompt: String) -> Self {
        let game_id = utils::generate_time_ordered_id("game");

        Self {
            game_id,
            users_in_game: users.iter().map(|user| user.user_id.clone()).collect(),
            game_status: GameStatus::Init,
            prompt,
        }
    }
}

impl User {
    pub fn new() -> Self {
        let user_id = utils::generate_time_ordered_id("user");
        let user_name = utils::generate_name();
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

#[derive(Clone, serde::Deserialize, serde::Serialize)]
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
