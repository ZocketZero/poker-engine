use poker_engine::action::Action;
use poker_engine::events::Stage;
use poker_engine::player::Player;
use poker_engine::table::{Table, TableConfig};

#[test]
fn test_preflop_fold_around_heads_up() {
    let mut table = Table::new(TableConfig {
        small_blind: 10,
        big_blind: 20,
        ante: 0,
        max_players: 2,
    });

    table.sit_player(0, Player::new(0, "Alice", 1000)).unwrap();
    table.sit_player(1, Player::new(1, "Bob", 1000)).unwrap();

    // Start hand:
    // Seat 0 (Alice) is button/SB (posts 10, chips=990)
    // Seat 1 (Bob) is BB (posts 20, chips=980)
    // First to act is SB (Alice)
    table.start_hand().unwrap();
    assert_eq!(table.stage, Stage::PreFlop);
    assert_eq!(table.current_player, Some(0));

    // Alice folds
    table.apply_action(Action::Fold).unwrap();

    // Hand should end immediately
    assert_eq!(table.stage, Stage::HandEnded);
    assert_eq!(table.current_player, None);

    // Bob wins total pot of 30: Bob's chips = 980 + 30 = 1010
    // Alice's chips = 990
    assert_eq!(table.player(0).unwrap().chips, 990);
    assert_eq!(table.player(1).unwrap().chips, 1010);
    assert_eq!(
        table.player(0).unwrap().chips + table.player(1).unwrap().chips,
        2000
    );
}

#[test]
fn test_full_hand_check_down() {
    let mut table = Table::new(TableConfig {
        small_blind: 10,
        big_blind: 20,
        ante: 0,
        max_players: 2,
    });

    table.sit_player(0, Player::new(0, "Alice", 1000)).unwrap();
    table.sit_player(1, Player::new(1, "Bob", 1000)).unwrap();

    table.start_hand().unwrap();

    // Preflop:
    // Alice (SB) calls 20 (chips committed: 10 more)
    assert_eq!(table.current_player, Some(0));
    table.apply_action(Action::Call).unwrap();

    // Bob (BB) checks
    assert_eq!(table.current_player, Some(1));
    table.apply_action(Action::Check).unwrap();

    // Flop dealt (3 cards)
    assert_eq!(table.stage, Stage::Flop);
    assert_eq!(table.board.len(), 3);
    // Postflop in heads-up: BB (Bob, seat 1) acts first
    assert_eq!(table.current_player, Some(1));

    // Bob checks
    table.apply_action(Action::Check).unwrap();
    // Alice checks
    table.apply_action(Action::Check).unwrap();

    // Turn dealt (1 card)
    assert_eq!(table.stage, Stage::Turn);
    assert_eq!(table.board.len(), 4);

    // Bob checks
    table.apply_action(Action::Check).unwrap();
    // Alice checks
    table.apply_action(Action::Check).unwrap();

    // River dealt (1 card)
    assert_eq!(table.stage, Stage::River);
    assert_eq!(table.board.len(), 5);

    // Bob checks
    table.apply_action(Action::Check).unwrap();
    // Alice checks
    table.apply_action(Action::Check).unwrap();

    // Showdown!
    assert_eq!(table.stage, Stage::HandEnded);
    assert_eq!(table.current_player, None);

    // Chips conserved
    let total_chips = table.player(0).unwrap().chips + table.player(1).unwrap().chips;
    assert_eq!(total_chips, 2000);
}

#[test]
fn test_all_in_showdown_runout() {
    let mut table = Table::new(TableConfig {
        small_blind: 10,
        big_blind: 20,
        ante: 0,
        max_players: 2,
    });

    table.sit_player(0, Player::new(0, "Alice", 500)).unwrap();
    table.sit_player(1, Player::new(1, "Bob", 500)).unwrap();

    table.start_hand().unwrap();

    // Alice goes all-in preflop
    table.apply_action(Action::AllIn).unwrap();

    // Bob calls all-in
    table.apply_action(Action::Call).unwrap();

    // Both all in: board should be automatically dealt out to 5 cards and showdown resolved
    assert_eq!(table.stage, Stage::HandEnded);
    assert_eq!(table.board.len(), 5);

    let total_chips = table.player(0).unwrap().chips + table.player(1).unwrap().chips;
    assert_eq!(total_chips, 1000);
}

