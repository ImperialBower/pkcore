use crate::analysis::eval::Eval;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::cards::Cards;
use crate::cards_cell::CardsCell;
use crate::util::Util;
use crate::{PKError, Pile, Plurable, SOK, TheNuts, Unumable};
use std::fmt::{Display, Formatter};
use std::ops::Index;
use std::str::FromStr;

/// A `Board` is a type that represents a single instance of the face up `Cards`
/// of one `Game` of `Texas hold 'em`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Board {
    pub flop: Three,
    pub turn: Card,
    pub river: Card,
}

impl Board {
    #[must_use]
    pub fn new(flop: Three, turn: Card, river: Card) -> Self {
        Board { flop, turn, river }
    }

    #[must_use]
    pub fn turn_cards(&self) -> Cards {
        let mut cards = self.flop.to_vec();
        if self.turn.is_dealt() {
            cards.push(self.turn);
        }
        Cards::from(cards)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FLOP: {}, TURN: {}, RIVER: {}", self.flop, self.turn, self.river)
    }
}

impl From<Five> for Board {
    fn from(value: Five) -> Self {
        Board::new(
            Three::from([value.first(), value.second(), value.third()]),
            value.forth(),
            value.fifth(),
        )
    }
}

impl From<[Card; 5]> for Board {
    fn from(value: [Card; 5]) -> Self {
        Board::from(Five::from(value))
    }
}

impl FromStr for Board {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Board::try_from(Cards::from_str(s)?)
    }
}

impl Plurable for Board {
    /// The Pluribus format for a board is `3h7s5c/Qs/6c`.
    fn from_pluribus(s: &str) -> Result<Self, PKError>
    where
        Self: Sized,
    {
        if s.is_empty() {
            return Ok(Board::default());
        }
        let v = Util::str_splitter(s, "/");

        match v.len() {
            1 => Ok(Board::new(
                Three::from_str(Util::str_len_splitter(v.index(0), 2).as_str())?,
                Card::BLANK,
                Card::BLANK,
            )),
            2 => Ok(Board::new(
                Three::from_str(Util::str_len_splitter(v.index(0), 2).as_str())?,
                Card::from_str(v.index(1))?,
                Card::BLANK,
            )),
            3 => Ok(Board::new(
                Three::from_str(Util::str_len_splitter(v.index(0), 2).as_str())?,
                Card::from_str(v.index(1))?,
                Card::from_str(v.index(2))?,
            )),
            _ => Err(PKError::InvalidPluribusIndex),
        }
    }
}

impl Unumable for Board {
    /// `3h7s5c/Qs/6c`, `3h7s5c/Qs`, `3h7s5c`, or `""` for a hand that never
    /// saw a flop.
    ///
    /// The inverse of [`Board::from_pluribus`], which pads the streets a hand
    /// never reached with [`Card::BLANK`]. This truncates at the first blank
    /// instead of padding, so a blank suit (`_`) can never reach a log line,
    /// and a hand that ended pre-flop renders as the empty string rather than
    /// a stray `/`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert_eq!(Board::from_pluribus("3h7s5c/Qs/6c").unwrap().to_pluribus(), "3h7s5c/Qs/6c");
    /// assert_eq!(Board::from_pluribus("3h7s5c").unwrap().to_pluribus(), "3h7s5c");
    /// assert_eq!(Board::from_pluribus("").unwrap().to_pluribus(), "");
    /// ```
    fn to_pluribus(&self) -> String {
        if !self.flop.is_dealt() {
            return String::new();
        }

        let mut rendered = self.flop.to_pluribus();

        if !self.turn.is_dealt() {
            return rendered;
        }
        rendered.push('/');
        rendered.push_str(&self.turn.to_pluribus());

        if !self.river.is_dealt() {
            return rendered;
        }
        rendered.push('/');
        rendered.push_str(&self.river.to_pluribus());

        rendered
    }
}

impl Pile for Board {
    fn add<P: Pile>(&self, _other: P) -> Self
    where
        Self: Sized,
    {
        unimplemented!("Board cannot be added; it's a fixed 5-card hand")
    }

