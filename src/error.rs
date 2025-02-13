use actix_web::{web, HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Database error: {0}")]
    Db(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Authentication failed: {0}")]
    Auth(String),
}

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
