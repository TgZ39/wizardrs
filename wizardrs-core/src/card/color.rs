use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, EnumIter, Display, PartialEq)]
pub enum CardColor {
    Blue,
    Red,
    Green,
    Yellow,
}
