use crate::AppState;
use actix_web::{get, web, Responder};
use serde::Serialize;
use std::collections::HashMap;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use actix_web::web::Bytes;
    use actix_web::{test, App};
    use std::collections::HashSet;
    use surrealdb::engine::any;

    #[actix_web::test]
    async fn test_get_versions() {
        let app = test::init_service(App::new().service(versions)).await;
        let req = test::TestRequest::get().uri("/_matrix/client/versions").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let result = test::read_body(resp).await;
        assert_eq!(result, Bytes::from_static(b"{\"versions\":[\"v1.13\"]}"));
    }

    #[actix_web::test]
    async fn test_get_server_names() {
        let conf = Config::test();
        let db = any::connect("mem://").await.unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    config: conf.clone(),
                    db: db,
                }))
                .service(server_names)
        ).await;
        let req = test::TestRequest::get().uri("/.well-known/matrix/client").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let result: HashMap<String, String> = test::read_body_json(resp).await;
        let keys: HashSet<String> = result.keys().cloned().collect();
        let expected_keys: HashSet<String> = vec!["m.homeserver", "m.identity_server"].into_iter().map(|m| m.to_string()).collect();
        assert_eq!(keys, expected_keys);

        assert!(result.get("m.homeserver").unwrap().eq(&conf.server.base_url));
        assert!(result.get("m.identity_server").unwrap().eq(&conf.server.identity_server));
    }
}
