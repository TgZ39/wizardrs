use crate::card::color::CardColor;
use crate::card::value::CardValue;
use crate::error::*;
use std::cmp::PartialEq;
use strum::IntoEnumIterator;

pub mod color;
pub mod value;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Card {
    pub color: CardColor,
    pub value: CardValue,
}

impl Card {
    pub fn new(value: u8, color: CardColor) -> Result<Self> {
        let value = CardValue::try_from(value)?;

        Ok(Self { color, value })
    }

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

    /// Returns
    /// - `true` if self beats other
    /// - `false` if other beats self
    /// - `true` if self and other are equal
    pub fn beats(&self, other: &Card, trump_color: CardColor) -> bool {
        use crate::card::CardValue::*;

        match (self.color == trump_color, other.color == trump_color) {
            (true, false) => match (self.value, other.value) {
                (Fool, Fool)
                | (Wizard, Wizard)
                | (Wizard, Fool)
                | (Simple(_), Fool)
                | (Wizard, Simple(_))
                | (Simple(_), Simple(_)) => true,
                (Fool, Wizard) | (Fool, Simple(_)) | (Simple(_), Wizard) => false,
            },
            (false, true) => match (self.value, other.value) {
                (Fool, Fool)
                | (Wizard, Wizard)
                | (Wizard, Fool)
                | (Simple(_), Fool)
                | (Wizard, Simple(_)) => true,
                (Fool, Wizard)
                | (Fool, Simple(_))
                | (Simple(_), Wizard)
                | (Simple(_), Simple(_)) => false,
            },
            (true, true) | (false, false) => match (self.value, other.value) {
                (Fool, Fool)
                | (Wizard, Wizard)
                | (Wizard, Fool)
                | (Simple(_), Fool)
                | (Wizard, Simple(_)) => true,
                (Fool, Wizard) | (Fool, Simple(_)) | (Simple(_), Wizard) => false,
                (Simple(slf), Simple(oth)) => slf >= oth,
            },
        }
    }
}
