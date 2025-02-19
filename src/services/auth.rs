use actix_web::web;
use sqlx::PgPool;
use crate::error::Error;
use crate::store::auth::Session;
use crate::routes::auth::LoginRequest;
use crate::{store, services};

/// Possible results of calling [`log_in()`]
pub enum LoginResult {
    LoggedIn { access_token: String, device_id: String, username: String, expires_in_ms: u64 },
    CredentialsInvalid,
    NotSupported,
    BadRequest,
}

/// Authenticates a user and, if successful, returns a `LoginResult` with a token
///
/// If the request specifies a `device_id`, any previous `Session` for that device
/// will be deleted. If the request does not specify a `device_id`, one will be
/// generated.
///
pub async fn log_in(login_request: web::Json<LoginRequest>, pool: &PgPool) -> Result<LoginResult, Error> {
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

    let user_id_opt = store::auth::validate_user_and_password(
        &username,
        &login_request.password.as_ref().unwrap(),
        pool
    ).await?;

    if user_id_opt.is_none() {
        return Ok(LoginResult::CredentialsInvalid);
    }

    // Delete existing Session if `device_id` specified; else generate a
    // `device_id`.
    let user_id = user_id_opt.unwrap();
    let device_id = match login_request.device_id.clone() {
        Some(device_id) => {
            store::auth::invalidate_existing_sessions(user_id, &device_id, pool).await?;
            device_id
        },
        None =>
            uuid::Uuid::new_v4().to_string()
    };

    // Create Session and JWT
    let session = store::auth::create_session(
        user_id,
        &device_id,
        &login_request.initial_device_display_name,
        pool
    ).await?;

    let access_token = services::jwt::create_jwt(&session.uuid.to_string(), 0)?;

    Ok(LoginResult::LoggedIn { access_token, device_id, username, expires_in_ms: services::jwt::JWT_TTL_SECONDS })
}

/// Validates the JWT signature and validates the referenced `Session`; returns
/// `Ok(s: Session)` on success
pub async fn authorize_request(access_token: &String, pool: &PgPool) -> Result<Session, Error> {
    let claims = services::jwt::validate_jwt(&access_token)?;
    let uuid = claims.sub;

    store::auth::validate_session(&uuid, pool).await
}

/// Logs out a user, invalidating any held access tokens
pub async fn log_out(session_id: i64, pool: &PgPool) -> Result<(), Error> {
    store::auth::log_out(session_id, pool).await
}

/// Logs out a user from all devices, invalidating any held access tokens
pub async fn log_out_all(user_id: i64, pool: &PgPool) -> Result<(), Error> {
    store::auth::log_out_all(user_id, pool).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use crate::services;
    use crate::store::auth::invalidate_existing_sessions;
    use crate::store::auth::tests::{create_test_session, create_test_user};

    #[sqlx::test]
    async fn test_authorize_request (pool: PgPool) {
        let (user, _password) = create_test_user(&pool).await;
        let (_session, jwt) = create_test_session(user.id, 0, &pool).await;

        let result = authorize_request(&jwt, &pool).await;
        assert!(result.is_ok());
    }

    #[sqlx::test]
    async fn test_authorize_request_without_session (pool: PgPool) {
        let (user, _password) = create_test_user(&pool).await;
        let (session, jwt) = create_test_session(user.id, 0, &pool).await;
        let _ = invalidate_existing_sessions(user.id, &session.device_identifier, &pool).await;

        let result = authorize_request(&jwt, &pool).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_authorize_request_with_expired (pool: PgPool) {
        let (user, _password) = create_test_user(&pool).await;
        let (_session, jwt) = create_test_session(user.id, -(services::jwt::JWT_TTL_SECONDS as i64) - 300, &pool).await;

        let result = authorize_request(&jwt, &pool).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_log_out(pool: PgPool) {
        let (user, _password) = create_test_user(&pool).await;
        let (session, jwt) = create_test_session(user.id, 0, &pool).await;
        assert!(authorize_request(&jwt, &pool).await.is_ok());

        assert!(log_out(session.id, &pool).await.is_ok());
        assert!(authorize_request(&jwt, &pool).await.is_err());
    }
}
