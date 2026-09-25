use crate::action::{Action, LegalActions};
use crate::card::Card;
use crate::deck::Deck;
use crate::evaluator::evaluate_7;
use crate::events::{GameEvent, Stage};
use crate::player::{Player, PlayerStatus};
use crate::pot::{PotManager, PotPayout};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    pub small_blind: u64,
    pub big_blind: u64,
    pub ante: u64,
    pub max_players: usize,
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            small_blind: 10,
            big_blind: 20,
            ante: 0,
            max_players: 6,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub config: TableConfig,
    pub seats: Vec<Option<Player>>,
    pub button: usize,
    pub stage: Stage,
    pub board: Vec<Card>,
    pub deck: Deck,
    pub pot_manager: PotManager,
    pub current_player: Option<usize>,
    pub highest_bet: u64,
    pub min_raise_size: u64,
    pub hand_count: u64,
    pub events: Vec<GameEvent>,
}

impl Table {
    pub fn new(config: TableConfig) -> Self {
        let max_players = config.max_players;
        Self {
            config,
            seats: (0..max_players).map(|_| None).collect(),
            button: 0,
            stage: Stage::HandEnded,
            board: Vec::with_capacity(5),
            deck: Deck::new(),
            pot_manager: PotManager::new(max_players),
            current_player: None,
            highest_bet: 0,
            min_raise_size: 0,
            hand_count: 0,
            events: Vec::new(),
        }
    }

    /// Add a player to a specific seat. Returns error if seat is occupied or out of bounds.
    pub fn sit_player(&mut self, seat: usize, player: Player) -> Result<(), String> {
        if seat >= self.config.max_players {
            return Err(format!(
                "Seat {} is out of table bounds (max {})",
                seat, self.config.max_players
            ));
        }
        if self.seats[seat].is_some() {
            return Err(format!("Seat {} is already occupied", seat));
        }
        let mut p = player;
        p.id = seat;
        if self.stage != Stage::HandEnded {
            p.status = PlayerStatus::SittingOut;
        }
        self.seats[seat] = Some(p);
        Ok(())
    }

    /// Remove a player from a seat.
    pub fn remove_player(&mut self, seat: usize) -> Option<Player> {
        if seat < self.seats.len() {
            self.seats[seat].take()
        } else {
            None
        }
    }

    pub fn player(&self, seat: usize) -> Option<&Player> {
        self.seats.get(seat).and_then(|opt| opt.as_ref())
    }

    pub fn player_mut(&mut self, seat: usize) -> Option<&mut Player> {
        self.seats.get_mut(seat).and_then(|opt| opt.as_mut())
    }

    /// Total number of seated players with chips > 0
    pub fn seated_players_with_chips(&self) -> usize {
        self.seats
            .iter()
            .filter_map(|p| p.as_ref())
            .filter(|p| p.chips > 0)
            .count()
    }

    /// List of seated player IDs who have chips > 0
    pub fn eligible_seat_indices(&self) -> Vec<usize> {
        self.seats
            .iter()
            .enumerate()
            .filter_map(|(idx, p)| p.as_ref().filter(|p| p.chips > 0).map(|_| idx))
            .collect()
    }

    /// Active players in the hand (status == Active)
    pub fn active_players_count(&self) -> usize {
        self.seats
            .iter()
            .filter_map(|p| p.as_ref())
            .filter(|p| p.status == PlayerStatus::Active)
            .count()
    }

    /// Players still in the hand (Active or AllIn)
    pub fn in_hand_players_count(&self) -> usize {
        self.seats
            .iter()
            .filter_map(|p| p.as_ref())
            .filter(|p| p.is_in_hand())
            .count()
    }

    /// Next occupied seat clockwise from `from`
    fn next_seat_with_chips(&self, from: usize) -> usize {
        let n = self.config.max_players;
        for i in 1..=n {
            let seat = (from + i) % n;
            if let Some(p) = &self.seats[seat]
                && p.chips > 0
            {
                return seat;
            }
        }
        from
    }

