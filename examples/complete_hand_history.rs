use pkcore::casino::hand_history::HandHistoryExt;
use pkcore::prelude::*;
use pkcore::util::data::TestData;

/// Example showing a complete hand with hand history export
///
/// This demonstrates:
/// - Playing a complete hand
/// - Capturing all actions
/// - Exporting to hand history format
/// - Saving as YAML
///
/// Run with: cargo run --example complete_hand_history
fn main() -> Result<(), PKError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== Complete Hand History Example ===\n");

    // Use "The Hand" - a famous Negreanu vs Hansen hand
    let table = TestData::the_hand_table();

    println!("Table: {}", table.name);
    println!("Game: {:?}", table.game);
    println!("Blinds: {}/{}\n", table.forced.small_blind, table.forced.big_blind);

    // Display initial state
    println!("--- Initial Table State ---");
    for (i, seat_cell) in table.seats.borrow_all().iter().enumerate() {
        let seat = seat_cell.borrow();
        println!(
            "Seat {}: {} - {} chips - Cards: {}",
            i,
            seat.player.handle,
            seat.player.chips.count(),
            seat.cards
        );
    }

    println!("\n--- Playing Hand ---");

    // Shuffle and deal
    table.act_shuffle_deck();
    let _ = table.act_forced_bets();
    println!("✓ Blinds posted - Pot: {} chips", table.pot.count());

    // In a real scenario, you would have actual betting actions here
    // For now, we'll just capture the current state

    println!("\n--- Exporting Hand History ---");

    // Convert to hand history
    let mut history = table.to_hand_history()?;

    // Add some metadata
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("event".to_string(), "The Hand - Negreanu vs Hansen".to_string());
    metadata.insert(
        "stakes".to_string(),
        format!("{}/{}", history.game.blinds.small_blind, history.game.blinds.big_blind),
    );
    metadata.insert("format".to_string(), "Heads-Up".to_string());
    history.metadata = Some(metadata);

    // Serialize to YAML
    let yaml = history.to_yaml()?;

    println!("\n--- Generated YAML ---");
    println!("{}", yaml);

    // Save to file
    let filename = "generated/the_hand_history.yaml";
    history.save_to_file(filename)?;

    println!("\n✓ Hand history saved to: {}", filename);

    // Show statistics
    println!("\n--- Hand History Statistics ---");
    println!("Hand ID: {}", history.hand_id);
    println!("Players: {}", history.players.len());
    println!("Button: Seat {}", history.button);

    if let Some(preflop_actions) = &history.actions.preflop {
        println!("Preflop actions: {}", preflop_actions.len());
    }

    if let Some(board) = &history.board {
        if board.flop.is_some() {
            println!("Board: Flop dealt");
        }
        if board.turn.is_some() {
            println!("        Turn dealt");
        }
        if board.river.is_some() {
            println!("        River dealt");
        }
    }

    if let Some(meta) = &history.metadata {
        println!("\nMetadata:");
        for (key, value) in meta {
            println!("  {}: {}", key, value);
        }
    }

    println!("\n=== Example Complete ===");
    println!("\nYou can now:");
    println!("  1. View the YAML file: cat {}", filename);
    println!("  2. Load it back: PokerHandHistory::load_from_file(\"{}\")", filename);
    println!("  3. Parse it with any YAML-compatible tool");

    Ok(())
}
