use pkcore::casino::hand_history::{HandHistoryExt, PokerHandHistory};
use pkcore::prelude::*;

/// Example demonstrating how to save and load poker hand histories in YAML format
///
/// This shows how to:
/// - Convert a Table to a PokerHandHistory
/// - Serialize to YAML
/// - Save to file
/// - Load from file
/// - Access hand history data
///
/// Run with: cargo run --example hand_history_demo
fn main() -> Result<(), PKError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== Poker Hand History Demo ===\n");

    // Create a table with some action
    let table = Table::default();

    // Shuffle and deal
    table.act_shuffle_deck();
    let _ = table.deal_cards_to_seats();

    // Post blinds
    let _ = table.act_forced_bets();

    println!("Table setup complete:");
    println!("  Players: {}", table.seats.size());
    println!("  Button: Seat {}", table.button.value());
    println!("  Pot: {} chips\n", table.pot.count());

    // Convert table to hand history
    println!("Converting table to hand history...");
    let history = table.to_hand_history()?;

    println!("Hand History created:");
    println!("  Hand ID: {}", history.hand_id);
    println!("  Table: {}", history.table.name);
    println!("  Game: {}", history.game.variant);
    println!(
        "  Blinds: {}/{}",
        history.game.blinds.small_blind, history.game.blinds.big_blind
    );
    println!("  Players: {}", history.players.len());

    // Serialize to YAML
    println!("\n--- YAML Output ---");
    let yaml = history.to_yaml()?;
    println!("{}", yaml);

    // Save to file
    let filename = "generated/example_hand.yaml";
    println!("Saving to file: {}", filename);
    history.save_to_file(filename)?;
    println!("✓ Hand history saved successfully!");

    // Load from file
    println!("\nLoading hand history from file...");
    let loaded_history = PokerHandHistory::load_from_file(filename)?;

    println!("✓ Hand history loaded successfully!");
    println!("  Loaded Hand ID: {}", loaded_history.hand_id);
    println!("  Players in loaded history: {}", loaded_history.players.len());

    // Verify they match
    if history == loaded_history {
        println!("\n✓ Original and loaded histories match!");
    } else {
        println!("\n✗ Histories don't match!");
    }

    // Display player information from loaded history
    println!("\n--- Player Details ---");
    for player in &loaded_history.players {
        println!("Seat {}: {} (Stack: {} chips)", player.seat, player.name, player.stack);
        if let Some(cards) = &player.hole_cards {
            println!("  Cards: {}", cards.join(" "));
        }
    }

    // Show board cards if any
    if let Some(board) = &loaded_history.board {
        println!("\n--- Community Cards ---");
        if let Some(flop) = &board.flop {
            println!("Flop: {}", flop.join(" "));
        }
        if let Some(turn) = &board.turn {
            println!("Turn: {}", turn);
        }
        if let Some(river) = &board.river {
            println!("River: {}", river);
        }
    }

    // Show actions if any
    println!("\n--- Actions ---");
    if let Some(preflop) = &loaded_history.actions.preflop {
        println!("Preflop: {} actions", preflop.len());
        for action in preflop {
            if let Some(amount) = action.amount {
                println!("  Seat {} {}s {}", action.seat, action.action, amount);
            } else {
                println!("  Seat {} {}s", action.seat, action.action);
            }
        }
    }

    println!("\n=== Demo Complete ===");
    println!("\nThe hand history has been saved to: {}", filename);
    println!("You can view it with: cat {}", filename);

    Ok(())
}
