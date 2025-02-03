use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct RoundEntry {
    pub score: Option<i32>,
    pub bid: Option<u8>,
    pub won_tricks: u8,
}
