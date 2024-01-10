use config::{Config, Environment, File, FileFormat};
use serde::Deserialize;

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

/// Read a file from local storage
///
/// Return `None` if file is not present
pub fn read_local_storage<'a, T>(file_name: &str) -> Option<T>
where
    T: Deserialize<'a>,
{
    let config_builder = Config::builder().add_source(File::new(file_name, FileFormat::Toml));

    // Unwrap here because config details must not be wrong
    let data = config_builder.build();

    data.ok().map(|data| data.try_deserialize().unwrap())
}
