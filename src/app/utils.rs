use tokio::{fs, io::AsyncWriteExt};

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
pub async fn read_local_storage<T>(file_name: &str) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    let file_name = replace_home_dir(file_name);
    fs::read_to_string(file_name)
        .await
        .ok()
        .map(|file_contents| toml::from_str::<T>(&file_contents).expect("Invalid data in file"))
}

/// Write the given string to file in local storage
///
/// Create the file if it does not exist
pub async fn write_local_storage<T>(file_name: &str, data: T)
where
    T: serde::Serialize,
{
    let file_name = replace_home_dir(file_name);
    let file_contents = toml::to_string(&data).expect("Cannot convert data to toml representation");

    if fs::write(&file_name, file_contents.as_bytes())
        .await
        .is_err()
    {
        let mut file = fs::File::create(file_name)
            .await
            .expect("Cannot create the file");

        file.write_all(file_contents.as_bytes())
            .await
            .expect("Cannot write to file");
    }
}
