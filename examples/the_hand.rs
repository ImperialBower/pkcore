use pkcore::PKError;
use pkcore::casino::table::Table;
use pkcore::util::data::TestData;

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

    let table = TestData::the_hand_table();

    setup(&table)?;
    preflop(&table)?;
    flop(&table)?;
    turn(&table)?;
    river(&table)?;

    println!("\nDump event logs? (y/n): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        // table.commentary_dump();
        println!("{}", table.event_log);
    }

    let pk_state = pkstate::PKState::from(&table);
    match serde_yaml_bw::to_string(&pk_state) {
        Ok(yaml) => println!("\n=== PKState YAML ===\n{yaml}"),
        Err(e) => eprintln!("Failed to serialize PKState: {e}"),
    }

    Ok(())
}

fn setup(table: &Table) -> Result<(), PKError> {
    table.act_forced_bets().expect("ActForcedBets failed");
    table.deal_cards_to_seats().expect("Failed to deal cards to seats");

    println!();
    table.commentary_dump();
    println!("\n{table}");
    commentary_action_to(table);

    Ok(())
}

fn preflop(table: &Table) -> Result<(), PKError> {
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

    let _pot = table.bring_it_in()?;

    Ok(())
}

fn flop(table: &Table) -> Result<(), PKError> {
    table.deal_flop().expect("No flop");

    table.eval_flop_display();

    println!();
    println!("The Nuts @ Flop:");
    println!("{}", table.eval_flop_the_nuts()?);

    let _gus = table.act_check(3)?;
    let _daniel = table.act_bet(4, 8_000)?;
    let _gus = table.act_raise(3, 26_000)?;
    let _daniel = table.act_call(4)?;
    let _pot = table.bring_it_in()?;

    Ok(())
}

fn turn(table: &Table) -> Result<(), PKError> {
    table.deal_turn().expect("No turn");
    table.eval_turn_display();

    let _gus = table.act_bet(3, 24_000)?;
    commentary_action_to(table);

    let _daniel = table.act_call(4)?;
    commentary_action_to(table);

    let _pot = table.bring_it_in()?;

    Ok(())
}

fn river(table: &Table) -> Result<(), PKError> {
    table.deal_river().expect("No river");

    table.eval_river_display();

    let _gus = table.act_check(3)?;
    commentary_action_to(table);

    let _daniel = table.act_bet(4, 65_000)?;
    commentary_action_to(table);

    let _gus = table.act_all_in(3)?;
    commentary_action_to(table);

    let _daniel = table.act_call(4)?;
    commentary_action_to(table);

    let hand_result = table.end_hand()?;
    commentary_action_to(table);

    println!("{hand_result}");

    // table.eval_river_display();

    Ok(())
}

fn commentary_action_to(table: &Table) {
    println!();
    if let Some(action) = table.commentary_last_player_action() {
        println!("{action}");
    }
    println!("{}", table.commentary_action_to());
    println!();
}
