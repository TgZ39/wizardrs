use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[derive(
    Clone,
    Copy,
    Debug,
    EnumIter,
    Display,
    PartialEq,
    Serialize,
    Deserialize,
    Hash,
    Ord,
    PartialOrd,
    Eq,
)]
pub enum CardColor {
    Blue,
    Red,
    Green,
    Yellow,
}
