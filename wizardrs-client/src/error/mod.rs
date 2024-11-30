use derive_more::Display;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Display, Debug)]
pub enum Error {
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    EFrame(#[from] eframe::Error),
    WizardServer(#[from] wizardrs_server::error::Error),
    ConnectionClosed,
}
