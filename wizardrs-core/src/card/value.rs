use crate::error::Error;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CardValue {
    Fool,
    Simple(u8),
    Wizard,
}

impl TryFrom<u8> for CardValue {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Fool),
            1..=13 => Ok(Self::Simple(value)),
            14 => Ok(Self::Wizard),
            _ => Err(Error::CardValueError),
        }
    }
}

impl CardValue {
    pub fn value(&self) -> u8 {
        match self {
            CardValue::Fool => 0,
            CardValue::Simple(value) => *value,
            CardValue::Wizard => 14,
        }
    }
}