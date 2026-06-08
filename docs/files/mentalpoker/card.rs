//! Minimal card model for trick-taking. On a real build these map to
//! `cardpack`'s `Rank`/`Suit`/`Card`; kept local here so the engine builds
//! dependency-free. Trick-taking wants **Ace high**, so `Rank` is ordered
//! Two < .. < King < Ace via its discriminant.

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Suit {
    Clubs = 0,
    Diamonds = 1,
    Hearts = 2,
    Spades = 3,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Rank {
    Two = 0,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub const fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven, Rank::Eight,
        Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
    ];
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let r = match self.rank {
            Rank::Two => "2", Rank::Three => "3", Rank::Four => "4", Rank::Five => "5",
            Rank::Six => "6", Rank::Seven => "7", Rank::Eight => "8", Rank::Nine => "9",
            Rank::Ten => "T", Rank::Jack => "J", Rank::Queen => "Q", Rank::King => "K",
            Rank::Ace => "A",
        };
        let s = match self.suit {
            Suit::Clubs => "c", Suit::Diamonds => "d", Suit::Hearts => "h", Suit::Spades => "s",
        };
        write!(f, "{r}{s}")
    }
}
