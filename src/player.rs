use crate::card::Card;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerStatus {
    /// In the hand and able to take actions
    Active,
    /// Has folded cards this hand
    Folded,
    /// All chips are committed; will participate in showdown
    AllIn,
    /// Not participating in current hand
    SittingOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub chips: u64,
    pub hole_cards: Option<[Card; 2]>,
    /// Chips bet in the current street (preflop, flop, turn, river)
    pub current_bet: u64,
    /// Total chips invested in the entire hand
    pub total_invested: u64,
    pub status: PlayerStatus,
    /// Has this player acted in the current betting round since the last full bet/raise?
    pub acted_this_round: bool,
}

impl Player {
    pub fn new(id: usize, name: impl Into<String>, starting_chips: u64) -> Self {
        Self {
            id,
            name: name.into(),
            chips: starting_chips,
            hole_cards: None,
            current_bet: 0,
            total_invested: 0,
            status: PlayerStatus::Active,
            acted_this_round: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == PlayerStatus::Active
    }

    pub fn is_in_hand(&self) -> bool {
        self.status == PlayerStatus::Active || self.status == PlayerStatus::AllIn
    }

    pub fn reset_for_hand(&mut self) {
        self.hole_cards = None;
        self.current_bet = 0;
        self.total_invested = 0;
        self.acted_this_round = false;
        if self.chips > 0 {
            self.status = PlayerStatus::Active;
        } else {
            self.status = PlayerStatus::SittingOut;
        }
    }

    pub fn reset_for_street(&mut self) {
        self.current_bet = 0;
        self.acted_this_round = false;
    }

    /// Deduct chips from stack and add to current bet and total invested.
    /// Returns the actual amount deducted (capped by player chips).
    pub fn commit_chips(&mut self, amount: u64) -> u64 {
        let committed = amount.min(self.chips);
        self.chips -= committed;
        self.current_bet += committed;
        self.total_invested += committed;
        if self.chips == 0 && self.status == PlayerStatus::Active {
            self.status = PlayerStatus::AllIn;
        }
        committed
    }
}
