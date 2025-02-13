use actix_web::{FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{err, ok, Ready};
use crate::error;
use crate::repo::auth::Session;

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
