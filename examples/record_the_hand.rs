//! Records "The Hand" (Hansen vs. Negreanu, High Stakes Poker Season 2,
//! Episode 11) from the engine and prints it as a hand-history YAML file.
//!
//! The file in `data/hands/the_hand.yaml` is this program's output, not
//! something typed (`DEFECT_040`): the table is
//! [`TestData::the_hand_broadcast_table`], the moves are
//! [`TestData::the_hand_line`], and the engine works out the pots, the nets
//! and the hand ranks.
//!
//! ```text
//! cargo run --features full --example record_the_hand > data/hands/the_hand.yaml
//! ```
//!
//! Then put the header comment back on top of the file: the sources, and the
//! stacks that are estimates.

use pkcore::PKError;
use pkcore::casino::session::PokerSession;
use pkcore::hand_history::HandHistory;
use pkcore::util::data::TestData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let history = record()?;
    print!("{}", history.to_yaml()?);
    Ok(())
}

/// Plays the hand and builds its history.
fn record() -> Result<HandHistory, PKError> {
    let table = TestData::the_hand_broadcast_table();
    let button = table.button;
    let forced = table.forced;
    let seat_count = table.seats.size();

    // Stacks before the forced bets.
    let stacks: Vec<(u8, String, usize)> = (0..seat_count)
        .filter_map(|seat| {
            table
                .seats
                .get_seat(seat)
                .map(|occupant| (seat, occupant.player.handle.clone(), occupant.player.chips))
        })
        .collect();

    let mut session = PokerSession::new(table);
    let event_log_start = session.table.event_log.len();
    let deck = session.table.deck.clone();
    session.start_hand_with_deck(deck)?;

    // Hole cards in the order they were dealt, so `stacked_deck` rebuilds the
    // same deck.
    let player_snapshot: Vec<(u8, String, usize, Option<String>)> = stacks
        .into_iter()
        .map(|(seat, name, stack)| {
            let hole = session
                .table
                .seats
                .get_seat(seat)
                .filter(|occupant| occupant.cards.has_cards())
                .map(|occupant| occupant.cards.to_string());
            (seat, name, stack, hole)
        })
        .collect();

    for (street, seat, action) in TestData::the_hand_line() {
        let next = session.next_actor()?;
        if next != Some(seat) {
            eprintln!("{street}: expected seat {seat} to act, the engine has {next:?}");
            return Err(PKError::InvalidAction);
        }
        session.apply_action(seat, action)?;
    }
    if session.next_actor()?.is_some() {
        eprintln!("the line is over and the hand is not");
        return Err(PKError::ActionIsntFinished);
    }

    let board = session.table.board.to_string();
    let event_log = session.table.event_log[event_log_start..].to_vec();
    let winnings = session.end_hand()?;
    let ending_stacks: Vec<(u8, usize)> = (0..seat_count)
        .filter_map(|seat| {
            session
                .table
                .seats
                .get_seat(seat)
                .map(|occupant| (seat, occupant.player.chips))
        })
        .collect();

    let source = "High Stakes Poker Season 2, Episode 11";
    let mut history = HandHistory::from_table_state(
        1,
        0,
        button,
        &forced,
        &player_snapshot,
        &board,
        &winnings,
        &event_log,
        &ending_stacks,
        source,
        None,
    )
    .with_table_size(seat_count as usize);
    history.pkcore_version = Some(env!("CARGO_PKG_VERSION").to_string());
    history.hand.id = "hsp-s2e11-the-hand".to_string();
    history.hand.timestamp = None;
    history.hand.description = Some("The Hand: Gus Hansen's quad fives over Daniel Negreanu's sixes full".to_string());
    history.table.name = Some("High Stakes Poker".to_string());
    Ok(history)
}