#[test]
fn test_chip_conservation_over_multiple_hands() {
    let mut table = Table::new(TableConfig {
        small_blind: 10,
        big_blind: 20,
        ante: 0,
        max_players: 3,
    });

    table.sit_player(0, Player::new(0, "P0", 1000)).unwrap();
    table.sit_player(1, Player::new(1, "P1", 1000)).unwrap();
    table.sit_player(2, Player::new(2, "P2", 1000)).unwrap();

    let initial_total = 3000;

    for _ in 0..50 {
        if table.seated_players_with_chips() < 2 {
            break;
        }

        table.start_hand().unwrap();

        // Simulate passive play (call / check / fold)
        while table.stage != Stage::HandEnded {
            if let Some(cp) = table.current_player {
                let legal = table.legal_actions(cp).unwrap();
                if legal.can_check {
                    table.apply_action(Action::Check).unwrap();
                } else if legal.can_call {
                    table.apply_action(Action::Call).unwrap();
                } else {
                    table.apply_action(Action::Fold).unwrap();
                }
            } else {
                break;
            }
        }

        let current_total: u64 = table.seats.iter().flatten().map(|p| p.chips).sum();
        assert_eq!(
            current_total, initial_total,
            "Chips must be strictly conserved!"
        );
    }
}

#[test]
fn test_uncalled_bet_against_folded_player() {
    let mut table = Table::new(TableConfig {
        small_blind: 10,
        big_blind: 20,
        ante: 0,
        max_players: 3,
    });

    table.sit_player(0, Player::new(0, "P0_Short", 20)).unwrap();
    table.sit_player(1, Player::new(1, "P1_Mid", 1000)).unwrap();
    table.sit_player(2, Player::new(2, "P2_Deep", 1000)).unwrap();

    table.start_hand().unwrap();
    // Hand 1: Button is P0.
    // SB is P1 (posts 10), BB is P2 (posts 20).
    // Preflop:
    // First to act: P0 (UTG). P0 calls 20 (all-in).
    assert_eq!(table.current_player, Some(0));
    table.apply_action(Action::Call).unwrap();

    // P1 (SB) calls 10 (total 20).
    assert_eq!(table.current_player, Some(1));
    table.apply_action(Action::Call).unwrap();

    // P2 (BB) checks.
    assert_eq!(table.current_player, Some(2));
    table.apply_action(Action::Check).unwrap();

    // Flop:
    assert_eq!(table.stage, Stage::Flop);
    // P1 acts first postflop
    assert_eq!(table.current_player, Some(1));
    table.apply_action(Action::Bet(100)).unwrap();

    // P2 raises to 300
    assert_eq!(table.current_player, Some(2));
    table.apply_action(Action::Raise(300)).unwrap();

    // P1 folds
    assert_eq!(table.current_player, Some(1));
    table.apply_action(Action::Fold).unwrap();

    // Betting is complete. Hand should go straight to showdown since P0 is all-in and P1 folded.
    assert_eq!(table.stage, Stage::HandEnded);

    // P2 committed 300 on flop against P1's 100 bet.
    // P2's uncalled amount should be 200 (300 - 100).
    // P2 should have 100 chips committed to the flop pot, matching P1's 100.
    // P1 put in 20 preflop + 100 flop = 120 total. P1 folded and lost 120.
    // P1's remaining chips must be 1000 - 120 = 880.
    assert_eq!(table.player(1).unwrap().chips, 880);

    // P2 put in 20 preflop + 100 flop = 120 total.
    // P2 is guaranteed to win the side pot (since P1 folded and P0 was all-in for 20).
    // If P0 wins main pot (60), P0 chips = 60, P2 chips = 1000 - 120 + (100+100-20*2) + ...
    // Total chips across all 3 players MUST be 2020.
    let total: u64 = table.seats.iter().flatten().map(|p| p.chips).sum();
    assert_eq!(total, 2020);

    // If P0 won main pot (60): P0 has 60. P2 has 880 + 200 (side pot) = 1080.
    // If P2 won main pot (60): P0 has 0. P2 has 880 + 260 (all pots) = 1140.
    let p0_chips = table.player(0).unwrap().chips;
    let p2_chips = table.player(2).unwrap().chips;
    if p0_chips == 60 {
        assert_eq!(p2_chips, 1080, "P2 should have 1080 when P0 wins main pot");
    } else {
        assert_eq!(p0_chips, 0);
        assert_eq!(p2_chips, 1140, "P2 should have 1140 when P2 wins all pots");
    }
}

