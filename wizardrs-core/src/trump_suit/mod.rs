use serde::{Deserialize, Serialize};

use crate::card::{color::CardColor, value::CardValue, Card};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TrumpSuit {
    Card(Card),                     // normal card
    Color(Card, Option<CardColor>), // card is a wizard and color is chosen by dealer or fool
    None,                           // there is no trump card
}

impl TrumpSuit {
    pub fn from_card(card: Option<Card>) -> Self {
        if let Some(card) = card {
            match card.value {
                CardValue::Simple(_) => Self::Card(card),
                CardValue::Wizard | CardValue::Fool => Self::Color(card, None),
            }
        } else {
            Self::None
        }
    }

    pub fn color(&self) -> Option<CardColor> {
        match self {
            Self::Card(card) => Some(card.color),
            Self::Color(_, color) => *color,
            Self::None => None,
        }
    }

    /// Sets the color of the card if it is of type Color
    pub fn set_color(&mut self, color: CardColor) {
        if let TrumpSuit::Color(_, maybe) = self {
            *maybe = Some(color);
        }
    }
}
