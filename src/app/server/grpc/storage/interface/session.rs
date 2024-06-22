use crate::app::server::grpc::{
    storage::{StorageResult, Store},
    types::RoomMessage,
};

type SessionChannel = tokio::sync::mpsc::Sender<RoomMessage>;

/// To store the user channels who are connected
pub trait SessionInterface {
    fn insert_channel(&self, user_id: &str, channel: SessionChannel) -> StorageResult<()>;
    fn get_channel(&self, user_id: &str) -> StorageResult<SessionChannel>;
    fn remove_channel(&self, user_id: &str) -> StorageResult<()>;
}

impl SessionInterface for Store {
    fn insert_channel(&self, user_id: &str, channel: SessionChannel) -> StorageResult<()> {
        let mut connected_users = self.session_state.lock().unwrap();
        connected_users.insert(user_id.to_string(), channel);
        Ok(())
    }

    #[track_caller]
    fn get_channel(&self, user_id: &str) -> StorageResult<SessionChannel> {
        let connected_users = self.session_state.lock().unwrap();
        let user_channel = connected_users.get(user_id).unwrap().clone();
        Ok(user_channel)
    }

    fn remove_channel(&self, user_id: &str) -> StorageResult<()> {
        let mut connected_users = self.session_state.lock().unwrap();
        let user_channel = connected_users.remove(user_id).unwrap().clone();
        drop(user_channel);
        Ok(())
    }
}
