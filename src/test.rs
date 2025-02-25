use std::future::Future;
use surrealdb::engine::any;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

pub async fn run_with_db<F, Fut>(f: F)
where
    F: FnOnce(Surreal<Any>) -> Fut,
    Fut: Future<Output = ()>,
{
    let db = any::connect("mem://").await.unwrap();
    db.use_ns("spelt").use_db("test").await.unwrap();
    f(db).await;
    ()
}
