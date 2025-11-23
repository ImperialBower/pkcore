use crate::PKError;
use crate::analysis::gto::combos::Combos;
use crate::arrays::hole_cards::twos::StartingHands;
use crate::cards::Cards;
use std::io::{Write, stdin, stdout};
use std::str::FromStr;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub struct Terminal;

impl Terminal {
    /// Cleans up the `Card` index string by replacing commas and dashes with spaces.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use pkcore::card::Card;
    /// use pkcore::cards::Cards;
    ///
    /// let expected = Cards::from(vec![Card::ACE_SPADES, Card::ACE_HEARTS,Card::ACE_DIAMONDS, Card::KING_DIAMONDS]);
    ///
    /// assert_eq!(expected, Cards::from_str("A♠ A♥ A♦ K♦").unwrap());
    /// assert_eq!(expected, Cards::from_str("A♠ A♥ - A♦ K♦").unwrap());
    /// assert_eq!(expected, Cards::from_str("A♠ A♥,A♦ K♦").unwrap());
    /// ```
    #[must_use]
    pub fn index_cleaner(index: &str) -> String {
        index.replace([',', '-'], " ")
    }

    /// # Errors
    ///
    /// If unable to read input.
    pub fn pause(prompt: &str) -> std::io::Result<()> {
        let mut stdout = stdout().into_raw_mode()?;
        write!(stdout, "{prompt}")?;
        stdout.flush()?;
        if let Some(key_res) = stdin().keys().next() {
            key_res?;
        }
        Ok(())
    }

    /// # Panics
    ///
    /// If it somehow wigs out on the input.
    #[must_use]
    pub fn receive_cards(prompt: &str) -> Option<Cards> {
        print!("{prompt}");
        let _ = stdout().flush();
        let mut input_text = String::new();
        stdin().read_line(&mut input_text).unwrap_or_default();

        Cards::from_str(input_text.as_str()).ok()
    }

    /// # Errors
    ///
    /// `PKError::InvalidIndex` if `str` doesn't translate into `Cards`
    /// `PKError::InvalidCardCount` if number of cards isn't divisible by two
    pub fn receive_cards_in_twos(prompt: &str) -> Result<StartingHands, PKError> {
        let Some(cards) = Terminal::receive_cards(prompt) else {
            return Err(PKError::InvalidCardIndex);
        };

        StartingHands::try_from(cards)
    }

    /// # Errors
    ///
    /// TODO
    pub fn receive_range(prompt: &str) -> Result<Combos, PKError> {
        print!("{prompt}");
        Combos::from_str(prompt)
    }

    /// Then goal of the functions in this module is to isolate and standardize the patterns we've been
    /// using in our example files.
    ///
    /// TODO use [RustyLine](https://github.com/kkawakam/rustyline)
    ///
    /// # Panics
    ///
    /// If it somehow wigs out on the input.
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn receive_usize(prompt: &str) -> usize {
        print!("{prompt}");
        let _ = stdout().flush();
        let mut input_text = String::new();
        stdin().read_line(&mut input_text).expect("Failed to receive value");
        let trimmed = input_text.trim();
        trimmed.parse::<usize>().unwrap_or_default()
    }

    /// # Errors
    ///
    /// `PKError::NotEnoughCards` if `Cards` is less than `x`.
    /// `PKError::TooManyCards` if `Cards` is greater than `x`.
    /// `PKError::InvalidIndex` if the string entered isn't a valid `Cards` index.
    pub fn receive_x_cards(prompt: &str, x: usize) -> Result<Cards, PKError> {
        if x < 1 {
            return Err(PKError::NotEnoughCards);
        }
        if x > 52 {
            return Err(PKError::TooManyCards);
        }

        let Some(cards) = Terminal::receive_cards(prompt) else {
            return Err(PKError::InvalidCardIndex);
        };

        if cards.len() < x {
            return Err(PKError::NotEnoughCards);
        }
        if cards.len() > x {
            return Err(PKError::TooManyCards);
        }
        Ok(cards)
    }
}
