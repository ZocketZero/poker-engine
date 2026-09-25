# poker-engine ♠️♥️♦️♣️

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-26%20passed-brightgreen.svg)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A high-performance, modular, and feature-complete **No-Limit Texas Hold'em (NLHE)** poker engine written in idiomatic Rust.

Designed for AI/bot development, Monte Carlo simulations, server backends, and interactive gameplay.

---

## Features

- ⚡ **High-Speed Bitwise Hand Evaluator**:
  - Implements the classic Cactus Kev 32-bit card representation and perfect-hash/binary-search lookup tables.
  - Supports 5-card, 6-card, and 7-card evaluations.
  - Invertible monotonic `HandRank` score ($1$ to $7462$, with $7462$ being Royal Flush) implementing standard Rust `Ord`.
  - Natural human-readable hand descriptions (*"Full House, Aces full of Kings"*, *"Two Pair, Kings and Fives with Jack kicker"*).
- 🎲 **Complete NLHE Betting State Machine**:
  - Strict round transitions: **Preflop → Flop → Turn → River → Showdown → Hand Ended**.
  - Small Blind & Big Blind rotation, dealer button management, and heads-up rules.
  - Full action set: `Fold`, `Check`, `Call`, `Bet`, `Raise`, and `AllIn`.
  - Enforces min-raise increment rules and handles all-in under-raise action reopening.
  - Optional ante support — collected from all eligible seats before blinds.
  - Automatic board runout when remaining players are all-in.
  - Early round termination when players fold around to a single survivor.
  - Accurate event log: `Bet` and `Raise` events always reflect the *actual* committed amount (not the requested amount for short-stack all-ins).
- 💰 **Robust Side Pot & Split Pot Accounting**:
  - Automatically calculates discrete main pots and multi-tier side pots for arbitrary uneven all-in player stacks.
  - Absorbs dead money from folded players into the correct pot tier.
  - Automatic refund of uncalled bets at the end of each street.
  - Official WSOP/casino odd-chip rule: remainder chips go to the player closest clockwise to the dealer button.
  - Strictly verified zero-leak chip conservation across multi-hand simulations.
- 📡 **Event-Driven Architecture**:
  - Emits richly typed `GameEvent`s: `HandStarted`, `AntePosted`, `BlindPosted`, `HoleCardsDealt`, `PlayerTurn`, `PlayerActed`, `StreetStarted`, `Showdown`, `PotAwarded`, `HandEnded`.
  - Every event carries structured data — no stringly-typed payloads.
  - Suitable for GUI frontends, replay systems, bots, and structured loggers.
- 🎮 **Interactive CLI & Benchmark Suite**:
  - Playable colored terminal interface against heuristic AI bots (`--play`).
  - Headless benchmark runner simulating **~30,000 full multi-street hands per second** (`--simulate`).

---

## Architecture Overview

```mermaid
flowchart TD
    subgraph Core ["Core Data Types"]
        Suit["Suit: ♣, ♦, ♥, ♠"]
        Rank["Rank: 2..Ace"]
        Card["Card: 32-bit Cactus Kev encoding"]
        Deck["Deck: 52 cards, Fisher-Yates shuffle"]
    end

    subgraph Evaluator ["Evaluator Module"]
        Tables["Cactus Kev Lookup Tables"]
        Eval5["5-Card Evaluator"]
        Eval7["7-Card Evaluator: Best 5 of 7"]
        HandRank["HandRank: Score 1..7462, Ord, Category"]
    end

    subgraph Accounting ["Pot & Actions"]
        PotManager["PotManager: Main & Side Pots"]
        Actions["LegalActions & Action Validation"]
    end

    subgraph Engine ["Game State Machine"]
        Player["Player: Seat, Stack, Hole Cards, Status"]
        Table["Table: Dealer Button, Blinds, Antes, Streets, Events"]
    end

    Core --> Evaluator
    Core --> Engine
    Evaluator --> Engine
    Accounting --> Engine
```

