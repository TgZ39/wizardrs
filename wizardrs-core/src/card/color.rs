use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, EnumIter, Display)]
pub enum CardColor {
    Blue,
    Red,
    Green,
    Yellow
}