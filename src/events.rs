use crate::action::{Action, LegalActions};
use crate::card::Card;
use crate::evaluator::HandRank;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
    HandEnded,
}

impl Stage {
    pub fn name(&self) -> &'static str {
        match self {
            Stage::PreFlop => "Pre-flop",
            Stage::Flop => "Flop",
            Stage::Turn => "Turn",
            Stage::River => "River",
            Stage::Showdown => "Showdown",
            Stage::HandEnded => "Hand Ended",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    HandStarted {
        hand_id: u64,
        button: usize,
        small_blind: u64,
        big_blind: u64,
    },
    BlindPosted {
        player_id: usize,
        amount: u64,
        is_small_blind: bool,
    },
    AntePosted {
        player_id: usize,
        amount: u64,
    },
    HoleCardsDealt {
        player_id: usize,
        cards: [Card; 2],
    },
    PlayerTurn {
        player_id: usize,
        legal_actions: LegalActions,
    },
    PlayerActed {
        player_id: usize,
        action: Action,
        chips_committed: u64,
    },
    StreetStarted {
        stage: Stage,
        board: Vec<Card>,
    },
    Showdown {
        players: Vec<(usize, [Card; 2], HandRank)>,
    },
    PotAwarded {
        pot_index: usize,
        player_id: usize,
        amount: u64,
        hand_rank: Option<HandRank>,
    },
    HandEnded,
}
