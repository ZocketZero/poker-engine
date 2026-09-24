pub mod tables;

use crate::card::{Card, Rank};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HandCategory {
    HighCard = 0,
    OnePair = 1,
    TwoPair = 2,
    ThreeOfAKind = 3,
    Straight = 4,
    Flush = 5,
    FullHouse = 6,
    FourOfAKind = 7,
    StraightFlush = 8,
}

impl HandCategory {
    pub fn name(&self) -> &'static str {
        match self {
            HandCategory::HighCard => "High Card",
            HandCategory::OnePair => "One Pair",
            HandCategory::TwoPair => "Two Pair",
            HandCategory::ThreeOfAKind => "Three of a Kind",
            HandCategory::Straight => "Straight",
            HandCategory::Flush => "Flush",
            HandCategory::FullHouse => "Full House",
            HandCategory::FourOfAKind => "Four of a Kind",
            HandCategory::StraightFlush => "Straight Flush",
        }
    }
}

impl fmt::Display for HandCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct HandRank {
    /// Inverted Cactus Kev score: 1 (worst, 7-5-4-3-2 unsuited) to 7462 (Royal Flush).
    /// Higher is strictly better.
    pub score: u16,
    pub category: HandCategory,
    pub description: String,
}

impl Ord for HandRank {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for HandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for HandRank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HandRank(score: {}, {})", self.score, self.description)
    }
}

impl fmt::Display for HandRank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

pub fn category_from_score(score: u16) -> HandCategory {
    if score >= 7453 {
        HandCategory::StraightFlush
    } else if score >= 7297 {
        HandCategory::FourOfAKind
    } else if score >= 7141 {
        HandCategory::FullHouse
    } else if score >= 5864 {
        HandCategory::Flush
    } else if score >= 5854 {
        HandCategory::Straight
    } else if score >= 4996 {
        HandCategory::ThreeOfAKind
    } else if score >= 4138 {
        HandCategory::TwoPair
    } else if score >= 1278 {
        HandCategory::OnePair
    } else {
        HandCategory::HighCard
    }
}

/// Evaluates a 5-card poker hand and returns its rank.
pub fn evaluate_5(cards: &[Card; 5]) -> HandRank {
    let tables = tables::get_tables();

    let c0 = cards[0].raw();
    let c1 = cards[1].raw();
    let c2 = cards[2].raw();
    let c3 = cards[3].raw();
    let c4 = cards[4].raw();

    // Check for flush
    let is_flush = (c0 & c1 & c2 & c3 & c4 & 0xF000) != 0;
    let rank_mask = ((c0 | c1 | c2 | c3 | c4) >> 16) as usize;

    let ck_rank = if is_flush {
        tables.flushes[rank_mask]
    } else {
        let unique_rank = tables.unique5[rank_mask];
        if unique_rank != 0 {
            unique_rank
        } else {
            let prime_prod = (c0 & 0xFF) * (c1 & 0xFF) * (c2 & 0xFF) * (c3 & 0xFF) * (c4 & 0xFF);
            match tables.products.binary_search(&prime_prod) {
                Ok(idx) => tables.values[idx],
                Err(_) => unreachable!("Invalid card prime product: {}", prime_prod),
            }
        }
    };

    let score = 7463 - ck_rank;
    let category = category_from_score(score);
    let description = describe_hand(cards, category, score);

    HandRank {
        score,
        category,
        description,
    }
}

/// Evaluates any set of 5 to 7 cards (e.g. 7-card Texas Hold'em showdown)
/// and returns the best 5-card hand rank.
pub fn evaluate_7(cards: &[Card]) -> HandRank {
    let (rank, _) = evaluate_best(cards);
    rank
}

