use crate::app::server::grpc::storage::{models, StorageResult, Store};

#[allow(async_fn_in_trait)]
pub trait RoomInterface {
    async fn insert_room(&self, room: models::Room) -> StorageResult<models::Room>;
    async fn find_room(&self, room_id: &str) -> StorageResult<models::Room>;
}

impl RoomInterface for Store {
    async fn insert_room(&self, room: models::Room) -> StorageResult<models::Room> {
        let room_id = room.room_id.clone();
        self.redis_client.serialize_and_set(room_id, room).await
    }

    async fn find_room(&self, room_id: &str) -> StorageResult<models::Room> {
        self.redis_client.get_and_deserialize(room_id).await
    }
}
