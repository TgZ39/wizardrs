use crate::client::WizardClient;
use crate::error::*;
use crate::server::builder::WizardServerBuilder;
use futures::StreamExt;
use indexmap::IndexMap;
use ngrok::prelude::*;
use ngrok::tunnel::TcpTunnel;
use rand::prelude::SliceRandom;
use rand::rng;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, watch, RwLock};
use tracing::{debug, info};
use url::Url;
use uuid::Uuid;
use wizardrs_core::card::value::CardValue;
use wizardrs_core::card::Card;
use wizardrs_core::game_phase::GamePhase;
use wizardrs_core::scoreboard::ScoreBoard;
use wizardrs_core::server_event::ServerEvent;
use wizardrs_core::trump_suit::TrumpSuit;

pub mod builder;

#[derive(Debug)]
pub struct WizardServer {
    pub(crate) clients: Arc<RwLock<IndexMap<Uuid, Arc<WizardClient>>>>,
    broadcast_tx: broadcast::Sender<ServerEvent>,
    shutdown_tx: watch::Sender<bool>,
    pub local_url: Url,
    pub ngrok_url: Option<Url>,

    pub(crate) played_cards: Arc<RwLock<Vec<(Card, Arc<WizardClient>)>>>,
    pub(crate) game_phase: Arc<RwLock<GamePhase>>,
    pub(crate) current_round: Arc<AtomicU8>,
    pub(crate) current_trick: Arc<AtomicU8>, // keeps track of the current trick in the round. used to check if the next trick or the next round needs to start
    pub(crate) trump_suit: Arc<RwLock<TrumpSuit>>,
    pub(crate) player_on_turn: Arc<AtomicU8>, // index of player who is currently on turn playing a card or bidding
    pub(crate) scoreboard: Arc<RwLock<ScoreBoard>>,
}

impl WizardServer {
    pub fn builder() -> WizardServerBuilder {
        WizardServerBuilder::default()
    }

    pub async fn new(port: u16, ngrok_authtoken: Option<String>) -> Result<Arc<Self>> {
        // start local TcpListener
        let addr = format!("0.0.0.0:{port}");
        info!("starting TcpListener on {addr}");
        let listener = TcpListener::bind(addr).await?;

        // start ngrok tunnel
        let tunnel = match ngrok_authtoken {
            Some(token) => {
                let tunnel = ngrok::Session::builder()
                    .authtoken(token)
                    .connect()
                    .await?
                    .tcp_endpoint()
                    .listen()
                    .await;

                match tunnel {
                    Ok(tunnel) => {
                        info!("started ngrok tunnel on {}", tunnel.url());
                        Some(tunnel)
                    }
                    Err(_) => None,
                }
            }
            None => None,
        };

        // used to broadcast server events to all clients
        let (broadcast_tx, _broadcast_rx) = broadcast::channel(128);
        // used to signal server shutdown
        let (shutdown_tx, _shutdown_rx) = watch::channel(false);

        let local_url = format!("ws://0.0.0.0:{port}").parse::<Url>().unwrap();
        let ngrok_url = if let Some(tunnel) = &tunnel {
            let url = tunnel.url().to_string();

            let url = url.trim_start_matches("tcp");
            let mut ws = "ws".to_string();
            ws.push_str(url);
            Some(ws.parse::<Url>().unwrap())
        } else {
            None
        };

        let server = Arc::new(Self {
            clients: Arc::new(RwLock::new(IndexMap::new())),
            broadcast_tx,
            shutdown_tx,
            local_url,
            ngrok_url,
            played_cards: Arc::new(RwLock::new(Vec::new())),
            game_phase: Arc::new(RwLock::new(GamePhase::Lobby)),
            current_round: Arc::new(AtomicU8::from(0)),
            current_trick: Arc::new(AtomicU8::from(0)),
            trump_suit: Arc::new(RwLock::new(TrumpSuit::None)),
            player_on_turn: Arc::new(AtomicU8::from(0)),
            scoreboard: Arc::new(RwLock::new(ScoreBoard::new(vec![]))),
        });

        // add local TcpListener listener
        server.clone().with_tcp_listener(listener);
        // add ngrok tunnel listener if provided
        if let Some(tunnel) = tunnel {
            server.clone().with_tunnel_listener(tunnel);
        }

        Ok(server)
    }