/// Returns the best 5-card HandRank and the specific 5 cards that formed it.
pub fn evaluate_best(cards: &[Card]) -> (HandRank, [Card; 5]) {
    match cards.len() {
        5 => {
            let five = [cards[0], cards[1], cards[2], cards[3], cards[4]];
            let rank = evaluate_5(&five);
            (rank, five)
        }
        6 => {
            let mut best_rank: Option<HandRank> = None;
            let mut best_cards = [cards[0], cards[1], cards[2], cards[3], cards[4]];

            for skip in 0..6 {
                let mut five = [cards[0]; 5];
                let mut idx = 0;
                for (i, &card) in cards.iter().enumerate().take(6) {
                    if i != skip {
                        five[idx] = card;
                        idx += 1;
                    }
                }
                let rank = evaluate_5(&five);
                if best_rank.as_ref().is_none_or(|b| rank.score > b.score) {
                    best_rank = Some(rank);
                    best_cards = five;
                }
            }
            (best_rank.unwrap(), best_cards)
        }
        7 => {
            const COMBOS_7_CHOOSE_5: [[usize; 5]; 21] = [
                [0, 1, 2, 3, 4],
                [0, 1, 2, 3, 5],
                [0, 1, 2, 3, 6],
                [0, 1, 2, 4, 5],
                [0, 1, 2, 4, 6],
                [0, 1, 2, 5, 6],
                [0, 1, 3, 4, 5],
                [0, 1, 3, 4, 6],
                [0, 1, 3, 5, 6],
                [0, 1, 4, 5, 6],
                [0, 2, 3, 4, 5],
                [0, 2, 3, 4, 6],
                [0, 2, 3, 5, 6],
                [0, 2, 4, 5, 6],
                [0, 3, 4, 5, 6],
                [1, 2, 3, 4, 5],
                [1, 2, 3, 4, 6],
                [1, 2, 3, 5, 6],
                [1, 2, 4, 5, 6],
                [1, 3, 4, 5, 6],
                [2, 3, 4, 5, 6],
            ];

            let mut best_rank: Option<HandRank> = None;
            let mut best_cards = [cards[0], cards[1], cards[2], cards[3], cards[4]];

            for &idx in &COMBOS_7_CHOOSE_5 {
                let five = [
                    cards[idx[0]],
                    cards[idx[1]],
                    cards[idx[2]],
                    cards[idx[3]],
                    cards[idx[4]],
                ];
                let rank = evaluate_5(&five);
                if best_rank.as_ref().is_none_or(|b| rank.score > b.score) {
                    best_rank = Some(rank);
                    best_cards = five;
                }
            }
            (best_rank.unwrap(), best_cards)
        }
        n => panic!("evaluate_best expects 5, 6, or 7 cards, got {}", n),
    }
}

