use crate::client::WizardClient;
use crate::error::*;
use crate::server::builder::WizardServerBuilder;
use futures::StreamExt;
use indexmap::IndexMap;
use ngrok::prelude::*;
use ngrok::tunnel::TcpTunnel;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, watch, RwLock};
use tracing::{debug, error, info, trace};
use uuid::Uuid;
use url::Url;
use wizardrs_core::server_event::ServerEvent;

pub mod builder;

pub struct WizardServer {
    clients: Arc<RwLock<IndexMap<Uuid, Arc<WizardClient>>>>,
    broadcast_tx: broadcast::Sender<ServerEvent>,
    shutdown_tx: watch::Sender<bool>,
    pub local_url: Url,
    pub ngrok_url: Option<Url>,
}

impl WizardServer {
    pub fn builder() -> WizardServerBuilder {
        WizardServerBuilder::default()
    }

    pub async fn new(port: u16, ngrok_authtoken: Option<String>) -> Result<Arc<Self>> {
        // start local TcpListener
        let addr = format!("127.0.0.1:{port}");
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

        let (broadcast_tx, _broadcast_rx) = broadcast::channel(128);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let local_url = format!("ws://127.0.0.1:{port}").parse::<Url>().unwrap();
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
            ngrok_url
        });

        server.clone().with_tcp_listener(listener);
        if let Some(tunnel) = tunnel {
            server.clone().with_tunnel_listener(tunnel);
        }

        Ok(server)
    }

    fn with_tcp_listener(self: Arc<Self>, listener: TcpListener) {
        let server = self.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            debug!("starting TcpListener task");

            let recv_fut = async move {
                while let Ok((stream, addr)) = listener.accept().await {
                    debug!("new connection: {addr}");

                    let client =
                        WizardClient::new(stream, server.clone(), self.broadcast_tx.subscribe())
                            .await;

                    match client {
                        Ok(client) => server.add_client(client).await,
                        Err(_) => {}
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

    fn with_tunnel_listener(self: Arc<Self>, mut tunnel: TcpTunnel) {
        let server = self.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            debug!("starting TcpTunnel listener task");

            let recv_fut = async move {
                while let Some(Ok(conn)) = tunnel.next().await {
                    let client =
                        WizardClient::new(conn, server.clone(), self.broadcast_tx.subscribe())
                            .await;

                    match client {
                        Ok(client) => server.add_client(client).await,
                        Err(_) => {}
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

    /// Adds WizardClient to server and broadcasts a join event
    async fn add_client(self: &Arc<Self>, client: Arc<WizardClient>) {
        debug!("successfully established connection to {}", client.uuid);

        // add client to server list
        self.clients
            .write()
            .await
            .insert(client.uuid, client.clone());

        self.update_player_list().await;
    }

    pub fn broadcast_event(self: &Arc<Self>, event: ServerEvent) {
        trace!("broadcasting event: {:?}", event);

        let _ = self.broadcast_tx.send(event);
    }

    pub(crate) async fn remove_client(self: &Arc<Self>, client: Arc<WizardClient>) {
        if self.clients.write().await.shift_remove(&client.uuid).is_some() {
            debug!("disconnected client: {}", client.uuid);
            self.update_player_list().await;
        }
    }

    pub async fn shutdown(self: &Arc<Self>) {
        // disconnect clients
        for client in self.clients.read().await.values() {
            let client = client.clone();
            tokio::spawn(async move {
                client.disconnect().await;
            });
        }

        // stop own tasks
        self.shutdown_tx.send_replace(true);
    }

    async fn update_player_list(self: &Arc<Self>) {
        let mut players = Vec::new();
        for player in self.clients.read().await.values() {
            players.push((player.username.clone(), player.uuid));
        }

        let event = ServerEvent::UpdatePlayerList {
            players,
        };
        self.broadcast_event(event);
    }
}
