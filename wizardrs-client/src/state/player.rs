use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Player {
    pub username: String,
    pub uuid: Uuid,
    pub bid: Option<u8>,
}