fn describe_hand(cards: &[Card; 5], category: HandCategory, score: u16) -> String {
    // Count rank frequencies
    let mut counts = [0u8; 13];
    for c in cards {
        counts[c.rank() as usize] += 1;
    }

    let mut ranked_items: Vec<(u8, Rank)> = counts
        .iter()
        .enumerate()
        .filter(|&(_, &count)| count > 0)
        .map(|(r_idx, &count)| (count, Rank::ALL[r_idx]))
        .collect();

    // Sort by count descending, then rank descending
    ranked_items.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));

    match category {
        HandCategory::StraightFlush => {
            if score == 7462 {
                "Royal Flush".to_string()
            } else {
                // Check if 5-high (wheel)
                let is_wheel = cards.iter().any(|c| c.rank() == Rank::Five)
                    && cards.iter().any(|c| c.rank() == Rank::Ace);
                let high_rank = if is_wheel {
                    Rank::Five
                } else {
                    ranked_items[0].1
                };
                format!("Straight Flush, {} high", high_rank.name())
            }
        }
        HandCategory::FourOfAKind => {
            let quads = ranked_items[0].1;
            let kicker = ranked_items[1].1;
            format!(
                "Four of a Kind, {} with {} kicker",
                quads.plural_name(),
                kicker.name()
            )
        }
        HandCategory::FullHouse => {
            let trips = ranked_items[0].1;
            let pair = ranked_items[1].1;
            format!(
                "Full House, {} full of {}",
                trips.plural_name(),
                pair.plural_name()
            )
        }
        HandCategory::Flush => {
            let high = ranked_items[0].1;
            format!("Flush, {} high", high.name())
        }
        HandCategory::Straight => {
            let is_wheel = cards.iter().any(|c| c.rank() == Rank::Five)
                && cards.iter().any(|c| c.rank() == Rank::Ace);
            let high_rank = if is_wheel {
                Rank::Five
            } else {
                ranked_items[0].1
            };
            format!("Straight, {} high", high_rank.name())
        }
        HandCategory::ThreeOfAKind => {
            let trips = ranked_items[0].1;
            format!("Three of a Kind, {}", trips.plural_name())
        }
        HandCategory::TwoPair => {
            let p1 = ranked_items[0].1;
            let p2 = ranked_items[1].1;
            let kicker = ranked_items[2].1;
            format!(
                "Two Pair, {} and {} with {} kicker",
                p1.plural_name(),
                p2.plural_name(),
                kicker.name()
            )
        }
        HandCategory::OnePair => {
            let pair = ranked_items[0].1;
            format!("Pair of {}", pair.plural_name())
        }
        HandCategory::HighCard => {
            let high = ranked_items[0].1;
            format!("High Card, {}", high.name())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn parse_5(s: &str) -> [Card; 5] {
        let parts: Vec<&str> = s.split_whitespace().collect();
        assert_eq!(parts.len(), 5);
        [
            Card::from_str(parts[0]).unwrap(),
            Card::from_str(parts[1]).unwrap(),
            Card::from_str(parts[2]).unwrap(),
            Card::from_str(parts[3]).unwrap(),
            Card::from_str(parts[4]).unwrap(),
        ]
    }

    #[test]
    fn test_hand_rankings_hierarchy() {
        let royal_flush = evaluate_5(&parse_5("As Ks Qs Js Ts"));
        let king_straight_flush = evaluate_5(&parse_5("Ks Qs Js Ts 9s"));
        let wheel_straight_flush = evaluate_5(&parse_5("5s 4s 3s 2s As"));
        let quads_aces = evaluate_5(&parse_5("As Ac Ad Ah Ks"));
        let quads_kings = evaluate_5(&parse_5("Ks Kc Kd Kh As"));
        let full_house_aces = evaluate_5(&parse_5("As Ac Ad Kh Ks"));
        let full_house_kings = evaluate_5(&parse_5("Ks Kc Kd Ah As"));
        let flush_ace_high = evaluate_5(&parse_5("As Js 8s 6s 2s"));
        let straight_broadway = evaluate_5(&parse_5("As Kh Qc Jd Ts"));
        let straight_wheel = evaluate_5(&parse_5("5s 4h 3c 2d As"));
        let trips_queens = evaluate_5(&parse_5("Qs Qc Qd Kh 2s"));
        let two_pair_aces_kings = evaluate_5(&parse_5("As Ac Ks Kc Qd"));
        let two_pair_aces_queens = evaluate_5(&parse_5("As Ac Qs Qc Kd"));
        let pair_aces = evaluate_5(&parse_5("As Ac Kd Qs Jc"));
        let high_card_ace = evaluate_5(&parse_5("As Kd Qs Jc 9h"));
        let high_card_king = evaluate_5(&parse_5("Ks Qd Jc 9h 8s"));

        assert_eq!(royal_flush.category, HandCategory::StraightFlush);
        assert_eq!(royal_flush.score, 7462);
        assert_eq!(royal_flush.description, "Royal Flush");

        assert!(royal_flush > king_straight_flush);
        assert!(king_straight_flush > wheel_straight_flush);
        assert!(wheel_straight_flush > quads_aces);
        assert!(quads_aces > quads_kings);
        assert!(quads_kings > full_house_aces);
        assert!(full_house_aces > full_house_kings);
        assert!(full_house_kings > flush_ace_high);
        assert!(flush_ace_high > straight_broadway);
        assert!(straight_broadway > straight_wheel);
        assert!(straight_wheel > trips_queens);
        assert!(trips_queens > two_pair_aces_kings);
        assert!(two_pair_aces_kings > two_pair_aces_queens);
        assert!(two_pair_aces_queens > pair_aces);
        assert!(pair_aces > high_card_ace);
        assert!(high_card_ace > high_card_king);
    }

    #[test]
    fn test_evaluate_7() {
        let cards = vec![
            Card::from_str("As").unwrap(),
            Card::from_str("Ks").unwrap(),
            Card::from_str("Qs").unwrap(),
            Card::from_str("Js").unwrap(),
            Card::from_str("Ts").unwrap(),
            Card::from_str("2c").unwrap(),
            Card::from_str("3d").unwrap(),
        ];
        let rank = evaluate_7(&cards);
        assert_eq!(rank.category, HandCategory::StraightFlush);
        assert_eq!(rank.score, 7462);
    }
}
