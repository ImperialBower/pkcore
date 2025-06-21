use crate::util::Util;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::wins::Wins;
use std::fmt::{Display, Formatter};

/// # PHASE 2.2/Step 4: Results
///
/// Results is a utility state class designed to make it as easy as possible to get and display
/// winning and tie percentages for any game.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Results {
    pub case_count: usize,
    pub player_count: usize,
    pub v: Vec<(usize, usize)>,
}

impl Results {
    /// It would be great if I could just figure out the number of players by what `Win` bit flag is
    /// set. The problem is that it would take too long to figure out. Some of these wins are going
    /// to contain hundreds of thousands of possibilities. It feels to me like it would be easier
    /// to just pass in the number of players when you instantiate the result. I already know that
    /// number. *Don't overthink things. Quit being so smart.*
    ///
    /// # Refactoring
    ///
    /// I'm feeling the need to update this struct so that it stores the total number of cases and
    /// players so that I don't need to keep computing them. Right now we have:
    ///
    /// ```
    /// pub struct Results(Vec<(usize, usize)>);
    /// ```
    ///
    /// What I'm thinking of changing it to is:
    ///
    /// ```
    /// pub struct Results {
    ///     pub case_count: usize,
    ///     pub player_count: usize,
    ///     pub v: Vec<(usize, usize)>,
    /// }
    /// ```
    ///
    /// One of the bad habits I've collected from my Java days is a phobia of public fields,
    /// instead, relying on assessors of the `.getCaseCount()` and `.setCaseCount()` variety. The
    /// problem is, that `Rust` by default doesn't have mutable state so you don't need to be
    /// battening down the hatches all the time.
    ///
    /// With this refactoring I can take the same constructor and have all the state I need to
    /// calculate winning percentages.
    ///
    /// ## Second test
    ///
    /// OK, so we have `The Hand` in our `TestData` util. The problem is that it's got only two
    /// players. I want a three way hand just to kick the tires a little harder. I'm going to add
    /// a second hand from `S09E13` of High Stakes Poker between Daniel Negreanu, Patrik Antonius,
    /// and Phil Ivey where Daniel folded the best hand at the river because of a wild all in
    /// from Patrik Antonius. Daniel did a great
    /// [breakdown of the hand](https://www.youtube.com/watch?v=SE3BP0KFqTA) on his channel.
    ///
    /// For now, I'm going to table this test. Let's get the code so that it works with The Hand
    /// before we get fancy.
    ///
    /// ## Clippy to the rescue!
    ///
    /// Initially, I had this function as:
    ///
    /// ```
    /// use pkcore::util::wincounter::results::Results;
    /// use pkcore::util::wincounter::wins::Wins;
    ///
    /// pub fn from_wins(wins: &Wins, player_count: usize) -> Results {
    ///     let mut results = Results::default();
    ///     results.case_count = wins.len();
    ///     results.player_count = player_count;
    ///     // ...
    ///     results
    /// }
    /// ```
    ///
    /// Clippy came back with this wonderful refactoring:
    ///
    /// ```
    /// use pkcore::util::wincounter::results::Results;
    /// use pkcore::util::wincounter::wins::Wins;
    /// pub fn from_wins(wins: &Wins, player_count: usize) -> Results {
    ///     let mut results = Results {
    ///         case_count: wins.len(),
    ///         player_count,
    ///         ..Default::default()
    ///     };
    ///     // ...
    ///     results
    /// }
    /// ```
    #[must_use]
    pub fn from_wins(wins: &Wins, player_count: usize) -> Results {
        let mut results = Self {
            case_count: wins.len(),
            player_count,
            ..Default::default()
        };

        for i in 0..player_count {
            let (total_wins, ties) = wins.wins_for(Win::from_index(i));
            results.v.push((total_wins - ties, ties));
        }

        results
    }

    /// This function is there to make it easy to create text based displays of a player's chances
    /// of winning at a particular point. It will be the foundation of the `Results` display trait
    /// implementation.
    ///
    /// REFACTORING: I refactored this so that it would be easier to compare my results
    /// to what I am getting from `Fudd`, which shows a simple one decimal place win percentage:
    /// (`97.7%`).
    #[must_use]
    pub fn player_to_string(&self, player_index: usize) -> String {
        let (wins, ties) = self.wins_and_ties(player_index);
        let (win_percentage, tie_percentage) = self.wins_and_ties_percentages(player_index);
        let percentage = win_percentage + tie_percentage;
        if percentage == 0.00 {
            "0.00%".to_string()
        } else {
            format!("{percentage:.1}% ({win_percentage:.2}%/{tie_percentage:.2}%) [{wins}/{ties}]")
        }
    }

    #[must_use]
    pub fn wins_and_ties(&self, player_index: usize) -> (usize, usize) {
        match self.v.get(player_index) {
            None => (0, 0),
            Some((wins, ties)) => (*wins, *ties),
        }
    }

