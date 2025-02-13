use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpMessage};
use actix_web::middleware::Next;
use twelf::reexports::log;
use crate::repo::auth::Session;

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
