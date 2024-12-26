use crate::card::color::CardColor;
use crate::card::value::CardValue;
use crate::card::Card;
use uuid::Uuid;

/// Evaluates the winner for the played trick.
///
/// # Panics
/// Panics if the cards are empty.
pub fn evaluate_trick_winner(
    cards: &[(Uuid, Card)],
    trump_color: Option<CardColor>,
) -> (Uuid, Card) {
    assert!(!cards.is_empty());

    // early find wizards
    for (uuid, card) in cards {
        if card.value == CardValue::Wizard {
            return (*uuid, card.to_owned());
        }
    }

    // all fools case
    if cards.iter().all(|(_, card)| card.value == CardValue::Fool) {
        return cards[0];
    }

    let contains_trump_color = match trump_color {
        Some(trump_color) => cards.iter().any(|(_, card)| {
            match card.value {
                CardValue::Simple(_) => card.color == trump_color,
                _ => false, // wizards and fools should not count towards trump color
            }
        }),
        None => false,
    };

    let only_cards = cards
        .iter()
        .map(|(_, card)| card.to_owned())
        .collect::<Vec<_>>();
    let leading_color = leading_color(&only_cards[..]);
    let contains_leading_color = leading_color.is_some();

    // all cards from are not wizards because they have been checked
    match (contains_trump_color, contains_leading_color) {
        (true, _) => {
            // trump color takes priority
            let trump_color = trump_color.unwrap();

            let mut winner = cards[0];

            for (uuid, card) in cards {
                if card.color == trump_color {
                    // card is trump color
                    // check if value is higher then winner
                    match card.value {
                        CardValue::Fool => continue,
                        CardValue::Simple(value) => {
                            match winner.1.value {
                                CardValue::Fool => winner = (*uuid, card.to_owned()),
                                CardValue::Simple(winner_value) => {
                                    if winner.1.color == trump_color {
                                        if value > winner_value {
                                            // new card is better
                                            winner = (*uuid, card.to_owned());
                                        }
                                    } else {
                                        // winner is not trump color so new card always wins
                                        winner = (*uuid, card.to_owned());
                                    }
                                }
                                CardValue::Wizard => unreachable!("card should not be wizard"),
                            }
                        }
                        CardValue::Wizard => unreachable!("card should not be wizard"),
                    }
                }

                // card is not trump color but at least one card is trump color so we can skip this card
            }

            winner
        }
        (false, true) => {
            // there is no trump color in cards
            // leading color takes priority
            let leading_color = leading_color.unwrap();

            let mut winner = cards[0];

            for (uuid, card) in cards {
                if card.color == leading_color {
                    // card is leading color
                    // check if value is higher then winner
                    match card.value {
                        CardValue::Fool => continue,
                        CardValue::Simple(value) => {
                            match winner.1.value {
                                CardValue::Fool => winner = (*uuid, card.to_owned()),
                                CardValue::Simple(winner_value) => {
                                    if winner.1.color == leading_color {
                                        if value > winner_value {
                                            // new card is better
                                            winner = (*uuid, card.to_owned());
                                        }
                                    } else {
                                        // winner is not leading color so new card always wins
                                        winner = (*uuid, card.to_owned());
                                    }
                                }
                                CardValue::Wizard => unreachable!("card should not be wizard"),
                            }
                        }
                        CardValue::Wizard => unreachable!("card should not be wizard"),
                    }
                }

                // card is not leading color but at least one card is leading color so we can skip this card
            }

            winner
        }
        (false, false) => {
            // no card color takes priority

            let mut winner = cards[0];

            for (uuid, card) in cards {
                match card.value {
                    CardValue::Fool => continue,
                    CardValue::Simple(value) => {
                        match winner.1.value {
                            CardValue::Fool => winner = (*uuid, card.to_owned()),
                            CardValue::Simple(winner_value) => {
                                if value > winner_value {
                                    // new card is better
                                    winner = (*uuid, card.to_owned());
                                }
                            }
                            CardValue::Wizard => unreachable!("card should not be wizard"),
                        }
                    }
                    CardValue::Wizard => unreachable!("card should not be wizard"),
                }
            }

            winner
        }
    }
}

pub fn leading_color(cards: &[Card]) -> Option<CardColor> {
    for card in cards {
        match card.value {
            CardValue::Fool => continue,
            CardValue::Simple(_) => return Some(card.color),
            CardValue::Wizard => return None,
        }
    }

    None
}
