use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder, ResponseError};
use serde::{Deserialize, Serialize};
use crate::{repo, services, AppState};
use crate::error::ErrorResponse;
use crate::repo::auth::Session;
use crate::services::auth::LoginResult;

const VALIDITY_RESPONSE_JSON: &str = r#"{"valid":false}"#;
const SUPPORTED_LOGIN_TYPES_JSON: &str = r#"{"flows":[{"type":"m.login.password"}]}"#;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub identifier: Option<UserIdentifier>,
    pub device_id: Option<String>,
    pub initial_device_display_name: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<bool>,
    pub token: Option<String>,
    pub r#type: String,
    pub user: Option<String>, // Deprecated
    pub address: Option<String>, // Deprecated
    pub medium: Option<String>, // Deprecated
}

#[derive(Debug, Deserialize)]
pub struct UserIdentifier {
    pub r#type: String,
    pub user: Option<String>,
    pub address: Option<String>,
    pub medium: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
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
async fn log_in(login_request: web::Json<LoginRequest>, data: web::Data<AppState>) -> impl Responder {
    let pool = data.db_pool.as_ref().unwrap();
    let homeserver = data.config.server.base_url.clone();

    println!("{:?}", login_request);

    match services::auth::log_in(login_request, pool).await {
        Ok(LoginResult::LoggedIn { access_token, device_id, username, expires_in_ms }) =>
            HttpResponse::Ok().json(LoginSuccess {
                access_token,
                device_id,
                user_id: format!("@{}:{}", username, homeserver),
                expires_in_ms
            }),
        Ok(LoginResult::BadRequest) => {
            HttpResponse::BadRequest().json(ErrorResponse {
                errcode: String::from("M_UNKNOWN"),
                error: String::from("Malformed request")
            })
        },
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

#[post("/_matrix/client/v3/logout")]
async fn log_out(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let pool = data.db_pool.as_ref().unwrap();
    let extensions = req.extensions();
    let session = extensions.get::<Session>().unwrap();

    match repo::auth::log_out(session.id, pool).await {
        Ok(_) =>
            HttpResponse::Ok().json("{}"),
        Err(err) => err.error_response()
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use actix_web::http::StatusCode;
    use actix_web::web;
    use sqlx::PgPool;
    use crate::config::Config;
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

    #[derive(Serialize)]
    struct RequestWithIdentifier {
        r#type: String,
        identifier: RequestIdentifier,
        password: String,
    }

    #[derive(Serialize)]
    struct RequestIdentifier {
        r#type: String,
        user: String,
    }

    #[sqlx::test]
    async fn test_log_in(pool: PgPool) {
        let (user, password) = crate::repo::auth::tests::create_test_user(&pool).await;
        let payload = RequestWithIdentifier {
            r#type: "m.login.password".to_string(),
            identifier: RequestIdentifier {
                r#type: String::from("m.id.user"),
                user: user.name,
            },
            password
        };

        let state = AppState { config: Config::test(), db_pool: Some(pool) };
        let app = test::init_service(App::new().app_data(web::Data::new(state)).service(log_in)).await;

        let req = test::TestRequest::post().uri("/_matrix/client/v3/login").set_json(payload).to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[sqlx::test]
    async fn test_log_in_with_bad_password(pool: PgPool) {
        let (user, _password) = crate::repo::auth::tests::create_test_user(&pool).await;
        let payload = RequestWithIdentifier {
            r#type: "m.login.password".to_string(),
            identifier: RequestIdentifier {
                r#type: String::from("m.id.user"),
                user: user.name,
            },
            password: String::from("foobar"),
        };

        let state = AppState { config: Config::test(), db_pool: Some(pool) };
        let app = test::init_service(App::new().app_data(web::Data::new(state)).service(log_in)).await;

        let req = test::TestRequest::post().uri("/_matrix/client/v3/login").set_json(payload).to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[derive(Serialize)]
    struct RequestWithUser {
        r#type: String,
        user: String,
        password: String,
    }

    #[sqlx::test]
    async fn test_log_in_with_user(pool: PgPool) {
        let (user, password) = crate::repo::auth::tests::create_test_user(&pool).await;
        let payload = RequestWithUser {
            r#type: "m.login.password".to_string(),
            user: user.name,
            password
        };

        let state = AppState { config: Config::test(), db_pool: Some(pool) };
        let app = test::init_service(App::new().app_data(web::Data::new(state)).service(log_in)).await;

        let req = test::TestRequest::post().uri("/_matrix/client/v3/login").set_json(payload).to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[derive(Serialize)]
    struct RequestWithAddress {
        r#type: String,
        address: String,
        password: String,
    }

    #[sqlx::test]
    async fn test_log_in_with_address(pool: PgPool) {
        let (user, password) = crate::repo::auth::tests::create_test_user(&pool).await;
        let payload = RequestWithAddress {
            r#type: "m.login.password".to_string(),
            address: user.name,
            password
        };

        let state = AppState { config: Config::test(), db_pool: Some(pool) };
        let app = test::init_service(App::new().app_data(web::Data::new(state)).service(log_in)).await;

        let req = test::TestRequest::post().uri("/_matrix/client/v3/login").set_json(payload).to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
