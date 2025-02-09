use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::{services, AppState};

const VALIDITY_RESPONSE_JSON: &str = r#"{"valid":false}"#;
const SUPPORTED_LOGIN_TYPES_JSON: &str = r#"{"flows":[{"type":"m.login.password"}]}"#;

#[derive(Deserialize)]
pub struct LoginRequest {
    identifier: UserIdentifier,
    device_id: String,
    initial_device_display_name: String,
    password: String,
    refresh_token: bool,
    token: String,
    r#type: String,
    user: String, // Deprecated
    address: String, // Deprecated
    medium: String, // Deprecated
}

#[derive(Deserialize)]
pub struct UserIdentifier {
    r#type: String,
}

#[derive(Serialize)]
pub struct LoginResponse {

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
    services::auth::log_in(login, &data.db_pool).await;
    Ok::<&str, crate::error::Error>("foo")
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
