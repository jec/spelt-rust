use actix_web::web;
use sqlx::PgPool;
use crate::error::Error;
use crate::routes::auth::LoginRequest;
use crate::services;

pub enum LoginResult {
    LoggedIn { access_token: String, device_id: String, username: String, expires_in_ms: u64 },
    CredentialsInvalid,
    NotSupported,
    BadRequest,
}

/// Authenticates a user and, if successful, returns a LoginResult with a token
///
/// If the request specifies a `device_id`, any previous Session for that device
/// will be deleted. If the request does not specify a `device_id`, one will be
/// generated.
///
pub async fn log_in(login_request: web::Json<LoginRequest>, db_pool: &PgPool) -> Result<LoginResult, Error> {
    // Check authentication type
    let login = login_request.into_inner();
    if login.r#type != "m.login.password" {
        return Ok(LoginResult::NotSupported);
    }

    // Authenticate User
    let username = if login.address.is_empty() {
        if login.user.is_empty() {
            if login.identifier.r#type == "m.id.user" {
                login.identifier.user
            } else {
                return Ok(LoginResult::BadRequest);
            }
        } else {
            return Ok(LoginResult::BadRequest);
        }
    } else {
        login.address
    };

    let user_id_opt = crate::repo::auth::validate_user_and_password(&username, &login.password, db_pool).await?;

    if user_id_opt.is_none() {
        return Ok(LoginResult::CredentialsInvalid);
    }

    // Delete existing Session if `device_id` specified; else generate a
    // `device_id`.
    let user_id = user_id_opt.unwrap();
    let device_id = if login.device_id.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        crate::repo::auth::invalidate_existing_sessions(user_id, &login.device_id, db_pool).await?;
        login.device_id
    };

    // Create Session and JWT
    let session_uuid = crate::repo::auth::create_session(user_id, &device_id, &login.initial_device_display_name, db_pool).await?;
    let access_token = services::jwt::create_jwt(&session_uuid)?;

    Ok(LoginResult::LoggedIn { access_token, device_id, username, expires_in_ms: services::jwt::JWT_TTL_SECONDS })
}