    /// Next player with status == Active clockwise from `from`
    fn next_active_player(&self, from: usize) -> Option<usize> {
        let n = self.config.max_players;
        for i in 1..=n {
            let seat = (from + i) % n;
            if let Some(p) = &self.seats[seat]
                && p.status == PlayerStatus::Active
            {
                return Some(seat);
            }
        }
        None
    }

    /// Starts a new hand.
    pub fn start_hand(&mut self) -> Result<(), String> {
        let eligible = self.eligible_seat_indices();
        if eligible.len() < 2 {
            return Err("At least 2 players with chips are required to start a hand".to_string());
        }
        if self.config.big_blind == 0 {
            return Err("Big blind must be greater than 0".to_string());
        }

        self.hand_count += 1;
        self.events.clear();
        self.board.clear();
        self.deck.reset();
        self.deck.shuffle();
        self.pot_manager.reset(self.config.max_players);

        // Advance dealer button
        if self.hand_count > 1 {
            self.button = self.next_seat_with_chips(self.button);
        } else {
            // First hand: button is the first eligible seat
            self.button = eligible[0];
        }

        // Reset all seated players
        for seat in 0..self.config.max_players {
            if let Some(p) = self.seats[seat].as_mut() {
                p.reset_for_hand();
            }
        }

        let is_heads_up = eligible.len() == 2;
        let (sb_seat, bb_seat) = if is_heads_up {
            // In Heads-up: button is Small Blind, the other is Big Blind
            let sb = self.button;
            let bb = self.next_seat_with_chips(sb);
            (sb, bb)
        } else {
            let sb = self.next_seat_with_chips(self.button);
            let bb = self.next_seat_with_chips(sb);
            (sb, bb)
        };

        self.events.push(GameEvent::HandStarted {
            hand_id: self.hand_count,
            button: self.button,
            small_blind: self.config.small_blind,
            big_blind: self.config.big_blind,
        });

        // Collect antes if configured
        if self.config.ante > 0 {
            for &seat in &eligible {
                if let Some(p) = self.seats[seat].as_mut() {
                    let ante_paid = p.post_ante(self.config.ante);
                    self.pot_manager.contribute(seat, ante_paid);
                    self.events.push(GameEvent::AntePosted {
                        player_id: seat,
                        amount: ante_paid,
                    });
                }
            }
        }

        // Post Small Blind
        if let Some(sb_player) = self.seats[sb_seat].as_mut() {
            let paid = sb_player.commit_chips(self.config.small_blind);
            self.pot_manager.contribute(sb_seat, paid);
            self.events.push(GameEvent::BlindPosted {
                player_id: sb_seat,
                amount: paid,
                is_small_blind: true,
            });
        }

        // Post Big Blind
        if let Some(bb_player) = self.seats[bb_seat].as_mut() {
            let paid = bb_player.commit_chips(self.config.big_blind);
            self.pot_manager.contribute(bb_seat, paid);
            self.events.push(GameEvent::BlindPosted {
                player_id: bb_seat,
                amount: paid,
                is_small_blind: false,
            });
        }

        // Deal 2 hole cards to each participating player
        for &seat in &eligible {
            let c1 = self.deck.deal().ok_or("Deck ran out of cards")?;
            let c2 = self.deck.deal().ok_or("Deck ran out of cards")?;
            if let Some(p) = self.seats[seat].as_mut() {
                p.hole_cards = Some([c1, c2]);
                self.events.push(GameEvent::HoleCardsDealt {
                    player_id: seat,
                    cards: [c1, c2],
                });
            }
        }

        self.stage = Stage::PreFlop;
        self.highest_bet = self.config.big_blind;
        self.min_raise_size = self.config.big_blind;

        // First to act preflop:
        // Heads up: SB (the button) acts first preflop!
        // Multi-way: Under the gun (UTG) = seat after Big Blind
        let first_to_act = if is_heads_up {
            sb_seat
        } else {
            self.next_seat_with_chips(bb_seat)
        };

        // If the first to act is all-in from blind, find next active
        if let Some(p) = &self.seats[first_to_act] {
            if p.status == PlayerStatus::Active {
                self.current_player = Some(first_to_act);
            } else {
                self.current_player = self.next_active_player(first_to_act);
            }
        } else {
            self.current_player = self.next_active_player(first_to_act);
        }

        // Check if betting is already needed or if all are all-in
        self.check_round_status();

        Ok(())
    }

