use crate::card::color::CardColor;
use crate::card::color::CardColor::*;
use crate::card::Card;
use crate::utils::evaluate_trick_winner;
use uuid::Uuid;

fn with_uuid(cards: Vec<Card>) -> Vec<(Uuid, Card)> {
    cards
        .into_iter()
        .map(|card| (Uuid::new_v4(), card))
        .collect()
}

fn new_card(value: u8, color: CardColor) -> Card {
    Card::new(value, color).unwrap()
}

#[test]
fn eval_winner_1() {
    let cards = vec![
        new_card(1, Blue),
        new_card(2, Blue),
        new_card(14, Blue),
        new_card(3, Blue),
    ];
    let cards = with_uuid(cards);
    let actual = cards[2];
    let eval = evaluate_trick_winner(&cards[..], None);

    assert_eq!(actual, eval);
}

#[test]
fn eval_winner_2() {
    let cards = vec![
        new_card(1, Blue),
        new_card(2, Blue),
        new_card(14, Red),
        new_card(3, Blue),
    ];
    let cards = with_uuid(cards);
    let actual = cards[2];
    let eval = evaluate_trick_winner(&cards[..], Some(Blue));

    assert_eq!(actual, eval);
}

#[test]
fn eval_winner_3() {
    let cards = vec![
        new_card(1, Blue),
        new_card(14, Red),
        new_card(14, Blue),
        new_card(4, Blue),
    ];
    let cards = with_uuid(cards);
    let actual = cards[1];
    let eval = evaluate_trick_winner(&cards[..], Some(Blue));

    assert_eq!(actual, eval);
}

#[test]
fn eval_winner_4() {
    let cards = vec![
        new_card(1, Blue),
        new_card(14, Red),
        new_card(14, Blue),
        new_card(4, Blue),
    ];
    let cards = with_uuid(cards);
    let actual = cards[1];
    let eval = evaluate_trick_winner(&cards[..], None);

    assert_eq!(actual, eval);
}

#[test]
fn eval_winner_5() {
    let cards = vec![
        new_card(1, Blue),
        new_card(10, Blue),
        new_card(9, Blue),
        new_card(4, Blue),
    ];
    let cards = with_uuid(cards);
    let actual = cards[1];
    let eval = evaluate_trick_winner(&cards[..], Some(Blue));

    assert_eq!(actual, eval);
}

#[test]
fn eval_winner_6() {
    let cards = vec![
        new_card(1, Blue),
        new_card(10, Blue),
        new_card(9, Blue),
        new_card(11, Red),
    ];
    let cards = with_uuid(cards);
    let actual = cards[1];
    let eval = evaluate_trick_winner(&cards[..], None);

    assert_eq!(actual, eval);
}

// TODO add more test cases
