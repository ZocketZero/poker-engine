use poker_engine::pot::PotManager;
use std::collections::HashSet;

#[test]
fn test_complex_4way_side_pots() {
    // Player 0: all-in for 50
    // Player 1: all-in for 150
    // Player 2: all-in for 300
    // Player 3: calls 300
    let mut pm = PotManager::new(4);
    pm.contribute(0, 50);
    pm.contribute(1, 150);
    pm.contribute(2, 300);
    pm.contribute(3, 300);

    let mut eligible = HashSet::new();
    eligible.insert(0);
    eligible.insert(1);
    eligible.insert(2);
    eligible.insert(3);

    let pots = pm.calculate_pots(&eligible);
    assert_eq!(pots.len(), 3);

    // Main pot: 50 * 4 = 200 (all 4 eligible)
    assert_eq!(pots[0].amount, 200);
    assert_eq!(pots[0].eligible_players.len(), 4);

    // Side pot 1: 100 * 3 = 300 (P1, P2, P3 eligible)
    assert_eq!(pots[1].amount, 300);
    assert_eq!(pots[1].eligible_players.len(), 3);
    assert!(!pots[1].eligible_players.contains(&0));

    // Side pot 2: 150 * 2 = 300 (P2, P3 eligible)
    assert_eq!(pots[2].amount, 300);
    assert_eq!(pots[2].eligible_players.len(), 2);
    assert!(pots[2].eligible_players.contains(&2));
    assert!(pots[2].eligible_players.contains(&3));

    // Total pot sum must match total contributions: 50 + 150 + 300 + 300 = 800
    let total_in_pots: u64 = pots.iter().map(|p| p.amount).sum();
    assert_eq!(total_in_pots, 800);
}

#[test]
fn test_folded_chips_absorbed_into_side_pots() {
    // Player 0 (all-in for 100)
    // Player 1 (folded after contributing 250)
    // Player 2 (all-in for 400)
    let mut pm = PotManager::new(3);
    pm.contribute(0, 100);
    pm.contribute(1, 250);
    pm.contribute(2, 400);

    let mut eligible = HashSet::new();
    eligible.insert(0);
    eligible.insert(2);

    let pots = pm.calculate_pots(&eligible);
    assert_eq!(pots.len(), 2);

    // Main pot: P0 puts 100, P1 puts 100, P2 puts 100 -> 300
    assert_eq!(pots[0].amount, 300);
    assert_eq!(pots[0].eligible_players, [0, 2].into_iter().collect());

    // Side pot: P1 had 150 remaining, P2 had 300 remaining.
    // Cutoff for P2 is 300, absorbs P1's remaining 150 -> 300 + 150 = 450.
    // Eligible: only P2.
    assert_eq!(pots[1].amount, 450);
    assert_eq!(pots[1].eligible_players, [2].into_iter().collect());

    let total: u64 = pots.iter().map(|p| p.amount).sum();
    assert_eq!(total, 100 + 250 + 400);
}
