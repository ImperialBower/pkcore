/// Helper module for Text-to-Speech in poker examples
///
/// This module provides convenient functions for announcing poker-related
/// events using text-to-speech.
use pkcore::prelude::*;
use tts::Tts;

/// A wrapper around the TTS engine with poker-specific announcement methods
pub struct PokerNarrator {
    tts: Tts,
    enabled: bool,
}

impl PokerNarrator {
    /// Create a new poker narrator
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            tts: Tts::default()?,
            enabled: true,
        })
    }

    /// Create a silent narrator (for testing or when TTS is unavailable)
    pub fn silent() -> Self {
        // This will fail but we'll catch it
        match Tts::default() {
            Ok(tts) => Self { tts, enabled: false },
            Err(_) => Self {
                tts: Tts::default().unwrap(),  // This is just a dummy
                enabled: false
            }
        }
    }

    /// Enable or disable speech
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Speak text if enabled
    pub fn speak(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.enabled {
            self.tts.speak(text, false)?;
        }
        Ok(())
    }

    /// Announce game state
    pub fn announce_state(&mut self, state: &GameState) -> Result<(), Box<dyn std::error::Error>> {
        let msg = format!(
            "Phase: {}. Button on seat {}. Action on seat {}. Pot: {} chips.",
            state.phase,
            state.button_position,
            state.next_to_act,
            state.pot_size
        );
        self.speak(&msg)
    }

    /// Announce a player action
    pub fn announce_action(&mut self, seat: u8, action: &str, amount: Option<usize>) -> Result<(), Box<dyn std::error::Error>> {
        let msg = if let Some(amt) = amount {
            format!("Seat {} {}s {} chips", seat, action, amt)
        } else {
            format!("Seat {} {}s", seat, action)
        };
        self.speak(&msg)
    }

    /// Announce blind posting
    pub fn announce_blinds(&mut self, small: usize, big: usize) -> Result<(), Box<dyn std::error::Error>> {
        let msg = format!(
            "Posting blinds. Small blind {} chips. Big blind {} chips.",
            small, big
        );
        self.speak(&msg)
    }

    /// Announce board cards
    pub fn announce_board(&mut self, phase: &str, cards: &[Bard]) -> Result<(), Box<dyn std::error::Error>> {
        let card_text = cards.iter()
            .map(|c| self.card_to_speech(c))
            .collect::<Vec<_>>()
            .join(", ");

        let msg = format!("Dealing the {}. {}", phase, card_text);
        self.speak(&msg)
    }

    /// Announce pot size
    pub fn announce_pot(&mut self, amount: usize) -> Result<(), Box<dyn std::error::Error>> {
        let msg = format!("The pot is now {} chips", amount);
        self.speak(&msg)
    }

    /// Announce winner
    pub fn announce_winner(&mut self, seat: u8, amount: usize, hand: &str) -> Result<(), Box<dyn std::error::Error>> {
        let msg = format!("Seat {} wins {} chips with {}", seat, amount, hand);
        self.speak(&msg)
    }

    /// Announce start of hand
    pub fn announce_new_hand(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.speak("New hand. Shuffling and dealing.")
    }

    /// Convert a card to speech-friendly text
    fn card_to_speech(&self, card: &Bard) -> String {
        // Convert card symbols to speakable text
        // This is a simplified version - you may want to expand this
        let s = card.to_string();
        s.replace('♠', " of spades")
            .replace('♥', " of hearts")
            .replace('♦', " of diamonds")
            .replace('♣', " of clubs")
            .replace('A', "Ace")
            .replace('K', "King")
            .replace('Q', "Queen")
            .replace('J', "Jack")
            .replace('T', "Ten")
    }

    /// Set speech rate (0.0 - 10.0, platform dependent)
    pub fn set_rate(&mut self, rate: f32) -> Result<(), Box<dyn std::error::Error>> {
        self.tts.set_rate(rate)?;
        Ok(())
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&mut self, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
        self.tts.set_volume(volume)?;
        Ok(())
    }
}

/// Example usage:
///
/// ```no_run
/// use poker_narrator::PokerNarrator;
/// use pkcore::prelude::*;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut narrator = PokerNarrator::new()?;
///     let table = Table::default();
///
///     narrator.announce_new_hand()?;
///     narrator.announce_blinds(50, 100)?;
///
///     let state = table.get_game_state();
///     narrator.announce_state(&state)?;
///
///     Ok(())
/// }
/// ```

