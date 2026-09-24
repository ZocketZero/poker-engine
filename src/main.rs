use colored::*;
use poker_engine::action::Action;
use poker_engine::card::{Card, Suit};
use poker_engine::events::{GameEvent, Stage};
use poker_engine::player::Player;
use poker_engine::table::{Table, TableConfig};
use rand::Rng;
use std::io::{self, BufRead, Write};
use std::time::Instant;

fn format_card(card: &Card) -> ColoredString {
    let s = format!("{}{}", card.rank(), card.suit());
    match card.suit() {
        Suit::Hearts | Suit::Diamonds => s.red().bold(),
        Suit::Spades => s.white().bold(),
        Suit::Clubs => s.green().bold(),
    }
}

fn format_cards(cards: &[Card]) -> String {
    cards
        .iter()
        .map(|c| format!("{}", format_card(c)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn bot_decide(table: &Table, seat: usize) -> Action {
    let legal = match table.legal_actions(seat) {
        Some(l) => l,
        None => return Action::Fold,
    };

    let mut rng = rand::thread_rng();

    // If check is free, bots usually check (90%) or occasionally bet (10%)
    if legal.can_check {
        if legal.can_bet && rng.gen_bool(0.15) {
            let bet_amt = (legal.min_bet * 2).min(legal.max_bet);
            return Action::Bet(bet_amt);
        }
        return Action::Check;
    }

    // Facing a bet:
    // If call amount is small relative to stack (< 15%), call or raise
    let player = table.player(seat).unwrap();
    let call_ratio = (legal.call_amount as f64) / (player.chips.max(1) as f64);

    if call_ratio <= 0.25 {
        // Can call or raise
        if legal.can_raise && rng.gen_bool(0.20) {
            let raise_target = (legal.min_raise + table.config.big_blind).min(legal.max_raise);
            Action::Raise(raise_target)
        } else if legal.can_call {
            Action::Call
        } else {
            Action::Fold
        }
    } else if call_ratio <= 0.60 {
        // Medium bet: 60% call, 40% fold
        if rng.gen_bool(0.60) && legal.can_call {
            Action::Call
        } else {
            Action::Fold
        }
    } else {
        // Big bet / all-in: 30% call, 70% fold
        if rng.gen_bool(0.30) && legal.can_call {
            Action::Call
        } else {
            Action::Fold
        }
    }
}

fn run_interactive_game() {
    println!(
        "{}",
        "=================================================".bright_yellow()
    );
    println!(
        "{}",
        "       WELCOME TO THE RUST POKER ENGINE          "
            .bright_yellow()
            .bold()
    );
    println!(
        "{}",
        "=================================================".bright_yellow()
    );
    println!("You are sitting at Seat 0 (Hero) against 3 automated bots.\n");

    let config = TableConfig {
        small_blind: 10,
        big_blind: 20,
        ante: 0,
        max_players: 4,
    };

    let mut table = Table::new(config);
    table
        .sit_player(0, Player::new(0, "Hero (You)", 1000))
        .unwrap();
    table
        .sit_player(1, Player::new(1, "Bot Bob", 1000))
        .unwrap();
    table
        .sit_player(2, Player::new(2, "Bot Charlie", 1000))
        .unwrap();
    table
        .sit_player(3, Player::new(3, "Bot Diana", 1000))
        .unwrap();

    let stdin = io::stdin();
    let mut reader = stdin.lock();

    while table.seated_players_with_chips() > 1 {
        // Check if Hero has busted
        if table.player(0).unwrap().chips == 0 {
            println!(
                "{}",
                "\nYou ran out of chips! Game Over.".bright_red().bold()
            );
            break;
        }

        println!(
            "\n{}",
            "-------------------------------------------------".bright_cyan()
        );
        println!(
            "Starting Hand #{} | Blinds: {}/{}",
            table.hand_count + 1,
            table.config.small_blind,
            table.config.big_blind
        );

        if let Err(e) = table.start_hand() {
            println!("Could not start hand: {}", e);
            break;
        }

        // Print initial hand state
        println!("Dealer Button is at Seat {}", table.button);
        if let Some(hero) = table.player(0)
            && let Some(cards) = hero.hole_cards
        {
            println!("Your Hole Cards: [{}]", format_cards(&cards));
        }

        // Event / Action Loop
        while table.stage != Stage::HandEnded {
            if let Some(current_seat) = table.current_player {
                let player_name = table.player(current_seat).unwrap().name.clone();

                if current_seat == 0 {
                    // Hero's turn
                    println!(
                        "\n--- Your Turn (Pot: {}) ---",
                        table.pot_manager.total_pot()
                    );
                    println!("Board: [{}]", format_cards(&table.board));
                    let legal = table.legal_actions(0).unwrap();

                    let mut prompt_opts = Vec::new();
                    if legal.can_fold {
                        prompt_opts.push("[f]old".to_string());
                    }
                    if legal.can_check {
                        prompt_opts.push("[ch]eck".to_string());
                    }
                    if legal.can_call {
                        prompt_opts.push(format!("[c]all {}", legal.call_amount));
                    }
                    if legal.can_bet {
                        prompt_opts.push(format!("[b]et <{}-{}>", legal.min_bet, legal.max_bet));
                    }
                    if legal.can_raise {
                        prompt_opts
                            .push(format!("[r]aise <{}-{}>", legal.min_raise, legal.max_raise));
                    }
                    if legal.can_all_in {
                        prompt_opts.push(format!("[a]ll-in ({})", legal.all_in_cost));
                    }

                    print!("Options ({}): ", prompt_opts.join(" | "));
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    let bytes = reader.read_line(&mut input).unwrap_or(0);
                    if bytes == 0 {
                        println!("\nInput stream closed. Exiting game.");
                        return;
                    }
                    let parts: Vec<&str> = input.split_whitespace().collect();
                    if parts.is_empty() {
                        continue;
                    }

                    let action = match parts[0].to_lowercase().as_str() {
                        "f" | "fold" => Action::Fold,
                        "ch" | "check" if legal.can_check => Action::Check,
                        "c" | "call" if legal.can_call => Action::Call,
                        "b" | "bet" if legal.can_bet => {
                            let amt = parts
                                .get(1)
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or(legal.min_bet);
                            Action::Bet(amt.clamp(legal.min_bet, legal.max_bet))
                        }
                        "r" | "raise" if legal.can_raise => {
                            let amt = parts
                                .get(1)
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or(legal.min_raise);
                            Action::Raise(amt.clamp(legal.min_raise, legal.max_raise))
                        }
                        "a" | "allin" | "all-in" if legal.can_all_in => Action::AllIn,
                        _ => {
                            if legal.can_check {
                                Action::Check
                            } else if legal.can_call {
                                Action::Call
                            } else {
                                Action::Fold
                            }
                        }
                    };

                    if let Err(err) = table.apply_action(action) {
                        println!("Action error: {}", err);
                    }
                } else {
                    // Bot's turn
                    let action = bot_decide(&table, current_seat);
                    println!(
                        "{} chooses: {}",
                        player_name.dimmed(),
                        format!("{:?}", action).bright_white()
                    );
                    if let Err(err) = table.apply_action(action) {
                        println!("Bot error: {}", err);
                    }
                }
            } else {
                break;
            }
        }

        // Print Hand Results
        println!("\n{}", "=== Hand Results ===".bright_magenta().bold());
        if !table.board.is_empty() {
            println!("Final Board: [{}]", format_cards(&table.board));
        }
        for ev in &table.events {
            match ev {
                GameEvent::Showdown { players } => {
                    for (seat, hole, rank) in players {
                        let name = &table.player(*seat).unwrap().name;
                        println!(
                            "  {} shows [{}] -> {}",
                            name,
                            format_cards(hole),
                            rank.description.bright_green()
                        );
                    }
                }
                GameEvent::PotAwarded {
                    pot_index,
                    player_id,
                    amount,
                    hand_rank,
                } => {
                    let name = &table.player(*player_id).unwrap().name;
                    if let Some(rank) = hand_rank {
                        println!(
                            "  {} wins Pot #{} ({} chips) with {}",
                            name.bright_yellow().bold(),
                            pot_index,
                            amount,
                            rank
                        );
                    } else {
                        println!(
                            "  {} wins Pot #{} ({} chips, uncontested)",
                            name.bright_yellow().bold(),
                            pot_index,
                            amount
                        );
                    }
                }
                _ => {}
            }
        }

        // Print Stacks
        println!("\nChip Stacks:");
        for seat in 0..table.config.max_players {
            if let Some(p) = table.player(seat) {
                println!("  {}: {} chips", p.name, p.chips);
            }
        }

        print!("\nPress Enter to play next hand (or 'q' to quit)... ");
        io::stdout().flush().unwrap();
        let mut next_line = String::new();
        let bytes = reader.read_line(&mut next_line).unwrap_or(0);
        if bytes == 0 || next_line.trim().eq_ignore_ascii_case("q") {
            break;
        }
    }

    println!("\nThanks for playing Poker Engine!");
}

fn run_simulation_benchmark(total_hands: u64) {
    println!("Starting simulation benchmark of {} hands...", total_hands);

    let config = TableConfig {
        small_blind: 10,
        big_blind: 20,
        ante: 0,
        max_players: 6,
    };

    let mut table = Table::new(config);
    for i in 0..6 {
        table
            .sit_player(i, Player::new(i, format!("Bot_{}", i), 5000))
            .unwrap();
    }

    let _initial_total_chips = 6 * 5000;
    let start_time = Instant::now();
    let mut hands_completed = 0;

    for _ in 0..total_hands {
        // Rebuy any busted bots so simulation continues continuously
        for i in 0..6 {
            if let Some(p) = table.player_mut(i)
                && p.chips < 20
            {
                p.chips += 5000;
            }
        }

        if let Err(e) = table.start_hand() {
            eprintln!("Failed to start hand: {}", e);
            break;
        }

        while table.stage != Stage::HandEnded {
            if let Some(cp) = table.current_player {
                let action = bot_decide(&table, cp);
                if let Err(e) = table.apply_action(action) {
                    eprintln!("Action error: {}", e);
                    break;
                }
            } else {
                break;
            }
        }

        hands_completed += 1;
    }

    let elapsed = start_time.elapsed();
    let hands_per_sec = (hands_completed as f64) / elapsed.as_secs_f64();

    println!(
        "\n{}",
        "=== Simulation Benchmark Results ===".bright_green().bold()
    );
    println!("Hands Completed: {}", hands_completed);
    println!("Time Elapsed:    {:.3} seconds", elapsed.as_secs_f64());
    println!("Throughput:      {:.0} hands/second", hands_per_sec);
}

fn print_help() {
    println!("Poker Engine CLI");
    println!("Usage:");
    println!("  cargo run -- --play             Play interactive game vs bots");
    println!("  cargo run -- --simulate <hands> Run headless simulation benchmark");
    println!("  cargo run -- --help             Show this help message");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 || args[1] == "--play" {
        run_interactive_game();
    } else if args[1] == "--simulate" || args[1] == "-s" || args[1] == "--benchmark" {
        let count = args
            .get(2)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(20_000);
        run_simulation_benchmark(count);
    } else {
        print_help();
    }
}
