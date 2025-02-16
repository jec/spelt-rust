use actix_web::{FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{err, ok, Ready};
use crate::error;
use crate::repo::auth::Session;

/// An Actix extractor that retrieves the current authenticated User ID and
/// Session ID
///
/// Endpoints that require authentication must include this as a parameter in
/// their handler functions.
///
/// This extractor relies on the middleware function [`authenticator()`] to have
/// found and validated an Authorization header, which places the authenticated
/// [`Session`] in the request Extensions.
///
/// If this extractor fails to find a [`Session`] in the request Extensions,
/// then there is no valid authentication for the request, and this will return
/// a 401/Unauthorized.
///
/// # Example
///
/// Notice the `AuthenticatedUser` parameter in the handler function.
///
/// ```
/// #[post("/_matrix/client/v3/logout")]
/// async fn log_out(auth: AuthenticatedUser, data: web::Data<AppState>) -> impl Responder {
///     //...
/// }
/// ```
///
/// [`authenticator()`]: crate::middleware::auth::authenticator
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub session_id: i64,
}

impl FromRequest for AuthenticatedUser {
    type Error = error::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let extensions = req.extensions();
        match extensions.get::<Session>() {
            Some(session) =>
                ok(Self {
                    user_id: session.user_id,
                    session_id: session.id,
                }),
            None =>
                err(error::Error::Auth(String::from("Request not authenticated"))),
        }
    }
}
