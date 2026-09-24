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
