use crate::error;
use crate::models::auth::{Session, User};
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{err, ok, Ready};

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
pub struct AuthenticatedUser<'a> {
    pub user: &'a User,
    pub session: &'a Session,
}

impl<'a> FromRequest for AuthenticatedUser<'a> {
    type Error = error::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let extensions = req.extensions();
        let user = extensions.get::<User>();
        let session = extensions.get::<Session>();

        if user.is_none() | session.is_none() {
            return err(error::Error::Auth(String::from("Request not authenticated")));
        }

        ok(Self {
            user: user.unwrap(),
            session: session.unwrap()
        })
    }
}
