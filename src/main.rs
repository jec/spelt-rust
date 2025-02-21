use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::{from_fn, Logger, Next};
use actix_web::{web, App, Error, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::path::PathBuf;
use twelf::reexports::log;
use twelf::reexports::log::LevelFilter;

mod cli;
mod config;
mod error;
mod extractors;
mod middleware;
mod routes;
mod services;
mod store;

#[derive(Debug)]
struct AppState {
    config: config::Config,
    db_pool: Option<Pool<Postgres>>,
}

/// TODO: Redact secrets or remove this.
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

    // If command is given, run it and exit.
    if args.command.is_some() {
        cli::run_command(&args, &pool).await;
        return Ok(());
    }

    env_logger::Builder::new().filter_level(LevelFilter::Debug).init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(from_fn(middleware::auth::authenticator))
            .wrap(from_fn(log_request_params))
            .app_data(web::Data::new(AppState {
                config: conf.clone(),
                db_pool: Some(pool.clone()),
            }))
            .service(routes::info::versions)
            .service(routes::info::server_names)
            .service(routes::auth::check_validity)
            .service(routes::auth::login_types)
            .service(routes::auth::log_in)
            .service(routes::auth::log_out)
    })
        .bind((bind_address, port))?
        .run()
        .await?;

    Ok(())
}
