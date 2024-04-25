use fred::interfaces::ClientLike;

use super::redis_client::RedisClient;
use crate::app::types::RedisConfig;

pub async fn create_redis_client(
    redis_config: RedisConfig,
) -> Result<RedisClient, fred::error::RedisError> {
    let config = fred::types::RedisConfig {
        server: fred::types::ServerConfig::Centralized {
            server: fred::types::Server {
                host: redis_config.host.into(),
                port: redis_config.port,
            },
        },
        username: redis_config.username,
        password: redis_config.password,
        ..fred::types::RedisConfig::default()
    };

    let client = fred::clients::RedisClient::new(config, None, None, None);

    // connect to the server, returning a handle to a task that drives the connection
    client.connect();

    // wait for the client to connect
    let _ = client.wait_for_connect().await;

    Ok(RedisClient::new(client))
}
