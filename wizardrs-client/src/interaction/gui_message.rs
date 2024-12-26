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
    Ready, // ready button clicked
           // MakeBid {
           //     bid: u8
           // },
           // SetTrumpColor {
           //     color: CardColor
           // }
}
