use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use twelf::reexports::{log, serde_json};
use uuid::Uuid;
use crate::error::Error;
use crate::{store, AppState};
use crate::store::events::CreateRoomEvent;
use crate::routes::rooms::CreateRoomRequest;

impl From<CreateRoomRequest> for CreateRoomEvent {
    fn from(request: CreateRoomRequest) -> Self {
        Self {
            r#type: "m.room.create".to_string(),
        }
    }
}

/// Creates a new Room and returns `Ok(room_id)`
pub async fn create_room(request: CreateRoomRequest, user_id: i64, state: &AppState) -> Result<String, Error> {
    let pool = state.db_pool.as_ref().unwrap();
    let user = store::auth::get_user(user_id, &pool).await?;

    if user.is_none() {
        log::error!("Authenticated user with ID {} not found", user_id);
        return Err(Error::Auth("Authenticated user is invalid".to_string()));
    }

    let base_url = state.config.server.base_url.clone();
    let name = format!("!{}:{}", Uuid::new_v4(), base_url);
    store::rooms::create_room(&name, &pool).await?;

    let event = store::events::CreateRoomEvent::from(request);

    store::events::create_event(&event.r#type, &event, &pool).await?;
    Ok(name)
}
