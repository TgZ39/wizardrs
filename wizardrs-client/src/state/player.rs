use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Player {
    pub username: String,
    pub uuid: Uuid,
}
