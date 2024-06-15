use crate::app::server::grpc::storage::{models, StorageResult, Store};

#[allow(async_fn_in_trait)]
pub trait UserInterface {
    async fn insert_user(&self, user: models::User) -> StorageResult<models::User>;
    async fn find_user(&self, user_id: &str) -> StorageResult<models::User>;
    async fn get_multiple_users(&self, user_ids: Vec<String>) -> StorageResult<Vec<models::User>>;
}

impl UserInterface for Store {
    async fn insert_user(&self, user: models::User) -> StorageResult<models::User> {
        let user_id = user.user_id.clone();
        self.redis_client.serialize_and_set(user_id, user).await
    }

    async fn find_user(&self, user_id: &str) -> StorageResult<models::User> {
        self.redis_client.get_and_deserialize(user_id).await
    }

    async fn get_multiple_users(&self, user_ids: Vec<String>) -> StorageResult<Vec<models::User>> {
        self.redis_client.get_multiple_keys(user_ids).await
    }
}
