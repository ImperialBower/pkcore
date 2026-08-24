//! Recreation of "The Hand" (Negreanu vs. Hansen) using [`Table`].
//!
//! Every function receives `&mut Table`. The mutability contract is explicit at
//! every call site: the compiler enforces exclusive access rather than hiding it
//! behind `Cell`/`RefCell`, which is what the retired `TableCelled` engine did.
//!
//! ```
//! cargo run --example the_hand
//! ```
//!
//! ## The Hand
//! Season 2, Episode 11 of High Stakes Poker.
//! <https://www.youtube.com/watch?v=vjM60lqRhPg>

use pkcore::PKError;
use pkcore::Pile;
use pkcore::analysis::ev::Ev;
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::vs::Versus;
use pkcore::analysis::pot_odds::PotOdds;
use pkcore::analysis::range_equity::RangeEquity;
use pkcore::arrays::two::Two;
use pkcore::cards::Cards;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::{Player, Seat, Seats, Table};
use pkcore::play::board::Board;
use pkcore::play::stages::flop_eval::FlopEval;
use pkcore::play::stages::river_eval::RiverEval;
use pkcore::play::stages::turn_eval::TurnEval;
use pkcore::util::data::TestData;
use std::str::FromStr;

fn main() -> Result<(), PKError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut table = the_hand_table();

    setup(&mut table)?;
    let preflop_pot = preflop(&mut table)?;
    let flop_pot = flop(&mut table, preflop_pot)?;
    let turn_pot = turn(&mut table, flop_pot)?;
    river(&mut table, turn_pot)?;

    println!("\nDump event logs? (y/n): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        for action in &table.event_log {
            println!("{action}");
        }
    }

    Ok(())
}

/// Constructs a [`Table`] primed with the exact deck order used in The Hand.
///
/// The key difference from `Table::nlh_primed`: because `Table.deck` is a
/// plain `pub` field, we can inject the pre-ordered deck with a direct assignment
/// after construction rather than through a specialised constructor.
fn the_hand_table() -> Table {
    let seats = Seats::new(vec![
        Seat::new(Player::new_with_chips("Doyle Brunson".to_string(), 1_000_000)),
        Seat::new(Player::new_with_chips("Eli Elezra".to_string(), 1_000_000)),
        Seat::new(Player::new_with_chips("Antonio Esfandiari".to_string(), 1_000_000)),
        Seat::new(Player::new_with_chips("Gus Hansen".to_string(), 1_000_000)),
        Seat::new(Player::new_with_chips("Daniel Negreanu".to_string(), 1_000_000)),
        Seat::new(Player::new_with_chips("Cory Zeidman".to_string(), 1_000_000)),
        Seat::new(Player::new_with_chips("Barry Greenstein".to_string(), 1_000_000)),
        Seat::new(Player::new_with_chips("Amnon Filippi".to_string(), 1_000_000)),
    ]);

    let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));

    // Inject the pre-ordered deck so cards deal in the documented order.
    // With Table, `deck` is a plain public field — no wrapper needed.
    table.deck = Cards::deck_primed(&TestData::the_hand_cards_dealable());

    table
}

// ── Phases ────────────────────────────────────────────────────────────────────

fn setup(table: &mut Table) -> Result<(), PKError> {
    table.act_forced_bets().expect("forced bets failed");
    table.deal_cards_to_seats().expect("failed to deal hole cards");

    println!();
    // The event log is a plain Vec<TableAction>; walk it directly.
    for action in &table.event_log {
        if let Some(seat_num) = action.get_seat() {
            if let Some(seat) = table.seats.get_seat(seat_num) {
                println!("--- {} {action}", seat.player.handle);
                continue;
            }
        }
        println!("--- {action}");
    }

    println!("\n{table}");
    commentary_action_to(table);

    Ok(())
}