    /// Get legal actions for a specific player on their turn
    pub fn legal_actions(&self, seat: usize) -> Option<LegalActions> {
        let player = self.seats.get(seat)?.as_ref()?;
        if player.status != PlayerStatus::Active || self.current_player != Some(seat) {
            return None;
        }

        let to_call = self.highest_bet.saturating_sub(player.current_bet);
        let can_check = to_call == 0;
        let can_fold = true;
        let can_call = to_call > 0 && player.chips > 0;
        let call_amount = to_call.min(player.chips);

        let can_bet = self.highest_bet == 0 && player.chips > 0;
        let min_bet = if can_bet {
            self.config.big_blind.min(player.chips)
        } else {
            0
        };
        let max_bet = if can_bet { player.chips } else { 0 };

        let has_active_opponent = self.seats.iter().enumerate().any(|(idx, opt)| {
            idx != seat && opt.as_ref().is_some_and(|p| p.status == PlayerStatus::Active)
        });

        let can_raise = self.highest_bet > 0 && player.chips > to_call && has_active_opponent;
        let min_raise = if can_raise {
            let target = self.highest_bet + self.min_raise_size;
            let max_total = player.current_bet + player.chips;
            target.min(max_total)
        } else {
            0
        };
        let max_raise = if can_raise {
            player.current_bet + player.chips
        } else {
            0
        };

        let can_all_in = player.chips > 0
            && (has_active_opponent || to_call >= player.chips || self.highest_bet == 0);
        let all_in_cost = if !has_active_opponent && self.highest_bet > 0 && to_call < player.chips {
            to_call
        } else {
            player.chips
        };

        Some(LegalActions {
            can_fold,
            can_check,
            can_call,
            call_amount,
            can_bet,
            min_bet,
            max_bet,
            can_raise,
            min_raise,
            max_raise,
            can_all_in,
            all_in_cost,
        })
    }

