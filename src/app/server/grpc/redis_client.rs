use fred::{interfaces::KeysInterface, types::MultipleKeys};

use crate::app::server::errors;

#[derive(Clone)]
pub struct RedisClient {
    client: fred::clients::RedisClient,
}

impl RedisClient {
    pub fn new(inner_client: fred::clients::RedisClient) -> Self {
        Self {
            client: inner_client,
        }
    }
}

type DbResult<T> = Result<T, errors::DbError>;

impl RedisClient {
    pub async fn get_and_deserialize<
        K: Into<fred::types::RedisKey> + Send,
        V: serde::de::DeserializeOwned,
    >(
        &self,
        key: K,
    ) -> DbResult<V> {
        let get_command_result = self.client.get::<Option<String>, _>(key).await;

        match get_command_result {
            Ok(value_string_optional) => match value_string_optional {
                Some(value_string) => match serde_json::from_str::<V>(&value_string) {
                    Ok(value) => Ok(value),
                    Err(deserialize_error) => {
                        log::error!("{deserialize_error:?}");
                        Err(errors::DbError::NotFound)
                    }
                },
                None => Err(errors::DbError::NotFound),
            },
            Err(error) => Err(errors::DbError::Others(error)),
        }
    }

    pub async fn serialize_and_set<
        K: Into<fred::types::RedisKey> + Send,
        V: serde::Serialize + serde::de::DeserializeOwned,
    >(
        &self,
        key: K,
        value: V,
    ) -> DbResult<V> {
        let serialized_value = serde_json::to_string(&value);

        match serialized_value {
            Ok(serialized_value) => {
                match self
                    .client
                    .set::<String, _, _>(key, serialized_value, None, None, false)
                    .await
                {
                    Ok(_) => Ok(value),
                    Err(error) => Err(errors::DbError::Others(error)),
                }
            }
            Err(serialization_error) => {
                log::error!("serialization_error {serialization_error:?}");
                Err(errors::DbError::ParsingFailure)
            }
        }
    }

    pub async fn get_multiple_keys<
        K: Into<MultipleKeys> + Send,
        V: serde::Serialize + serde::de::DeserializeOwned,
    >(
        &self,
        keys: K,
    ) -> DbResult<Vec<V>> {
        let get_command_result = self.client.mget::<Vec<String>, _>(keys).await;

        match get_command_result {
            Ok(value_string_optional) => {
                let result = value_string_optional
                    .iter()
                    .map(|value_string| serde_json::from_str::<V>(value_string))
                    .collect::<Result<Vec<_>, _>>();

                result.map_err(|serialize_error| {
                    tracing::error!(?serialize_error);
                    errors::DbError::ParsingFailure
                })
            }
            Err(error) => Err(errors::DbError::Others(error)),
        }
    }

    pub async fn delete_key(&self, key: &str) -> DbResult<()> {
        let delete_result = self.client.del::<usize, _>(key).await;
        match delete_result {
            Ok(_) => {
                tracing::info!("Key {key} has been successfully deleted");
                Ok(())
            }
            Err(error) => {
                tracing::error!(?error);
                Err(errors::DbError::Others(error))
            }
        }
    }
}
