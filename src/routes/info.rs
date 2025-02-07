use std::collections::HashMap;
use actix_web::{get, web, Responder};
use serde::Serialize;
use crate::AppState;

const VERSIONS: [&str; 1] = ["v1.13"];

#[derive(Serialize)]
struct Versions {
    versions: [&'static str; 1],
}

#[get("/_matrix/client/versions")]
async fn versions() -> actix_web::Result<impl Responder> {
    let versions = Versions { versions: VERSIONS };

    Ok(web::Json(versions))
}

#[get("/.well-known/matrix/client")]
async fn server_names(data: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    let mut names: HashMap<String, String> = HashMap::new();

    names.insert(String::from("m.homeserver"), data.config.server.base_url.clone());
    names.insert(String::from("m.identity_server"), data.config.server.identity_server.clone());

    Ok(web::Json(names))
}
