//! Interactive REPL for the [`Dealer`] struct.
//!
//! Drive a full poker hand from the command line — seat players, start hands,
//! act on each street, and resolve the winner at showdown.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example dealer_repl
//! ```
//!
//! # Quick-start session
//!
//! ```text
//! dealer❯ seat Alice 10000
//! dealer❯ seat Bob 10000
//! dealer❯ seat Carol 10000
//! dealer❯ start
//! dealer❯ status
//! dealer❯ bet 2 300
//! dealer❯ call 3
//! dealer❯ fold 0
//! dealer❯ street          # consolidate bets → deal flop
//! dealer❯ check 1
//! dealer❯ check 2
//! dealer❯ street          # consolidate bets → deal turn
//! dealer❯ street          # consolidate bets → deal river
//! dealer❯ end
//! dealer❯ quit
//! ```

use clap::Parser;
use clap_repl::ClapEditor;
use pkcore::casino::dealer::{Dealer, DealerAction, DealerError};
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::Player;
use reedline::{DefaultPrompt, DefaultPromptSegment, FileBackedHistory};

// ── Commands ─────────────────────────────────────────────────────────────────

#[derive(Debug, Parser)]
#[command(name = "", about = "pkcore Dealer REPL — drive a poker hand from the command line")]
enum Command {
    /// Seat a new player. Chips default to 10 000 if omitted.
    ///
    /// Example: seat Alice 25000
    #[command(alias = "s")]
    Seat {
        /// Player name (no spaces)
        name: String,
        /// Starting chip count
        #[arg(default_value_t = 10_000)]
        chips: usize,
    },

    /// Seat a player at a specific seat number.
    ///
    /// Example: seat-at 3 Alice 25000
    #[command(alias = "sa")]
    SeatAt {
        /// Seat number (0-based)
        seat: u8,
        /// Player name (no spaces)
        name: String,
        /// Starting chip count
        #[arg(default_value_t = 10_000)]
        chips: usize,
    },

    /// Lets a player indicate to the `Table` that they are ready
    /// to play.
    #[command(alias = "re")]
    Ready { seat: u8 },

    /// Remove a player from their seat (between hands only).
    ///
    /// Example: remove 2
    #[command(alias = "rm")]
    Remove {
        /// Seat number to vacate
        seat: u8,
    },

    /// Start a new hand (shuffle, post blinds, deal hole cards).
    #[command(alias = "st")]
    Start,

    /// Advance to the next street (consolidate bets → deal flop/turn/river).
    #[command(alias = "sv")]
    Street,

    /// End the current hand and pay out the winner(s).
    #[command(alias = "e")]
    End,

    // ── Player actions ────────────────────────────────────────────────────────
    /// Bet a specific amount.
    ///
    /// Example: bet 2 400
    #[command(alias = "b")]
    Bet {
        /// Seat number
        seat: u8,
        /// Chip amount
        amount: usize,
    },

    /// Call the current bet.
    ///
    /// Example: call 3
    #[command(alias = "c")]
    Call {
        /// Seat number
        seat: u8,
    },

    /// Check (pass the action with no bet).
    ///
    /// Example: check 1
    #[command(alias = "ck")]
    Check {
        /// Seat number
        seat: u8,
    },

    /// Raise to a total amount.
    ///
    /// Example: raise 0 900
    #[command(alias = "r")]
    Raise {
        /// Seat number
        seat: u8,
        /// Total raise amount
        amount: usize,
    },

    /// Go all-in.
    ///
    /// Example: allin 4
    #[command(alias = "ai")]
    Allin {
        /// Seat number
        seat: u8,
    },

    /// Fold.
    ///
    /// Example: fold 2
    #[command(alias = "f")]
    Fold {
        /// Seat number
        seat: u8,
    },

    // ── Information ───────────────────────────────────────────────────────────
    /// Show the full table state.
    #[command(alias = "sh")]
    Status,

    /// Show who is next to act and what their options are.
    #[command(alias = "n")]
    Next,

    /// Show the community cards (board).
    #[command(alias = "bo")]
    Board,

    /// Show each seated player's chip count.
    #[command(alias = "ch")]
    Chips,

    /// Show the current pot size.
    #[command(alias = "p")]
    Pot,

    /// Show the full event log.
    #[command(alias = "l")]
    Log,

    /// Exit the REPL.
    #[command(alias = "q")]
    Quit,
}

// ── main ─────────────────────────────────────────────────────────────────────

fn main() {
    env_logger::init();

    println!("╔══════════════════════════════════════════════════╗");
    println!("║         pkcore  Dealer  REPL  v0.1               ║");
    println!("║  Tab-complete commands · Ctrl-D or quit to exit  ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("  Blinds: SB 50 / BB 100  ·  Up to 6 seats");
    println!("  Type a command or press Tab to see all options.");
    println!();

    let mut dealer = Dealer::new(ForcedBets::new(50, 100), 6);

    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("dealer".to_owned()),
        ..DefaultPrompt::default()
    };

    let rl = ClapEditor::<Command>::builder()
        .with_prompt(Box::new(prompt))
        .with_editor_hook(|reed| {
            reed.with_history(Box::new(
                FileBackedHistory::with_file(1_000, "./generated/dealer-repl-history".into()).unwrap_or_default(),
            ))
        })
        .build();

    rl.repl(|command| handle(&mut dealer, command));
}

// ── Command dispatch ──────────────────────────────────────────────────────────

