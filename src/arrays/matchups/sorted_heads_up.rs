use crate::analysis::the_nuts::TheNuts;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use crate::util::wincounter::wins::Wins;
use crate::{PKError, Pile, SuitShift};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct SortedHeadsUp {
    higher: Two,
    lower: Two,
}

impl SortedHeadsUp {
    #[must_use]
    pub fn new(first: Two, second: Two) -> SortedHeadsUp {
        if first > second {
            SortedHeadsUp {
                higher: first,
                lower: second,
            }
        } else {
            SortedHeadsUp {
                higher: second,
                lower: first,
            }
        }
    }

    #[must_use]
    pub fn contains(&self, two: &Two) -> bool {
        &self.lower == two || &self.higher == two
    }

    #[must_use]
    pub fn higher(&self) -> Two {
        self.higher
    }

    #[must_use]
    pub fn higher_as_bard(&self) -> Bard {
        self.higher.bard()
    }

    #[must_use]
    pub fn lower(&self) -> Two {
        self.lower
    }

    #[must_use]
    pub fn lower_as_bard(&self) -> Bard {
        self.lower.bard()
    }

    /// This is going to be a heavy calculation.
    ///
    /// Oops!
    /// ```txt
    /// error[E0599]: the method `insert` exists for struct `HashSet<SortedHeadsUp>`, but its trait bounds were not satisfied
    ///   --> src/arrays/matchups/mod.rs:63:20
    ///    |
    /// 13 | pub struct SortedHeadsUp {
    ///    | ------------------------ doesn't satisfy `SortedHeadsUp: Hash`
    /// ...
    /// 63 |                 hs.insert(SortedHeadsUp::new(hero, villain));
    ///    |                    ^^^^^^
    ///    |
    ///    = note: the following trait bounds were not satisfied:
    ///            `SortedHeadsUp: Hash`
    /// help: consider annotating `SortedHeadsUp` with `#[derive(Hash)]`
    ///    |
    /// 13 + #[derive(Hash)]
    /// 14 | pub struct SortedHeadsUp {
    ///    |
    /// ```
    ///
    /// Man, I love the [Rust compiler](https://doc.rust-lang.org/std/collections/struct.HashSet.html#examples).
    /// Much love to everyone who worked on this thing.
    ///
    /// # Errors
    ///
    /// Shrugs.
    pub fn all_possible() -> Result<HashSet<SortedHeadsUp>, PKError> {
        let mut hs: HashSet<SortedHeadsUp> = HashSet::new();
        for v in Cards::deck().combinations(2) {
            let hero = Two::try_from(v.as_slice())?;
            for r in hero.remaining().combinations(2) {
                let villain = Two::try_from(r.as_slice())?;
                hs.insert(SortedHeadsUp::new(hero, villain));
            }
        }
        Ok(hs)
    }

    pub fn possible_sorts(&self) -> HashSet<Self> {
        let mut v = HashSet::new();
        let mut shifty = self.clone();
        v.insert(shifty);

        for i in 1..= 3 {
            shifty = shifty.shift_suit_up();
            v.insert(shifty);
        }

        v
    }

    /// For now, returning default is all part of the blueprint. Still, let's write a test
    /// that fails that we can ignore later on when we get everything wired in.
    #[must_use]
    pub fn wins(&self) -> Wins {
        Wins::default()
    }
}

impl Display for SortedHeadsUp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.higher, self.lower)
    }
}

impl Pile for SortedHeadsUp {
    /// Shoot. Forgot about my frequency mask idea. Still has potential, but later.
    fn clean(&self) -> Self {
        todo!()
    }

    /// Implementing this would be interesting. What's the best possible hand from either of these
    /// two hands?
    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    /// This is the only one we need to implement for what we want. Maybe this interface is doing
    /// too much.
    fn to_vec(&self) -> Vec<Card> {
        let mut v = self.higher.to_vec();
        v.extend(self.lower.to_vec());
        v
    }
}

impl SuitShift for SortedHeadsUp {
    /// I'm not convinced that this is going to work, but I want to try.
    fn shift_suit_down(&self) -> Self {
        SortedHeadsUp::new(self.higher.shift_suit_down(), self.lower.shift_suit_down())
    }

    fn shift_suit_up(&self) -> Self {
        SortedHeadsUp::new(self.higher.shift_suit_up(), self.lower.shift_suit_up())
    }

    /// I especially don't know about opposite.
    fn opposite(&self) -> Self {
        SortedHeadsUp::new(self.higher.opposite(), self.lower.opposite())
    }
}

impl TryFrom<Vec<Two>> for SortedHeadsUp {
    type Error = PKError;

    fn try_from(v: Vec<Two>) -> Result<Self, Self::Error> {
        match v.len() {
            0..=1 => Err(PKError::NotEnoughHands),
            2 => Ok(SortedHeadsUp::new(
                *v.get(0).ok_or(PKError::InvalidHand)?,
                *v.get(1).ok_or(PKError::InvalidHand)?,
            )),
            _ => Err(PKError::TooManyHands),
        }
    }
}

/// TODO do I need this? I'm overthinking and underknowing.
impl TryFrom<Vec<&Two>> for SortedHeadsUp {
    type Error = PKError;

