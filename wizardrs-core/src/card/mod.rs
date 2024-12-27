use crate::card::color::CardColor;
use crate::card::value::CardValue;
use crate::error::*;
use egui::{include_image, ImageSource};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;

pub mod color;
pub mod value;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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

    pub fn image(&self) -> ImageSource {
        // TODO find a better way to load images
        // TODO if there are no images due to copyright. maybe make an optional feature for this

        match self.color {
            CardColor::Blue => match self.value {
                CardValue::Fool => include_image!("../../../assets/cards/blue-fool.jpg"),
                CardValue::Simple(1) => include_image!("../../../assets/cards/blue-1.jpg"),
                CardValue::Simple(2) => include_image!("../../../assets/cards/blue-2.jpg"),
                CardValue::Simple(3) => include_image!("../../../assets/cards/blue-3.jpg"),
                CardValue::Simple(4) => include_image!("../../../assets/cards/blue-4.jpg"),
                CardValue::Simple(5) => include_image!("../../../assets/cards/blue-5.jpg"),
                CardValue::Simple(6) => include_image!("../../../assets/cards/blue-6.jpg"),
                CardValue::Simple(7) => include_image!("../../../assets/cards/blue-7.jpg"),
                CardValue::Simple(8) => include_image!("../../../assets/cards/blue-8.jpg"),
                CardValue::Simple(9) => include_image!("../../../assets/cards/blue-9.jpg"),
                CardValue::Simple(10) => include_image!("../../../assets/cards/blue-10.jpg"),
                CardValue::Simple(11) => include_image!("../../../assets/cards/blue-11.jpg"),
                CardValue::Simple(12) => include_image!("../../../assets/cards/blue-12.jpg"),
                CardValue::Simple(13) => include_image!("../../../assets/cards/blue-13.jpg"),
                CardValue::Wizard => include_image!("../../../assets/cards/blue-wizard.jpg"),
                _ => unreachable!("invalid card value: {}", self.value),
            },
            CardColor::Red => {
                let source = match self.value {
                    CardValue::Fool => include_image!("../../../assets/cards/red-fool.jpg"),
                    CardValue::Simple(1) => include_image!("../../../assets/cards/red-1.jpg"),
                    CardValue::Simple(2) => include_image!("../../../assets/cards/red-2.jpg"),
                    CardValue::Simple(3) => include_image!("../../../assets/cards/red-3.jpg"),
                    CardValue::Simple(4) => include_image!("../../../assets/cards/red-4.jpg"),
                    CardValue::Simple(5) => include_image!("../../../assets/cards/red-5.jpg"),
                    CardValue::Simple(6) => include_image!("../../../assets/cards/red-6.jpg"),
                    CardValue::Simple(7) => include_image!("../../../assets/cards/red-7.jpg"),
                    CardValue::Simple(8) => include_image!("../../../assets/cards/red-8.jpg"),
                    CardValue::Simple(9) => include_image!("../../../assets/cards/red-9.jpg"),
                    CardValue::Simple(10) => include_image!("../../../assets/cards/red-10.jpg"),
                    CardValue::Simple(11) => include_image!("../../../assets/cards/red-11.jpg"),
                    CardValue::Simple(12) => include_image!("../../../assets/cards/red-12.jpg"),
                    CardValue::Simple(13) => include_image!("../../../assets/cards/red-13.jpg"),
                    CardValue::Wizard => include_image!("../../../assets/cards/red-wizard.jpg"),
                    _ => unreachable!("invalid card value: {}", self.value),
                };

                source
            }
            CardColor::Green => match self.value {
                CardValue::Fool => include_image!("../../../assets/cards/green-fool.jpg"),
                CardValue::Simple(1) => include_image!("../../../assets/cards/green-1.jpg"),
                CardValue::Simple(2) => include_image!("../../../assets/cards/green-2.jpg"),
                CardValue::Simple(3) => include_image!("../../../assets/cards/green-3.jpg"),
                CardValue::Simple(4) => include_image!("../../../assets/cards/green-4.jpg"),
                CardValue::Simple(5) => include_image!("../../../assets/cards/green-5.jpg"),
                CardValue::Simple(6) => include_image!("../../../assets/cards/green-6.jpg"),
                CardValue::Simple(7) => include_image!("../../../assets/cards/green-7.jpg"),
                CardValue::Simple(8) => include_image!("../../../assets/cards/green-8.jpg"),
                CardValue::Simple(9) => include_image!("../../../assets/cards/green-9.jpg"),
                CardValue::Simple(10) => include_image!("../../../assets/cards/green-10.jpg"),
                CardValue::Simple(11) => include_image!("../../../assets/cards/green-11.jpg"),
                CardValue::Simple(12) => include_image!("../../../assets/cards/green-12.jpg"),
                CardValue::Simple(13) => include_image!("../../../assets/cards/green-13.jpg"),
                CardValue::Wizard => include_image!("../../../assets/cards/green-wizard.jpg"),
                _ => unreachable!("invalid card value: {}", self.value),
            },
            CardColor::Yellow => match self.value {
                CardValue::Fool => include_image!("../../../assets/cards/yellow-fool.jpg"),
                CardValue::Simple(1) => include_image!("../../../assets/cards/yellow-1.jpg"),
                CardValue::Simple(2) => include_image!("../../../assets/cards/yellow-2.jpg"),
                CardValue::Simple(3) => include_image!("../../../assets/cards/yellow-3.jpg"),
                CardValue::Simple(4) => include_image!("../../../assets/cards/yellow-4.jpg"),
                CardValue::Simple(5) => include_image!("../../../assets/cards/yellow-5.jpg"),
                CardValue::Simple(6) => include_image!("../../../assets/cards/yellow-6.jpg"),
                CardValue::Simple(7) => include_image!("../../../assets/cards/yellow-7.jpg"),
                CardValue::Simple(8) => include_image!("../../../assets/cards/yellow-8.jpg"),
                CardValue::Simple(9) => include_image!("../../../assets/cards/yellow-9.jpg"),
                CardValue::Simple(10) => include_image!("../../../assets/cards/yellow-10.jpg"),
                CardValue::Simple(11) => include_image!("../../../assets/cards/yellow-11.jpg"),
                CardValue::Simple(12) => include_image!("../../../assets/cards/yellow-12.jpg"),
                CardValue::Simple(13) => include_image!("../../../assets/cards/yellow-13.jpg"),
                CardValue::Wizard => include_image!("../../../assets/cards/yellow-wizard.jpg"),
                _ => unreachable!("invalid card value: {}", self.value),
            },
        }
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.color, self.value)
    }
}