#[test]
fn test_ante_preflop_calling() {
    let mut table = Table::new(TableConfig {
        small_blind: 10,
        big_blind: 20,
        ante: 5,
        max_players: 3,
    });

    table.sit_player(0, Player::new(0, "P0", 1000)).unwrap();
    table.sit_player(1, Player::new(1, "P1", 1000)).unwrap();
    table.sit_player(2, Player::new(2, "P2", 1000)).unwrap();

    table.start_hand().unwrap();
    // P0 is button (UTG preflop).
    // P1 is SB. P2 is BB.
    // Antes paid: 5 each = 15 total in pot.
    // SB paid: 10. BB paid: 20.
    // Pot before action: 15 + 10 + 20 = 45.
    assert_eq!(table.pot_manager.total_pot(), 45);

    // P0 (first to act) must call the full big blind (20 chips)!
    // Under the ante bug, P0's current_bet was set to 5 by ante,
    // so legal.call_amount was 15 instead of 20!
    let legal = table.legal_actions(0).unwrap();
    assert_eq!(legal.call_amount, 20, "Calling the big blind must cost 20, not 15!");
}

#[test]
fn test_cannot_raise_when_opponent_is_all_in() {
    let mut table = Table::new(TableConfig {
        small_blind: 10,
        big_blind: 20,
        ante: 0,
        max_players: 2,
    });

    table.sit_player(0, Player::new(0, "Alice", 1000)).unwrap();
    table.sit_player(1, Player::new(1, "Bob", 100)).unwrap();

    table.start_hand().unwrap();
    // Preflop:
    // Alice (SB) is button, acts first.
    // Alice raises to 50.
    table.apply_action(Action::Raise(50)).unwrap();
    // Bob goes all in for 100 total.
    table.apply_action(Action::AllIn).unwrap();

    // Now it's Alice's turn. Bob is ALL-IN with 0 chips left.
    // There are NO OTHER active players.
    // Can Alice raise?!
    // In poker, Alice CANNOT raise, because Bob is already all-in and cannot call a raise!
    let legal = table.legal_actions(0).unwrap();
    assert!(!legal.can_raise, "Alice must not be allowed to raise when opponent is all-in with no active opponents left!");
}

#[test]
fn test_min_raise_after_short_all_in_bet() {
    let mut table = Table::new(TableConfig {
        small_blind: 10,
        big_blind: 20,
        ante: 0,
        max_players: 3,
    });

    table.sit_player(0, Player::new(0, "Alice", 1000)).unwrap();
    table.sit_player(1, Player::new(1, "Bob", 25)).unwrap();
    table.sit_player(2, Player::new(2, "Charlie", 1000)).unwrap();

    table.start_hand().unwrap();
    // Preflop:
    // P0 calls 20, P1 calls 10 (now 20), P2 checks.
    table.apply_action(Action::Call).unwrap();
    table.apply_action(Action::Call).unwrap();
    table.apply_action(Action::Check).unwrap();

    // Flop:
    // Bob (P1) has 5 chips left. Bob bets 5 all-in!
    // P2 is next.
    assert_eq!(table.stage, Stage::Flop);
    assert_eq!(table.current_player, Some(1));
    table.apply_action(Action::Bet(5)).unwrap();

    // Now Charlie (P2) acts. P0 (Alice) is also active behind Charlie.
    // Highest bet is 5.
    // But table config big_blind is 20!
    // A legal raise must be at least to 20 (the big blind size above 0), not to 10!
    let legal = table.legal_actions(2).unwrap();
    assert!(legal.can_raise);
    assert!(legal.min_raise >= 20, "Min raise must be at least the big blind (20), got {}", legal.min_raise);
}





