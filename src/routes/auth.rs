use actix_web::{get, post, web, HttpResponse, Responder, ResponseError};
use serde::{Deserialize, Serialize};
use crate::{services, AppState};
use crate::error::ErrorResponse;
use crate::services::auth::LoginResult;

const VALIDITY_RESPONSE_JSON: &str = r#"{"valid":false}"#;
const SUPPORTED_LOGIN_TYPES_JSON: &str = r#"{"flows":[{"type":"m.login.password"}]}"#;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub identifier: UserIdentifier,
    pub device_id: String,
    pub initial_device_display_name: String,
    pub password: String,
    pub refresh_token: bool,
    pub token: String,
    pub r#type: String,
    pub user: String, // Deprecated
    pub address: String, // Deprecated
    pub medium: String, // Deprecated
}

#[derive(Deserialize)]
pub struct UserIdentifier {
    pub r#type: String,
    pub user: String,
    pub address: String,
    pub medium: String,
    pub country: String,
    pub phone: String,
}

#[derive(Serialize)]
struct LoginSuccess {
    access_token: String,
    device_id: String,
    user_id: String,
    expires_in_ms: u64,
}

/// Checks the validity of a login token
///
/// Token Authenticated Registration is not supported by this server, so this
/// always returns a 200 with `valid: false` in the payload.
///
/// See https://spec.matrix.org/v1.13/client-server-api/#token-authenticated-registration
#[get("/_matrix/client/v1/register/m.login.registration_token/validity")]
async fn check_validity() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(VALIDITY_RESPONSE_JSON)
}

/// Responds with the login flows supported by this server
///
/// Currently, the only supported login type is `m.login.password`.
///
/// See https://spec.matrix.org/v1.13/client-server-api/#get_matrixclientv3login
#[get("/_matrix/client/v3/login")]
async fn login_types() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(SUPPORTED_LOGIN_TYPES_JSON)
}

/// Authenticates a user and, if successful, responds with a token
///
/// See https://spec.matrix.org/v1.13/client-server-api/#post_matrixclientv3login
#[post("/_matrix/client/v3/login")]
async fn log_in(login: web::Json<LoginRequest>, data: web::Data<AppState>) -> impl Responder {
    let pool = data.db_pool.as_ref().unwrap();
    let homeserver = data.config.server.base_url.clone();

    match services::auth::log_in(login, pool).await {
        Ok(LoginResult::LoggedIn { access_token, device_id, username, expires_in_ms }) =>
            HttpResponse::Ok().json(LoginSuccess {
                access_token,
                device_id,
                user_id: format!("@{}:{}", username, homeserver),
                expires_in_ms
            }),
        Ok(LoginResult::BadRequest) =>
            HttpResponse::BadRequest().json(ErrorResponse {
                errcode: String::from("M_UNKNOWN"),
                error: String::from("Malformed request")
            }),
        Ok(LoginResult::NotSupported) =>
            HttpResponse::BadRequest().json(ErrorResponse {
                errcode: String::from("M_UNKNOWN"),
                error: String::from("Unsupported login type")
            }),
        Ok(LoginResult::CredentialsInvalid) =>
            HttpResponse::Forbidden().json(ErrorResponse {
                errcode: String::from("M_FORBIDDEN"),
                error: String::from("Username or password invalid")
            }),
        Err(err) => err.error_response()
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use super::*;

    #[actix_web::test]
    async fn test_check_validity() {
        let app = test::init_service(App::new().service(check_validity)).await;
        let req = test::TestRequest::get().uri("/_matrix/client/v1/register/m.login.registration_token/validity").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_login_types() {
        let app = test::init_service(App::new().service(login_types)).await;
        let req = test::TestRequest::get().uri("/_matrix/client/v3/login").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_log_in() {
        let app = test::init_service(App::new().service(log_in)).await;
        let req = test::TestRequest::post().uri("/_matrix/client/v3/login").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
