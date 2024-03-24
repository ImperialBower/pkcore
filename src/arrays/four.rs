use crate::analysis::eval::Eval;
use crate::analysis::hand_rank::{HandRank, HandRankValue, NO_HAND_RANK_VALUE};
use crate::arrays::five::Five;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::arrays::HandRanker;
use crate::{Card, Pile, TheNuts};
use std::fmt;
use std::fmt::{Display, Formatter};

/// This is a convenience struct for Game. I'm not writing many tests *WHAT???* for it because I don't
/// feel it is necessary right now. Later on, who knows, but for now that's OK.
///
/// I mainly want this struct for the `From<Vec<Card>>` trait, which is there to make things
/// easier for me with the analysis code.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Four([Card; 4]);

impl Four {
    pub const OMAHA_PERMUTATIONS: [[u8; 2]; 6] = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];

    #[must_use]
    pub fn from_twos(first: Two, second: Two) -> Self {
        Four::from([first.first(), first.second(), second.first(), second.second()])
    }

    #[must_use]
    pub fn from_turn(flop: Three, turn: Card) -> Four {
        Four([flop.first(), flop.second(), flop.third(), turn])
    }

    //region accessors
    #[must_use]
    pub fn first(&self) -> Card {
        self.0[0]
    }

    #[must_use]
    pub fn second(&self) -> Card {
        self.0[1]
    }

    #[must_use]
    pub fn third(&self) -> Card {
        self.0[2]
    }

    #[must_use]
    pub fn forth(&self) -> Card {
        self.0[3]
    }

    #[must_use]
    pub fn to_arr(&self) -> [Card; 4] {
        self.0
    }
    //endregion

    #[must_use]
    pub fn omaha_high(&self, board: Five) -> Eval {
        let mut best_hrv: HandRankValue = NO_HAND_RANK_VALUE;
        let mut best_hand = Five::default();

        todo!();

        // for perm in Self::OMAHA_PERMUTATIONS.iter() {
        //     let mut hand = Five::default();
        //     hand.0[0] = self.0[perm[0] as usize];
        //     hand.0[1] = self.0[perm[1] as usize];
        //     hand.0[2] = _board.first();
        //     hand.0[3] = _board.second();
        //     hand.0[4] = _board.third();
        //
        //     let eval = hand.eval();
        //     if eval.hand_rank.value < best_hrv {
        //         best_hrv = eval.hand_rank.value;
        //         best_hand = hand;
        //     }
        // }
        //
        // Eval::new(HandRank::from(best_hrv), best_hand)
    }
}

impl Display for Four {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.first(),
            self.second(),
            self.third(),
            self.forth()
        )
    }
}

impl From<[Card; 4]> for Four {
    fn from(array: [Card; 4]) -> Self {
        let mut array = array;
        array.sort();
        array.reverse();
        Four(array)
    }
}

impl From<Vec<Card>> for Four {
    /// I do want to test this baby, since it's the main reason we are here.
    fn from(v: Vec<Card>) -> Self {
        let mut v = v.clone();
        v.sort();
        v.reverse();
        match v.len() {
            4 => {
                let one = match v.first() {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let two = match v.get(1) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let three = match v.get(2) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let four = match v.get(3) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let four = Four([one, two, three, four]);
                if four.is_dealt() {
                    four
                } else {
                    Four::default()
                }
            }
            _ => Four::default(),
        }
    }
}

impl Pile for Four {
    fn clean(&self) -> Self {
        Four([
            self.first().clean(),
            self.second().clean(),
            self.third().clean(),
            self.forth().clean(),
        ])
    }

    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__four_tests {
    use super::*;

    #[test]
    fn from__array() {
        let cards = [
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_HEARTS,
            Card::FIVE_SPADES,
        ];
        let expected = Four([
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_SPADES,
            Card::FIVE_HEARTS,
        ]);

        let actual = Four::from(cards);

        assert_eq!(expected, actual);
    }

    #[test]
    fn from__vec() {
        let cards = vec![
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_HEARTS,
            Card::FIVE_SPADES,
        ];
        let expected = Four([
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_SPADES,
            Card::FIVE_HEARTS,
        ]);

        let actual = Four::from(cards);

        assert_eq!(expected, actual);
    }
}