fn preflop(table: &mut Table) -> Result<usize, PKError> {
    let _gus = table.act_bet(3, 2_100)?;
    commentary_action_to(table);

    let _daniel = table.act_raise(4, 5_000)?;
    commentary_action_to(table);

    let _seat5 = table.act_fold(5)?;
    commentary_action_to(table);

    let _seat6 = table.act_fold(6)?;
    commentary_action_to(table);

    let _seat7 = table.act_fold(7)?;
    commentary_action_to(table);

    let _seat0 = table.act_fold(0)?;
    commentary_action_to(table);

    let _seat1 = table.act_fold(1)?;
    commentary_action_to(table);

    let _seat2 = table.act_fold(2)?;
    commentary_action_to(table);

    table.act_call(3)?;
    commentary_action_to(table);

    let pot = table.bring_it_in()?;
    Ok(pot)
}

fn flop(table: &mut Table, _preflop_pot: usize) -> Result<usize, PKError> {
    table.deal_flop().expect("no flop");

    // Evaluation via build_game — the NoCell equivalent of table.eval_flop_display().
    if let Ok(game) = table.build_game() {
        if let Ok(fe) = FlopEval::try_from(game) {
            println!("{fe}");
        }
    }

    println!();
    println!("The Nuts @ Flop:");
    if let Ok(game) = table.build_game() {
        println!("{}", game.board.flop.evals());
    }

    let _gus = table.act_check(3)?;
    let _daniel = table.act_bet(4, 8_000)?;
    let _gus = table.act_raise(3, 26_000)?;
    let _daniel = table.act_call(4)?;
    let pot = table.bring_it_in()?;

    Ok(pot)
}

fn turn(table: &mut Table, _flop_pot: usize) -> Result<usize, PKError> {
    table.deal_turn().expect("no turn");

    // Evaluation via build_game — the NoCell equivalent of table.eval_turn_display().
    if let Ok(game) = table.build_game() {
        if let Ok(te) = TurnEval::try_from(&game) {
            println!("{te}");
        }
    }

    let _gus = table.act_bet(3, 24_000)?;
    commentary_action_to(table);

    let _daniel = table.act_call(4)?;
    commentary_action_to(table);

    let pot = table.bring_it_in()?;
    Ok(pot)
}

fn river(table: &mut Table, turn_pot: usize) -> Result<(), PKError> {
    table.deal_river().expect("no river");

    // Evaluation via build_game — the NoCell equivalent of table.eval_river_display().
    if let Ok(game) = table.build_game() {
        game.river_display_results();
    }

    // ── River hand breakdown (identical to the_hand.rs — independent of table type) ──
    let game = TestData::the_hand();
    if let Ok(re) = RiverEval::try_from(game) {
        println!("\n=== River Hand Breakdown ===");
        print!("{re}");

        match (re.rank_for_player(0), re.rank_for_player(1)) {
            (Ok(daniel), Ok(gus)) => {
                let winner = if gus > daniel {
                    "Gus Hansen (quads)"
                } else {
                    "Daniel Negreanu (full house)"
                };
                println!("  Winner: {winner}");
            }
            _ => {}
        }
    }

    let _gus = table.act_check(3)?;
    commentary_action_to(table);

    // ── Pot odds & EV for Gus facing Daniel's 65k bet ────────────────────────
    let daniel_bet: u64 = 65_000;
    let _daniel = table.act_bet(4, daniel_bet as usize)?;
    commentary_action_to(table);

    let pot_after_bet = turn_pot as u64 + daniel_bet;
    let pot_odds = PotOdds::new(pot_after_bet, daniel_bet);

    let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap_or_default();
    let gus_vs_fullhouse = Versus::new_with_board(Two::HAND_5D_5C, Combos::from_str("66").unwrap_or_default(), board);

    println!("\n=== Gus Hansen's Decision (facing {} chip bet) ===", daniel_bet);
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

    let _gus = table.act_all_in(3)?;
    commentary_action_to(table);

    let _daniel = table.act_call(4)?;
    commentary_action_to(table);

    let hand_result = table.end_hand()?;
    commentary_action_to(table);

    println!("{hand_result}");

    // ── Range equity: sets vs overpairs ──────────────────────────────────────
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

    Ok(())
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
            if let Some(seat_num) = action.get_seat() {
                if let Some(seat) = table.seats.get_seat(seat_num) {
                    println!("{} {action}", seat.player.handle);
                }
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
