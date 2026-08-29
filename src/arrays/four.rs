use crate::analysis::eval::Eval;
use crate::arrays::HandRanker;
use crate::arrays::seven::Seven;
use crate::arrays::six::Six;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::cards::Cards;
use crate::play::board::Board;
use crate::util::Util;
use crate::{Card, PKError, Pile, Plurable, TheNuts, Unumable};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// This is a convenience struct for Game. I'm not writing many tests *WHAT???* for it because I don't
/// feel it is necessary right now. Later on, who knows, but for now that's OK.
///
/// I mainly want this struct for the `From<Vec<Card>>` trait, which is there to make things
/// easier for me with the analysis code.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Four(pub(crate) [Card; 4]);

impl Four {
    pub const OMAHA_PERMUTATIONS: [[usize; 2]; 6] = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];

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

    /// There's a serious flaw in this logic. Omaha requires that you use exactly two of the cards
    /// from the four in your hand, unlike NLHE where you can play the board. This method evaluates
    /// two hole cards plus the whole board as a best-5-of-7, so it will happily play the board and
    /// return a hand no Omaha player is allowed to make.
    ///
    /// Use [`crate::games::omaha::OmahaHigh::eval`] instead. It enumerates the 60 legal
    /// 2-from-hand + 3-from-board combinations. Note that it carried this same flaw until
    /// `DEFECT_017` fixed it in 0.5.4 — before that, the two methods were the same wrong logic in
    /// two places, and this comment pointed at the other copy.
    ///
    /// This is kept for historical reasons and is deprecated; it has no callers in the crate.
    #[must_use]
    #[deprecated]
    pub fn omaha_high(&self, board: &Board) -> Eval {
        let mut best_eval = Eval::default();

        for perm in &Self::OMAHA_PERMUTATIONS {
            let two = Two::from([self.0[perm[0]], self.0[perm[1]]]);
            let seven = Seven::from_case_and_board(&two, board);

            let eval = seven.eval();
            if eval > best_eval {
                best_eval = eval;
            }
        }

        best_eval
    }

    #[must_use]
    pub fn two_from_permutation(&self, permutation: &[usize; 2]) -> Two {
        Two::from([self.0[permutation[0]], self.0[permutation[1]]])
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
    /// Constructs a `Four` from a fixed array, sorting cards high-to-low.
    ///
    /// The sort normalizes the representation so that two `Four`s containing
    /// the same cards compare equal regardless of the order they were passed in.
    /// This matters for Omaha hole cards, where `[A♠ K♠ Q♠ J♠]` and
    /// `[J♠ Q♠ K♠ A♠]` are the same hand.
    ///
    /// Use [`Four::from_turn`] when constructing a board representation where
    /// insertion order is meaningful — that constructor bypasses this sort.
    fn from(array: [Card; 4]) -> Self {
        let mut array = array;
        array.sort();
        array.reverse();
        Four(array)
    }
}

impl From<Vec<Card>> for Four {
    fn from(mut v: Vec<Card>) -> Self {
        v.sort();
        v.reverse();
        match v.len() {
            4 => {
                let four = Four([v[0], v[1], v[2], v[3]]);
                if four.is_dealt() { four } else { Four::default() }
            }
            _ => Four::default(),
        }
    }
}

impl FromStr for Four {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Four::try_from(Cards::from_str(s)?)
    }
}

impl Plurable for Four {
    fn from_pluribus(s: &str) -> Result<Self, PKError> {
        let s = s.trim();
        match s.len() {
            0..=7 => Err(PKError::NotEnoughCards),
            8 => Self::from_str(Util::str_len_splitter(s, 2).as_str()),
            _ => Err(PKError::TooManyCards),
        }
    }
}

impl Unumable for Four {
    /// `Qs7s5c3h` — four cards back to back.
    ///
    /// Note that [`From<[Card; 4]>`](Four::from) sorts high-to-low, so a
    /// `Four` renders in its normalized order, not the order it was parsed
    /// from. Use [`Four::from_turn`] when insertion order matters.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert_eq!(Four::from_pluribus("Qs7s5c3h").unwrap().to_pluribus(), "Qs7s5c3h");
    /// // Parsing normalizes: the input order is not preserved.
    /// assert_eq!(Four::from_pluribus("3h7s5cQs").unwrap().to_pluribus(), "Qs7s5c3h");
    /// ```
    fn to_pluribus(&self) -> String {
        self.0.iter().map(Card::to_pluribus).collect()
    }
}
impl Pile for Four {
    fn add<P: Pile>(&self, _other: P) -> Self
    where
        Self: Sized,
    {
        unimplemented!("Four cannot be added; it's a fixed 4-card hand")
    }

    impl_pile_uniqueness_checks!();

    fn card_at(self, _index: usize) -> Option<Card> {
        unimplemented!("Four is a fixed 4-card hand; use `.cards().card_at(index)` for positional access")
    }

    fn clean(&self) -> Self {
        Four([
            self.first().clean(),
            self.second().clean(),
            self.third().clean(),
            self.forth().clean(),
        ])
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        unimplemented!("Four is a fixed 4-card hand; use `.cards()` for a swappable set")
    }

