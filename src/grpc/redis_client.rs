use fred::interfaces::KeysInterface;

use crate::{app::errors, grpc::models};

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
    async fn get_and_deserialize<
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

    async fn serialize_and_set<
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

    pub async fn get_user_optional(&self, user_id: String) -> DbResult<Option<models::User>> {
        let res = self.get_and_deserialize::<_, models::User>(user_id).await;
        match res {
            Ok(user) => Ok(Some(user)),
            Err(db_error) => {
                if db_error.is_not_found() {
                    Ok(None)
                } else {
                    Err(db_error)
                }
            }
        }
    }

    pub async fn insert_user(&self, user: models::User) -> DbResult<models::User> {
        let user_id = user.user_id.clone();
        self.serialize_and_set(user_id, user).await
    }
}
