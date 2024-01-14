use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("The requested resource was not found")]
    NotFound,
    #[error("The value already exists")]
    DuplicateValue,
    #[error("Failed to parse value")]
    ParsingFailure,
    #[error("Unknown Database error")]
    Others(#[from] fred::error::RedisError),
}

impl DbError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, DbError::NotFound)
    }
}

impl From<DbError> for tonic::Status {
    fn from(value: DbError) -> Self {
        log::error!("db_error={value:?}");
        match value {
            DbError::NotFound => {
                Self::new(tonic::Code::NotFound, "The requested resource is not found")
            }
            DbError::DuplicateValue => Self::new(tonic::Code::AlreadyExists, "Data already exists"),
            DbError::ParsingFailure => Self::new(
                tonic::Code::InvalidArgument,
                "The data is not in the right format",
            ),
            DbError::Others(_) => Self::new(tonic::Code::Internal, "Something Fucked up"),
        }
    }
}
