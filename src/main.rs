use std::path::PathBuf;
use actix_web::{web, App, HttpServer};
use neo4rs::{query, Graph};

// mod error;
mod cli;
mod routes;
mod config;

struct AppState {
    config: config::Config,
    graph: Graph,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::parse();
    let conf = config::load(PathBuf::from(args.config_file))?;
    // Make copies of these to use in the bind() call.
    let bind_address = conf.server.bind_address.clone();
    let port = conf.server.port;

    let graph = Graph::new(
        &conf.database.uri,
        &conf.database.username,
        &conf.database.password
    ).await.unwrap();

    assert!(graph.run(query("RETURN 1")).await.is_ok());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                config: conf.clone(),
                graph: graph.clone(),
            }))
            .service(routes::info::versions)
            .service(routes::info::server_names)
    })
        .bind((bind_address, port))?
        .run()
        .await?;

    Ok(())
}
