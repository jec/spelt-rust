use crate::error::ErrorResponse;
use crate::extractors::authenticated_user::AuthenticatedUser;
use crate::services::auth::LoginResult;
use crate::{services, AppState};
use actix_web::{get, post, web, HttpResponse, Responder, ResponseError};
use serde::{Deserialize, Serialize};

/// JSON response to [`check_validity()`]
const VALIDITY_RESPONSE_JSON: &str = r#"{"valid":false}"#;

/// JSON response to [`login_types()`]
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
    let db = &data.db;
    let homeserver = data.config.server.base_url.clone();

    match services::auth::log_in(login_request, db).await {
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

/// Logs out a user
///
/// See https://spec.matrix.org/v1.13/client-server-api/#post_matrixclientv3logout
#[post("/_matrix/client/v3/logout")]
async fn log_out(auth: AuthenticatedUser, data: web::Data<AppState>) -> impl Responder {
    let db = &data.db;

    match services::auth::log_out(&auth.session_id, db).await {
        Ok(_) =>
            HttpResponse::Ok().json("{}"),
        Err(err) => err.error_response()
    }
}

/// Logs out a user across all sessions and devices
///
/// See https://spec.matrix.org/v1.13/client-server-api/#post_matrixclientv3logoutall
#[post("/_matrix/client/v3/logout/all")]
async fn log_out_all(auth: AuthenticatedUser, data: web::Data<AppState>) -> impl Responder {
    let db = &data.db;

    match services::auth::log_out_all(&auth.user_id, db).await {
        Ok(_) =>
            HttpResponse::Ok().json("{}"),
        Err(err) => err.error_response()
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use super::*;
    use crate::config::Config;
    use crate::{middleware, services, store};
    use actix_web::body::to_bytes;
    use actix_web::dev::ServiceResponse;
    use actix_web::http::StatusCode;
    use actix_web::middleware::from_fn;
    use actix_web::web;
    use actix_web::{test, App};
    use surrealdb::engine::any;
    use surrealdb::engine::any::Any;
    use surrealdb::Surreal;
    use twelf::reexports::serde_json;

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

    async fn test_log_in() {
        run_with_db(async |db| {
            let (user, password) = store::auth::tests::create_test_user(&db).await;
            let payload = RequestWithIdentifier {
                r#type: "m.login.password".to_string(),
                identifier: RequestIdentifier {
                    r#type: String::from("m.id.user"),
                    user: user.name,
                },
                password
            };

            let state = AppState { config: Config::test(), db: db.clone() };
            let app = test::init_service(App::new().app_data(web::Data::new(state)).service(log_in)).await;

            let req = test::TestRequest::post()
                .uri("/_matrix/client/v3/login")
                .set_json(payload)
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());

            let jwt = access_token_from_body(resp).await;
            assert!(services::auth::authorize_request(&jwt, &db).await.is_ok());
        }.await).await;
    }

    async fn test_log_in_with_bad_password() {
        run_with_db(async |db| {
            let (user, _password) = store::auth::tests::create_test_user(&db).await;
            let payload = RequestWithIdentifier {
                r#type: "m.login.password".to_string(),
                identifier: RequestIdentifier {
                    r#type: String::from("m.id.user"),
                    user: user.name,
                },
                password: String::from("foobar"),
            };

            let state = AppState { config: Config::test(), db: db.clone() };
            let app = test::init_service(App::new().app_data(web::Data::new(state)).service(log_in)).await;

            let req = test::TestRequest::post()
                .uri("/_matrix/client/v3/login")
                .set_json(payload)
                .to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }.await).await;
    }

    #[derive(Serialize)]
    struct RequestWithUser {
        r#type: String,
        user: String,
        password: String,
    }

    async fn test_log_in_with_user() {
        run_with_db(async |db| {
            let (user, password) = store::auth::tests::create_test_user(&db).await;
            let payload = RequestWithUser {
                r#type: "m.login.password".to_string(),
                user: user.name,
                password
            };

            let state = AppState { config: Config::test(), db: db.clone() };
            let app = test::init_service(App::new().app_data(web::Data::new(state)).service(log_in)).await;

            let req = test::TestRequest::post()
                .uri("/_matrix/client/v3/login")
                .set_json(payload)
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());

            let jwt = access_token_from_body(resp).await;
            assert!(services::auth::authorize_request(&jwt, &db).await.is_ok());
        }.await).await;
    }

    #[derive(Serialize)]
    struct RequestWithAddress {
        r#type: String,
        address: String,
        password: String,
    }

    async fn test_log_in_with_address() {
        run_with_db(async |db| {
            let (user, password) = store::auth::tests::create_test_user(&db).await;
            let payload = RequestWithAddress {
                r#type: "m.login.password".to_string(),
                address: user.name,
                password
            };

            let state = AppState { config: Config::test(), db: db.clone() };
            let app = test::init_service(App::new().app_data(web::Data::new(state)).service(log_in)).await;

            let req = test::TestRequest::post()
                .uri("/_matrix/client/v3/login")
                .set_json(payload)
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());

            let jwt = access_token_from_body(resp).await;
            assert!(services::auth::authorize_request(&jwt, &db).await.is_ok());
        }.await).await;
    }

    async fn test_log_out() {
        run_with_db(async |db| {
            let (user, _password) = store::auth::tests::create_test_user(&db).await;
            let (_session, jwt) = store::auth::tests::create_test_session(&user.id, 0, &db).await;

            let state = AppState { config: Config::test(), db: db.clone() };
            let app = test::init_service(
                App::new()
                    .wrap(from_fn(middleware::auth::authenticator))
                    .app_data(web::Data::new(state))
                    .service(log_out)
            ).await;

            let req = test::TestRequest::post()
                .uri("/_matrix/client/v3/logout")
                .append_header(("Authorization", format!("Bearer {}", jwt)))
                .to_request();
            let resp = test::call_service(&app, req).await;

            assert!(resp.status().is_success());
            assert!(services::auth::authorize_request(&jwt, &db).await.is_err());
        }.await).await;
    }

    #[test]
    async fn test_log_out_all() {
        run_with_db(test_log_out_all_impl).await;
    }

    async fn test_log_out_all_impl(db: Surreal<Any>) {
        let (user, _password) = store::auth::tests::create_test_user(&db).await;
        let (_session, jwt_1) = store::auth::tests::create_test_session(&user.id, 0, &db).await;
        let (_session, jwt_2) = store::auth::tests::create_test_session(&user.id, 0, &db).await;

        let state = AppState { config: Config::test(), db: db.clone() };
        let app = test::init_service(
            App::new()
                .wrap(from_fn(middleware::auth::authenticator))
                .app_data(web::Data::new(state))
                .service(log_out_all)
        ).await;

        let req = test::TestRequest::post()
            .uri("/_matrix/client/v3/logout/all")
            .append_header(("Authorization", format!("Bearer {}", jwt_1)))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        assert!(services::auth::authorize_request(&jwt_1, &db).await.is_err());
        assert!(services::auth::authorize_request(&jwt_2, &db).await.is_err());
    }

    async fn access_token_from_body(resp: ServiceResponse) -> String {
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = std::str::from_utf8(&body_bytes).unwrap();
        let json: serde_json::Value = serde_json::from_str(body).unwrap();
        json["access_token"].as_str().unwrap().to_string()
    }

    async fn run_with_db(f: fn(Surreal<Any>) -> dyn Future<Output = ()>) {
        let db = any::connect("mem://").await.unwrap();
        f(db);
    }
}