    /// Applies an action by the current player.
    pub fn apply_action(&mut self, action: Action) -> Result<(), String> {
        let seat = self.current_player.ok_or("No player to act")?;
        let legal = self
            .legal_actions(seat)
            .ok_or("No legal actions for current player")?;

        if !legal.is_legal(&action) {
            return Err(format!(
                "Action {:?} is not legal for player {seat}",
                action
            ));
        }

        let mut chips_committed = 0;
        let actual_action = match action {
            Action::Fold => {
                if let Some(p) = self.seats[seat].as_mut() {
                    p.status = PlayerStatus::Folded;
                    p.acted_this_round = true;
                }
                Action::Fold
            }
            Action::Check => {
                if let Some(p) = self.seats[seat].as_mut() {
                    p.acted_this_round = true;
                }
                Action::Check
            }
            Action::Call => {
                let to_call = self
                    .highest_bet
                    .saturating_sub(self.seats[seat].as_ref().unwrap().current_bet);
                if let Some(p) = self.seats[seat].as_mut() {
                    chips_committed = p.commit_chips(to_call);
                    p.acted_this_round = true;
                }
                self.pot_manager.contribute(seat, chips_committed);
                Action::Call
            }
            Action::Bet(amount) => {
                if let Some(p) = self.seats[seat].as_mut() {
                    chips_committed = p.commit_chips(amount);
                    p.acted_this_round = true;
                    self.highest_bet = p.current_bet;
                    // Use chips_committed (actual chips put in) rather than the requested
                    // `amount`, because a short-stacked player may commit less than `amount`
                    // (all-in for less). The raise increment must be based on the real bet size.
                    self.min_raise_size = chips_committed.max(self.config.big_blind);
                }
                self.pot_manager.contribute(seat, chips_committed);
                // Reset acted_this_round for all other active players
                for (idx, other) in self.seats.iter_mut().enumerate() {
                    if idx != seat
                        && let Some(p) = other
                        && p.status == PlayerStatus::Active
                    {
                        p.acted_this_round = false;
                    }
                }
                Action::Bet(amount)
            }
            Action::Raise(total) => {
                let current_bet = self.seats[seat].as_ref().unwrap().current_bet;
                let needed = total.saturating_sub(current_bet);
                let raise_increment = total.saturating_sub(self.highest_bet);

                if let Some(p) = self.seats[seat].as_mut() {
                    chips_committed = p.commit_chips(needed);
                    p.acted_this_round = true;
                    self.highest_bet = p.current_bet;
                }
                self.pot_manager.contribute(seat, chips_committed);

                if raise_increment >= self.min_raise_size {
                    // Full raise: reopens betting and updates min_raise_size
                    self.min_raise_size = raise_increment;
                    for (idx, other) in self.seats.iter_mut().enumerate() {
                        if idx != seat
                            && let Some(p) = other
                            && p.status == PlayerStatus::Active
                        {
                            p.acted_this_round = false;
                        }
                    }
                } else {
                    // Under-raise all-in: reopens action only for players who have not faced this bet
                    for (idx, other) in self.seats.iter_mut().enumerate() {
                        if idx != seat
                            && let Some(p) = other
                            && p.status == PlayerStatus::Active
                            && p.current_bet < self.highest_bet
                        {
                            p.acted_this_round = false;
                        }
                    }
                }
                Action::Raise(total)
            }
            Action::AllIn => {
                let p = self.seats[seat].as_ref().unwrap();
                let chips = p.chips;
                let current_bet = p.current_bet;
                let total = current_bet + chips;

                let has_active_opponent = self.seats.iter().enumerate().any(|(idx, opt)| {
                    idx != seat && opt.as_ref().is_some_and(|p| p.status == PlayerStatus::Active)
                });

                if self.highest_bet == 0 {
                    return self.apply_action(Action::Bet(chips));
                } else if total <= self.highest_bet || !has_active_opponent {
                    return self.apply_action(Action::Call);
                } else {
                    return self.apply_action(Action::Raise(total));
                }
            }
        };

        self.events.push(GameEvent::PlayerActed {
            player_id: seat,
            action: actual_action,
            chips_committed,
        });

        // After player action, check if only 1 player remains in hand
        if self.in_hand_players_count() <= 1 {
            self.handle_lone_survivor();
            return Ok(());
        }

        self.advance_action_or_street(seat);
        Ok(())
    }

    fn advance_action_or_street(&mut self, last_seat: usize) {
        // Find next active player
        let next = self.next_active_player(last_seat);

        // Check if betting round is complete
        let round_complete = self.is_betting_round_complete();

        if round_complete {
            self.advance_street();
        } else {
            self.current_player = next;
            if let Some(p_id) = self.current_player
                && let Some(legal) = self.legal_actions(p_id)
            {
                self.events.push(GameEvent::PlayerTurn {
                    player_id: p_id,
                    legal_actions: legal,
                });
            }
        }
    }

    fn is_betting_round_complete(&self) -> bool {
        let active_players: Vec<&Player> = self
            .seats
            .iter()
            .filter_map(|p| p.as_ref())
            .filter(|p| p.status == PlayerStatus::Active)
            .collect();

        if active_players.is_empty() {
            return true;
        }

        if active_players.len() == 1 {
            let solo = active_players[0];
            return solo.acted_this_round && solo.current_bet >= self.highest_bet;
        }

        // All active players must have acted AND have current_bet == highest_bet
        active_players
            .iter()
            .all(|p| p.acted_this_round && p.current_bet == self.highest_bet)
    }

