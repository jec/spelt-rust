use std::path::PathBuf;
// use actix_web::{App, HttpServer};
use neo4rs::{query, Graph};

// mod error;
mod cli;
mod routes;
mod config;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::parse();
    let conf = config::load(PathBuf::from(args.config_file))?;

    let graph = Graph::new(
        conf.database.uri,
        conf.database.username,
        conf.database.password
    ).await.unwrap();

    assert!(graph.run(query("RETURN 1")).await.is_ok());

    // HttpServer::new(|| {
    //     App::new()
    //         .service(routes::info::versions)
    //         .service(routes::info::server_names)
    // })
    //     .bind(("localhost", 8080))?
    //     .run()
    //     .await?;

    Ok(())
}
