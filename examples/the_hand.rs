use pkcore::PKError;
use pkcore::analysis::ev::Ev;
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::vs::Versus;
use pkcore::analysis::pot_odds::PotOdds;
use pkcore::analysis::range_equity::RangeEquity;
use pkcore::arrays::two::Two;
use pkcore::casino::table_celled::TableCelled;
use pkcore::play::board::Board;
use pkcore::play::stages::river_eval::RiverEval;
use pkcore::util::data::TestData;
use std::str::FromStr;

/// Here's a recreation of "The Hand" between Daniel Negreanu and Gus Hansen, using strict
/// assertions to validate that the `Table` engine is working correctly.
///
/// TODO: Can I add a way to automate tests based on the logs?
///
/// `cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"` HSP THE HAND Negreanu/Hansen
///     https://www.youtube.com/watch?v=vjM60lqRhPg
///     https://www.youtube.com/watch?v=fEEW06iX4n8
///
/// Season 2, Episode 11
/// `cargo run --example the_hand`
fn main() -> Result<(), PKError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let table = TestData::the_hand_table_celled();

    setup(&table)?;
    let preflop_pot = preflop(&table)?;
    let flop_pot = flop(&table, preflop_pot)?;
    let turn_pot = turn(&table, flop_pot)?;
    river(&table, turn_pot)?;

    Ok(())
}

fn print_street_header(name: &str) {
    println!("\n{}", "═".repeat(56));
    println!("  {name}");
    println!("{}", "═".repeat(56));
}

fn setup(table: &TableCelled) -> Result<(), PKError> {
    table.act_forced_bets().expect("ActForcedBets failed");
    table.deal_cards_to_seats().expect("Failed to deal cards to seats");

    print_street_header("SETUP — Blinds & Hole Cards");
    table.commentary_dump();
    println!("\n{table}");
    commentary_action_to(table);

    Ok(())
}

fn preflop(table: &TableCelled) -> Result<usize, PKError> {
    print_street_header("PREFLOP");

    let _gus = table.act_bet(3, 2100)?;
    commentary_action_to(table);
    let _daniel = table.act_raise(4, 5000)?;
    commentary_action_to(table);

    let _seat5_remaining = table.act_fold(5)?;
    commentary_action_to(table);

    let _seat6_remaining = table.act_fold(6)?;
    commentary_action_to(table);

    let _seat7_remaining = table.act_fold(7)?;
    commentary_action_to(table);

    let _seat0_remaining = table.act_fold(0)?;
    commentary_action_to(table);

    let _seat1_remaining = table.act_fold(1)?;
    commentary_action_to(table);

    let _seat2_remaining = table.act_fold(2)?;
    commentary_action_to(table);

    table.act_call(3)?;
    commentary_action_to(table);

    let pot = table.bring_it_in()?;
    println!("  >>> Pot heading to flop: {} chips", pot);

    Ok(pot)
}

fn flop(table: &TableCelled, _preflop_pot: usize) -> Result<usize, PKError> {
    table.deal_flop().expect("No flop");
    print_street_header("FLOP");

    table.eval_flop_display();

    println!();
    println!("The Nuts @ Flop:");
    println!("{}", table.eval_flop_the_nuts()?);

    let _gus = table.act_check(3)?;
    commentary_action_to(table);

    let _daniel = table.act_bet(4, 8_000)?;
    commentary_action_to(table);

    let _gus = table.act_raise(3, 26_000)?;
    commentary_action_to(table);

    let _daniel = table.act_call(4)?;
    commentary_action_to(table);

    let pot = table.bring_it_in()?;
    println!("  >>> Pot heading to turn: {} chips", pot);

    Ok(pot)
}

fn turn(table: &TableCelled, _flop_pot: usize) -> Result<usize, PKError> {
    table.deal_turn().expect("No turn");
    print_street_header("TURN");
    table.eval_turn_display();

    let _gus = table.act_bet(3, 24_000)?;
    commentary_action_to(table);

    let _daniel = table.act_call(4)?;
    commentary_action_to(table);

    let pot = table.bring_it_in()?;
    println!("  >>> Pot heading to river: {} chips", pot);

    Ok(pot)
}

fn river(table: &TableCelled, turn_pot: usize) -> Result<(), PKError> {
    table.deal_river().expect("No river");
    print_street_header("RIVER");
    table.eval_river_display();

    // ── River Hand Analysis ──────────────────────────────────────────────────
    // RiverEval gives us a deterministic single-outcome ranking for each player
    // on the complete five-card board — no runout enumeration needed.
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

    // ── Pot Odds & EV for Gus facing Daniel's 65k bet ───────────────────────
    // Daniel fires 65k into the pot built over preflop, flop, and turn.
    // We compute Gus's breakeven equity requirement and his actual EV.
    let daniel_bet: u64 = 65_000;
    let _daniel = table.act_bet(4, daniel_bet as usize)?;
    commentary_action_to(table);

    // pot_after_bet = chips in pot before Gus's call decision
    let pot_after_bet = turn_pot as u64 + daniel_bet;
    let pot_odds = PotOdds::new(pot_after_bet, daniel_bet);

    // Compute Gus's exact equity (5♦ 5♣ quads) vs Daniel's full-house range (66)
    // on the completed board. remaining_at_river() handles card-removal blocking.
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

    // ── Range Equity: sets & full houses vs overpairs on this board ─────────
    // Illustrates how dramatically board texture shifts equity — the paired
    // board with a set destroys overpair equity.
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

fn commentary_action_to(table: &TableCelled) {
    println!();
    if let Some(action) = table.commentary_last_player_action() {
        println!("{action}");
    }
    println!("{}", table.commentary_action_to());
    println!();
}
