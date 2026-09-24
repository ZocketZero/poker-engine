use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pot {
    pub amount: u64,
    /// Player IDs / seat indices eligible to win this pot
    pub eligible_players: HashSet<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PotPayout {
    pub pot_index: usize,
    pub player_id: usize,
    pub amount: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PotManager {
    /// Total chips contributed by each player across the entire hand (indexed by player_id / seat)
    total_contributions: Vec<u64>,
}

impl PotManager {
    pub fn new(num_seats: usize) -> Self {
        Self {
            total_contributions: vec![0; num_seats],
        }
    }

    /// Reset pot manager for a new hand with given number of seats
    pub fn reset(&mut self, num_seats: usize) {
        self.total_contributions = vec![0; num_seats];
    }

    /// Add chips contributed by a player
    pub fn contribute(&mut self, player_id: usize, amount: u64) {
        if player_id >= self.total_contributions.len() {
            self.total_contributions.resize(player_id + 1, 0);
        }
        self.total_contributions[player_id] += amount;
    }

    /// Refund uncalled chips to a player (e.g. when bet/raise is not matched)
    pub fn refund(&mut self, player_id: usize, amount: u64) {
        if player_id < self.total_contributions.len() {
            self.total_contributions[player_id] =
                self.total_contributions[player_id].saturating_sub(amount);
        }
    }

    pub fn contribution_of(&self, player_id: usize) -> u64 {
        self.total_contributions
            .get(player_id)
            .copied()
            .unwrap_or(0)
    }

    pub fn total_pot(&self) -> u64 {
        self.total_contributions.iter().sum()
    }

    /// Calculates the main pot and side pots based on total contributions and eligible players.
    /// `eligible_players`: set of players who have NOT folded.
    pub fn calculate_pots(&self, eligible_players: &HashSet<usize>) -> Vec<Pot> {
        let mut pots = Vec::new();
        if eligible_players.is_empty() || self.total_pot() == 0 {
            return pots;
        }

        // Remaining contribution for each player
        let mut remaining = self.total_contributions.clone();

        loop {
            // Find the lowest positive contribution among ELIGIBLE players
            let min_eligible = remaining
                .iter()
                .copied()
                .enumerate()
                .filter(|&(p_id, rem)| eligible_players.contains(&p_id) && rem > 0)
                .map(|(_, rem)| rem)
                .min();

            let target_cutoff = match min_eligible {
                Some(min) => min,
                None => {
                    // No eligible players have remaining chips.
                    // If there's any dead money remaining from folded players, add it to the last pot.
                    let dead_chips: u64 = remaining.iter().sum();
                    if dead_chips > 0 && !pots.is_empty() {
                        let last_idx = pots.len() - 1;
                        pots[last_idx].amount += dead_chips;
                    }
                    break;
                }
            };

            let mut pot_amount = 0u64;
            let mut pot_eligible = HashSet::new();

            // Collect up to target_cutoff from all players who still have chips
            for (p_id, rem) in remaining.iter_mut().enumerate() {
                if *rem > 0 {
                    let take = (*rem).min(target_cutoff);
                    pot_amount += take;
                    *rem -= take;

                    if eligible_players.contains(&p_id) {
                        pot_eligible.insert(p_id);
                    }
                }
            }

            if pot_amount > 0 && !pot_eligible.is_empty() {
                pots.push(Pot {
                    amount: pot_amount,
                    eligible_players: pot_eligible,
                });
            }

            // If all eligible players have 0 remaining, check dead money and stop
            let any_eligible_left = remaining
                .iter()
                .copied()
                .enumerate()
                .any(|(p_id, rem)| eligible_players.contains(&p_id) && rem > 0);

            if !any_eligible_left {
                let dead_chips: u64 = remaining.iter().sum();
                if dead_chips > 0 && !pots.is_empty() {
                    let last_idx = pots.len() - 1;
                    pots[last_idx].amount += dead_chips;
                }
                break;
            }
        }

        pots
    }

    /// Distributes pots among winners.
    /// `pots`: list of pots from `calculate_pots`
    /// `get_winners_for_pot`: closure that takes `&Pot` and returns a list of winner player IDs for that pot
    /// `button_seat`: dealer button position (used to break ties for odd chips)
    /// `num_seats`: total number of table seats
    pub fn distribute_pots<F>(
        pots: &[Pot],
        mut get_winners_for_pot: F,
        button_seat: usize,
        num_seats: usize,
    ) -> Vec<PotPayout>
    where
        F: FnMut(&Pot) -> Vec<usize>,
    {
        let mut payouts = Vec::new();

        for (pot_idx, pot) in pots.iter().enumerate() {
            if pot.amount == 0 {
                continue;
            }

            let mut winners = get_winners_for_pot(pot);
            if winners.is_empty() {
                continue;
            }

            let share = pot.amount / (winners.len() as u64);
            let mut odd_chips = pot.amount % (winners.len() as u64);

            // Sort winners by clockwise position from button for odd chip allocation:
            // Closest to the left of the button gets the odd chip first.
            // Distance = (seat + num_seats - button_seat - 1) % num_seats
            if winners.len() > 1 && odd_chips > 0 {
                winners.sort_by_key(|&seat| (seat + num_seats - button_seat - 1) % num_seats);
            }

            for winner_id in winners {
                let mut amount = share;
                if odd_chips > 0 {
                    amount += 1;
                    odd_chips -= 1;
                }
                payouts.push(PotPayout {
                    pot_index: pot_idx,
                    player_id: winner_id,
                    amount,
                });
            }
        }

        payouts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_heads_up_pot() {
        let mut pm = PotManager::new(2);
        pm.contribute(0, 100);
        pm.contribute(1, 100);
        assert_eq!(pm.total_pot(), 200);

        let mut eligible = HashSet::new();
        eligible.insert(0);
        eligible.insert(1);

        let pots = pm.calculate_pots(&eligible);
        assert_eq!(pots.len(), 1);
        assert_eq!(pots[0].amount, 200);
        assert_eq!(pots[0].eligible_players.len(), 2);
    }

    #[test]
    fn test_multiway_side_pots() {
        // Player 0: all-in for 100
        // Player 1: all-in for 300
        // Player 2: active with 500
        // Player 3: folded after putting in 50
        let mut pm = PotManager::new(4);
        pm.contribute(0, 100);
        pm.contribute(1, 300);
        pm.contribute(2, 500);
        pm.contribute(3, 50);

        // Player 2 had an uncalled bet of 200 (since player 1 only had 300)
        pm.refund(2, 200); // Player 2 now in for 300

        let mut eligible = HashSet::new();
        eligible.insert(0);
        eligible.insert(1);
        eligible.insert(2);
        // Player 3 folded, so not eligible

        let pots = pm.calculate_pots(&eligible);
        // Pot 0 (Main Pot):
        // Everyone pays up to 100:
        // P0: 100, P1: 100, P2: 100, P3: 50 -> 350 chips.
        // Eligible: 0, 1, 2.
        assert_eq!(pots.len(), 2);
        assert_eq!(pots[0].amount, 350);
        assert_eq!(pots[0].eligible_players, [0, 1, 2].into_iter().collect());

        // Pot 1 (Side Pot):
        // P1 has 200 left, P2 has 200 left -> 400 chips.
        // Eligible: 1, 2.
        assert_eq!(pots[1].amount, 400);
        assert_eq!(pots[1].eligible_players, [1, 2].into_iter().collect());
    }

    #[test]
    fn test_split_pot_odd_chips() {
        let pots = vec![Pot {
            amount: 101,
            eligible_players: [0, 1].into_iter().collect(),
        }];

        // Button is at seat 0.
        // Closest to left of button: seat 1 has distance 0, seat 0 has distance 1.
        // So seat 1 gets the odd chip!
        let payouts = PotManager::distribute_pots(&pots, |_| vec![0, 1], 0, 2);

        let p0 = payouts.iter().find(|p| p.player_id == 0).unwrap();
        let p1 = payouts.iter().find(|p| p.player_id == 1).unwrap();
        assert_eq!(p0.amount, 50);
        assert_eq!(p1.amount, 51);
    }
}
