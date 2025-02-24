use crate::error::Error;
use crate::routes::rooms::CreateRoomRequest;
use crate::{store, AppState};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use twelf::reexports::{log, serde_json};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CreateRoomEvent {
    pub r#type: String,
}

impl From<CreateRoomRequest> for CreateRoomEvent {
    fn from(request: CreateRoomRequest) -> Self {
        Self {
            r#type: "m.room.create".to_string(),
        }
    }
}

/// Creates a new Room and returns `Ok(room_id)`
pub async fn create_room(request: CreateRoomRequest, user_id: &RecordId, state: &AppState) -> Result<String, Error> {
    let db = state.db.clone();
    let user = store::auth::get_user(user_id, &db).await?;

    if user.is_none() {
        log::error!("Authenticated user with ID {} not found", user_id);
        return Err(Error::Auth("Authenticated user is invalid".to_string()));
    }

    let base_url = state.config.server.base_url.clone();
    let name = format!("!{}:{}", Uuid::new_v4(), base_url);
    store::rooms::create_room(&name, &db).await?;

    let event = CreateRoomEvent::from(request);

    store::events::create_event(&event.r#type, &event, &db).await?;
    Ok(name)
}
