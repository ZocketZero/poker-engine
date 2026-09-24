use poker_engine::card::Card;
use poker_engine::evaluator::{HandCategory, evaluate_5, evaluate_7};
use std::str::FromStr;

fn parse_cards(s: &str) -> Vec<Card> {
    s.split_whitespace()
        .map(|token| Card::from_str(token).unwrap())
        .collect()
}

fn parse_5(s: &str) -> [Card; 5] {
    let v = parse_cards(s);
    assert_eq!(v.len(), 5);
    [v[0], v[1], v[2], v[3], v[4]]
}

#[test]
fn test_straight_wheel_vs_six_high() {
    let wheel = evaluate_5(&parse_5("5s 4d 3c 2h As"));
    let six_high = evaluate_5(&parse_5("6s 5d 4c 3h 2c"));
    assert_eq!(wheel.category, HandCategory::Straight);
    assert_eq!(six_high.category, HandCategory::Straight);
    assert!(
        six_high > wheel,
        "6-high straight must beat 5-high straight"
    );
}

#[test]
fn test_two_pair_kicker_tiebreak() {
    let tp_high_kicker = evaluate_5(&parse_5("As Ah Ks Kh Qd"));
    let tp_low_kicker = evaluate_5(&parse_5("Ac Ad Kc Kd Jd"));
    assert_eq!(tp_high_kicker.category, HandCategory::TwoPair);
    assert_eq!(tp_low_kicker.category, HandCategory::TwoPair);
    assert!(
        tp_high_kicker > tp_low_kicker,
        "Aces and Kings with Queen kicker beats Jack kicker"
    );
}

#[test]
fn test_flush_kicker_tiebreak() {
    let f1 = evaluate_5(&parse_5("As Ks Qs Ts 2s"));
    let f2 = evaluate_5(&parse_5("As Ks Qs 9s 8s"));
    assert_eq!(f1.category, HandCategory::Flush);
    assert_eq!(f2.category, HandCategory::Flush);
    assert!(f1 > f2, "Ten kicker beats Nine kicker in flush");
}

#[test]
fn test_full_house_tiebreak() {
    let fh_aces_over_kings = evaluate_5(&parse_5("As Ah Ad Ks Kh"));
    let fh_aces_over_queens = evaluate_5(&parse_5("Ac As Ad Qs Qh"));
    let fh_kings_over_aces = evaluate_5(&parse_5("Ks Kh Kd As Ah"));

    assert!(fh_aces_over_kings > fh_aces_over_queens);
    assert!(fh_aces_over_queens > fh_kings_over_aces);
}

#[test]
fn test_four_of_a_kind_kicker() {
    let quads_k = evaluate_5(&parse_5("Ts Th Td Tc Kh"));
    let quads_q = evaluate_5(&parse_5("Ts Th Td Tc Qh"));
    assert!(quads_k > quads_q);
}

#[test]
fn test_evaluate_7_best_combination() {
    // 7 cards containing a full house and a flush: Aces full of Kings vs King-high flush
    let cards = parse_cards("As Ah Ad Ks Kh 2s 3s");
    let rank = evaluate_7(&cards);
    assert_eq!(rank.category, HandCategory::FullHouse);
    assert!(rank.description.contains("Full House"));
}

#[test]
fn test_board_plays() {
    // Board has a royal flush: both players should tie with Royal Flush
    let p1_hole = parse_cards("2c 3d");
    let p2_hole = parse_cards("4c 5d");
    let board = parse_cards("As Ks Qs Js Ts");

    let mut hand1 = p1_hole;
    hand1.extend_from_slice(&board);
    let mut hand2 = p2_hole;
    hand2.extend_from_slice(&board);

    let rank1 = evaluate_7(&hand1);
    let rank2 = evaluate_7(&hand2);

    assert_eq!(rank1.category, HandCategory::StraightFlush);
    assert_eq!(rank2.category, HandCategory::StraightFlush);
    assert_eq!(rank1.score, rank2.score);
}
