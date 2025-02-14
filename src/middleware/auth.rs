use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpMessage};
use actix_web::middleware::Next;
use actix_web::web::Data;
use twelf::reexports::log;
use crate::{services, AppState};

/// Authenticates the request using the Bearer token, if any
///
/// Looks for an `Authorization: Bearer xxx` header in the request and, if
/// found, validates the token and looks up the referenced Session. If
/// successful, adds the authenticated [`Session`] to the request [`Extensions`].
///
/// See [`AuthenticatedUser`] for the mechanism that request handlers must use
/// to enforce authentication.
///
/// [`Session`]: crate::repo::auth::Session
/// [`Extensions`]: actix_web::dev::Extensions
/// [`AuthenticatedUser`]: crate::extractors::authenticated_user::AuthenticatedUser
pub async fn authenticator(req: ServiceRequest, next: Next<impl MessageBody>) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if let Some(auth) = req.headers().get("Authorization") {
        if let Ok(auth) = auth.to_str() {
            if auth.starts_with("Bearer") {
                let token = &auth[7..auth.len()].trim();
                log::info!("Bearer token found");

                if let Some(state) = req.app_data::<Data<AppState>>() {
                    if let Some(pool) = state.db_pool.as_ref() {
                        match services::auth::authorize_request(&token.to_string(), pool).await {
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