    fn refund_uncalled_bets(&mut self) {
        // Collect current_bet of ALL seated players who participated in the round
        let bets: Vec<(usize, u64)> = self
            .seats
            .iter()
            .enumerate()
            .filter_map(|(idx, p)| p.as_ref().map(|p| (idx, p.current_bet)))
            .collect();

        if bets.is_empty() {
            return;
        }

        let mut sorted_bets = bets.clone();
        sorted_bets.sort_by_key(|&(_, bet)| bet);

        let (leader_seat, highest) = *sorted_bets.last().unwrap();
        if highest == 0 {
            return;
        }

        let second_highest = if sorted_bets.len() >= 2 {
            sorted_bets[sorted_bets.len() - 2].1
        } else {
            0
        };

        let baseline = if self.stage == Stage::PreFlop {
            self.config.big_blind
        } else {
            0
        };

        if highest > second_highest {
            let uncalled = if self.stage == Stage::PreFlop {
                if highest > baseline {
                    highest - second_highest.max(baseline)
                } else {
                    let active_or_allin_opponents = self.seats.iter().enumerate().any(|(idx, opt)| {
                        idx != leader_seat && opt.as_ref().is_some_and(|p| p.is_in_hand())
                    });
                    if active_or_allin_opponents {
                        highest - second_highest
                    } else {
                        0
                    }
                }
            } else {
                highest - second_highest
            };

            if uncalled > 0 {
                if let Some(p) = self.seats[leader_seat].as_mut() {
                    p.chips += uncalled;
                    p.current_bet -= uncalled;
                    p.total_invested -= uncalled;
                    if p.chips > 0 && p.status == PlayerStatus::AllIn {
                        p.status = PlayerStatus::Active;
                    }
                }
                self.pot_manager.refund(leader_seat, uncalled);
                self.highest_bet = self.highest_bet.min(second_highest.max(baseline));
            }
        }
    }

    fn deal_runout_board(&mut self) {
        if self.board.is_empty() {
            for _ in 0..3 {
                let card = self.deck.deal().expect("Deck ran out of cards while dealing the flop");
                self.board.push(card);
            }
            self.events.push(GameEvent::StreetStarted {
                stage: Stage::Flop,
                board: self.board.clone(),
            });
        }
        if self.board.len() == 3 {
            let card = self.deck.deal().expect("Deck ran out of cards while dealing the turn");
            self.board.push(card);
            self.events.push(GameEvent::StreetStarted {
                stage: Stage::Turn,
                board: self.board.clone(),
            });
        }
        if self.board.len() == 4 {
            let card = self.deck.deal().expect("Deck ran out of cards while dealing the river");
            self.board.push(card);
            self.events.push(GameEvent::StreetStarted {
                stage: Stage::River,
                board: self.board.clone(),
            });
        }
    }

    pub fn advance_street(&mut self) {
        // Refund any uncalled bets in this round
        self.refund_uncalled_bets();

        // Reset player round bets
        for p in self.seats.iter_mut().flatten() {
            p.reset_for_street();
        }
        self.highest_bet = 0;
        self.min_raise_size = self.config.big_blind;

        let active_count = self.active_players_count();
        let in_hand_count = self.in_hand_players_count();

        if in_hand_count <= 1 {
            self.handle_lone_survivor();
            return;
        }

        // If fewer than 2 active players (e.g. 0 or 1 active, others all-in), no more betting rounds!
        if active_count <= 1 {
            self.deal_runout_board();
            self.stage = Stage::Showdown;
            self.handle_showdown();
            return;
        }

        match self.stage {
            Stage::PreFlop => {
                self.stage = Stage::Flop;
                for _ in 0..3 {
                    let card = self.deck.deal().expect("Deck ran out of cards while dealing the flop");
                    self.board.push(card);
                }
                self.events.push(GameEvent::StreetStarted {
                    stage: Stage::Flop,
                    board: self.board.clone(),
                });
            }
            Stage::Flop => {
                self.stage = Stage::Turn;
                let card = self.deck.deal().expect("Deck ran out of cards while dealing the turn");
                self.board.push(card);
                self.events.push(GameEvent::StreetStarted {
                    stage: Stage::Turn,
                    board: self.board.clone(),
                });
            }
            Stage::Turn => {
                self.stage = Stage::River;
                let card = self.deck.deal().expect("Deck ran out of cards while dealing the river");
                self.board.push(card);
                self.events.push(GameEvent::StreetStarted {
                    stage: Stage::River,
                    board: self.board.clone(),
                });
            }
            Stage::River => {
                self.stage = Stage::Showdown;
                self.handle_showdown();
                return;
            }
            Stage::Showdown | Stage::HandEnded => return,
        }

        // Post-flop: first active player clockwise from the button acts first
        self.current_player = self.next_active_player(self.button);
        if let Some(p_id) = self.current_player
            && let Some(legal) = self.legal_actions(p_id)
        {
            self.events.push(GameEvent::PlayerTurn {
                player_id: p_id,
                legal_actions: legal,
            });
        }
    }

