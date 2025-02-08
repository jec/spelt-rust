use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error")]
    Config(String),
    #[error("Database error")]
    Db(String),
    #[error("IO error")]
    Io(String),
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::Config(e) | Error::Db(e) | Error::Io(e) =>
                HttpResponse::InternalServerError().body(e.to_string()),
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

impl From<twelf::Error> for Error {
    fn from(error: twelf::Error) -> Self {
        eprintln!("{error}");
        Self::Config(error.to_string())
    }
}
