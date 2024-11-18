use strum::IntoEnumIterator;
use crate::card::color::CardColor;
use crate::card::value::CardValue;
use crate::error::*;

pub mod color;
pub mod value;

#[derive(Copy, Clone, Debug)]
pub struct Card {
    pub color: CardColor,
    pub value: CardValue
}

impl Card {
    pub fn new(value: u8, color: CardColor) -> Result<Self> {
        let value = CardValue::try_from(value)?;

        Ok(Self {
            color,
            value
        })
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
}