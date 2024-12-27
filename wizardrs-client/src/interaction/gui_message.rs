use std::path::PathBuf;
use wizardrs_core::card::Card;

pub enum GuiMessage {
    CreateServer {
        port: u16,
        authtoken: Option<String>,
    },
    JoinGame {
        url: String,
        username: String,
    },
    PlayCard {
        card: Card,
    },
    Ready,
    RequestImageCache {
        path: PathBuf,
    },
}
