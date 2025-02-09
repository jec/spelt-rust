use actix_web::web;
use sqlx::PgPool;
use crate::error::Error;
use crate::routes::auth::{LoginRequest, LoginResponse};

/// Authenticates a user and, if successful, returns a LoginResponse with a token
///
/// If the request specifies a `device_id`, any previous Session for that device
/// will be deleted. If the request does not specify a `device_id`, one will be
/// generated.
///
pub async fn log_in(login: web::Json<LoginRequest>, db_pool: &Option<PgPool>) -> Result<LoginResponse, Error> {
    Ok(LoginResponse{})
}
