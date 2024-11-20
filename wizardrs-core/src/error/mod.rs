use derive_more::Display;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Display, Clone, Debug)]
pub enum Error {
    CardValueError,
}
