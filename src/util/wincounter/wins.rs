use crate::util::Util;
use crate::util::wincounter::heads_up::HeadsUp;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::{PlayerFlag, Result};

/// I've moved wincounter into the library so that I can make updates to the library
/// as a part of this work. The plan is to later on move the updated module back to
/// its own crate.
///
/// When I originally wrote the crate I was just focused on heads up play.
#[derive(Clone, Debug, Default)]
pub struct Wins(Vec<PlayerFlag>);

impl Wins {
    pub fn add(&mut self, count: PlayerFlag) {
        self.0.push(count);
    }

    /// Adds a count x number of times. Primarily used for testing.
    pub fn add_x(&mut self, count: PlayerFlag, x: usize) {
        for _ in 0..x {
            self.0.push(count);
        }
    }

    pub fn add_win_first(&mut self) {
        self.0.push(Win::FIRST);
    }

    pub fn add_win_second(&mut self) {
        self.0.push(Win::SECOND);
    }

    pub fn add_win_third(&mut self) {
        self.0.push(Win::THIRD);
    }

    pub fn extend(&mut self, other: &Wins) {
        self.0.extend(other.get());
    }

    #[must_use]
    pub fn get(&self) -> &Vec<PlayerFlag> {
        &self.0
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the cumulative wins for a specific `PlayerFlag`, and their number
    /// of ties.
    #[must_use]
    pub fn wins_for(&self, result: PlayerFlag) -> (usize, usize) {
        let wins: Vec<PlayerFlag> = self.0.clone().into_iter().filter(|r| r.win_for(result)).collect();
        (wins.len(), wins.into_iter().filter(Result::is_tie).count())
    }

    /// Pass in a zero based player index and the function will return the
    /// win alone percentage of the player, as well as the number of times
    /// the player tied for first.
    #[must_use]
    pub fn percentage_for_player(&self, index: usize) -> (f32, f32) {
        let total = self.len();
        let (wins, ties) = self.wins_for(Win::from_index(index));
        let pure_wins = wins - ties;

        (
            Util::calculate_percentage(pure_wins, total),
            Util::calculate_percentage(ties, total),
        )
    }

    /// Forgiving percentage calculator. It will return zero if you try
    /// to divide by zero.
    /// ```txt
    /// #[must_use]
    /// #[allow(clippy::cast_precision_loss)]
    /// pub fn percent_calculator(number: usize, total: usize) -> f32 {
    ///     match total {
    ///         0 => 0_f32,
    ///         _ => ((number as f32 * 100.0) / total as f32) as f32,
    ///     }
    /// }
    /// ```
    ///
    #[must_use]
    pub fn results_heads_up(&self) -> HeadsUp {
        let (first, ties) = self.wins_for(Win::FIRST);
        let (second, _) = self.wins_for(Win::SECOND);
        HeadsUp::new(first - ties, second - ties, ties)
    }
}

impl From<Vec<PlayerFlag>> for Wins {
    fn from(counts: Vec<PlayerFlag>) -> Self {
        Wins(counts)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__wincounter__wins__tests {
    use super::*;

    #[test]
    fn extend() {
        let mut wins = Wins::default();
        let more_wins = Wins::from(vec![Win::FIRST, Win::FIRST, Win::SECOND]);
        let even_more_wins = Wins::from(vec![Win::FIRST, Win::SECOND, Win::SECOND]);

        wins.extend(&more_wins);
        wins.extend(&even_more_wins);

        assert!(!wins.is_empty());
        assert_eq!(more_wins.len() + even_more_wins.len(), wins.len());
    }

    #[test]
    fn get() {
        let v = vec![Win::FIRST, Win::FIRST, Win::SECOND, Win::FIRST];

        let wins = Wins::from(v.clone());

        assert_eq!(&v, wins.get())
    }

    #[test]
    fn add() {
        let mut counter = Wins::default();

        counter.add_win_first();
        counter.add_win_second();
        counter.add_win_first();
        counter.add_win_third();
        counter.add(Win::FIRST | Win::SECOND);
        counter.add(Win::FIFTH);

        assert_eq!(6, counter.len())
    }

    #[test]
    fn add_x() {
        let mut wins = Wins::default();

        wins.add_x(Win::FIRST, 1_365_284); // Daniel Wins
        wins.add_x(Win::SECOND, 314_904); // Gus Wins
        wins.add_x(Win::FIRST | Win::SECOND, 32116); // Ties

        // Since the result returned is a tuple containing the number of wins, including
        // ties, followed by just the ties, we need to add the two numbers together.
        assert_eq!((1_365_284 + 32116, 32116), wins.wins_for(Win::FIRST));
        assert_eq!((314_904 + 32116, 32116), wins.wins_for(Win::SECOND));
        assert_eq!((32116, 32116), wins.wins_for(Win::FIRST | Win::SECOND));
    }

    #[test]
    fn is_empty() {
        let mut counter = Wins::default();

        counter.add(Win::FIRST);

        assert!(!counter.is_empty());
        assert!(Wins::default().is_empty());
    }

    #[test]
    fn len() {
        let mut counter = Wins::default();

        counter.add(Win::FIRST);
        counter.add(Win::FIRST);
        counter.add(Win::FIRST);
        counter.add(Win::FIRST);

        assert_eq!(4, counter.len());
        assert_eq!(0, Wins::default().len());
    }

    #[test]
    fn wins_for() {
        let mut counter = Wins::default();

        counter.add_win_first();
        counter.add(Win::FIRST | Win::SECOND);
        counter.add_win_third();
        counter.add_win_third();
        counter.add_win_third();
        counter.add(Win::FORTH);

        assert_eq!((2, 1), counter.wins_for(Win::FIRST));
        assert_eq!((1, 1), counter.wins_for(Win::SECOND));
        assert_eq!((3, 0), counter.wins_for(Win::THIRD));
        assert_eq!((1, 0), counter.wins_for(Win::FORTH));
    }

    #[test]
    fn results_heads_up() {
        let mut counter = Wins::default();
        counter.add_x(Win::FIRST, 1_365_284);
        counter.add_x(Win::SECOND, 314_904);
        counter.add_x(Win::FIRST | Win::SECOND, 32_116);

        let hup = counter.results_heads_up();

        assert_eq!(79.73374, hup.percentage_first());
        assert_eq!(18.39066, hup.percentage_second());
        assert_eq!(1.8756015, hup.percentage_ties());
        assert_eq!(100.0, hup.percentage_total());
    }
}