    fn check_round_status(&mut self) {
        if self.in_hand_players_count() <= 1 {
            self.handle_lone_survivor();
            return;
        }

        if self.active_players_count() <= 1 {
            // Everyone is all-in preflop! Run out the board
            self.deal_runout_board();
            self.stage = Stage::Showdown;
            self.handle_showdown();
            return;
        }

        if let Some(p_id) = self.current_player
            && let Some(legal) = self.legal_actions(p_id)
        {
            self.events.push(GameEvent::PlayerTurn {
                player_id: p_id,
                legal_actions: legal,
            });
        }
    }

    fn handle_lone_survivor(&mut self) {
        self.refund_uncalled_bets();
        let winner_seat = self
            .seats
            .iter()
            .enumerate()
            .find_map(|(idx, p)| p.as_ref().filter(|p| p.is_in_hand()).map(|_| idx));

        if let Some(seat) = winner_seat {
            let total_won = self.pot_manager.total_pot();
            if let Some(p) = self.seats[seat].as_mut() {
                p.chips += total_won;
            }
            self.events.push(GameEvent::PotAwarded {
                pot_index: 0,
                player_id: seat,
                amount: total_won,
                hand_rank: None,
            });
        }

        self.stage = Stage::HandEnded;
        self.current_player = None;
        self.events.push(GameEvent::HandEnded);
    }

    fn handle_showdown(&mut self) {
        self.refund_uncalled_bets();
        let eligible_seats: HashSet<usize> = self
            .seats
            .iter()
            .enumerate()
            .filter_map(|(idx, p)| p.as_ref().filter(|p| p.is_in_hand()).map(|_| idx))
            .collect();

        // Evaluate all hands
        let mut showdown_hands = Vec::new();
        let mut hand_ranks = std::collections::HashMap::new();

        // Sanity check: board must have exactly 5 cards at showdown.
        assert_eq!(
            self.board.len(),
            5,
            "Showdown reached with {} board card(s) instead of 5 — deck or street dealing is broken",
            self.board.len()
        );

        for &seat in &eligible_seats {
            let p = self.seats[seat].as_ref().unwrap();
            let hole = p.hole_cards.unwrap();
            let mut all_7 = Vec::with_capacity(7);
            all_7.extend_from_slice(&hole);
            all_7.extend_from_slice(&self.board);
            let rank = evaluate_7(&all_7);
            hand_ranks.insert(seat, rank.clone());
            showdown_hands.push((seat, hole, rank));
        }

        self.events.push(GameEvent::Showdown {
            players: showdown_hands,
        });

        // Calculate pots
        let pots = self.pot_manager.calculate_pots(&eligible_seats);

        // Distribute each pot to the best hand among eligible players
        let payouts: Vec<PotPayout> = PotManager::distribute_pots(
            &pots,
            |pot| {
                let mut best_score = 0u16;
                let mut winners = Vec::new();
                for &p_id in &pot.eligible_players {
                    let score = hand_ranks[&p_id].score;
                    if score > best_score {
                        best_score = score;
                        winners.clear();
                        winners.push(p_id);
                    } else if score == best_score {
                        winners.push(p_id);
                    }
                }
                winners
            },
            self.button,
            self.config.max_players,
        );

        for payout in payouts {
            if let Some(p) = self.seats[payout.player_id].as_mut() {
                p.chips += payout.amount;
            }
            self.events.push(GameEvent::PotAwarded {
                pot_index: payout.pot_index,
                player_id: payout.player_id,
                amount: payout.amount,
                hand_rank: hand_ranks.get(&payout.player_id).cloned(),
            });
        }

        self.stage = Stage::HandEnded;
        self.current_player = None;
        self.events.push(GameEvent::HandEnded);
    }
}
