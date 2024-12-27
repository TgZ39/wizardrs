use crate::card::color::CardColor;
use crate::card::value::CardValue;
use crate::error::*;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;

pub mod color;
pub mod value;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq)]
pub struct Card {
    pub color: CardColor,
    pub value: CardValue,
}

impl Card {
    pub fn new(value: u8, color: CardColor) -> Result<Self> {
        let value = CardValue::new(value)?;

        Ok(Self { color, value })
    }

    /// Returns an array of all 60 cards in a wizard deck.
    pub fn all() -> [Card; 60] {
        let default = Card::new(0, CardColor::Blue).unwrap();
        let mut cards = [default; 60];

        let mut index = 0;
        for color in CardColor::iter() {
            for value in 0..=14u8 {
                let card = Card::new(value, color).unwrap();
                cards[index] = card;
                index += 1;
            }
        }

        cards
    }

    pub fn is_wizard(&self) -> bool {
        matches!(self.value, CardValue::Wizard)
    }

    pub fn is_fool(&self) -> bool {
        matches!(self.value, CardValue::Fool)
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.color, self.value)
    }
}
