use pkcore::prelude::*;

/// Shows how to read a snapshot of a hand in progress straight off the
/// [`Table`].
///
/// The celled engine exposed this through a `get_game_state()` method that
/// packed everything into a `GameState` struct. `Table` keeps the same
/// information in plain public fields, so no wrapper type is needed — read
/// what you want, when you want it. EPIC-83 removed the wrapper.
///
/// What a table tells you:
/// - identity: `id`, `name`, `game`
/// - where the hand is: `phase`, `button`, `next_to_act()`
/// - the money: `pot`, `bet`, `forced`
/// - the cards: `board`, `deck`
/// - the people: `seats`
fn main() {
    println!("=== Table State Demo ===\n");

    // A default 6-handed No Limit Hold'em table.
    let mut table = Table::default();

    println!("Initial State:");
    println!("{table}");

    println!("\n--- Individual Field Access ---");
    println!("Table Name: {}", table.name);
    println!("Game Type: {:?}", table.game);
    println!("Current Phase: {}", table.phase);
    println!("Button Position: Seat {}", table.button);
    println!("Next to Act: Seat {}", table.next_to_act());
    println!("Blinds: {}", table.forced);
    println!(
        "Players: {} active out of {} total",
        table.seats.count_active_in_hand(),
        table.seats.count_occupied()
    );

    println!("\n=== Simulating Game Progress ===\n");

    table.act_shuffle_deck();
    let _ = table.act_forced_bets();

    println!("After Forced Bets:");
    println!("{table}");

    if table.pot > 0 {
        println!("\n✓ Blinds have been posted!");
        println!("  Pot now contains {} chips", table.pot);
    }

    if table.bet > 0 {
        println!("  Current bet to call: {} chips", table.bet);
    }

    // Reading state straight off the table is useful for:
    // - logging game progress
    // - making AI decisions
    // - debugging game flow
    // - showing the table to players

    println!("\n=== Demo Complete ===");
}
