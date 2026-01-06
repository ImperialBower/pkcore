use pkcore::casino::table::Table;
use pkcore::casino::table::event::TableAction;
use pkcore::play::game::Game;
use pkcore::play::stages::flop_eval::FlopEval;
use pkcore::play::stages::turn_eval::TurnEval;
use pkcore::util::data::TestData;
use pkcore::{PKError, Pile};

/// cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠" HSP THE HAND Negreanu/Hansen
///     https://www.youtube.com/watch?v=vjM60lqRhPg
///     https://www.youtube.com/watch?v=fEEW06iX4n8
///
/// Season 2, Episode 11
/// `cargo run --example the_hand`
fn main() -> Result<(), PKError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    // TODO: Add ante of 200
    let table = TestData::the_hand_table();
    assert_eq!(8_000_000, table.table_chip_count());

    assert!(!table.seats.is_betting_complete());
    assert_eq!(0, table.button.value());
    assert_eq!(3, table.determine_utg());
    assert_eq!(1, table.determine_small_blind());
    assert_eq!(2, table.determine_big_blind());

    let _ = table.act_forced_bets();

    assert_eq!(8_000_000, table.table_chip_count());

    if let Some(seat) = table.get_seat(1) {
        assert_eq!(999_950, seat.player.chips.count());
        assert_eq!(50, seat.player.bet.count());
        assert_eq!(50, table.to_call(1));
    } else {
        panic!("Failed to get seat 1");
    }

    if let Some(seat) = table.get_seat(2) {
        assert_eq!(999_900, seat.player.chips.count());
        assert_eq!(100, seat.player.bet.count());
        assert_eq!(0, table.to_call(2));
    } else {
        panic!("Failed to get seat 2");
    }

    table.deal_cards_to_seats().expect("Failed to deal cards to seats");
    println!();
    table.commentary_dump();
    assert_eq!(
        "T♠ 2♥, 8♣ 3♥, A♦ Q♣, 5♦ 5♣, 6♠ 6♥, K♠ J♦, 4♦ 4♣, 7♣ 2♦",
        table.seats.cards_string()
    );

    println!("\n{table}");

    commentary_action_to(&table);

    let gus = table.act_bet(3, 2100)?;
    assert_eq!(997_900, gus);
    assert_eq!(table.event_log.last_player_action().unwrap(), TableAction::Bet(3, 2100));

    commentary_action_to(&table);

    let _daniel = table.act_raise(4, 5000)?;
    commentary_action_to(&table);

    let _seat5_remaining = table.act_fold(5)?;
    commentary_action_to(&table);

    let _seat6_remaining = table.act_fold(6)?;
    commentary_action_to(&table);

    let _seat7_remaining = table.act_fold(7)?;
    commentary_action_to(&table);

    let _seat0_remaining = table.act_fold(0)?;
    assert_eq!(1, table.get_action_to());
    commentary_action_to(&table);

    let _seat1_remaining = table.act_fold(1)?;
    commentary_action_to(&table);

    let _seat2_remaining = table.act_fold(2)?;
    commentary_action_to(&table);

    table.act_call(3)?;
    commentary_action_to(&table);
    assert!(table.seats.is_betting_complete());

    // The Flop
    let pot = table.bring_it_in()?;
    assert_eq!(10150, pot);
    assert!(!table.seats.is_betting_complete());

    table.deal_flop().expect("No flop");

    let flop_eval = FlopEval::try_from(&table)?;
    println!("\n{}", flop_eval);

    println!();
    println!("The Nuts @ Flop:");
    println!("{}", Game::try_from(&table)?.board.flop.evals());

    table.deal_turn().expect("No turn");
    let turn_eval = TurnEval::try_from(&table)?;
    println!("\n{}", turn_eval);

    let _gus = table.act_bet(3, 24_000)?;
    commentary_action_to(&table);
    let _daniel = table.act_call(4)?;

    assert_eq!(3, table.get_action_to());

    commentary_action_to(&table);

    assert!(table.seats.is_betting_complete());

    //
    // println!("{table}");
    // table.commentary_dump();
    //
    // println!("{}", table.commentary_action_to());

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
