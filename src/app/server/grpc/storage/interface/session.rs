use crate::app::server::grpc::{
    storage::{StorageResult, Store},
    types::Message,
};

type SessionChannel = tokio::sync::mpsc::Sender<Message>;

#[allow(async_fn_in_trait)]
pub trait SessionInterface {
    async fn insert_channel(&self, user_id: &str, channel: SessionChannel) -> StorageResult<()>;
    async fn get_channel(&self, user_id: &str) -> StorageResult<SessionChannel>;
}

impl SessionInterface for Store {
    async fn insert_channel(&self, user_id: &str, channel: SessionChannel) -> StorageResult<()> {
        let mut connected_users = self.session_state.lock().unwrap();
        connected_users.insert(user_id.to_string(), channel);
        Ok(())
    }

    async fn get_channel(&self, user_id: &str) -> StorageResult<SessionChannel> {
        let connected_users = self.session_state.lock().unwrap();
        let user_channel = connected_users.get(user_id).unwrap().clone();
        Ok(user_channel)
    }
}
