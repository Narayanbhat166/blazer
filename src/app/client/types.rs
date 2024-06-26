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

#[derive(Debug, PartialEq)]
pub enum AppStateUpdate {
    UserIdUpdate {
        user_id: String,
    },
    RoomUpdate {
        room_id: String,
        users: Vec<UserDetails>,
    },
    UserRoomJoin {
        users: Vec<UserDetails>,
    },
    GameStart {
        room_id: String,
        users: Vec<UserDetails>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct UserDetails {
    pub user_id: String,
    pub user_name: String,
    pub games_played: u32,
    pub rank: u32,
}

/// The messages handled by the client application
/// These messages are generated by the user ( key events ) or by external entities ( network )
#[derive(Debug, PartialEq)]
pub enum Msg {
    AppClose,
    BottomBarUpdate,
    Menu(MenuMessage),
    StateUpdate(AppStateUpdate),
    ReDraw,
}

// Let's define the component ids for our application
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    Menu,
    BottomBar,
    RoomDetails,
    NetworkReceptor,
    Help,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MenuMessage {
    MenuChange,
    MenuDataChange,
    MenuSelect(MenuSelection),
}

#[derive(Debug, PartialEq, Eq)]
pub enum MenuSelection {
    NewGame,
    CreateRoom,
    JoinRoom { room_id: String },
}

#[derive(Debug, PartialEq, Clone)]
pub struct RoomState {
    room_id: String,
    room_users: Vec<UserDetails>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GameState {
    room_id: String,
    game_id: String,
    has_started: bool,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct AppState {
    user_id: Option<String>,
    current_user: Option<UserDetails>,
    room_details: Option<RoomState>,
    game_details: Option<GameState>,
}

impl AppState {
    pub fn apply_update(self, update: AppStateUpdate) -> Self {
        match update {
            AppStateUpdate::UserIdUpdate { user_id } => Self {
                user_id: Some(user_id),
                ..self
            },
            AppStateUpdate::RoomUpdate { room_id, users } => {
                let room_state = RoomState {
                    room_id,
                    room_users: users,
                };

                Self {
                    room_details: Some(room_state),
                    ..self
                }
            }
            AppStateUpdate::UserRoomJoin { users } => {
                let previous_room_state = self.room_details.expect(
                    "Message ordering is invalid. Expected room details before user room join",
                );

                let new_room_state = RoomState {
                    room_users: users,
                    ..previous_room_state
                };

                Self {
                    room_details: Some(new_room_state),
                    ..self
                }
            }
            AppStateUpdate::GameStart { room_id, users } => {
                let room_state = RoomState {
                    room_id,
                    room_users: users,
                };

                Self {
                    room_details: Some(room_state),
                    ..self
                }
            }
        }
    }
}
