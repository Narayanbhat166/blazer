pub mod game;
pub mod room;
pub mod session;
pub mod user;

pub trait StorageInterface:
    user::UserInterface + room::RoomInterface + session::SessionInterface + game::GameInterface
{
}
