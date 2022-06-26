use crate::util::wincounter::win::Win;
use crate::util::wincounter::wins::Wins;

/// # PHASE 2.2/Step 4: Results
///
/// Results is a utility state class designed to make it as easy as possible to get and display
/// winning and tie percentages for any game.
#[derive(Clone, Debug, Default, PartialEq)]
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
    #[must_use]
    pub fn from_wins(wins: &Wins, player_count: usize) -> Results {
        let mut results = Results::default();
        for i in 0..player_count {
            let (total_wins, ties) = wins.wins_for(Win::from_index(i));
            results.v.push((total_wins - ties, ties));
        }
        results
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__wincounter__results__tests {
    use super::*;
    use crate::util::data::TestData;

    #[test]
    fn from_wins() {
        let results = Results::from_wins(&TestData::wins_the_hand(), 2);

        assert_eq!(&(1_365_284, 32116), results.v.get(0).unwrap());
        assert_eq!(&(314_904, 32116), results.v.get(1).unwrap());
    }
}
