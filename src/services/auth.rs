use actix_web::web;
use sqlx::PgPool;
use crate::error::Error;
use crate::routes::auth::LoginRequest;

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

    // Check for existing Session

    // Create Session
    let user_id = user_id_opt.unwrap();

    let access_token = String::from("foo"); // TODO: Implement
    let device_id = String::from("bar"); // TODO: Implement
    Ok(LoginResult::LoggedIn { access_token, device_id, username, expires_in_ms: 0 })
}