    /// Start TcpListener for local connections
    fn with_tcp_listener(self: Arc<Self>, listener: TcpListener) {
        let server = self.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            debug!("starting TcpListener task");

            let recv_fut = async move {
                while let Ok((mut stream, addr)) = listener.accept().await {
                    // check if game has started
                    if !matches!(*self.game_phase.read().await, GamePhase::Lobby) {
                        let _ = stream.shutdown().await;
                        drop(stream);
                        continue;
                    }
                    debug!(?addr, "new connection");

                    let client =
                        WizardClient::new(stream, server.clone(), self.broadcast_tx.subscribe())
                            .await;

                    if let Ok(client) = client {
                        server.add_client(client).await;
                    }
                }
            };

            tokio::select! {
                _ = shutdown_rx.changed() => {}
                _ = recv_fut => {}
            }

            debug!("stopping TcpListener task");
        });
    }

    /// Start TcpListener for ngrok tunnel
    fn with_tunnel_listener(self: Arc<Self>, mut tunnel: TcpTunnel) {
        let server = self.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            debug!("starting TcpTunnel listener task");

            let recv_fut = async move {
                while let Some(Ok(mut conn)) = tunnel.next().await {
                    // check if game has started
                    if !matches!(*self.game_phase.read().await, GamePhase::Lobby) {
                        let _ = conn.shutdown().await;
                        drop(conn);
                        continue;
                    }
                    debug!(addr = ?conn.remote_addr(), "new connection");

                    let client =
                        WizardClient::new(conn, server.clone(), self.broadcast_tx.subscribe())
                            .await;

                    if let Ok(client) = client {
                        server.add_client(client).await;
                    }
                }
            };

            tokio::select! {
                _ = shutdown_rx.changed() => {}
                _ = recv_fut => {}
            }

            debug!("stopping TcpTunnel listener task");
        });
    }

    /// Adds WizardClient to player list, broadcasts a join event and updates the scoreboard.
    async fn add_client(self: &Arc<Self>, client: Arc<WizardClient>) {
        debug!(?client.uuid, "successfully established connection to client");

        // add client to server list
        self.clients
            .write()
            .await
            .insert(client.uuid, client.clone());

        self.update_player_list().await;

        // add client to scoreboard
        let players = self
            .clients
            .read()
            .await
            .iter()
            .map(|(uuid, client)| (client.username.to_owned(), *uuid))
            .collect();
        *self.scoreboard.write().await = ScoreBoard::new(players);

        // broadcast scoreboard change
        self.update_scoreboard().await;
    }

    /// Send ServerEvent to all clients
    pub fn broadcast_event(self: &Arc<Self>, event: ServerEvent) {
        debug!(?event, "broadcasting event");

        let _ = self.broadcast_tx.send(event);
    }

    pub(crate) async fn remove_client(self: &Arc<Self>, client: Arc<WizardClient>) {
        if self
            .clients
            .write()
            .await
            .shift_remove(&client.uuid)
            .is_some()
        {
            // remove client from scoreboard
            let players = self
                .clients
                .read()
                .await
                .iter()
                .map(|(uuid, client)| (client.username.to_owned(), *uuid))
                .collect();
            *self.scoreboard.write().await = ScoreBoard::new(players);

            // broadcast scoreboard change
            self.update_scoreboard().await;

            debug!(?client.uuid, "disconnected client");
            self.update_player_list().await;
        }
    }

    /// Shut down the server and disconnect all clients.
    pub async fn shutdown(self: &Arc<Self>) {
        // disconnect clients
        for client in self.clients.read().await.values() {
            let client = client.clone();
            tokio::spawn(async move {
                client.disconnect().await;
            });
        }

        // stop listener tasks
        self.shutdown_tx.send_replace(true);
    }

    /// Send UpdatePlayerList event to all clients
    async fn update_player_list(self: &Arc<Self>) {
        let mut players = Vec::new();
        for player in self.clients.read().await.values() {
            players.push((player.username.clone(), player.uuid));
        }

        let event = ServerEvent::UpdatePlayerList { players };
        self.broadcast_event(event);
    }

    /// Get the number of rounds to play for the current amount of players
    pub async fn max_rounds(self: &Arc<Self>) -> Option<u8> {
        if (3..=6).contains(&self.clients.read().await.len()) {
            Some((60 / self.clients.read().await.len()) as u8)
        } else {
            None
        }
    }

    /// Start round n and initiate bidding phase
    pub async fn start_round(self: &Arc<Self>, round: u8) {
        // set current round
        self.current_round.store(round, Ordering::SeqCst);

        // set current trick
        self.current_trick.store(1, Ordering::SeqCst);

        // cleanup earlier rounds
        for client in self.clients.read().await.values() {
            client.clean_data().await;
        }

        // clear played cards
        self.played_cards.write().await.clear();
        // broadcast clear cards
        let event = ServerEvent::ClearPlayedCards;
        self.broadcast_event(event);

        // shuffle deck
        let mut deck = Card::all().to_vec();
        deck.shuffle(&mut rng());

        // deal cards
        for client in self.clients.read().await.values() {
            // get num round cards of the deck
            let hand = {
                let mut hand = Vec::new();
                for _ in 0..round {
                    hand.push(deck.pop().unwrap());
                }
                hand
            };
            // send hand to client
            client.set_hand(hand).await;
        }

        // set trump suit
        let trump_card = deck.pop();
        let trump_suit = TrumpSuit::from_card(trump_card);

        *self.trump_suit.write().await = trump_suit.clone();

        // broadcast trump suit to all clients
        let event = ServerEvent::SetTrumpSuit {
            trump_suit: trump_suit.clone(),
        };
        self.broadcast_event(event);

        // start bidding phase
        *self.game_phase.write().await = GamePhase::Bidding;

        // broadcast bidding phase
        let event = ServerEvent::SetGamePhase {
            phase: GamePhase::Bidding,
        };
        self.broadcast_event(event);

        // set player on turn to first player to bid
        // first player to bid equals current round because at round 1 player at index 0 is the dealer so player at index 1 is the first player to bid
        self.set_player_on_turn(round % self.clients.read().await.len() as u8)
            .await;

        // check if trump suit is wizard and if so ask dealer to select trump color
        if matches!(
            trump_suit,
            TrumpSuit::Color(
                Card {
                    value: CardValue::Wizard,
                    ..
                },
                _
            )
        ) {
            // set dealer to player on turn
            let dealer = self.get_dealer().await;
            let dealer_index = dealer.index().await;
            self.set_player_on_turn(dealer_index).await;

            // notify dealer to select trump color
            let event = ServerEvent::RequestSelectTrumpColor;
            dealer.send_event(event);
        }

        // update scoreboard
        self.scoreboard.write().await.set_current_round(round);

        // broadcast scoreboard change to all clients
        self.update_scoreboard().await;
    }

    /// Check if the player index is the last player to bid in the current round
    pub async fn is_last_player_to_bid(self: &Arc<Self>, player_index: usize) -> bool {
        let current_round = self.current_round.load(Ordering::SeqCst);
        let num_players = self.clients.read().await.len();

        // In round 1, player index 0 is the dealer and the last player to bid.
        // So if the player index == round - 1 then the player is the last player to bid but for larger rounds the bidder jumps from e.g. player index 3 to 0.
        // To avoid this problem we modulo the current round with the number of players.
        // if (current_round % num_players) == 0 -> num_players - 1 is the index as 0 - 1 is the last player
        let index = if (current_round as usize % num_players) == 0 {
            num_players - 1
        } else {
            (current_round as usize % num_players) - 1
        };
        player_index == index
    }

    /// Returns the client that is currently dealer according to self.current_round and self.clients
    pub(crate) async fn get_dealer(self: &Arc<Self>) -> Arc<WizardClient> {
        let current_round = self.current_round.load(Ordering::SeqCst);
        let num_players = self.clients.read().await.len();

        // If current round is 1 player at index 0 is the dealer.
        // If current round is 2 player at index 1 is the dealer.
        // Index of dealer is (current_round % num_players) - 1
        // if (current_round % num_players) == 0 -> num_players - 1 is the index as 0 - 1 is the last player
        let index = if (current_round as usize % num_players) == 0 {
            num_players - 1
        } else {
            (current_round as usize % num_players) - 1
        };

        // index is always valid so we can unwrap safely
        let lock = self.clients.read().await;
        let client = lock
            .get_index(index)
            .expect("index of dealer should always be valid")
            .1;

        client.clone()
    }

    /// Returns the current client on turn according to self.player_on_turn
    pub(crate) async fn get_player_on_turn(self: &Arc<Self>) -> Arc<WizardClient> {
        let player_on_turn = self.player_on_turn.load(Ordering::SeqCst) as usize;
        let lock = self.clients.read().await;
        let client = lock
            .get_index(player_on_turn)
            .expect("player_on_turn index should always be valid")
            .1;

        client.clone()
    }

    pub async fn num_players(self: &Arc<Self>) -> usize {
        self.clients.read().await.len()
    }

    /// Returns the sum of bids of every player in the current round
    pub async fn sum_bids(self: &Arc<Self>) -> u32 {
        self.scoreboard.read().await.sum_bids()
    }

    /// Broadcasts the scoreboard to all clients
    pub async fn update_scoreboard(self: &Arc<Self>) {
        let event = ServerEvent::UpdateScoreBoard {
            scoreboard: self.scoreboard.read().await.to_owned(),
        };
        self.broadcast_event(event);
    }

    /// Sets the index of the current player on turn and broadcasts it to all clients
    pub async fn set_player_on_turn(self: &Arc<Self>, index: u8) {
        self.player_on_turn
            .store(index % self.num_players().await as u8, Ordering::SeqCst);

        // broadcast event
        let event = ServerEvent::SetPlayerOnTurn {
            index: index % self.clients.read().await.len() as u8,
        };
        self.broadcast_event(event);
    }

    /// Returns the client that is currently dealer according to self.current_round and self.clients
    pub(crate) async fn get_first_bidder(self: &Arc<Self>) -> Arc<WizardClient> {
        // If current round is 1 player at index 0 is the dealer so player at index 1 is the first bidder.
        // If current round is 2 player at index 1 is the dealer so player at index 2 is the first bidder.
        // Index of first bidder is (current_round % num_players)
        let index =
            self.current_round.load(Ordering::SeqCst) as usize % self.clients.read().await.len();

        // index is always valid so we can unwrap safely
        let lock = self.clients.read().await;
        let client = lock
            .get_index(index)
            .expect("index of first bidder should always be valid")
            .1;

        client.clone()
    }

    /// Returns the player given the index
    #[allow(dead_code)]
    pub(crate) async fn get_player(self: &Arc<Self>, index: u8) -> Arc<WizardClient> {
        let index = index % self.clients.read().await.len() as u8;

        let lock = self.clients.read().await;
        let client = lock.get_index(index as usize).unwrap().1;
        client.clone()
    }

    /// Returns whether the currently being played trick is the last trick in the round
    pub async fn is_last_trick(self: &Arc<Self>) -> bool {
        let current_round = self.current_round.load(Ordering::SeqCst);
        let current_trick = self.current_trick.load(Ordering::SeqCst);

        current_trick >= current_round
    }

    /// Returns whether every player is ready
    pub async fn everyone_ready(self: &Arc<Self>) -> bool {
        self.clients
            .read()
            .await
            .values()
            .all(|client| client.ready.load(Ordering::SeqCst))
    }
}
