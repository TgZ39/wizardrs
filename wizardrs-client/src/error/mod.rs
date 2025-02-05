use derive_more::Display;
use std::io;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Display, Debug)]
pub enum Error {
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    EFrame(#[from] eframe::Error),
    WizardServer(#[from] wizardrs_server::error::Error),
    Io(#[from] io::Error),
    Serde(#[from] serde_json::error::Error),
    SelfUpdate(#[from] self_update::errors::Error),
    ConnectionClosed,
    Other(String),
}