    fn card_at(self, _index: usize) -> Option<Card> {
        unimplemented!("Board is a fixed 5-card hand; use `.cards().card_at(index)` for positional access")
    }

    fn clean(&self) -> Self {
        Board {
            flop: self.flop.clean(),
            turn: self.turn.clean(),
            river: self.river.clean(),
        }
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        unimplemented!("Board is a fixed 5-card hand; use `.cards()` for a swappable set")
    }

    /// The best seven-card hand every possible two-card holding makes with
    /// this board, strongest first — the classic "what is the nuts here?"
    /// question. Empty for an incomplete board.
    fn the_nuts(&self) -> TheNuts {
        if !self.is_dealt() {
            return TheNuts::default();
        }

        let mut the_nuts = TheNuts::default();

        for v in self.remaining().combinations(2) {
            let hole = Two::from(v);
            the_nuts.push(Eval::from(Seven::from_case_and_board(&hole, self)));
        }
        the_nuts.sort_in_place();
        the_nuts
    }

    fn to_vec(&self) -> Vec<Card> {
        let mut v: Vec<Card> = Vec::default();
        v.append(&mut self.flop.clone().to_vec());
        v.push(self.turn);
        v.push(self.river);
        v
    }
}

impl SOK for Board {
    fn salright(&self) -> bool {
        self != &Board::default()
    }
}

impl TryFrom<CardsCell> for Board {
    type Error = PKError;

    fn try_from(cards_cell: CardsCell) -> Result<Self, Self::Error> {
        Board::try_from(cards_cell.cards())
    }
}

