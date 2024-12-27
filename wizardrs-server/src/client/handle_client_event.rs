use crate::client::WizardClient;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::warn;
use wizardrs_core::card::value::CardValue;
use wizardrs_core::card::Card;
use wizardrs_core::client_event::ClientEvent;
use wizardrs_core::game_phase::GamePhase;
use wizardrs_core::scoreboard::ScoreBoard;
use wizardrs_core::server_event::ServerEvent;
use wizardrs_core::trump_suit::TrumpSuit;
use wizardrs_core::utils::evaluate_trick_winner;

impl WizardClient {
    // Handle events being sent from the remote client to the server
    pub async fn handle_client_event(self: &Arc<Self>, event: ClientEvent) {
        match event {
            ClientEvent::SetUsername { .. } => {}
            ClientEvent::SendChatMessage { content } => {
                let event = ServerEvent::PlayerChatMessage {
                    username: self.username.clone(),
                    uuid: self.uuid,
                    content,
                };
                self.server.broadcast_event(event);
            }
            ClientEvent::StartGame => {
                warn!("StartGame by {}", self.username);
                // only start game if it hasn't started yet and enough players are online
                if matches!(*self.server.game_phase.read().await, GamePhase::Lobby)
                    && (3..=6).contains(&self.server.num_players().await)
                {
                    self.server.start_round(1).await;
                }
            }
            ClientEvent::MakeBid { bid } => {
                warn!("MakeBid: {bid} by {}", self.username);

                let current_phase = self.server.game_phase.read().await.to_owned();
                if current_phase == GamePhase::Bidding // check if it is bidding phase
                && self.uuid == self.server.get_player_on_turn().await.uuid // check if self is player on turn
                && self.server.scoreboard.read().await.get_entry(self.uuid).is_some_and(|(_, bid, _)| bid.is_none())
                // check if self has already bid
                {
                    warn!("MakeBid: {bid} passed check by {}", self.username);

                    // check if self is last player to bid
                    if self.is_last_player_to_bid().await {
                        warn!("MakeBid: {bid} is last bid by {}", self.username);

                        // check if bid is allowed
                        let sum = self.server.sum_bids().await;
                        let current_round = self.server.current_round.load(Ordering::SeqCst);
                        let disallowed_bid = current_round as i32 - sum as i32;

                        if bid as i32 == disallowed_bid {
                            return;
                        }
                    }

                    // check if bid has valid range
                    let current_round = self.server.current_round.load(Ordering::SeqCst);
                    if !(0..=current_round).contains(&bid) {
                        warn!("MakeBid: {bid} has invalid range by {}", self.username);
                        return;
                    }

                    // set bid
                    self.server.scoreboard.write().await.set_bid(self.uuid, bid);

                    // broadcast scoreboard change
                    self.server.update_scoreboard().await;

                    if self.is_last_player_to_bid().await {
                        warn!(
                            "MakeBid: {bid} is last bid (start round) by {}",
                            self.username
                        );
                        // all players have made a bid
                        // now start the round

                        // start playing phase
                        *self.server.game_phase.write().await = GamePhase::Playing;
                        // broadcast playing phase
                        let event = ServerEvent::SetGamePhase {
                            phase: GamePhase::Playing,
                        };
                        self.server.broadcast_event(event);

                        // set player on turn to first bidder because he is the player left of the dealer
                        let first_bidder = self.server.get_first_bidder().await;
                        let index = first_bidder.index().await;
                        self.server.set_player_on_turn(index as u8).await;
                    } else {
                        warn!("MakeBid: {bid} is not last id by {}", self.username);
                        // there are other players who need to make a bid so increment the player on turn by 1
                        self.server
                            .set_player_on_turn(
                                self.server.player_on_turn.load(Ordering::SeqCst) + 1,
                            )
                            .await;
                    }
                }
            }
            ClientEvent::SetTrumpColor { color } => {
                warn!("SetTrumpColor: {color} received by {}", self.username);

                let current_phase = self.server.game_phase.read().await.to_owned();
                let trump_suit = self.server.trump_suit.read().await.to_owned();

                if current_phase == GamePhase::Bidding // check if it is bidding phase
                    && self.uuid == self.server.get_dealer().await.uuid // check if self is dealer
                    && matches!(trump_suit, TrumpSuit::Color(Card { value: CardValue::Wizard, .. }, None)) // check if trump suit is already set
                    && self.uuid == self.server.get_player_on_turn().await.uuid
                // check if self is player on turn
                {
                    warn!("SetTrumpColor: {color} passed check by {}", self.username);
                    // set trump suit color
                    self.server.trump_suit.write().await.set_color(color);

                    // broadcast trump suit
                    let event = ServerEvent::SetTrumpSuit {
                        trump_suit: self.server.trump_suit.read().await.to_owned(),
                    };
                    self.server.broadcast_event(event);

                    // set player on turn to the first player to bid
                    let first_bidder = self.server.get_first_bidder().await;
                    let index_first_bidder = first_bidder.index().await;

                    self.server
                        .set_player_on_turn(index_first_bidder as u8)
                        .await;
                }
            }
            ClientEvent::PlayCard { card } => {
                warn!("PlayCard: {card} received by {}", self.username);

                // check if we are in waiting for everyone ready
                if self.server.played_cards.read().await.len() >= self.server.num_players().await {
                    // we are waiting for everyone ready
                    // ignore this event
                    return;
                }

                let current_phase = self.server.game_phase.read().await.to_owned();

                if current_phase == GamePhase::Playing // check if it is playing phase
                && self.uuid == self.server.get_player_on_turn().await.uuid
                // check if self is player on turn
                {
                    warn!("PlayCard: {card} passed check by {}", self.username);

                    // TODO check if the played card is valid

                    // play card and broadcast to all clients
                    self.play_card(card).await;

                    let is_last_player_on_turn = self.server.num_players().await
                        == self.server.played_cards.read().await.len();
                    if is_last_player_on_turn {
                        // finish the trick and wait for everyone ready before starting the next round

                        warn!(
                            "PlayCard: {card} is last player on turn by {}",
                            self.username
                        );

                        // evaluate winner
                        let cards = self
                            .server
                            .played_cards
                            .read()
                            .await
                            .iter()
                            .map(|(card, client)| (client.uuid, card.to_owned()))
                            .collect::<Vec<_>>();
                        let trump_color = self.server.trump_suit.read().await.color();

                        let (winner_uuid, _) = evaluate_trick_winner(&cards[..], trump_color);

                        // update scoreboard
                        self.server
                            .scoreboard
                            .write()
                            .await
                            .increment_won_tricks(winner_uuid);
                        self.server.update_scoreboard().await;

                        // broadcast waiting for ready
                        let event = ServerEvent::WaitingForReady { waiting: true };
                        self.server.broadcast_event(event);
                    } else {
                        warn!(
                            "PlayCard: {card} is not last player on turn by {}",
                            self.username
                        );
                        // set next player to play card
                        self.server.set_player_on_turn(self.index().await + 1).await;
                    }
                }
            }
            ClientEvent::Ready => {
                warn!("Ready received by {}", self.username);

                self.ready.store(true, Ordering::SeqCst);
                // broadcast ready event
                let event = ServerEvent::PlayerReady {
                    uuid: self.uuid,
                    ready: true,
                };
                self.server.broadcast_event(event);

                // check if everyone is ready before proceeding
                if !self.server.everyone_ready().await {
                    // not everyone is ready
                    warn!("Ready: not everyone ready by {}", self.username);
                    return;
                }

                let set_waiting_ready = |waiting: bool| {
                    let event = ServerEvent::WaitingForReady { waiting };
                    self.server.broadcast_event(event);
                };
                let reset_ready = async {
                    for client in self.server.clients.read().await.values() {
                        client.ready.store(false, Ordering::SeqCst);

                        // broadcast ready event
                        let event = ServerEvent::PlayerReady {
                            uuid: client.uuid,
                            ready: false,
                        };
                        self.server.broadcast_event(event);
                    }
                };

                let game_phase = *self.server.game_phase.read().await;
                match game_phase {
                    GamePhase::Lobby => {}
                    GamePhase::Bidding => {}
                    GamePhase::Playing => {
                        warn!("Ready: playing phase ready by {}", self.username);
                        // the trick has already been evaluated
                        // now we just start the next trick or finish the game

                        // evaluate winner
                        let cards = self
                            .server
                            .played_cards
                            .read()
                            .await
                            .iter()
                            .map(|(card, client)| (client.uuid, card.to_owned()))
                            .collect::<Vec<_>>();
                        let trump_color = self.server.trump_suit.read().await.color();

                        let (winner_uuid, _) = evaluate_trick_winner(&cards[..], trump_color);

                        if self.server.is_last_trick().await {
                            warn!("Ready: is last trick {}", self.username);
                            // it was the last trick
                            // finish round

                            // evaluate scores
                            self.server.scoreboard.write().await.apply_scores();
                            self.server.update_scoreboard().await;

                            let current_round = self.server.current_round.load(Ordering::SeqCst);

                            // check if it was the last round
                            if current_round == self.server.max_rounds().await.unwrap() {
                                // TODO finish game

                                // set game phase
                                *self.server.game_phase.write().await = GamePhase::Finished;
                                // broadcast game phase
                                let event = ServerEvent::SetGamePhase {
                                    phase: GamePhase::Finished,
                                };
                                self.server.broadcast_event(event);
                            } else {
                                // more rounds need to be played
                                self.server.start_round(current_round + 1).await;

                                // reset ready
                                set_waiting_ready(false);
                                reset_ready.await;
                            }
                        } else {
                            warn!("Ready: is not last trick by {}", self.username);
                            // start next trick
                            self.server.current_trick.fetch_add(1, Ordering::SeqCst);

                            // clear played cards
                            self.server.played_cards.write().await.clear();
                            // broadcast clear cards
                            let event = ServerEvent::ClearPlayedCards;
                            self.server.broadcast_event(event);

                            // set player on turn to winner of previous trick
                            let index = self
                                .server
                                .clients
                                .read()
                                .await
                                .get_index_of(&winner_uuid)
                                .unwrap();
                            self.server.set_player_on_turn(index as u8).await;

                            // reset ready
                            set_waiting_ready(false);
                            reset_ready.await;
                        }
                    }
                    GamePhase::Finished => {
                        // everyone is ready for the next game
                        // full reset the lobby

                        // reset clients
                        for client in self.server.clients.read().await.values() {
                            client.clean_data().await;
                        }

                        // reset server
                        self.server.played_cards.write().await.clear();
                        let event = ServerEvent::ClearPlayedCards;
                        self.server.broadcast_event(event);

                        *self.server.game_phase.write().await = GamePhase::Lobby;
                        let event = ServerEvent::SetGamePhase {
                            phase: GamePhase::Lobby,
                        };
                        self.server.broadcast_event(event);

                        self.server.current_round.store(0, Ordering::SeqCst);
                        self.server.current_trick.store(0, Ordering::SeqCst);

                        *self.server.trump_suit.write().await = TrumpSuit::None;
                        let event = ServerEvent::SetTrumpSuit {
                            trump_suit: TrumpSuit::None,
                        };
                        self.server.broadcast_event(event);

                        self.server.set_player_on_turn(0).await;

                        let players = self
                            .server
                            .clients
                            .read()
                            .await
                            .values()
                            .map(|client| (client.username.to_owned(), client.uuid))
                            .collect();
                        *self.server.scoreboard.write().await = ScoreBoard::new(players);
                        self.server.update_scoreboard().await;

                        // reset ready
                        set_waiting_ready(false);
                        reset_ready.await;
                    }
                }
            }
        }
    }
}
