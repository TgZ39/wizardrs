use derive_more::Display;
use std::io;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Display, Debug)]
pub enum Error {
    WebSocket(#[from] tokio_tungstenite::tungstenite::error::Error),
    IO(#[from] io::Error),
    Ngrok(#[from] ngrok::session::ConnectError),
    WizardServerBuilder(#[from] crate::server::builder::WizardServerBuilderError),
    ConnectionClosed,
}
