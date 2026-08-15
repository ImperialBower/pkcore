//! Canonical card type and deck ordering.
//!
//! With the `pkcore` feature we use the real `pkcore::card::Card` and
//! `pkcore::deck::DECK_ARRAY`. Without it (default), a minimal local stub with
//! the same 52-card ordering as pkcore (Spades, Hearts, Diamonds, Clubs;
//! A K Q J T 9 8 7 6 5 4 3 2) is used.

#[cfg(feature = "pkcore")]
pub use pkcore::card::Card;
#[cfg(feature = "pkcore")]
pub use pkcore::deck::DECK_ARRAY;

#[cfg(not(feature = "pkcore"))]
pub use local::{Card, DECK_ARRAY};

#[cfg(not(feature = "pkcore"))]
mod local {
    use std::fmt;

    /// Stand-in for `pkcore::card::Card`: a canonical deck index `0..52`.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct Card(pub u8);

    const RANKS: [char; 13] = ['A', 'K', 'Q', 'J', 'T', '9', '8', '7', '6', '5', '4', '3', '2'];
    const SUITS: [char; 4] = ['s', 'h', 'd', 'c'];

    impl fmt::Display for Card {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let i = self.0 as usize;
            write!(f, "{}{}", RANKS[i % 13], SUITS[i / 13])
        }
    }

    pub const DECK_ARRAY: [Card; 52] = {
        let mut a = [Card(0); 52];
        let mut i = 0;
        while i < 52 {
            a[i] = Card(i as u8);
            i += 1;
        }
        a
    };
}
