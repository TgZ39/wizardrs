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
                CardValue::Fool => include_image!("../../../assets/blue-fool.jpg"),
                CardValue::Simple(1) => include_image!("../../../assets/blue-1.jpg"),
                CardValue::Simple(2) => include_image!("../../../assets/blue-2.jpg"),
                CardValue::Simple(3) => include_image!("../../../assets/blue-3.jpg"),
                CardValue::Simple(4) => include_image!("../../../assets/blue-4.jpg"),
                CardValue::Simple(5) => include_image!("../../../assets/blue-5.jpg"),
                CardValue::Simple(6) => include_image!("../../../assets/blue-6.jpg"),
                CardValue::Simple(7) => include_image!("../../../assets/blue-7.jpg"),
                CardValue::Simple(8) => include_image!("../../../assets/blue-8.jpg"),
                CardValue::Simple(9) => include_image!("../../../assets/blue-9.jpg"),
                CardValue::Simple(10) => include_image!("../../../assets/blue-10.jpg"),
                CardValue::Simple(11) => include_image!("../../../assets/blue-11.jpg"),
                CardValue::Simple(12) => include_image!("../../../assets/blue-12.jpg"),
                CardValue::Simple(13) => include_image!("../../../assets/blue-13.jpg"),
                CardValue::Wizard => include_image!("../../../assets/blue-wizard.jpg"),
                _ => unreachable!("invalid card value: {}", self.value),
            },
            CardColor::Red => {
                let source = match self.value {
                    CardValue::Fool => include_image!("../../../assets/red-fool.jpg"),
                    CardValue::Simple(1) => include_image!("../../../assets/red-1.jpg"),
                    CardValue::Simple(2) => include_image!("../../../assets/red-2.jpg"),
                    CardValue::Simple(3) => include_image!("../../../assets/red-3.jpg"),
                    CardValue::Simple(4) => include_image!("../../../assets/red-4.jpg"),
                    CardValue::Simple(5) => include_image!("../../../assets/red-5.jpg"),
                    CardValue::Simple(6) => include_image!("../../../assets/red-6.jpg"),
                    CardValue::Simple(7) => include_image!("../../../assets/red-7.jpg"),
                    CardValue::Simple(8) => include_image!("../../../assets/red-8.jpg"),
                    CardValue::Simple(9) => include_image!("../../../assets/red-9.jpg"),
                    CardValue::Simple(10) => include_image!("../../../assets/red-10.jpg"),
                    CardValue::Simple(11) => include_image!("../../../assets/red-11.jpg"),
                    CardValue::Simple(12) => include_image!("../../../assets/red-12.jpg"),
                    CardValue::Simple(13) => include_image!("../../../assets/red-13.jpg"),
                    CardValue::Wizard => include_image!("../../../assets/red-wizard.jpg"),
                    _ => unreachable!("invalid card value: {}", self.value),
                };

                source
            }
            CardColor::Green => match self.value {
                CardValue::Fool => include_image!("../../../assets/green-fool.jpg"),
                CardValue::Simple(1) => include_image!("../../../assets/green-1.jpg"),
                CardValue::Simple(2) => include_image!("../../../assets/green-2.jpg"),
                CardValue::Simple(3) => include_image!("../../../assets/green-3.jpg"),
                CardValue::Simple(4) => include_image!("../../../assets/green-4.jpg"),
                CardValue::Simple(5) => include_image!("../../../assets/green-5.jpg"),
                CardValue::Simple(6) => include_image!("../../../assets/green-6.jpg"),
                CardValue::Simple(7) => include_image!("../../../assets/green-7.jpg"),
                CardValue::Simple(8) => include_image!("../../../assets/green-8.jpg"),
                CardValue::Simple(9) => include_image!("../../../assets/green-9.jpg"),
                CardValue::Simple(10) => include_image!("../../../assets/green-10.jpg"),
                CardValue::Simple(11) => include_image!("../../../assets/green-11.jpg"),
                CardValue::Simple(12) => include_image!("../../../assets/green-12.jpg"),
                CardValue::Simple(13) => include_image!("../../../assets/green-13.jpg"),
                CardValue::Wizard => include_image!("../../../assets/green-wizard.jpg"),
                _ => unreachable!("invalid card value: {}", self.value),
            },
            CardColor::Yellow => match self.value {
                CardValue::Fool => include_image!("../../../assets/yellow-fool.jpg"),
                CardValue::Simple(1) => include_image!("../../../assets/yellow-1.jpg"),
                CardValue::Simple(2) => include_image!("../../../assets/yellow-2.jpg"),
                CardValue::Simple(3) => include_image!("../../../assets/yellow-3.jpg"),
                CardValue::Simple(4) => include_image!("../../../assets/yellow-4.jpg"),
                CardValue::Simple(5) => include_image!("../../../assets/yellow-5.jpg"),
                CardValue::Simple(6) => include_image!("../../../assets/yellow-6.jpg"),
                CardValue::Simple(7) => include_image!("../../../assets/yellow-7.jpg"),
                CardValue::Simple(8) => include_image!("../../../assets/yellow-8.jpg"),
                CardValue::Simple(9) => include_image!("../../../assets/yellow-9.jpg"),
                CardValue::Simple(10) => include_image!("../../../assets/yellow-10.jpg"),
                CardValue::Simple(11) => include_image!("../../../assets/yellow-11.jpg"),
                CardValue::Simple(12) => include_image!("../../../assets/yellow-12.jpg"),
                CardValue::Simple(13) => include_image!("../../../assets/yellow-13.jpg"),
                CardValue::Wizard => include_image!("../../../assets/yellow-wizard.jpg"),
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
