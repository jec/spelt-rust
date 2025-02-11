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
    if login_request.r#type != "m.login.password" {
        return Ok(LoginResult::NotSupported);
    }

    // Authenticate User
    // The `username` should be `address` or `user` or (if `identifier.type` is
    // "m.id.user") `identifier.user`.
    let username = match login_request.address {
        Some(ref address) => address.clone(),
        None =>
            match login_request.user {
                Some(ref user) => user.clone(),
                None =>
                    match login_request.identifier {
                        Some(ref identifier) => {
                            if identifier.r#type != "m.id.user" {
                                return Ok(LoginResult::BadRequest);
                            }
                            match identifier.user {
                                Some(ref user) => user.clone(),
                                None => return Ok(LoginResult::BadRequest),
                            }
                        }
                        None => return Ok(LoginResult::BadRequest),
                    }
            }
    };

    if login_request.password.is_none() {
        return Ok(LoginResult::BadRequest);
    }

    let user_id_opt = crate::repo::auth::validate_user_and_password(
        &username,
        &login_request.password.as_ref().unwrap(),
        db_pool
    ).await?;

    if user_id_opt.is_none() {
        return Ok(LoginResult::CredentialsInvalid);
    }

    // Delete existing Session if `device_id` specified; else generate a
    // `device_id`.
    let user_id = user_id_opt.unwrap();
    let device_id = match login_request.device_id.clone() {
        Some(device_id) => {
            crate::repo::auth::invalidate_existing_sessions(user_id, &device_id, db_pool).await?;
            device_id
        },
        None =>
            uuid::Uuid::new_v4().to_string()
    };

    // Create Session and JWT
    let session = crate::repo::auth::create_session(
        user_id,
        &device_id,
        &login_request.initial_device_display_name,
        db_pool
    ).await?;

    let access_token = services::jwt::create_jwt(&session.uuid.to_string())?;

    Ok(LoginResult::LoggedIn { access_token, device_id, username, expires_in_ms: services::jwt::JWT_TTL_SECONDS })
}