    #[must_use]
    pub fn wins_and_ties_percentages(&self, player_index: usize) -> (f32, f32) {
        let (wins, ties) = self.wins_and_ties(player_index);
        (
            Util::calculate_percentage(wins, self.case_count),
            Util::calculate_percentage(ties, self.case_count),
        )
    }

    #[must_use]
    pub fn wins_total(&self, player_index: usize) -> usize {
        let (wins, ties) = self.wins_and_ties(player_index);
        wins + ties
    }

    #[must_use]
    pub fn wins_total_percentage(&self, player_index: usize) -> f32 {
        let (wins, ties) = self.wins_and_ties(player_index);
        Util::calculate_percentage(wins + ties, self.case_count)
    }
}

/// Right now I am irritated that it ends with a new line, but I don't really want to deal with it
/// tight now. I've done this before in other languages, but I don't honestly remember how. I am
/// 56 years old. The brain cells are dying fast. R.I.P. 🪦
///
/// TODO TD: Trim final new line.
impl Display for Results {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.v.len() {
            writeln!(f, "Player #{} {}", i + 1, self.player_to_string(i))?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__wincounter__results__tests {
    use super::*;
    use crate::util::data::TestData;

    #[test]
    fn from_wins() {
        let results = Results::from_wins(&TestData::the_hand_as_wins(), 2);

        assert_eq!(&(1_365_284, 32_116), results.v.get(0).unwrap());
        assert_eq!(&(314_904, 32_116), results.v.get(1).unwrap());
    }

    #[test]
    fn player_to_string() {
        let results = Results::from_wins(&TestData::the_hand_as_wins(), 2);

        assert_eq!("81.6% (79.73%/1.88%) [1365284/32116]", results.player_to_string(0));
        assert_eq!("20.3% (18.39%/1.88%) [314904/32116]", results.player_to_string(1));
        assert_eq!("0.00%", results.player_to_string(2));
    }

    #[test]
    fn wins_and_ties() {
        let results = Results::from_wins(&TestData::the_hand_as_wins(), 2);

        assert_eq!((1_365_284, 32_116), results.wins_and_ties(0));
        assert_eq!((314_904, 32_116), results.wins_and_ties(1));
        assert_eq!((0, 0), results.wins_and_ties(2));
        assert_eq!((0, 0), results.wins_and_ties(3));
    }

    #[test]
    fn wins_and_ties_percentages() {
        let results = Results::from_wins(&TestData::the_hand_as_wins(), 2);

        assert_eq!((79.73374, 1.8756015), results.wins_and_ties_percentages(0));
        assert_eq!((18.39066, 1.8756015), results.wins_and_ties_percentages(1));
        assert_eq!((0.0, 0.0), results.wins_and_ties_percentages(2));
        assert_eq!((0.0, 0.0), results.wins_and_ties_percentages(3));
    }

    #[test]
    fn wins_total() {
        let results = Results::from_wins(&TestData::the_hand_as_wins(), 2);

        assert_eq!(1_397_400, results.wins_total(0));
        assert_eq!(347_020, results.wins_total(1));
        assert_eq!(0, results.wins_total(2));
        assert_eq!(0, results.wins_total(3));
    }

    #[test]
    fn wins_total_percentage() {
        let results = Results::from_wins(&TestData::the_hand_as_wins(), 2);

        assert_eq!(81.60934, results.wins_total_percentage(0));
        assert_eq!(20.266262, results.wins_total_percentage(1));
        assert_eq!(0.0, results.wins_total_percentage(2));
        assert_eq!(0.0, results.wins_total_percentage(3));
    }

    /// I like to organize my tests to match the order they fall in the source. I generally
    /// structure them as:
    ///
    /// * [Associated function "constructors"](https://rust-unofficial.github.io/patterns/idioms/ctor.html)
    /// * functions on self
    /// * static functions
    /// * trait implementations
    #[test]
    fn display() {
        let results = Results::from_wins(&TestData::the_hand_as_wins(), 2);
        assert_eq!(
            "Player #1 81.6% (79.73%/1.88%) [1365284/32116]\nPlayer #2 20.3% (18.39%/1.88%) [314904/32116]\n",
            results.to_string()
        );
    }
}

// Running `target/debug/examples/calc -d '5♠ 5♦ 9♠ 9♥ K♣ T♦' -b '5♣ 9♦ T♥ T♣ Q♦' -e -n`
// Seat #0 5♠ 5♦: 18.1% Seat #1 9♠ 9♥: 44.8% Seat #2 K♣ T♦: 37.6%
// Cards Dealt: 5♠ 5♦ 9♠ 9♥ K♣ T♦ 5♣ 9♦ T♥ T♣ Q♦
//
// [Seat 0: 5♠ 5♦, Seat 1: 9♠ 9♥, Seat 2: K♣ T♦]
// [FLOP:  5♣ 9♦ T♥, TURN:  T♣, RIVER: Q♦]
//
// The Flop: 5♣ 9♦ T♥
// Chances of winning:
// Seat #0 5♠ 5♦: 4.5% - CURRENT HAND: 5♠ 5♦ 5♣ T♥ 9♦ HandRank { value: 2242, name: ThreeOfAKind, class: ThreeFives }
// Seat #1 9♠ 9♥: 92.6% - CURRENT HAND: 9♠ 9♥ 9♦ T♥ 5♣ HandRank { value: 1981, name: ThreeOfAKind, class: ThreeNines }
// Seat #2 K♣ T♦: 2.9% - CURRENT HAND: T♥ T♦ K♣ 9♦ 5♣ HandRank { value: 4281, name: Pair, class: PairOfTens }
//
// The Nuts would be: T♠ T♥ T♦ 9♦ 5♣ HandRank { value: 1915, name: ThreeOfAKind, class: ThreeTens }
//
// Possible hands at the flop, sorted by strength:
// CKC #1915 T♠ T♥ T♦ 9♦ 5♣ HandRank { value: 1915, name: ThreeOfAKind, class: ThreeTens }
// CKC #1981 9♠ 9♥ 9♦ T♥ 5♣ HandRank { value: 1981, name: ThreeOfAKind, class: ThreeNines }
// CKC #2242 5♠ 5♥ 5♣ T♥ 9♦ HandRank { value: 2242, name: ThreeOfAKind, class: ThreeFives }
// CKC #2937 T♠ T♥ 9♠ 9♦ 5♣ HandRank { value: 2937, name: TwoPair, class: TensAndNines }
// CKC #2978 T♠ T♥ 5♠ 5♣ 9♦ HandRank { value: 2978, name: TwoPair, class: TensAndFives }
// CKC #3055 9♠ 9♦ 5♠ 5♣ T♥ HandRank { value: 3055, name: TwoPair, class: NinesAndFives }
// CKC #3465 A♠ A♥ T♥ 9♦ 5♣ HandRank { value: 3465, name: Pair, class: PairOfAces }
// CKC #3685 K♠ K♥ T♥ 9♦ 5♣ HandRank { value: 3685, name: Pair, class: PairOfKings }
// CKC #3905 Q♠ Q♥ T♥ 9♦ 5♣ HandRank { value: 3905, name: Pair, class: PairOfQueens }
// CKC #4125 J♠ J♥ T♥ 9♦ 5♣ HandRank { value: 4125, name: Pair, class: PairOfJacks }
// CKC #4236 T♠ T♥ A♠ 9♦ 5♣ HandRank { value: 4236, name: Pair, class: PairOfTens }
// CKC #4456 9♠ 9♦ A♠ T♥ 5♣ HandRank { value: 4456, name: Pair, class: PairOfNines }
// CKC #4812 8♠ 8♥ T♥ 9♦ 5♣ HandRank { value: 4812, name: Pair, class: PairOfEights }
// CKC #5032 7♠ 7♥ T♥ 9♦ 5♣ HandRank { value: 5032, name: Pair, class: PairOfSevens }
// CKC #5252 6♠ 6♥ T♥ 9♦ 5♣ HandRank { value: 5252, name: Pair, class: PairOfSixes }
// CKC #5333 5♠ 5♣ A♠ T♥ 9♦ HandRank { value: 5333, name: Pair, class: PairOfFives }
// CKC #5693 4♠ 4♥ T♥ 9♦ 5♣ HandRank { value: 5693, name: Pair, class: PairOfFours }
// CKC #5913 3♠ 3♥ T♥ 9♦ 5♣ HandRank { value: 5913, name: Pair, class: PairOfTreys }
// CKC #6133 2♠ 2♥ T♥ 9♦ 5♣ HandRank { value: 6133, name: Pair, class: PairOfDeuces }
// CKC #6269 A♠ K♠ T♥ 9♦ 5♣ HandRank { value: 6269, name: HighCard, class: AceHigh }
// CKC #6717 K♠ Q♠ T♥ 9♦ 5♣ HandRank { value: 6717, name: HighCard, class: KingHigh }
// CKC #7010 Q♠ J♠ T♥ 9♦ 5♣ HandRank { value: 7010, name: HighCard, class: QueenHigh }
// CKC #7218 J♠ T♥ 9♦ 8♠ 5♣ HandRank { value: 7218, name: HighCard, class: JackHigh }
// CKC #7342 T♥ 9♦ 8♠ 7♠ 5♣ HandRank { value: 7342, name: HighCard, class: TenHigh }
// See https://suffe.cool/poker/7462.html for a listing of all CKC numbers.
//
// The Turn: T♣
// Chances of winning:
// Seat 0: 2.4% - Outs: 5♥
// Seat 1: 88.1%
// Seat 2: 9.5% - Outs: K♠ T♠ K♥ K♦
//
// The River: Q♦
// Seat 0: 0.0%
// Seat 1: 100.0%
// Seat 2: 0.0%
//
// Winners:
//    Seat 1: 9♠ 9♥ 9♦ T♥ T♣ HandRank { value: 231, name: FullHouse, class: NinesOverTens }
// Time taken performing calc: 1294.483106271s
//
// Command:
// ❯ cargo run --example calc -- -d "5♠ 5♦ 9♠ 9♥ K♣ T♦" -b "5♣ 9♦ T♥ T♣ Q♦"
