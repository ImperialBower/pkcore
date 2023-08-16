use crate::cards::Cards;
use crate::PKError;
use std::io;
use std::io::Write;
use std::str::FromStr;

/// Then goal of the functions in this module is to isolate and standardize the patterns we've been
/// using in our example files.
///
/// TODO use [RustyLine](https://github.com/kkawakam/rustyline)
///
/// # Panics
///
/// If it somehow wigs out on the input.
#[must_use]
pub fn receive_usize(prompt: &str) -> usize {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut input_text = String::new();
    io::stdin()
        .read_line(&mut input_text)
        .expect("Failed to receive value");
    let trimmed = input_text.trim();
    match trimmed.parse::<usize>() {
        Ok(i) => i,
        Err(..) => 0,
    }
}

/// # Panics
///
/// If it somehow wigs out on the input.
#[must_use]
pub fn receive_cards(prompt: &str) -> Option<Cards> {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut input_text = String::new();
    io::stdin()
        .read_line(&mut input_text)
        .expect("Failed to receive value");

    match Cards::from_str(input_text.as_str()) {
        Ok(cards) => Some(cards),
        Err(_) => None,
    }
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

    let Some(cards) = receive_cards(prompt) else {
        return Err(PKError::InvalidIndex);
    };

    if cards.len() < x {
        return Err(PKError::NotEnoughCards);
    }
    if cards.len() > x {
        return Err(PKError::TooManyCards);
    }
    Ok(cards)
}