    fn the_nuts(&self) -> TheNuts {
        if !self.is_dealt() {
            return TheNuts::default();
        }

        let mut the_nuts = TheNuts::default();

        for v in self.remaining().combinations(2) {
            let hole = Two::from(v);
            let six = Six::from([
                hole.first(),
                hole.second(),
                self.first(),
                self.second(),
                self.third(),
                self.forth(),
            ]);
            the_nuts.push(six.eval());
        }
        the_nuts.sort_in_place();

        the_nuts
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

impl TryFrom<Cards> for Four {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=3 => Err(PKError::NotEnoughCards),
            4 => Ok(Four::from([
                *cards.get_index(0).ok_or(PKError::InvalidCard)?,
                *cards.get_index(1).ok_or(PKError::InvalidCard)?,
                *cards.get_index(2).ok_or(PKError::InvalidCard)?,
                *cards.get_index(3).ok_or(PKError::InvalidCard)?,
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__four_tests {
    use super::*;

    /// See `docs/perf/PROFILING.md`: the `Pile::is_dealt` default allocates
    /// twice via `to_vec`, which dominated hand evaluation.
    #[test]
    fn is_dealt_does_not_allocate() {
        let hand = Four([
            Card::ACE_DIAMONDS,
            Card::ACE_CLUBS,
            Card::KING_DIAMONDS,
            Card::KING_CLUBS,
        ]);
        assert!(hand.is_dealt());

        let (dealt, allocations) = crate::alloc_probe::count_allocs(|| hand.is_dealt());

        assert!(dealt);
        assert_eq!(allocations, 0, "Four::is_dealt made {allocations} heap allocation(s)");
    }

    #[test]
    fn from_twos() {
        let first = Two::from([Card::KING_CLUBS, Card::KING_DIAMONDS]);
        let second = Two::from([Card::ACE_CLUBS, Card::ACE_DIAMONDS]);
        let expected = Four([
            Card::ACE_DIAMONDS,
            Card::ACE_CLUBS,
            Card::KING_DIAMONDS,
            Card::KING_CLUBS,
        ]);

        let actual = Four::from_twos(first, second);

        assert_eq!(expected, actual);
    }
    // Test for flawed method
    // #[test]
    // fn omaha_high() {
    //     let four = Four::from([
    //         Card::ACE_DIAMONDS,
    //         Card::ACE_CLUBS,
    //         Card::KING_DIAMONDS,
    //         Card::KING_CLUBS,
    //     ]);
    //     let board = Board::from([
    //         Card::QUEEN_DIAMONDS,
    //         Card::QUEEN_HEARTS,
    //         Card::JACK_DIAMONDS,
    //         Card::TEN_CLUBS,
    //         Card::TEN_DIAMONDS,
    //     ]);
    //     let expected = Class::RoyalFlush;
    //
    //     let actual = four.omaha_high(&board).hand_rank.class;
    //
    //     assert_eq!(expected, actual);
    // }

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
    fn from_str() {
        let cards = "AS QS QD JC";
        let expected = Four([
            Card::ACE_SPADES,
            Card::QUEEN_SPADES,
            Card::QUEEN_DIAMONDS,
            Card::JACK_CLUBS,
        ]);

        let actual = Four::from_str(cards).unwrap();

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

    #[test]
    fn try_from__cards() {
        let cards = Cards::from_str("AS QS QD JC").unwrap();
        let expected = Four([
            Card::ACE_SPADES,
            Card::QUEEN_SPADES,
            Card::QUEEN_DIAMONDS,
            Card::JACK_CLUBS,
        ]);

        let actual = Four::try_from(cards).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn try_from__cards__error() {
        assert_eq!(PKError::NotEnoughCards, Four::try_from(Cards::default()).unwrap_err());
        assert_eq!(
            PKError::NotEnoughCards,
            Four::try_from(Cards::from_str("AS").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::NotEnoughCards,
            Four::try_from(Cards::from_str("AS KS").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::NotEnoughCards,
            Four::try_from(Cards::from_str("AS KS QC").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::TooManyCards,
            Four::try_from(Cards::from_str("AS KS QC JC TC").unwrap()).unwrap_err()
        );
    }

    #[test]
    fn from_pluribus() {
        let expected = Four::from_str("AS QS QD JC").unwrap();
        assert_eq!(expected, Four::from_pluribus("AsQsQdJc").unwrap());
        assert_eq!(expected, Four::from_pluribus(" AsQsQdJc").unwrap());
        assert_eq!(expected, Four::from_pluribus("AsQsQdJc ").unwrap());
        assert_eq!(PKError::NotEnoughCards, Four::from_pluribus("AsQsQd").unwrap_err());
        assert_eq!(PKError::TooManyCards, Four::from_pluribus("AsQsQdJcTc").unwrap_err());
    }

    #[test]
    fn pile__the_nuts__blank() {
        let four = Four::from([Card::BLANK, Card::SIX_DIAMONDS, Card::FIVE_HEARTS, Card::FOUR_CLUBS]);
        assert_eq!(TheNuts::default(), four.the_nuts());
    }

    #[test]
    fn pile__the_nuts__turn_board() {
        let four = Four::from([
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_HEARTS,
            Card::DEUCE_SPADES,
        ]);
        let the_nuts = four.the_nuts();
        // 31 distinct HandRankClass values achievable on this turn board
        assert_eq!(31, the_nuts.len());
    }

    #[test]
    fn four_renders_four_cards_back_to_back() {
        assert_eq!(Four::from_pluribus("Qs7s5c3h").unwrap().to_pluribus(), "Qs7s5c3h");
    }

    #[test]
    fn four_normalizes_high_to_low() {
        // `From<[Card; 4]>` sorts, so the render is canonical, not logged.
        assert_eq!(Four::from_pluribus("3h7s5cQs").unwrap().to_pluribus(), "Qs7s5c3h");
    }
}
