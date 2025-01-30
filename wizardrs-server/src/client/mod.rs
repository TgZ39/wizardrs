use crate::error::*;
use crate::server::WizardServer;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::{broadcast, mpsc, watch, RwLock};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
use tracing::debug;
use uuid::Uuid;
use wizardrs_core::card::Card;
use wizardrs_core::client_event::ClientEvent;
use wizardrs_core::server_event::ServerEvent;

pub(crate) mod handle_broadcast;
pub(crate) mod handle_client_event;

#[derive(Debug)]
pub(crate) struct WizardClient {
    pub username: String,
    pub uuid: Uuid,
    server: Arc<WizardServer>,
    event_tx: mpsc::UnboundedSender<ServerEvent>, // send events to client
    leave_tx: watch::Sender<bool>,                // used to notify tasks to shut down

    pub hand: Arc<RwLock<Vec<Card>>>,
    pub bid: Arc<AtomicU8>,
    pub won_tricks: Arc<AtomicU8>,
    pub ready: Arc<AtomicBool>,
}

impl WizardClient {
    pub async fn new<S: AsyncWrite + AsyncRead + Unpin + Send + 'static>(
        stream: S,
        server: Arc<WizardServer>,
        broadcast_rx: broadcast::Receiver<ServerEvent>,
    ) -> Result<Arc<Self>> {
        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        let (event_tx, event_rx) = unbounded_channel();
        let (leave_tx, _leave_rx) = watch::channel(false);

        let uuid = Uuid::new_v4();

        // send UUID to client
        let uuid_event = ServerEvent::SetUUID { uuid };
        let json = serde_json::to_string(&uuid_event).unwrap();
        let msg = Message::text(json);

        if write.send(msg).await.is_err() {
            return Err(Error::ConnectionClosed);
        }

        // get username event from client
        let username = {
            let mut name = None;

            while let Some(Ok(msg)) = read.next().await {
                if let Ok(ClientEvent::SetUsername { username }) =
                    serde_json::from_str::<ClientEvent>(&msg.to_string())
                {
                    name = Some(username);
                    break;
                }
            }

            if name.is_none() {
                debug!("connection closed unexpectedly");
                return Err(Error::ConnectionClosed);
            }

            name
        };

        let client = Arc::new(Self {
            uuid,
            username: username.unwrap(),
            event_tx,
            leave_tx,
            server,

            hand: Arc::new(RwLock::new(Vec::new())),
            bid: Arc::new(AtomicU8::new(0)),
            won_tricks: Arc::new(AtomicU8::new(0)),
            ready: Arc::new(AtomicBool::new(false)),
        });

        client.spawn_event_receiver(read);
        client.spawn_event_sender(write, event_rx, broadcast_rx);

        Ok(client)
    }

    fn spawn_event_receiver<S: AsyncWrite + AsyncRead + Unpin + Send + 'static>(
        self: &Arc<Self>,
        mut read: SplitStream<WebSocketStream<S>>,
    ) {
        let client = self.clone();
        let mut leave_rx = self.leave_tx.subscribe();

        // receives events from external client
        tokio::spawn(async move {
            debug!(?client.uuid, "starting event receiver task");

            let c = client.clone();
            let recv_fut = async move {
                while let Some(Ok(msg)) = read.next().await {
                    if let Ok(event) = serde_json::from_str::<ClientEvent>(&msg.to_string()) {
                        debug!(?event, "received event from client");

                        c.handle_client_event(event).await;
                    }
                }
            };

            tokio::select! {
                _ = leave_rx.changed() => {}
                _ = recv_fut => {}
            }

            debug!(?client.uuid, "stopping event receiver task");
            client.disconnect().await;
        });
    }

    fn spawn_event_sender<S: AsyncWrite + AsyncRead + Unpin + Send + 'static>(
        self: &Arc<Self>,
        mut write: SplitSink<WebSocketStream<S>, Message>,
        mut event_rx: mpsc::UnboundedReceiver<ServerEvent>,
        mut broadcast_rx: broadcast::Receiver<ServerEvent>,
    ) {
        // send events to client
        {
            let client = self.clone();
            let mut leave_rx = self.leave_tx.subscribe();

            tokio::spawn(async move {
                debug!(?client.uuid, "starting event sender task");

                let c = client.clone();
                let send_fut = async move {
                    while let Some(event) = event_rx.recv().await {
                        debug!(?event, "sending event to client");

                        let json = serde_json::to_string(&event).unwrap();
                        let msg = Message::text(json);

                        if write.send(msg).await.is_err() {
                            c.disconnect().await;
                        }
                    }
                };

                tokio::select! {
                    _ = leave_rx.changed() => {}
                    _ = send_fut => {}
                }

                debug!(?client.uuid, "stopping event sender task");
                client.disconnect().await;
            });
        }

        // forward events from broadcast to event sender
        {
            let client = self.clone();
            let mut leave_rx = self.leave_tx.subscribe();

            tokio::spawn(async move {
                debug!(?client.uuid, "starting event forwarding task");

                let c = client.clone();
                let broadcast_fut = async move {
                    while let Ok(event) = broadcast_rx.recv().await {
                        c.handle_broadcast_event(event).await;
                    }
                };

                tokio::select! {
                    _ = leave_rx.changed() => {}
                    _ = broadcast_fut => {}
                }

                debug!(?client.uuid, "stopping event forwarding task");
                client.disconnect().await;
            });
        }
    }

    /// Sends a ServerEvent to the remote client.
    pub fn send_event(self: &Arc<Self>, event: ServerEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Shuts down websocket and removes self from server.
    pub async fn disconnect(self: &Arc<Self>) {
        // tell tasks to shut down
        self.leave_tx.send_replace(true);

        // remove self from server
        self.server.remove_client(self.clone()).await;
    }

    /// Clears hand, bid, and won_tricks.
    pub async fn clean_data(self: &Arc<Self>) {
        self.ready.store(false, Ordering::SeqCst);
        self.hand.write().await.clear();
        self.won_tricks.store(0, Ordering::SeqCst);
        self.bid.store(0, Ordering::SeqCst);
    }

    /// Sets hand of self and sends it to the remote client
    pub async fn set_hand(self: &Arc<Self>, hand: Vec<Card>) {
        *self.hand.write().await = hand.clone();

        let event = ServerEvent::SetHand { hand };
        self.send_event(event);
    }

    /// Checks whether self is the last player to bid in current round.
    pub async fn is_last_player_to_bid(self: &Arc<Self>) -> bool {
        let current_round = self.server.current_round.load(Ordering::SeqCst);
        let num_players = self.server.num_players().await;
        let self_index = self
            .server
            .clients
            .read()
            .await
            .get_index_of(&self.uuid)
            .expect("self UUID should always be in server client list");

        // In round 1, player index 0 is the dealer and the last player to bid.
        // So if the player index == round - 1 then the player is the last player to bid but for larger rounds the bidder jumps from e.g. player index 3 to 0.
        // To avoid this problem we modulo the current round with the number of players.
        // If (current_round as usize % num_players) == 0 -> num_players is the index as 0 - 1 "underflows" to num_players
        let last_index = if (current_round as usize % num_players) == 0 {
            num_players - 1
        } else {
            (current_round as usize % num_players) - 1
        };
        self_index == last_index
    }

    /// Plays a card from the own hand and broadcasts it to all clients.
    pub async fn play_card(self: &Arc<Self>, card: Card) {
        // remove card from hand
        self.hand
            .write()
            .await
            .retain(|hand_card| *hand_card != card);

        // add card to current trick
        self.server
            .played_cards
            .write()
            .await
            .push((card, self.clone()));

        // broadcast play card event
        let event = ServerEvent::PlayerPlayCard {
            uuid: self.uuid,
            card,
        };
        self.server.broadcast_event(event);
    }

    /// Returns the index of self in server
    pub async fn index(self: &Arc<Self>) -> u8 {
        self.server
            .clients
            .read()
            .await
            .get_index_of(&self.uuid)
            .unwrap() as u8
    }
}
