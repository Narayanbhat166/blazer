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

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("The user with id {user_id} does not exist")]
    UserNotFound { user_id: String },
    #[error("The user with id {user_id} already exists")]
    UserAlreadyExists { user_id: String },
    #[error("The room with id {room_id} does not exist")]
    RoomNotFound { room_id: String },
    #[error("The room with id {room_id} already exists")]
    RoomAlreadyExists { room_id: String },
    #[error("Internal Server error")]
    InternalServerError,
}

impl DbError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, DbError::NotFound)
    }
}

pub trait ResultExtApp<T> {
    // Returns the provided api_error if it is a not found error in database or return `InternalServerError`
    fn to_not_found(self, api_error: ApiError) -> Result<T, ApiError>;
    // Returns the provided api_error if it is a duplicate error in database or return `InternalServerError`
    fn to_duplicate(self, api_error: ApiError) -> Result<T, ApiError>;

    fn to_internal_api_error(self) -> Result<T, ApiError>;
}

impl<T> ResultExtApp<T> for Result<T, DbError> {
    fn to_not_found(self, api_error: ApiError) -> Result<T, ApiError> {
        if let Err(DbError::NotFound) = &self {
            self.map_err(|_| api_error)
        } else {
            self.map_err(|_| ApiError::InternalServerError)
        }
    }

    fn to_duplicate(self, api_error: ApiError) -> Result<T, ApiError> {
        if let Err(DbError::DuplicateValue) = &self {
            self.map_err(|_| api_error)
        } else {
            self.map_err(|error| {
                tracing::error!(db_error=?error);
                ApiError::InternalServerError
            })
        }
    }

    fn to_internal_api_error(self) -> Result<T, ApiError> {
        self.map_err(|error| {
            tracing::error!(db_error=?error);
            ApiError::InternalServerError
        })
    }
}

impl From<ApiError> for tonic::Status {
    fn from(api_error: ApiError) -> Self {
        let code = match api_error {
            ApiError::UserNotFound { .. } => tonic::Code::NotFound,
            ApiError::RoomNotFound { .. } => tonic::Code::NotFound,
            ApiError::UserAlreadyExists { .. } => tonic::Code::AlreadyExists,
            ApiError::RoomAlreadyExists { .. } => tonic::Code::AlreadyExists,
            ApiError::InternalServerError => tonic::Code::Internal,
        };

        Self::new(code, api_error.to_string())
    }
}
