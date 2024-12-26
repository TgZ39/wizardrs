use egui::Color32;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, EnumIter, Display, PartialEq, Serialize, Deserialize)]
pub enum CardColor {
    Blue,
    Red,
    Green,
    Yellow,
}

impl Into<Color32> for CardColor {
    fn into(self) -> Color32 {
        match self {
            CardColor::Blue => Color32::LIGHT_BLUE,
            CardColor::Red => Color32::LIGHT_RED,
            CardColor::Green => Color32::LIGHT_GREEN,
            CardColor::Yellow => Color32::LIGHT_YELLOW,
        }
    }
}
