use crate::gui::{App, APPLICATION, ORGANIZATION, QUALIFIER};
use crate::image_cache::ImageCache;
use crate::{
    client::WizardClient,
    interaction::{Message, StateUpdate},
};
use directories::ProjectDirs;
use rfd::FileDialog;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{mpsc, Arc};
use std::{fs, thread};
use tokio::sync::Semaphore;
use tracing::{debug, error, info, instrument};
use wizardrs_core::card::value::CardValue;
use wizardrs_core::card::Card;
use wizardrs_core::client_event::ClientEvent;
use wizardrs_server::server::WizardServer;

impl App {
    #[instrument(skip(self))]
    pub fn handle_message(&self, message: Message) {
        let state_tx = self.state_tx.clone();
        let client = self.join_page.client.clone();

        tokio::spawn(async move {
            debug!(?message, "handling message");

            match message {
                Message::CreateServer { port, authtoken } => {
                    let server = WizardServer::new(port, authtoken).await.ok();
                    let update = StateUpdate::WizardServer(server);

                    state_tx
                        .send(update)
                        .expect("error sending WizardServer to GUI");
                }
                Message::JoinGame { url, username } => {
                    let (local_state_tx, local_state_rx) = mpsc::channel();

                    // forward state updates from WizardClient to GUI
                    {
                        let state_tx = state_tx.clone();

                        tokio::spawn(async move {
                            while let Ok(state) = local_state_rx.recv() {
                                let update = StateUpdate::GameState(Some(state));
                                if let Err(error) = state_tx.send(update) {
                                    error!(?error, "error sending state update to GUI");
                                }
                            }
                        });
                    }

                    // create client
                    let client = WizardClient::new(url, username, local_state_tx).await;
                    let update = StateUpdate::WizardClient(client.ok());
                    state_tx
                        .send(update)
                        .expect("error sending WizardClient to GUI");
                }
                Message::PlayCard { card } => {
                    if let Some(client) = client {
                        let event = ClientEvent::PlayCard { card };
                        client.send_event(event);
                    }
                }
                Message::Ready => {
                    if let Some(client) = client {
                        let event = ClientEvent::Ready;
                        client.send_event(event);
                    }
                }
                Message::RequestImageCache { path } => {
                    let cache = ImageCache::new(&path).ok();
                    let update = StateUpdate::ImageCache(cache);
                    state_tx
                        .send(update)
                        .expect("error sending ImageCache to GUI");
                }
                Message::DownloadAndrianKennardDeck => {
                    let download_progress = Arc::new(AtomicU8::new(0));
                    let update =
                        StateUpdate::DownloadingAdrianKennardProgress(download_progress.clone());
                    state_tx
                        .send(update)
                        .expect("error sending download progress to GUI");

                    let base_url = "https://raw.githubusercontent.com/TgZ39/wizardrs/refs/heads/master/adrian-kennard/".to_string();
                    let deck_base_path =
                        if let Some(proj_dirs) = ProjectDirs::from("de", "TgZ39", "Wizardrs") {
                            let mut path = proj_dirs.data_dir().to_path_buf();
                            path.push("decks");
                            path.push("adrian-kennard");

                            if !path.exists() {
                                fs::create_dir_all(&path).expect("error creating folders");
                            }
                            path
                        } else {
                            return;
                        };

                    // download images
                    let semaphore = Arc::new(Semaphore::new(5));
                    let mut handles = Vec::new();

                    for card in Card::all() {
                        let semaphore = semaphore.clone();

                        let deck_base_path = deck_base_path.clone();
                        let base_url = base_url.clone();

                        let progress = download_progress.clone();

                        let handle = tokio::spawn(async move {
                            let permit = semaphore.acquire().await.expect("error acquiring permit");

                            let file_name = match card.value {
                                CardValue::Fool => format!(
                                    "{}-fool.jpg",
                                    card.color.to_string().to_ascii_lowercase()
                                ),
                                CardValue::Simple(value) => format!(
                                    "{}-{value}.jpg",
                                    card.color.to_string().to_ascii_lowercase()
                                ),
                                CardValue::Wizard => format!(
                                    "{}-wizard.jpg",
                                    card.color.to_string().to_ascii_lowercase()
                                ),
                            };
                            let full_url = format!("{base_url}{file_name}");

                            // download file
                            let client = reqwest::Client::builder().build().unwrap();
                            let resp = client
                                .get(full_url)
                                .header("Host", "raw.githubusercontent.com")
                                .send()
                                .await
                                .unwrap()
                                .error_for_status()
                                .unwrap();
                            let bytes = resp.bytes().await.expect("error loading response body");

                            let mut full_path = deck_base_path.to_path_buf();
                            full_path.push(file_name);

                            let mut file = File::create(full_path).expect("error creating file");
                            file.write_all(&bytes).expect("error saving file");

                            drop(permit);
                            progress.fetch_add(1, Ordering::Relaxed);
                        });
                        handles.push(handle);
                    }

                    // wait for all tasks to finish
                    futures::future::join_all(handles).await;

                    // inform GUI download is finished
                    let update = StateUpdate::FinishedDownloadingAdrianKennard;
                    state_tx
                        .send(update)
                        .expect("error sending StateUpdate to GUI");
                }
                Message::LeaveLobby => {
                    if let Some(client) = client {
                        client.disconnect();
                    }

                    let update = StateUpdate::WizardClient(None);
                    state_tx
                        .send(update)
                        .expect("error sending StateUpdate to GUI");

                    let update = StateUpdate::GameState(None);
                    state_tx
                        .send(update)
                        .expect("error sending StateUpdate to GUI");
                }
                Message::MakeBid { bid } => {
                    if let Some(client) = client {
                        let event = ClientEvent::MakeBid { bid };
                        client.send_event(event);
                    }
                }
                Message::StartGame => {
                    if let Some(client) = client {
                        let event = ClientEvent::StartGame;
                        client.send_event(event);
                    }
                }
                Message::SetTrumpColor { color } => {
                    if let Some(client) = client {
                        let event = ClientEvent::SetTrumpColor { color };
                        client.send_event(event);
                    }
                }
                Message::SendChatMessage { msg } => {
                    if let Some(client) = client {
                        let event = ClientEvent::SendChatMessage { content: msg };
                        client.send_event(event);
                    }
                }
                Message::ImportDeck => {
                    // open file dialog
                    if let (Some(proj_dirs), Some(folder)) = (
                        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION),
                        FileDialog::new().pick_folder(),
                    ) {
                        // get folder name
                        let deck_name = match folder.file_name() {
                            Some(str) => str.to_os_string().to_string_lossy().to_string(),
                            None => return,
                        };

                        let mut deck_dir = proj_dirs.data_dir().to_path_buf();
                        deck_dir.push("decks");
                        deck_dir.push(&deck_name);

                        // check if dir already exists
                        let mut counter = 0;
                        while deck_dir.exists() {
                            counter += 1;
                            deck_dir.pop();
                            deck_dir.push(format!("{deck_name}{counter}"));
                        }

                        // create dir
                        fs::create_dir_all(&deck_dir).unwrap();

                        // copy files
                        for entry in fs::read_dir(folder).unwrap().flatten() {
                            if !entry.path().is_file() {
                                continue;
                            }

                            let file_name = entry.path().file_name().unwrap().to_os_string();
                            let mut new_path = deck_dir.clone();
                            new_path.push(file_name);

                            fs::copy(entry.path(), new_path).unwrap();
                        }

                        // update deck list
                        let mut deck_dir = proj_dirs.data_dir().to_path_buf();
                        deck_dir.push("decks");

                        let mut out = Vec::new();
                        for entry in fs::read_dir(deck_dir)
                            .expect("error reading deck dir")
                            .flatten()
                        {
                            if entry.path().is_dir() {
                                out.push(entry.path().to_path_buf());
                            }
                        }

                        let update = StateUpdate::UpdateDeckList(out);
                        state_tx
                            .send(update)
                            .expect("error sending StateUpdate to GUI");
                    }
                }
                Message::RequestUpdateDeckList => {
                    if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
                    {
                        let mut deck_dir = proj_dirs.data_dir().to_path_buf();
                        deck_dir.push("decks");
                        fs::create_dir_all(&deck_dir).expect("error creating deck dir");

                        let mut out = Vec::new();
                        for entry in fs::read_dir(deck_dir)
                            .expect("error reading deck dir")
                            .flatten()
                        {
                            if entry.path().is_dir() {
                                out.push(entry.path().to_path_buf());
                            }
                        }

                        let update = StateUpdate::UpdateDeckList(out);
                        state_tx
                            .send(update)
                            .expect("error sending StateUpdate to GUI");
                    }
                }
                Message::GetLatestRelease => {
                    thread::spawn(move || {
                        let releases = self_update::backends::github::ReleaseList::configure()
                            .repo_owner("TgZ39")
                            .repo_name("wizardrs")
                            .build()
                            .unwrap()
                            .fetch();

                        match releases {
                            Ok(releases) => {
                                info!("found releases: {releases:#?}");

                                let update = StateUpdate::LatestRelease(Some(releases[0].clone()));
                                state_tx
                                    .send(update)
                                    .expect("error sending StateUpdate to GUI");
                            }
                            Err(err) => {
                                error!(?err, "error checking for releases");

                                let update = StateUpdate::LatestRelease(None);
                                state_tx
                                    .send(update)
                                    .expect("error sending StateUpdate to GUI");
                            }
                        }
                    });
                }
            }
        });
    }
}
