use crate::app::server::grpc::storage::{models, StorageResult, Store};

#[allow(async_fn_in_trait)]
pub trait GameInterface {
    async fn insert_game(&self, game: models::Game) -> StorageResult<models::Game>;
    async fn find_game(&self, game_id: &str) -> StorageResult<models::Game>;
}

impl GameInterface for Store {
    async fn insert_game(&self, game: models::Game) -> StorageResult<models::Game> {
        let game_id = game.game_id.clone();
        self.redis_client.serialize_and_set(game_id, game).await
    }

    async fn find_game(&self, game_id: &str) -> StorageResult<models::Game> {
        self.redis_client.get_and_deserialize(game_id).await
    }
}
