use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::{from_fn, Logger, Next};
use actix_web::{web, App, Error, HttpServer};
use std::path::PathBuf;
use std::sync::LazyLock;
use surrealdb::engine::any;
use surrealdb::engine::any::Any;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Namespace;
use surrealdb::Surreal;
use twelf::reexports::log;
use twelf::reexports::log::LevelFilter;

mod cli;
mod config;
mod error;
mod extractors;
mod middleware;
mod models;
mod routes;
mod services;
mod store;
mod test;

#[derive(Debug)]
struct AppState {
    config: config::Config,
    db: Surreal<Any>,
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

    // Prepare database connection.
    let db = any::connect(format!("ws://{}:{}", conf.database.dev.hostname, conf.database.dev.port)).await?;
    db.signin(Namespace {
        namespace: conf.database.dev.namespace.as_str(),
        username: conf.database.dev.username.as_str(),
        password: conf.database.dev.password.as_str()
    }).await?;
    db.use_db(&conf.database.dev.database_name).await?;

    // If command is given, run it and exit.
    if args.command.is_some() {
        cli::run_command(&args, &db).await;
        return Ok(());
    }

    env_logger::Builder::new().filter_level(LevelFilter::Debug).init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(from_fn(middleware::auth::authenticator))
            .wrap(from_fn(log_request_params))
            .app_data(web::Data::new(AppState { config: conf.clone(), db: db.clone() }))
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
