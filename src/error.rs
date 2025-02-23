use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

/// Internal error types that implement [`ResponseError`] so they're rendered
/// appropriately in HTTP responses
#[derive(Error, Debug)]
pub enum Error {
    /// Represents an error reading the application config file
    #[error("Configuration error: {0}")]
    Config(String),

    /// Represents an error reading/writing the database
    #[error("Database error: {0}")]
    Db(String),

    /// Represents an unknown read/write error
    #[error("IO error: {0}")]
    Io(String),

    /// Represents a JWT validation error or missing authentication for an
    /// endpoint requiring it
    #[error("Authentication failed: {0}")]
    Auth(String),
}

/// JSON response payload in the case of an error, per the Matrix spec
#[derive(Serialize)]
pub struct ErrorResponse {
    pub errcode: String,
    pub error: String,
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Config(_) | Error::Db(_) | Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Auth(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Error::Config(e) | Error::Db(e) | Error::Io(e) =>
                HttpResponse::build(self.status_code())
                    .json(web::Json(ErrorResponse {
                        errcode: String::from("M_UNKNOWN"),
                        error: e.to_string()
                    })),
            Error::Auth(e) =>
                HttpResponse::build(self.status_code())
                    .json(web::Json(ErrorResponse {
                        errcode: String::from("M_UNKNOWN_TOKEN"),
                        error: e.to_string()
                    })),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        eprintln!("{error}");
        Self::Io(error.to_string())
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        eprintln!("{error}");
        Self::Db(error.to_string())
    }
}

impl From<surrealdb::Error> for Error {
    fn from(error: surrealdb::Error) -> Self {
        eprintln!("{error}");
        Self::Db(error.to_string())
    }
}

impl From<twelf::Error> for Error {
    fn from(error: twelf::Error) -> Self {
        eprintln!("{error}");
        Self::Config(error.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        eprintln!("{error}");
        Self::Auth(error.to_string())
    }
}
