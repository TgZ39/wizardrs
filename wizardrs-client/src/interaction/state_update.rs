use crate::image_cache::ImageCache;
use crate::{client::WizardClient, state::GameState};
use std::path::PathBuf;
use std::sync::atomic::AtomicU8;
use std::sync::Arc;
use wizardrs_server::server::WizardServer;

#[derive(Debug)]
pub enum StateUpdate {
    WizardClient(Option<Arc<WizardClient>>),
    WizardServer(Option<Arc<WizardServer>>),
    GameState(Option<GameState>),
    ImageCache(Option<ImageCache>),
    UpdateDeckList(Vec<PathBuf>),
    FinishedDownloadingAdrianKennard,
    DownloadingAdrianKennardProgress(Arc<AtomicU8>),
}