---

## Module Reference

| Module | Key Types | Description |
|---|---|---|
| `card` | `Card`, `Rank`, `Suit` | 32-bit Cactus Kev card encoding; parse from `"As"`, `"Td"`, `"10h"` |
| `deck` | `Deck` | 52-card deck with Fisher-Yates shuffle and deterministic RNG support |
| `evaluator` | `HandRank`, `HandCategory`, `evaluate_5`, `evaluate_7`, `evaluate_best` | Bitwise hand evaluator returning comparable, scored `HandRank` values |
| `player` | `Player`, `PlayerStatus` | Per-seat chip stack, hole cards, and status (`Active`, `Folded`, `AllIn`, `SittingOut`) |
| `action` | `Action`, `LegalActions` | Enum of all player actions; `LegalActions` snapshot for action validation |
| `pot` | `PotManager`, `Pot`, `PotPayout` | Side-pot calculation and chip distribution with odd-chip rules |
| `events` | `GameEvent`, `Stage` | Typed event stream emitted by the engine |
| `table` | `Table`, `TableConfig` | Central game state machine — seats, button, streets, and full NLHE rules |

---

## Quick Start

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (version 1.85+ / 2024 edition).

### Build & Run Tests
```bash
# Clone the repository
git clone https://github.com/ZocketZero/poker-engine.git
cd poker-engine

# Run the complete test suite (26 unit & integration tests)
cargo test --all-targets

# Check formatting and clippy lints
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

---

## CLI Usage

### Interactive Game vs Bots
Play Texas Hold'em directly in your terminal against 3 automated bots:
```bash
cargo run -- --play
```

Example output:
```text
=================================================
       WELCOME TO THE RUST POKER ENGINE          
=================================================
You are sitting at Seat 0 (Hero) against 3 automated bots.

-------------------------------------------------
Starting Hand #1 | Blinds: 10/20
Dealer Button is at Seat 0
Your Hole Cards: [7♠ 9♣]
Bot Diana chooses: Call

--- Your Turn (Pot: 50) ---
Board: [K♦ 5♠ 8♥]
Options ([f]old | [ch]eck | [b]et <20-980> | [a]ll-in (980)): ch
Bot Bob chooses: Check
Bot Charlie chooses: Bet(40)
...

=== Hand Results ===
Final Board: [K♦ 5♠ 8♥ J♦ 4♣]
  Bot Charlie shows [K♠ Q♣] -> Pair of Kings
  Bot Diana shows [K♣ 5♥] -> Two Pair, Kings and Fives with Jack kicker
  Bot Diana wins Pot #0 (180 chips) with Two Pair, Kings and Fives with Jack kicker
```

### High-Speed Simulation Benchmark
Benchmark the engine's throughput with simulated bot play:
```bash
cargo run --release -- --simulate 50000
```

Sample benchmark output:
```text
Starting simulation benchmark of 50000 hands...

=== Simulation Benchmark Results ===
Hands Completed: 50000
Time Elapsed:    1.682 seconds
Throughput:      29724 hands/second
```

### CLI Help
```bash
cargo run -- --help
```

---

## Library Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
poker-engine = { path = "." }
```

### 1. Evaluating Hands

