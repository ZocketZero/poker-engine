use crate::card::{Card, Rank, Suit};
use rand::Rng;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

impl Deck {
    /// Creates a standard 52-card deck in canonical order.
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        for &suit in &Suit::ALL {
            for &rank in &Rank::ALL {
                cards.push(Card::new(rank, suit));
            }
        }
        Self { cards }
    }

    /// Shuffles the deck using thread-local RNG.
    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        self.shuffle_with_rng(&mut rng);
    }

    /// Shuffles the deck using a provided RNG (useful for deterministic tests).
    pub fn shuffle_with_rng<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.cards.shuffle(rng);
    }

    /// Deals the top card from the deck.
    pub fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Deals `n` cards from the deck. Returns None if not enough cards remain.
    pub fn deal_n(&mut self, n: usize) -> Option<Vec<Card>> {
        if self.cards.len() < n {
            None
        } else {
            let mut dealt = Vec::with_capacity(n);
            for _ in 0..n {
                dealt.push(self.cards.pop().unwrap());
            }
            Some(dealt)
        }
    }

    /// Removes a specific card from the deck (e.g. for pre-specifying hole/board cards).
    pub fn remove_card(&mut self, card: Card) -> bool {
        if let Some(pos) = self.cards.iter().position(|&c| c == card) {
            self.cards.remove(pos);
            true
        } else {
            false
        }
    }

    /// Number of cards remaining in the deck.
    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    /// Resets the deck to full 52 cards.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_new_and_deal() {
        let mut deck = Deck::new();
        assert_eq!(deck.remaining(), 52);

        let c1 = deck.deal().unwrap();
        assert_eq!(deck.remaining(), 51);

        let cards = deck.deal_n(5).unwrap();
        assert_eq!(cards.len(), 5);
        assert_eq!(deck.remaining(), 46);

        // Remove card test
        assert!(deck.remove_card(Card::new(Rank::Two, Suit::Clubs)));
        assert_eq!(deck.remaining(), 45);
        assert!(!deck.remove_card(c1)); // already dealt
    }
}
