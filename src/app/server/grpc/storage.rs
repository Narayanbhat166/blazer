use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::app::server::{
    errors::DbError,
    grpc::{redis_client::RedisClient, storage::interface::StorageInterface},
};

pub mod interface;
pub mod models;

/// Store the client connections to this instance of the application
type SessionState = Arc<Mutex<HashMap<String, tokio::sync::mpsc::Sender<super::types::Message>>>>;

/// A store that holds the storage clients for various storage types
#[derive(Clone)]
pub struct Store {
    pub redis_client: RedisClient,
    pub session_state: SessionState,
}

impl StorageInterface for Store {}

type StorageResult<T> = Result<T, DbError>;
