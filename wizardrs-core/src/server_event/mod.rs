use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ServerEvent {
    PlayerJoinEvent { username: String, uuid: Uuid },
    PlayerLeaveEvent { username: String, uuid: Uuid }
}
