use super::App;
use crate::interaction::{Message, StateUpdate};
use get_if_addrs::get_if_addrs;
use tracing::{debug, instrument};

impl App {
    #[instrument(skip(self))]
    pub fn update_state(&mut self) {
        // update state
        while let Ok(update) = self.state_rx.try_recv() {
            debug!(?update, "processing state update");

            match update {
                StateUpdate::WizardClient(client) => {
                    self.join_page.is_loading = false;
                    self.join_page.client = client;
                }
                StateUpdate::WizardServer(server) => {
                    if let Some(server) = &server {
                        let mut interfaces = get_if_addrs()
                            .map(|interfaces| {
                                interfaces
                                    .into_iter()
                                    .filter_map(|interface| {
                                        let mut url = server.local_url.clone();

                                        if url.set_ip_host(interface.ip()).is_err()
                                            || interface.ip().is_ipv6()
                                        {
                                            return None;
                                        }

                                        Some((interface.name, url))
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or(vec![("unknown".to_string(), server.local_url.clone())]);

                        if let Some(url) = &server.ngrok_url {
                            interfaces.push(("ngrok".to_string(), url.clone()));
                        }

                        self.host_page.interfaces = interfaces;
                    }

                    self.host_page.is_loading = false;
                    self.host_page.server = server;
                }
                StateUpdate::GameState(game_state) => {
                    self.join_page.game_state = game_state;
                }
                StateUpdate::ImageCache(cache) => {
                    self.image_cache = cache;
                }
                StateUpdate::FinishedDownloadingAdrianKennard => {
                    self.settings_page.downloading_adrian_kennard = false;
                    self.settings_page.download_progress = None;

                    let message = Message::RequestUpdateDeckList;
                    self.handle_message(message);
                }
                StateUpdate::DownloadingAdrianKennardProgress(progress) => {
                    self.settings_page.download_progress = Some(progress);
                }
                StateUpdate::UpdateDeckList(deck) => {
                    self.settings_page.deck_paths = deck;
                }
                StateUpdate::LatestRelease(release) => {
                    self.settings_page.latest_release = Some(release);
                }
            }
        }
    }
}
