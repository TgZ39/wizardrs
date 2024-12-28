use std::path::PathBuf;
use wizardrs_core::card::color::CardColor;
use wizardrs_core::card::Card;

pub enum Message {
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
    DownloadAndrianKennardDeck,
    LeaveLobby,
    MakeBid {
        bid: u8,
    },
    StartGame,
    SetTrumpColor {
        color: CardColor,
    },
    SendChatMessage {
        msg: String,
    },
    ImportDeck,
    RequestUpdateDeckList,
}
