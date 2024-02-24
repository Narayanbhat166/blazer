use std::{fs, io::Write};

use crate::grpc;
use config::{Config, Environment, File, FileFormat};
use fred::interfaces::ClientLike;
use serde::Deserialize;

use crate::app::types::RedisConfig;

pub fn read_config<'a, T>(file_name: &str, env_prefix: Option<&str>) -> T
where
    T: Deserialize<'a>,
{
    let mut config_builder = Config::builder().add_source(File::new(file_name, FileFormat::Toml));

    if let Some(env_prefix) = env_prefix {
        config_builder = config_builder.add_source(Environment::with_prefix(env_prefix));
    }

    let data = config_builder.build();

    // Unwrap here because without config application cannot be run
    data.unwrap().try_deserialize().unwrap()
}

fn replace_home_dir(file_name: &str) -> String {
    let path_buf = std::path::PathBuf::from(file_name);
    path_buf
        .iter()
        .map(|dir| {
            if dir == "~" {
                std::env::var("HOME").unwrap()
            } else {
                dir.to_str().unwrap().to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Read a file from local storage
///
/// Return `None` if file is not present
pub fn read_local_storage<T>(file_name: &str) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    let file_name = replace_home_dir(file_name);
    fs::read_to_string(file_name)
        .ok()
        .map(|file_contents| toml::from_str::<T>(&file_contents).expect("Invalid data in file"))
}

/// Write the given string to file in local storage
///
/// Create the file if it does not exist
pub fn write_local_storage<T>(file_name: &str, data: T)
where
    T: serde::Serialize,
{
    let file_name = replace_home_dir(file_name);
    let file_contents = toml::to_string(&data).expect("Cannot convert data to toml representation");

    if fs::write(&file_name, file_contents.as_bytes()).is_err() {
        let mut file = fs::File::create(file_name).expect("Cannot create the file");
        file.write_all(file_contents.as_bytes())
            .expect("Cannot write to file");
    }
}

pub async fn create_redis_client(
    redis_config: RedisConfig,
) -> Result<grpc::redis_client::RedisClient, fred::error::RedisError> {
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

    Ok(grpc::redis_client::RedisClient::new(client))
}
