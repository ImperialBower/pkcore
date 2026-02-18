use pkcore::prelude::*;
use tts::Tts;

/// Advanced poker game narrator using text-to-speech
///
/// This example plays through a poker hand and narrates all the action
/// including dealing cards, betting, and results.
///
/// Run with: cargo run --example poker_narrator
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize TTS
    let mut tts = Tts::default()?;

    tts.speak("Welcome to Poker Narrator", false)?;
    println!("=== Poker Narrator ===\n");

    // Set up a table
    let table = Table::default();

    // Get initial state
    let state = table.get_game_state();

    // Opening announcement
    let intro = format!(
        "Welcome to {}. We're playing {}. {} players are seated. \
         The blinds are {} and {}. Let's shuffle up and deal!",
        state.table_name,
        "No Limit Hold'em",
        state.total_players,
        state.small_blind,
        state.big_blind
    );
    println!("{}\n", intro);
    tts.speak(&intro, false)?;

    // Shuffle
    println!("Shuffling the deck...");
    tts.speak("Shuffling the deck", false)?;
    table.act_shuffle_deck();

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Post blinds
    println!("\nPosting blinds...");
    tts.speak("Posting the blinds", false)?;
    let _ = table.act_forced_bets();

    let state = table.get_game_state();
    let blind_msg = format!(
        "The small blind of {} and big blind of {} have been posted. \
         The pot is now {} chips.",
        state.small_blind,
        state.big_blind,
        state.pot_size
    );
    println!("{}\n", blind_msg);
    tts.speak(&blind_msg, false)?;

    // Game phase update
    println!("Current phase: {}", state.phase);
    let phase_msg = format!("We are now in the {} phase", state.phase);
    tts.speak(&phase_msg, false)?;

    // Deck status
    let deck_msg = format!(
        "{} cards remain in the deck. {} players are active.",
        state.deck_remaining,
        state.active_players
    );
    println!("{}\n", deck_msg);
    tts.speak(&deck_msg, false)?;

    // Action summary
    println!("\n=== Game Status ===");
    println!("Button: Seat {}", state.button_position);
    println!("Action on: Seat {}", state.next_to_act);
    println!("Pot: {} chips", state.pot_size);
    println!("Current bet: {} chips", state.current_bet);

    let action_msg = format!(
        "The button is on seat {}. Action is on seat {}. \
         The pot contains {} chips. Current bet to call is {} chips.",
        state.button_position,
        state.next_to_act,
        state.pot_size,
        state.current_bet
    );
    tts.speak(&action_msg, false)?;

    // Closing
    println!("\n=== Narrator Demo Complete ===");
    tts.speak("Poker narrator demo complete. May the cards be with you!", false)?;

    Ok(())
}

