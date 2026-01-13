use pkcore::casino::table::Table;
use pkcore::casino::table::event::TableAction;
use pkcore::games::GamePhase;
use pkcore::play::game::Game;
use pkcore::play::stages::flop_eval::FlopEval;
use pkcore::play::stages::turn_eval::TurnEval;
use pkcore::prelude::PlayerState;
use pkcore::util::data::TestData;
use pkcore::{PKError, Pile};

/// cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠" HSP THE HAND Negreanu/Hansen
///     https://www.youtube.com/watch?v=vjM60lqRhPg
///     https://www.youtube.com/watch?v=fEEW06iX4n8
///
/// Season 2, Episode 11
/// `cargo run --example the_hand`
fn main() -> Result<(), PKError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // TODO: Add ante of 200
    let table = TestData::the_hand_table();

    setup(&table).expect("Error setting up Table");

    preflop(&table).expect("Error running preflop");
    flop(&table)?;
    turn(&table)?;

    river(&table)?;

    Ok(())
}

fn setup(table: &Table) -> Result<(), PKError> {
    assert_eq!(8_000_000, table.table_chip_count());

    assert!(!table.seats.is_betting_complete());
    assert_eq!(0, table.button.value());
    assert_eq!(3, table.determine_utg());
    assert_eq!(3, table.next_to_act());
    assert_eq!(1, table.determine_small_blind());
    assert_eq!(2, table.determine_big_blind());
    assert_eq!(GamePhase::NewHand, table.get_phase());

    table.act_forced_bets().expect("ActForcedBets failed");
    assert_eq!(GamePhase::ForcedBets, table.get_phase());
    assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());

    assert_eq!(8_000_000, table.table_chip_count());

    if let Some(seat) = table.get_seat(1) {
        assert_eq!(999_950, seat.player.chips.count());
        assert_eq!(50, seat.player.bet.count());
        assert_eq!(50, table.to_call(1));
        assert_eq!(seat.player.state.get(), PlayerState::Blind(50));
    } else {
        panic!("Failed to get seat 1");
    }

    if let Some(seat) = table.get_seat(2) {
        assert_eq!(999_900, seat.player.chips.count());
        assert_eq!(100, seat.player.bet.count());
        assert_eq!(0, table.to_call(2));
        assert_eq!(seat.player.state.get(), PlayerState::Blind(100));
    } else {
        panic!("Failed to get seat 2");
    }

    table.deal_cards_to_seats().expect("Failed to deal cards to seats");
    assert_eq!(GamePhase::DealHoleCards, table.get_phase());
    assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());

    assert_eq!(
        "T♠ 2♥, 8♣ 3♥, A♦ Q♣, 5♦ 5♣, 6♠ 6♥, K♠ J♦, 4♦ 4♣, 7♣ 2♦",
        table.seats.cards_string()
    );

    println!();
    table.commentary_dump();
    println!("\n{table}");
    commentary_action_to(&table);

    Ok(())
}

fn preflop(table: &Table) -> Result<(), PKError> {
    assert_eq!(3, table.next_to_act());

    let gus = table.act_bet(3, 2100)?;
    assert_eq!(997_900, gus);
    assert_eq!(table.event_log.last_player_action().unwrap(), TableAction::Bet(3, 2100));

    commentary_action_to(&table);
    assert_eq!(4, table.next_to_act());

    let daniel = table.act_raise(4, 5000)?;
    assert_eq!(995_000, daniel);
    commentary_action_to(&table);
    assert_eq!(5, table.next_to_act());

    let _seat5_remaining = table.act_fold(5)?;
    commentary_action_to(&table);
    assert_eq!(6, table.next_to_act());

    let _seat6_remaining = table.act_fold(6)?;
    commentary_action_to(&table);
    assert_eq!(7, table.next_to_act());

    let _seat7_remaining = table.act_fold(7)?;
    commentary_action_to(&table);
    assert_eq!(0, table.next_to_act());

    let _seat0_remaining = table.act_fold(0)?;
    commentary_action_to(&table);
    assert_eq!(1, table.next_to_act());

    let _seat1_remaining = table.act_fold(1)?;
    commentary_action_to(&table);
    assert_eq!(2, table.next_to_act());

    let _seat2_remaining = table.act_fold(2)?;
    commentary_action_to(&table);
    assert_eq!(3, table.next_to_act());

    table.act_call(3)?;
    commentary_action_to(&table);
    assert!(table.seats.is_betting_complete());
    assert_eq!(3, table.next_to_act());

    let pot = table.bring_it_in()?;
    assert_eq!(10150, pot);

    Ok(())
}

