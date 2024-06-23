use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::app::server::{
    errors::DbError,
    grpc::{
        redis_client::RedisClient, server::RoomServiceResponse,
        storage::interface::StorageInterface,
    },
};

pub mod interface;
pub mod models;

/// Store the client connections to this instance of the application
type SessionState<T> = Arc<Mutex<HashMap<String, T>>>;

/// A store that holds the storage clients for various storage types
#[derive(Clone)]
pub struct Store {
    pub redis_client: RedisClient,
    pub room_users_state:
        SessionState<tokio::sync::mpsc::Sender<Result<RoomServiceResponse, tonic::Status>>>,
}

impl StorageInterface for Store {}

type StorageResult<T> = Result<T, DbError>;
