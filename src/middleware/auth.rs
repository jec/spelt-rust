use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpMessage};
use actix_web::middleware::Next;
use actix_web::web::Data;
use twelf::reexports::log;
use crate::{repo, AppState};
use crate::repo::auth::Session;

/// Authenticates the request using the Bearer token, if any
///
/// Looks for an `Authorization: Bearer xxx` header in the request and, if
/// found, validates the token and looks up the referenced Session. If
/// successful, adds the authenticated `Session` to the request `Extensions`.
pub async fn authenticator(req: ServiceRequest, next: Next<impl MessageBody>) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if let Some(auth) = req.headers().get("Authorization") {
        if let Ok(auth) = auth.to_str() {
            if auth.starts_with("Bearer") {
                let token = &auth[7..auth.len()].trim();
                log::info!("Bearer token found");

                if let Some(state) = req.app_data::<Data<AppState>>() {
                    if let Some(pool) = state.db_pool.as_ref() {
                        match repo::auth::authorize_request(token.to_string(), pool).await {
                            Ok(session) => {
                                log::info!(
                                    "Authenticated user {} with session {} on device {}",
                                    session.user_id,
                                    session.id,
                                    session.device_identifier
                                );

                                let mut extensions = req.extensions_mut();
                                extensions.insert(session);
                            },
                            Err(err) =>
                                log::info!("{}", err.to_string()),
                        }
                    }
                }
            }
        }
    }

    // call next middleware
    next.call(req).await
}

pub async fn require_authenticated(req: ServiceRequest, next: Next<impl MessageBody>) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if req.extensions().get::<Session>().is_none() {
        log::info!("Authentication required: failed");
        // Abort the middleware call chain.
        return Err(Error::from(crate::error::Error::Auth(String::from("failed"))));
    }
    log::info!("Authentication required: passed");

    // call next middleware
    next.call(req).await
}
