use std::future::{ready, Ready};
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use twelf::reexports::log;

// There are two steps in middleware processing.
// 1. Middleware initialization: Middleware factory gets called with next
//    service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct AuthenticatorFactory;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for AuthenticatorFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = Authenticator<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(Authenticator { service }))
    }
}

pub struct Authenticator<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for Authenticator<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        log::info!("Hi from start. You requested: {}", req.path());
        // println!("Hi from start. You requested: {}", req.path());

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            log::info!("Hi from response");
            Ok(res)
        })
    }
}
