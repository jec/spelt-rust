use std::path::PathBuf;
use actix_web::{web, App, Error, HttpServer};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::{from_fn, Logger, Next};
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use twelf::reexports::log;
use twelf::reexports::log::LevelFilter;
use crate::middleware::authenticator::AuthenticatorFactory;

mod cli;
mod config;
mod error;
mod middleware;
mod repo;
mod routes;
mod services;

struct AppState {
    config: config::Config,
    db_pool: Option<Pool<Postgres>>,
}

async fn log_request_params(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    log::info!("Request: {:?}", req);
    next.call(req).await
}

#[actix_web::main]
async fn main() -> Result<(), error::Error> {
    let args = cli::parse();
    let conf = config::load(PathBuf::from(&args.config_file))?;

    // Make copies of these to use in the bind() call.
    let bind_address = conf.server.bind_address.clone();
    let port = conf.server.port;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&conf.database.dev_uri.as_ref().expect("Value not found for config key: database.dev_uri"))
        .await?;

    // Run command and exit if command is given.
    if args.command.is_some() {
        cli::run_command(&args, &pool).await;
        return Ok(());
    }

    env_logger::Builder::new().filter_level(LevelFilter::Debug).init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(AuthenticatorFactory)
            .wrap(from_fn(log_request_params))
            .app_data(web::Data::new(AppState {
                config: conf.clone(),
                db_pool: Some(pool.clone()),
            }))
            .service(routes::info::versions)
            .service(routes::info::server_names)
            .service(routes::auth::check_validity)
            .service(routes::auth::login_types)
    })
        .bind((bind_address, port))?
        .run()
        .await?;

    Ok(())
}