impl TryFrom<Cards> for Board {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        // TODO RF? Clunky
        match cards.len() {
            0..=2 => Err(PKError::NotEnoughCards),
            3 => Ok(Board {
                flop: Three::try_from(cards)?,
                turn: Card::default(),
                river: Card::default(),
            }),
            4 => {
                let mut cards = cards;
                Ok(Board {
                    flop: Three::try_from(cards.draw(3)?)?,
                    turn: cards.draw_one()?,
                    river: Card::default(),
                })
            }
            5 => {
                let mut cards = cards;
                Ok(Board {
                    flop: Three::try_from(cards.draw(3)?)?,
                    turn: cards.draw_one()?,
                    river: cards.draw_one()?,
                })
            }
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play_board_tests {
    use super::*;
    use crate::Forgiving;

    #[test]
    fn turn_cards() {
        let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap_or_default();

        let turn_cards = board.turn_cards();

        assert_eq!("9♣ 6♦ 5♥ 5♠", turn_cards.to_string());
    }

    #[test]
    fn display() {
        assert_eq!("FLOP: __ __ __, TURN: __, RIVER: __", Board::default().to_string());
    }

    #[test]
    fn from_str() {
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠",
            Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap().to_string()
        )
    }

    #[test]
    fn from_pluribus() {
        assert_eq!(
            Board::from_str("3h 7s 5c Qs 6c").unwrap(),
            Board::from_pluribus("3h7s5c/Qs/6c").unwrap()
        );
        assert_eq!(
            Board::from_str("3h 7s 5c Qs").unwrap(),
            Board::from_pluribus("3h7s5c/Qs").unwrap()
        );
        assert_eq!(
            Board::from_str("3h 7s 5c").unwrap(),
            Board::from_pluribus("3h7s5c").unwrap()
        );
        assert_eq!(
            PKError::InvalidPluribusIndex,
            Board::from_pluribus("/3h7s5c/Qs/6c").unwrap_err()
        );
        assert_eq!(
            PKError::InvalidPluribusIndex,
            Board::from_pluribus("3h7s5c/Qs/6c/2d").unwrap_err()
        );
        assert_eq!(
            PKError::InvalidCardIndex,
            Board::from_pluribus("3h7s55/Qs/6c").unwrap_err()
        );
        assert_eq!(
            PKError::InvalidCardIndex,
            Board::from_pluribus("3h7s5c/QQ/6c").unwrap_err()
        );
        assert_eq!(
            PKError::InvalidCardIndex,
            Board::from_pluribus("3h7s5c/Qs/6A").unwrap_err()
        );
    }

    #[test]
    fn try_from() {
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: __, RIVER: __",
            Board::try_from(Cards::from(vec![
                Card::NINE_CLUBS,
                Card::SIX_DIAMONDS,
                Card::FIVE_HEARTS
            ]))
            .unwrap()
            .to_string()
        );
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: __",
            Board::try_from(Cards::from(vec![
                Card::NINE_CLUBS,
                Card::SIX_DIAMONDS,
                Card::FIVE_HEARTS,
                Card::FIVE_SPADES,
            ]))
            .unwrap()
            .to_string()
        );
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠",
            Board::try_from(Cards::from(vec![
                Card::NINE_CLUBS,
                Card::SIX_DIAMONDS,
                Card::FIVE_HEARTS,
                Card::FIVE_SPADES,
                Card::EIGHT_SPADES,
            ]))
            .unwrap()
            .to_string()
        );
        assert_eq!(
            "FLOP: A♠ K♥ Q♣, TURN: J♦, RIVER: T♣",
            Board::try_from(cc!("AS KH QC JD TC")).unwrap().to_string()
        );
    }

    #[test]
    fn try_from__cards__not_enough() {
        assert_eq!(
            PKError::NotEnoughCards,
            Board::try_from(Cards::from_str("AS KS").unwrap()).unwrap_err()
        );
    }

    #[test]
    fn try_from__cards__too_many() {
        assert_eq!(
            PKError::TooManyCards,
            Board::try_from(Cards::from_str("AS KS QS JS TS 9S").unwrap()).unwrap_err()
        );
    }

    #[test]
    fn pile__clean__strips_frequency_bits() {
        let board = Board::from_str("A♠ K♠ Q♠ J♠ 9♠").unwrap();
        let flagged = Board {
            flop: board.flop,
            turn: board.turn.frequency_paired(),
            river: board.river.frequency_tripped(),
        };

        assert_ne!(board, flagged);
        assert_eq!(board, flagged.clean());
    }

    #[test]
    fn pile__the_nuts__is_the_best_seven_card_hand_over_every_holding() {
        let nuts = Board::from_str("A♠ K♠ Q♠ J♠ 9♠").unwrap().the_nuts();

        assert!(!nuts.is_empty());
        let best = nuts.get(0).unwrap();
        // Cactus Kev rank 1 is the royal flush; it needs the T♠ in hand.
        assert_eq!(1, best.hand_rank.value);
        assert!(best.hand.contains(&Card::TEN_SPADES));
    }

    #[test]
    fn board_renders_every_street_it_reached() {
        assert_eq!(
            Board::from_pluribus("3h7s5c/Qs/6c").unwrap().to_pluribus(),
            "3h7s5c/Qs/6c"
        );
    }

    #[test]
    fn board_omits_unreached_streets() {
        // Flop-only renders with no trailing `/`; an empty board renders as
        // the empty string, not as a stray divider. 4,662 of the 10,000
        // corpus hands end pre-flop, so the empty case is the plurality.
        assert_eq!(Board::from_pluribus("3h7s5c/Qs").unwrap().to_pluribus(), "3h7s5c/Qs");
        assert_eq!(Board::from_pluribus("3h7s5c").unwrap().to_pluribus(), "3h7s5c");
        assert_eq!(Board::from_pluribus("").unwrap().to_pluribus(), "");
        assert_eq!(Board::default().to_pluribus(), "");
    }

    #[test]
    fn board_never_renders_blank_suit() {
        // `Suit::BLANK.to_char_letter()` is `_`, and it must never reach a
        // log line — `from_pluribus` pads absent streets with `Card::BLANK`,
        // and the writer has to truncate rather than render them.
        for board in ["", "3h7s5c", "3h7s5c/Qs", "3h7s5c/Qs/6c"] {
            let rendered = Board::from_pluribus(board).unwrap().to_pluribus();
            assert!(!rendered.contains('_'), "{board} rendered as {rendered}");
        }
    }

    #[test]
    fn board_round_trips_the_flop_in_dealt_order() {
        // `Three` does not sort, so an out-of-order flop survives exactly.
        assert_eq!(Board::from_pluribus("5c7s3h").unwrap().to_pluribus(), "5c7s3h");
    }
}
