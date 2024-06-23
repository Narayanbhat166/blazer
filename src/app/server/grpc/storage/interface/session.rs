use crate::app::server::grpc::{
    server::RoomServiceResponse,
    storage::{StorageResult, Store},
    types::RoomMessage,
};

type SessionChannel = tokio::sync::mpsc::Sender<Result<RoomServiceResponse, tonic::Status>>;

/// To store the user channels who are connected
/// This has to be generic over the message
///
/// Channels can be inserted and removed for the same user based on the current interaction
pub trait SessionInterface {
    fn insert_channel(&self, user_id: &str, channel: SessionChannel) -> StorageResult<()>;
    fn remove_channel(&self, user_id: &str) -> StorageResult<()>;
    fn send_message_to_user(
        &self,
        user_id: &str,
        message: RoomMessage,
    ) -> impl std::future::Future<Output = StorageResult<()>>;
}

impl SessionInterface for Store {
    fn insert_channel(&self, user_id: &str, channel: SessionChannel) -> StorageResult<()> {
        let mut connected_users = self.room_users_state.lock().unwrap();
        connected_users.insert(user_id.to_string(), channel);
        Ok(())
    }

    fn remove_channel(&self, user_id: &str) -> StorageResult<()> {
        let mut connected_users = self.room_users_state.lock().unwrap();
        let user_channel = connected_users.remove(user_id).unwrap().clone();
        drop(user_channel);
        Ok(())
    }

    fn send_message_to_user(
        &self,
        user_id: &str,
        message: RoomMessage,
    ) -> impl std::future::Future<Output = StorageResult<()>> {
        let grpc_response = RoomServiceResponse::from(message);
        let user_channel = self.room_users_state.lock().unwrap();

        let user_channel = user_channel
            .get(user_id)
            .expect("The user channel cannot be found")
            .clone();

        async move {
            user_channel.send(Ok(grpc_response)).await.unwrap();
            Ok(())
        }
    }
}
