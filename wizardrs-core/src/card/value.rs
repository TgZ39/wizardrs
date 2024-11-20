use crate::error::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CardValue {
    Fool,
    Simple(u8),
    Wizard,
}

impl CardValue {
    pub fn new(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Fool),
            1..=13 => Ok(Self::Simple(value)),
            14 => Ok(Self::Wizard),
            _ => Err(Error::CardValueError),
        }
    }

    pub fn value(&self) -> u8 {
        match self {
            CardValue::Fool => 0,
            CardValue::Simple(value) => *value,
            CardValue::Wizard => 14,
        }
    }
}