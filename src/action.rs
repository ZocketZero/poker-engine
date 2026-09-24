use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Fold,
    Check,
    Call,
    /// Bet specified amount (when no bet has been made in the round yet)
    Bet(u64),
    /// Raise to a total round bet amount
    Raise(u64),
    /// Commit all remaining chips
    AllIn,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Fold => write!(f, "Fold"),
            Action::Check => write!(f, "Check"),
            Action::Call => write!(f, "Call"),
            Action::Bet(amount) => write!(f, "Bet {amount}"),
            Action::Raise(amount) => write!(f, "Raise to {amount}"),
            Action::AllIn => write!(f, "All-In"),
        }
    }
}

/// A snapshot of all legal actions available to a player on their turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegalActions {
    pub can_fold: bool,
    pub can_check: bool,
    pub can_call: bool,
    pub call_amount: u64,
    pub can_bet: bool,
    pub min_bet: u64,
    pub max_bet: u64,
    pub can_raise: bool,
    /// Minimum total round bet to raise to
    pub min_raise: u64,
    /// Maximum total round bet to raise to (all-in)
    pub max_raise: u64,
    pub can_all_in: bool,
    pub all_in_cost: u64,
}

impl LegalActions {
    pub fn is_legal(&self, action: &Action) -> bool {
        match action {
            Action::Fold => self.can_fold,
            Action::Check => self.can_check,
            Action::Call => self.can_call,
            Action::Bet(amount) => {
                self.can_bet && *amount >= self.min_bet && *amount <= self.max_bet
            }
            Action::Raise(total) => {
                self.can_raise && *total >= self.min_raise && *total <= self.max_raise
            }
            Action::AllIn => self.can_all_in,
        }
    }
}
