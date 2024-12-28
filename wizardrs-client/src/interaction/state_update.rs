use crate::image_cache::ImageCache;
use crate::{client::WizardClient, state::GameState};
use std::sync::Arc;
use wizardrs_server::server::WizardServer;

#[derive(Clone)]
pub enum StateUpdate {
    WizardClient(Option<Arc<WizardClient>>),
    WizardServer(Option<Arc<WizardServer>>),
    GameState(Option<GameState>),
    ImageCache(Option<ImageCache>),
    FinishedDownloadingAdrianKennard,
}