```rust
use poker_engine::card::Card;
use poker_engine::evaluator::{evaluate_5, evaluate_7, evaluate_best, HandCategory};
use std::str::FromStr;

fn main() {
    // 5-card evaluation
    let royal_flush = [
        Card::from_str("As").unwrap(),
        Card::from_str("Ks").unwrap(),
        Card::from_str("Qs").unwrap(),
        Card::from_str("Js").unwrap(),
        Card::from_str("Ts").unwrap(),
    ];
    let rank = evaluate_5(&royal_flush);
    assert_eq!(rank.category, HandCategory::StraightFlush);
    assert_eq!(rank.score, 7462); // Highest possible score
    println!("Hand: {}", rank); // "Royal Flush"

    // 7-card evaluation — returns the best 5-card HandRank
    let cards = vec![
        Card::from_str("Ah").unwrap(),
        Card::from_str("Kh").unwrap(),
        Card::from_str("Qh").unwrap(),
        Card::from_str("Jh").unwrap(),
        Card::from_str("Th").unwrap(),
        Card::from_str("2c").unwrap(),
        Card::from_str("3d").unwrap(),
    ];
    let best = evaluate_7(&cards);
    println!("Best 7-card hand: {}", best); // "Royal Flush"

    // evaluate_best also returns the specific 5 cards that formed the best hand
    let (rank, best_five) = evaluate_best(&cards);
    println!("Best 5 cards: {:?}", best_five);
}
```

### 2. Simulating a Table Game

```rust
use poker_engine::action::Action;
use poker_engine::events::Stage;
use poker_engine::player::Player;
use poker_engine::table::{Table, TableConfig};

fn main() {
    // Configure a 6-max table with 10/20 blinds and a 5-chip ante
    let config = TableConfig {
        small_blind: 10,
        big_blind: 20,
        ante: 5,
        max_players: 6,
    };
    let mut table = Table::new(config);

    // Sit players
    table.sit_player(0, Player::new(0, "Alice", 1000)).unwrap();
    table.sit_player(1, Player::new(1, "Bob", 1000)).unwrap();
    table.sit_player(2, Player::new(2, "Charlie", 1000)).unwrap();

    // Start a new hand
    table.start_hand().unwrap();

    // Drive the game forward using legal actions
    while table.stage != Stage::HandEnded {
        if let Some(seat) = table.current_player {
            let legal = table.legal_actions(seat).unwrap();
            if legal.can_check {
                table.apply_action(Action::Check).unwrap();
            } else if legal.can_call {
                table.apply_action(Action::Call).unwrap();
            } else {
                table.apply_action(Action::Fold).unwrap();
            }
        }
    }

    println!("Hand completed! Pot awarded to winners.");
}
```

### 3. Consuming the Event Stream

```rust
use poker_engine::events::GameEvent;

// After a hand completes, inspect the full event log:
for event in &table.events {
    match event {
        GameEvent::BlindPosted { player_id, amount, is_small_blind } => {
            println!("Seat {} posted {} ({})", player_id, amount,
                if *is_small_blind { "SB" } else { "BB" });
        }
        GameEvent::PlayerActed { player_id, action, chips_committed } => {
            println!("Seat {} -> {:?} ({} chips)", player_id, action, chips_committed);
        }
        GameEvent::PotAwarded { pot_index, player_id, amount, hand_rank } => {
            if let Some(rank) = hand_rank {
                println!("Seat {} wins Pot #{} ({} chips) with {}", player_id, pot_index, amount, rank);
            } else {
                println!("Seat {} wins Pot #{} ({} chips, uncontested)", player_id, pot_index, amount);
            }
        }
        _ => {}
    }
}
```

---

## Testing

The project includes an extensive test suite verifying game logic, edge cases, and invariants across **26 tests**:

```bash
cargo test --all-targets -- --nocapture
```

| Test Suite | Count | Coverage |
|---|---|---|
| `card` / `deck` (unit) | 3 | All 52 cards unique, parsing (`As`, `Td`, `10s`), rank/suit properties |
| `pot` (unit) | 5 | Heads-up pot, 4-way side pot tiers, dead money absorption, odd-chip split |
| `evaluator` (unit + integration) | 9 | Hand hierarchy, wheel straight, 2-pair/flush/quads/full-house kickers, 7-card best-of, board plays |
| `game` (integration) | 9 | Preflop fold-around, full check-down to showdown, all-in runout, chip conservation over 50 hands, ante call amount, raise-after-opponent-all-in guard, min-raise after short-stack bet |

---

## License

This project is licensed under the [MIT License](LICENSE).