fn handle(dealer: &mut Dealer, command: Command) {
    match command {
        // ── Seating ───────────────────────────────────────────────────────────
        Command::Seat { name, chips } => {
            let player = Player::new_with_chips(name.clone(), chips);
            match dealer.seat_player(player) {
                Ok(seat) => println!("✓ {name} seated at seat {seat} with {chips} chips"),
                Err(e) => print_error(&e),
            }
        }

        Command::SeatAt { seat, name, chips } => {
            let player = Player::new_with_chips(name.clone(), chips);
            match dealer.seat_player_at(player, seat) {
                Ok(()) => println!("✓ {name} seated at seat {seat} with {chips} chips"),
                Err(e) => print_error(&e),
            }
        }

        Command::Ready { seat } => match dealer.do_ready(seat) {
            Ok(player) => println!("✓ {} in seat {seat} is ready to play", player.handle),
            Err(e) => print_error(&e),
        },

        Command::Remove { seat } => match dealer.remove_player(seat) {
            Ok(player) => println!("✓ {} removed from seat {seat}", player.handle),
            Err(e) => print_error(&e),
        },

        // ── Hand lifecycle ────────────────────────────────────────────────────
        Command::Start => match dealer.start_hand() {
            Ok(()) => {
                println!("✓ Hand started — blinds posted and hole cards dealt");
                print_status(dealer);
            }
            Err(e) => print_error(&e),
        },

        Command::Street => match dealer.advance_street() {
            Ok(()) => {
                let board = dealer.table.board.to_string();
                if board.trim().is_empty() {
                    println!("✓ Bets consolidated");
                } else {
                    println!("✓ Board: {board}");
                }
                print_action_to(dealer);
            }
            Err(e) => print_error(&e),
        },

        Command::End => match dealer.end_hand() {
            Ok(result) => {
                println!("✓ Hand complete");
                println!("{result}");
                println!();
                print_chips(dealer);
            }
            Err(e) => print_error(&e),
        },

        // ── Player actions ────────────────────────────────────────────────────
        Command::Bet { seat, amount } => match dealer.act(DealerAction::Bet { seat, amount }) {
            Ok(()) => {
                println!("✓ Seat {seat} bets {amount}");
                print_action_to(dealer);
            }
            Err(e) => print_error(&e),
        },

        Command::Call { seat } => match dealer.act(DealerAction::Call { seat }) {
            Ok(()) => {
                println!("✓ Seat {seat} calls");
                print_action_to(dealer);
            }
            Err(e) => print_error(&e),
        },

        Command::Check { seat } => match dealer.act(DealerAction::Check { seat }) {
            Ok(()) => {
                println!("✓ Seat {seat} checks");
                print_action_to(dealer);
            }
            Err(e) => print_error(&e),
        },

        Command::Raise { seat, amount } => match dealer.act(DealerAction::Raise { seat, amount }) {
            Ok(()) => {
                println!("✓ Seat {seat} raises to {amount}");
                print_action_to(dealer);
            }
            Err(e) => print_error(&e),
        },

        Command::Allin { seat } => match dealer.act(DealerAction::AllIn { seat }) {
            Ok(()) => {
                println!("✓ Seat {seat} is all-in");
                print_action_to(dealer);
            }
            Err(e) => print_error(&e),
        },

        Command::Fold { seat } => match dealer.act(DealerAction::Fold { seat }) {
            Ok(()) => {
                println!("✓ Seat {seat} folds");
                print_action_to(dealer);
            }
            Err(e) => print_error(&e),
        },

        // ── Information ───────────────────────────────────────────────────────
        Command::Status => print_status(dealer),

        Command::Next => print_action_to(dealer),

        Command::Board => {
            let board = dealer.table.board.to_string();
            if board.trim().is_empty() {
                println!("Board: (no community cards yet)");
            } else {
                println!("Board: {board}");
            }
        }

        Command::Chips => print_chips(dealer),

        Command::Pot => println!("Pot: {}", dealer.pot()),

        Command::Log => {
            println!("{}", "─".repeat(60));
            for action in dealer.event_log() {
                println!("{action}");
            }
            println!("{}", "─".repeat(60));
        }

        Command::Quit => {
            println!("Goodbye! 👋");
            std::process::exit(0);
        }
    }
}

// ── Display helpers ───────────────────────────────────────────────────────────

fn print_error(e: &DealerError) {
    println!("✗ {e}");
}

fn print_status(dealer: &Dealer) {
    println!("{}", "═".repeat(60));
    println!("{}", dealer.table);
    println!("{}", "═".repeat(60));
}

fn print_action_to(dealer: &Dealer) {
    if dealer.table.is_game_over() {
        println!("  Hand is over — type 'end' to resolve it.");
        return;
    }
    if !dealer.is_hand_in_progress() {
        println!("  No hand in progress — type 'start' to begin a new hand.");
        return;
    }
    let seat = dealer.next_to_act();
    let pot = dealer.pot();
    print!("  Action to seat {seat}");
    if let Some(s) = dealer.table.seats.get_seat(seat) {
        print!(" ({})  chips: {}", s.player.handle, s.player.chips);
    }
    println!("  pot: {pot}");
}

fn print_chips(dealer: &Dealer) {
    println!("{}", "─".repeat(40));
    for i in 0..dealer.table.seats.size() {
        if let Some(seat) = dealer.table.seats.get_seat(i)
            && !seat.is_empty()
        {
            println!("  Seat {i}  {}  →  {} chips", seat.player.handle, seat.player.chips);
        }
    }
    println!("{}", "─".repeat(40));
}
