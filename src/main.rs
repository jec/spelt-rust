use std::path::PathBuf;
use actix_web::{web, App, HttpServer};
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

mod cli;
mod config;
mod error;
mod routes;

struct AppState {
    config: config::Config,
    db_pool: Option<Pool<Postgres>>,
}

#[actix_web::main]
async fn main() -> Result<(), error::Error> {
    let args = cli::parse();
    let conf = config::load(PathBuf::from(args.config_file))?;
    // Make copies of these to use in the bind() call.
    let bind_address = conf.server.bind_address.clone();
    let port = conf.server.port;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&conf.database.dev_uri.as_ref().expect("Value not found for config key: database.dev_uri"))
        .await?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                config: conf.clone(),
                db_pool: Some(pool.clone()),
            }))
            .service(routes::info::versions)
            .service(routes::info::server_names)
    })
        .bind((bind_address, port))?
        .run()
        .await?;

    Ok(())
}