    fn try_from(v: Vec<&Two>) -> Result<Self, Self::Error> {
        match v.len() {
            0..=1 => Err(PKError::NotEnoughHands),
            2 => Ok(SortedHeadsUp::new(
                **v.get(0).ok_or(PKError::InvalidHand)?,
                **v.get(1).ok_or(PKError::InvalidHand)?,
            )),
            _ => Err(PKError::TooManyHands),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__sorted_heads_up {
    use super::*;
    use crate::util::data::TestData;
    use crate::util::wincounter::win::Win;

    const EXPECTED: SortedHeadsUp = SortedHeadsUp {
        higher: Two::HAND_7D_7C,
        lower: Two::HAND_6S_6H,
    };

    #[test]
    fn new() {
        assert_eq!(
            EXPECTED,
            SortedHeadsUp::new(Two::HAND_6S_6H, Two::HAND_7D_7C)
        );
        assert_eq!(
            EXPECTED,
            SortedHeadsUp::new(Two::HAND_7D_7C, Two::HAND_6S_6H)
        );
    }

    #[test]
    fn contains() {
        assert!(EXPECTED.contains(&Two::HAND_6S_6H));
        assert!(EXPECTED.contains(&Two::HAND_7D_7C));
        assert!(!EXPECTED.contains(&Two::HAND_7S_7C));
    }

    /// 7♦ 7♣ - 6♠ 6♥
    /// 7♠ 7♣ - 6♥ 6♦
    /// 7♠ 7♥ - 6♦ 6♣
    /// 7♥ 7♦ - 6♠ 6♣
    #[test]
    fn possible_sorts() {
        let v = EXPECTED.possible_sorts();
        for shu in v.iter() {
            println!("{shu}");
        }
    }

    /// Wow, this test caused a panic:
    ///
    /// ```
    /// use pkcore::util::data::TestData;
    /// assert_eq!(TestData::the_hand_sorted_headsup().wins(), TestData::wins_the_hand());
    /// ```
    ///
    /// Let's try it a different way...
    ///
    /// ```
    /// use pkcore::util::data::TestData;
    /// use pkcore::util::wincounter::win::Win;
    /// assert_eq!(
    ///     TestData::the_hand_sorted_headsup().wins().wins_for(Win::FIRST),
    ///     TestData::wins_the_hand().wins_for(Win::FIRST)
    /// );
    /// ```
    ///
    /// Let's leave this test to fail for now, just so we don't forget it.
    #[test]
    fn wins() {
        assert_eq!(
            TestData::the_hand_sorted_headsup()
                .wins()
                .wins_for(Win::FIRST),
            TestData::wins_the_hand().wins_for(Win::FIRST)
        );
    }

    /// Here's the original test that panics, just for fun. I love it's error message:
    ///
    /// ```txt
    /// t": "thread 'arrays::matchups::sorted_heads_up::arrays__matchups__sorted_heads_up::wins_panic'
    /// panicked at 'assertion failed: `(left == right)`\n  left: `Wins([])`,\n right:
    /// `Wins([1, 1, 1, 1, 1, 1 ...
    /// ```
    ///
    /// for a few million entries. Run it if you want to see it. Let us ignore this test, shall we.
    /// See `docs/data/stacktrace.txt` for the full error.
    ///
    /// In hindsight, maybe deriving Eq, PartialEq on Wins wasn't such a good idea. Let's remove
    /// them, shall we...? Here';s the test for posterity's sake.
    ///
    /// ```txt
    /// #[test]
    /// #[ignore]
    /// fn wins_panic() {
    ///     assert_eq!(TestData::the_hand_sorted_headsup().wins(), TestData::wins_the_hand());
    /// }
    /// ```
    ///
    /// Moving on...
    #[test]
    fn display() {
        assert_eq!(
            "6♠ 6♥ - 5♦ 5♣",
            TestData::the_hand_sorted_headsup().to_string()
        );
    }

    #[test]
    fn pile__to_vec() {
        assert_eq!(
            EXPECTED.to_vec(),
            vec![
                Card::SEVEN_DIAMONDS,
                Card::SEVEN_CLUBS,
                Card::SIX_SPADES,
                Card::SIX_HEARTS
            ]
        );
    }

    /// I don't believe that I need this test. The foundations are already tested. Still, I like
    /// doing double checks. Part of me is just like how cool is it that I can even do this?!
    #[test]
    fn pile__remaining() {
        assert_eq!(EXPECTED.remaining().sort().to_string(), "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 6♣ 5♣ 4♣ 3♣ 2♣");
    }

    #[test]
    fn suit_shift() {
        let expected = SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D);
        assert_eq!(EXPECTED.shift_suit_down(), expected);
    }


    #[test]
    fn try_from() {
        let v = vec![Two::HAND_6S_6H, Two::HAND_7D_7C];

        assert_eq!(EXPECTED, SortedHeadsUp::try_from(v).unwrap());
    }

    #[test]
    fn try_from__error() {
        assert_eq!(
            PKError::NotEnoughHands,
            SortedHeadsUp::try_from(vec![Two::HAND_6S_6H]).unwrap_err()
        );
        assert_eq!(
            PKError::TooManyHands,
            SortedHeadsUp::try_from(vec![Two::HAND_6S_6H, Two::HAND_7S_7C, Two::HAND_2D_2C])
                .unwrap_err()
        );
    }
}
