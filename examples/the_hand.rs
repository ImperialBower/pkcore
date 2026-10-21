//! Recreation of "The Hand" (Negreanu vs. Hansen) using [`Table`], driven by a
//! [`PokerSession`] that deals each street as the betting closes.
//!
//! ```
//! cargo run --example the_hand
//! ```
//!
//! ## The Hand
//! Season 2, Episode 11 of High Stakes Poker, at the broadcast's stakes:
//! $300/$600 with a $100 ante. The table is
//! [`TestData::the_hand_broadcast_table`] and the moves are
//! [`TestData::the_hand_line`], the same ones that record
//! `data/hands/the_hand.yaml` (`examples/record_the_hand.rs`).
//! <https://www.youtube.com/watch?v=vjM60lqRhPg>

use pkcore::PKError;
use pkcore::Pile;
use pkcore::analysis::ev::Ev;
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::vs::Versus;
use pkcore::analysis::pot_odds::PotOdds;
use pkcore::analysis::range_equity::RangeEquity;
use pkcore::arrays::two::Two;
use pkcore::casino::action::PlayerAction;
use pkcore::casino::session::PokerSession;
use pkcore::casino::table::Table;
use pkcore::hand_history::ReplayStreet;
use pkcore::play::board::Board;
use pkcore::play::stages::flop_eval::FlopEval;
use pkcore::play::stages::river_eval::RiverEval;
use pkcore::play::stages::turn_eval::TurnEval;
use pkcore::util::data::TestData;
use std::str::FromStr;

fn main() -> Result<(), PKError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // The broadcast's table: $300/$600 with a $100 ante, Hansen on 287,100.
    // The other stacks are estimates (see `TestData::the_hand_broadcast_table`).
    let mut session = PokerSession::new(TestData::the_hand_broadcast_table());
    let deck = session.table.deck.clone();
    session.start_hand_with_deck(deck)?;
    setup(&session.table);

    let mut street = ReplayStreet::Preflop;
    for (next_street, seat, action) in TestData::the_hand_line() {
        // `next_actor` sweeps the bets in and deals the next street when the
        // betting round is over.
        if session.next_actor()? != Some(seat) {
            return Err(PKError::InvalidAction);
        }
        if next_street != street {
            street = next_street;
            on_street(&session.table, street);
        }
        if street == ReplayStreet::River && seat == 3 && action == PlayerAction::AllIn {
            gus_decision(&session.table);
        }
        session.apply_action(seat, action)?;
        commentary_action_to(&session.table);
    }
    if session.next_actor()?.is_some() {
        return Err(PKError::ActionIsntFinished);
    }

    let hand_result = session.end_hand()?;
    commentary_action_to(&session.table);
    println!("{hand_result}");
    range_equity();

    println!("\nDump event logs? (y/n): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        for action in &session.table.event_log {
            println!("{action}");
        }
    }

    Ok(())
}

// ── Phases ────────────────────────────────────────────────────────────────────

fn setup(table: &Table) {
    println!();
    // The event log is a plain Vec<TableAction>; walk it directly.
    for action in &table.event_log {
        if let Some(seat_num) = action.get_seat()
            && let Some(seat) = table.seats.get_seat(seat_num)
        {
            println!("--- {} {action}", seat.player.handle);
            continue;
        }
        println!("--- {action}");
    }

    println!("\n{table}");
    commentary_action_to(table);
}

/// What the new street means for the players, printed as it is dealt.
fn on_street(table: &Table, street: ReplayStreet) {
    let Ok(game) = table.build_game() else {
        return;
    };
    match street {
        ReplayStreet::Preflop => {}
        ReplayStreet::Flop => {
            if let Ok(fe) = FlopEval::try_from(game.clone()) {
                println!("{fe}");
            }
            println!();
            println!("The Nuts @ Flop:");
            println!("{}", game.board.flop.evals());
        }
        ReplayStreet::Turn => {
            if let Ok(te) = TurnEval::try_from(&game) {
                println!("{te}");
            }
        }
        ReplayStreet::River => river(&game),
    }
}

fn river(game: &pkcore::play::game::Game) {
    game.river_display_results();

    // ── River hand breakdown (independent of table type) ──
    let game = TestData::the_hand();
    if let Ok(re) = RiverEval::try_from(game) {
        println!("\n=== River Hand Breakdown ===");
        print!("{re}");

        if let (Ok(daniel), Ok(gus)) = (re.rank_for_player(0), re.rank_for_player(1)) {
            let winner = if gus > daniel {
                "Gus Hansen (quads)"
            } else {
                "Daniel Negreanu (full house)"
            };
            println!("  Winner: {winner}");
        }
    }
}

/// Pot odds and EV for Gus facing Daniel's 65,000 river bet.
fn gus_decision(table: &Table) {
    let daniel_bet = table.bet as u64;
    let pot_odds = PotOdds::new(table.effective_pot() as u64, daniel_bet);

    let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap_or_default();
    let gus_vs_fullhouse = Versus::new_with_board(Two::HAND_5D_5C, Combos::from_str("66").unwrap_or_default(), board);

    println!("\n=== Gus Hansen's Decision (facing {daniel_bet} chip bet) ===");
    println!("{pot_odds}");

    if let Ok(river_odds) = gus_vs_fullhouse.combined_odds_at_river() {
        println!(
            "Gus's equity vs 66: {:.1}%  (wins={}, losses={}, draws={})",
            river_odds.win_percentage(),
            river_odds.wins,
            river_odds.losses,
            river_odds.draws,
        );
        let ev = Ev::new(river_odds, pot_odds);
        println!(
            "EV of calling:  {:+.0} chips  (positive: {})",
            ev.as_chips(),
            ev.is_positive()
        );
    }
}

/// Range equity: sets vs overpairs on the final board.
fn range_equity() {
    let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap_or_default();
    let set_range = Combos::from_str("66,99").unwrap_or_default();
    let overpair_range = Combos::from_str("AA,KK,QQ,JJ,TT").unwrap_or_default();
    let range_equity = RangeEquity::new(set_range, overpair_range, board);
    if let Ok(re_odds) = range_equity.combined_odds() {
        println!("\n=== Range Equity on 9♣ 6♦ 5♥ 5♠ 8♠ ===");
        println!(
            "Sets/full-houses (66,99) vs overpairs (AA-TT): {:.1}% equity for sets",
            re_odds.win_percentage()
        );
    }
}

// ── Commentary helpers ────────────────────────────────────────────────────────

/// Prints the last player action and who acts next.
///
/// Equivalent to `Table::commentary_last_player_action` + `Table::commentary_action_to`,
/// implemented by reading `table.event_log` and `table.seats` directly.
fn commentary_action_to(table: &Table) {
    println!();

    // Last player action: walk the log backwards for the first player-action event.
    for action in table.event_log.iter().rev() {
        if action.is_player_action() {
            if let Some(seat_num) = action.get_seat()
                && let Some(seat) = table.seats.get_seat(seat_num)
            {
                println!("{} {action}", seat.player.handle);
            }
            break;
        }
    }

    // Next to act — mirrors Table::commentary_action_to.
    let next = table.next_to_act();
    if let Some(seat) = table.seats.get_seat(next) {
        if table.seats.is_betting_complete() {
            println!("All players have acted");
        } else {
            println!("Action to Seat {} {}", next, seat.player.handle);
        }
    }

    println!();
}