fn flop(table: &Table) -> Result<(), PKError> {
    assert!(!table.seats.is_betting_complete());
    assert_eq!(3, table.next_to_act());

    table.deal_flop().expect("No flop");
    assert_eq!(GamePhase::BettingFlop, table.determine_betting_phase());

    let flop_eval = FlopEval::try_from(table)?;
    println!("\n{}", flop_eval);

    println!();
    println!("The Nuts @ Flop:");
    println!("{}", Game::try_from(table)?.board.flop.evals());

    let gus = table.act_check(3)?;
    assert_eq!(995_000, gus);
    assert_eq!(4, table.next_to_act());
    assert_eq!(10150, table.pot.count());
    assert!(!table.seats.is_betting_complete());

    let daniel = table.act_bet(4, 8_000)?;
    assert_eq!(987_000, daniel);
    assert_eq!(8_000, table.seats.chips_in_play());
    assert_eq!(3, table.next_to_act());
    assert!(!table.seats.is_betting_complete());

    let gus = table.act_raise(3, 26_000)?;
    assert_eq!(969_000, gus);
    assert_eq!(34_000, table.seats.chips_in_play());
    assert_eq!(4, table.next_to_act());
    assert!(!table.seats.is_betting_complete());

    let daniel = table.act_call(4)?;
    assert_eq!(26_000, daniel);
    assert_eq!(52_000, table.seats.chips_in_play());
    assert_eq!(3, table.next_to_act());
    assert!(table.seats.is_betting_complete());

    let pot = table.bring_it_in()?;
    assert_eq!(62_150, pot);

    Ok(())
}

fn turn(table: &Table) -> Result<(), PKError> {
    assert!(!table.seats.is_betting_complete());
    assert_eq!(3, table.next_to_act());

    table.deal_turn().expect("No turn");
    assert_eq!(GamePhase::BettingTurn, table.determine_betting_phase());

    let turn_eval = TurnEval::try_from(table)?;
    println!("\n{}", turn_eval);

    let gus = table.act_bet(3, 24_000)?;
    assert_eq!(945_000, gus);

    commentary_action_to(table);

    assert_eq!(4, table.next_to_act());

    let _daniel = table.act_call(4)?;
    assert_eq!(3, table.next_to_act());

    commentary_action_to(table);

    assert!(table.seats.is_betting_complete());
    let pot = table.bring_it_in()?;
    assert_eq!(110_150, pot);

    Ok(())
}


fn river(table: &Table) -> Result<(), PKError> {
    assert!(!table.seats.is_betting_complete());
    assert_eq!(3, table.next_to_act());

    table.deal_river().expect("No river");
    assert_eq!(GamePhase::BettingRiver, table.determine_betting_phase());

    Game::try_from(table).expect("").river_display_results();

    let gus = table.act_check(3)?;
    assert_eq!(945_000, gus);
    assert_eq!(4, table.next_to_act());

    commentary_action_to(table);

    let daniel = table.act_bet(4, 65_000)?;
    assert_eq!(880_000, daniel);
    assert_eq!(
        table.event_log.last_player_action().unwrap(),
        TableAction::Bet(4, 65_000)
    );
    assert_eq!(3, table.next_to_act());

    commentary_action_to(table);

    let gus = table.act_all_in(3)?;
    assert_eq!(945_000, gus);
    assert_eq!(4, table.next_to_act());

    commentary_action_to(table);

    let daniel = table.act_call(4)?;
    assert_eq!(945_000, daniel);


    //
    // assert_eq!(1_000_000 - 94_000, daniel);
    // assert_eq!(3, table.next_to_act());
    //
    // let gus = table.act_all_in(3)?;
    // assert_eq!(971_000, gus);

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
