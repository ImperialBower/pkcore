#![warn(clippy::pedantic)]

use crate::util::wincounter::result::HeadsUp;

pub mod result;

/// I've moved wincounter into the library so that I can make updates to the library
/// as a part of this work. The plan is to later on move the updated module back to
/// its own crate.
///
/// When I originally wrote the crate I was just focused on heads up play.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wins(Vec<Count>);

impl Wins {
    pub fn add_win(&mut self, count: Count) {
        self.0.push(count);
    }

    /// Adds a count x number of times. Primarily used for testing.
    pub fn add_x(&mut self, count: Count, x: usize) {
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
    pub fn get(&self) -> &Vec<Count> {
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

    #[must_use]
    pub fn wins_for(&self, result: Count) -> (usize, usize) {
        let wins: Vec<Count> = self
            .0
            .clone()
            .into_iter()
            .filter(|r| r.win_for(result))
            .collect();
        (wins.len(), wins.into_iter().filter(Result::is_tie).count())
    }

    /// Forgiving percentage calculator. It will return zero if you try
    /// to divide by zero.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn percent(number: usize, total: usize) -> f32 {
        match total {
            0 => 0_f32,
            _ => ((number as f32 * 100.0) / total as f32) as f32,
        }
    }

    #[must_use]
    pub fn results_heads_up(&self) -> HeadsUp {
        let (first, ties) = self.wins_for(Win::FIRST);
        let (second, _) = self.wins_for(Win::SECOND);
        HeadsUp::new(first - ties, second - ties, ties)
    }
}

impl From<Vec<Count>> for Wins {
    fn from(counts: Vec<Count>) -> Self {
        Wins(counts)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests__wins {
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
    fn add_win() {
        let mut counter = Wins::default();

        counter.add_win_first();
        counter.add_win_second();
        counter.add_win_first();
        counter.add_win_third();
        counter.add_win(Win::FIRST | Win::SECOND);
        counter.add_win(Win::FIFTH);

        assert_eq!(6, counter.len())
    }

    #[test]
    fn is_empty() {
        let mut counter = Wins::default();

        counter.add_win(Win::FIRST);

        assert!(!counter.is_empty());
        assert!(Wins::default().is_empty());
    }

    #[test]
    fn len() {
        let mut counter = Wins::default();

        counter.add_win(Win::FIRST);
        counter.add_win(Win::FIRST);
        counter.add_win(Win::FIRST);
        counter.add_win(Win::FIRST);

        assert_eq!(4, counter.len());
        assert_eq!(0, Wins::default().len());
    }

    #[test]
    fn wins_for() {
        let mut counter = Wins::default();

        counter.add_win_first();
        counter.add_win(Win::FIRST | Win::SECOND);
        counter.add_win_third();
        counter.add_win_third();
        counter.add_win_third();
        counter.add_win(Win::FORTH);

        assert_eq!((2, 1), counter.wins_for(Win::FIRST));
        assert_eq!((1, 1), counter.wins_for(Win::SECOND));
        assert_eq!((3, 0), counter.wins_for(Win::THIRD));
        assert_eq!((1, 0), counter.wins_for(Win::FORTH));
    }

    #[test]
    fn percent() {
        let percentage = Wins::percent(48, 2_598_960);

        assert_eq!("0.00185%", format!("{:.5}%", percentage));
    }

    #[test]
    fn percent__zero_numerator() {
        let percentage = Wins::percent(0, 2_598_960);

        assert_eq!("0.00000%", format!("{:.5}%", percentage));
    }

    #[test]
    fn percent__zero_denominator() {
        let percentage = Wins::percent(48, 0);

        assert_eq!("0.00000%", format!("{:.5}%", percentage));
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

// TODO RF: Refactor this as a `struct Count(u16)`.
pub type Count = u16;

pub trait Result {
    #[must_use]
    fn is_tie(&self) -> bool;

    #[must_use]
    fn win_for(&self, count: Count) -> bool;
}

impl Result for Count {
    fn is_tie(&self) -> bool {
        self.count_ones() > 1
    }

    fn win_for(&self, count: Count) -> bool {
        self & count == count
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests__result {
    use super::*;

    #[test]
    fn is_tie() {
        let r = Win::FIRST | Win::SECOND;

        assert_eq!(2, r.count_ones());
        assert!(r.is_tie());
    }

    #[test]
    fn win_for() {
        let tie = Win::FIRST | Win::THIRD;

        assert!(Win::FIRST.win_for(Win::FIRST));
        assert!(tie.win_for(Win::FIRST));
        assert!(tie.win_for(Win::THIRD));
        assert!(!tie.win_for(Win::SECOND));
        assert!(!Win::FIRST.win_for(Win::SECOND));
        assert!(!Win::FIRST.win_for(Win::THIRD));
    }
}

#[derive(Debug)]
pub struct Win;

impl Win {
    pub const FIRST: Count = 0b0000_0001;
    pub const SECOND: Count = 0b0000_0010;
    pub const THIRD: Count = 0b0000_0100;
    pub const FORTH: Count = 0b0000_1000;
    pub const FIFTH: Count = 0b0001_0000;
    pub const SIXTH: Count = 0b0010_0000;
    pub const SEVENTH: Count = 0b0100_0000;
    pub const EIGHT: Count = 0b1000_0000;
    pub const NINTH: Count = 0b1_0000_0000;
    pub const TENTH: Count = 0b10_0000_0000;
    pub const ELEVENTH: Count = 0b100_0000_0000;
    pub const TWELFTH: Count = 0b1000_0000_0000;
    pub const THIRTEENTH: Count = 0b1_0000_0000_0000;
    pub const FOURTEENTH: Count = 0b10_0000_0000_0000;
    pub const FIFTEENTH: Count = 0b100_0000_0000_0000;
    pub const SIXTEENTH: Count = 0b1000_0000_0000_0000;

    /// `CaseEval` win count Test #2: TAKE TWO detour.
    ///
    /// Our heroic system has been sidelined. Our heroes need a way to translate a zero based
    /// index position of a vector into a bit flag representation of that index. So, in other words,
    /// the index for the first position in a vector is 0:
    ///
    /// ```
    /// let v: Vec<usize> = vec![10, 9, 8, 7, 6];
    ///
    /// assert_eq!(*v.get(0).unwrap(), 10);
    /// ```
    /// [rust playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=6cf71e83aa0d16dba23fd310d07efc3c)
    ///
    /// So we need a function that will return the bit flag for the specific position in the vector:
    /// `0` returns `0b0000_0001`... `1` returns `0b0000_0010`, etc...
    ///
    /// Now I suppose the smart way to write this function would be to create some logic that will
    /// convert the index into a bit flag. The problem is, that I don't want to think about that
    /// right now, and, as the founder of the dumb coder movement, I am going to write this function
    /// in the stupidest way I can think of, proud of the fact that later on, someone smarter than
    /// me will offer a cooler, more awesomer way of coding this, and will send me a pull request
    /// with their solution just to prove how much smarter they are than me.
    ///
    /// ASIDE: _When you think about it, how much of the shit we do boils down to basic primate
    /// behavior? Lord knows that's true with software developers. If only there was a way to
    /// harness this dick measuring energy into getting them to write this book for me 🤔_
    ///
    /// Me, I'm going to code a good ol' fashioned match statement:
    ///
    /// ```
    /// use pkcore::util::wincounter::{Count, Win};
    /// fn from_index(i: usize) -> Count {
    ///     match i {
    ///         0 => Win::FIRST,
    ///         1 => Win::SECOND,
    ///         2 => Win::THIRD,
    ///         _ => Count::default()
    ///     }
    /// }
    /// assert_eq!(Win::FIRST, from_index(0));
    /// assert_eq!(Win::SECOND, from_index(1));
    /// ```
    ///
    /// For now, the contract of `wincounter` is to support up to 16 players. The idea behind
    /// this library was to have an easy way to deal winning percentages for situations where
    /// more than only person in a game could win. Granted, the maximum number of players in a
    /// single deck poker game is generally less than that. I've heard numbers of
    /// [9, 10, 11,](https://poker.stackexchange.com/questions/4413/what-is-the-maximum-number-of-players-in-texas-holdem)
    /// and even [22](https://www.betfirm.com/max-number-of-players-in-texas-hold-em/). Me, I'm
    /// doing 16 so I don't have to think about it for a while.
    ///
    /// You will have to forgive me if I don't test drive through this function too much. I've got
    /// one failing `todo!()` test, and I'm going to just implement the match as seems write,
    /// write tests to verify all of the boundary conditions, and call it a day.
    ///
    /// So what are the boundary conditions?
    ///
    /// * Positive: unsigned integer between 0 and 15.
    /// * Negative: unsigned integer greater than 15.
    ///
    /// I'll be honest with you. This wincounter library isn't wowing me. My desire to avoid
    /// the hassle of wrapping an u16 `Count` in a struct is making me bend over backwards to
    /// deal with the fact that I can't write methods against `Count` because it isn't a struct
    /// or an enum. I feel a major refactoring coming on for this code. I'll hold off for now,
    /// but it's in my backlog.
    ///
    /// Adding a technical debt not to my code as a reminder.
    ///
    /// *NOTE:* This isn't the first time I've had to do this sort of refactoring. The initial
    /// version of the `Card` struct was a
    /// [simple type alias](https://github.com/ContractBridge/ckc-rs/blob/5f301f182eb579c9f8df4e243b6ebecd310b1b24/src/lib.rs#L33).
    /// For the instance in this book, I decided to write it as a struct to make the code cleaner
    /// and easier to manage. It's only a matter of time before I do the same thing to
    /// `wincounter::Count`. Not doing things right in order to save you some time will always
    /// end up taking more time in the long run. Count on it.
    ///
    #[must_use]
    pub fn from_index(i: usize) -> Count {
        match i {
            0 => Win::FIRST,
            1 => Win::SECOND,
            2 => Win::THIRD,
            3 => Win::FORTH,
            4 => Win::FIFTH,
            5 => Win::SIXTH,
            6 => Win::SEVENTH,
            7 => Win::EIGHT,
            8 => Win::NINTH,
            9 => Win::TENTH,
            10 => Win::ELEVENTH,
            11 => Win::TWELFTH,
            12 => Win::THIRTEENTH,
            13 => Win::FOURTEENTH,
            14 => Win::FIFTEENTH,
            15 => Win::SIXTEENTH,
            _ => Count::default(),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__wincounter__win_tests {
    use super::*;

    #[test]
    fn from_index() {
        assert_eq!(Win::FIRST, Win::from_index(0));
        assert_eq!(Win::SECOND, Win::from_index(1));
        assert_eq!(Win::THIRD, Win::from_index(2));
        assert_eq!(Win::FORTH, Win::from_index(3));
        assert_eq!(Win::FIFTH, Win::from_index(4));
        assert_eq!(Win::SIXTH, Win::from_index(5));
        assert_eq!(Win::SEVENTH, Win::from_index(6));
        assert_eq!(Win::EIGHT, Win::from_index(7));
        assert_eq!(Win::NINTH, Win::from_index(8));
        assert_eq!(Win::TENTH, Win::from_index(9));
        assert_eq!(Win::ELEVENTH, Win::from_index(10));
        assert_eq!(Win::TWELFTH, Win::from_index(11));
        assert_eq!(Win::THIRTEENTH, Win::from_index(12));
        assert_eq!(Win::FOURTEENTH, Win::from_index(13));
        assert_eq!(Win::FIFTEENTH, Win::from_index(14));
        assert_eq!(Win::SIXTEENTH, Win::from_index(15));
        assert_eq!(Count::default(), Win::from_index(16));
    }
}

//region utility functions

/// Forgiving percentage calculator. It will return zero if you try
/// to divide by zero.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn calculate_percentage(number: usize, total: usize) -> f32 {
    match total {
        0 => 0_f32,
        _ => ((number as f32 * 100.0) / total as f32) as f32,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests__utilities {
    use super::*;

    #[test]
    fn percent() {
        let percentage = calculate_percentage(48, 2_598_960);

        assert_eq!("0.00185%", format!("{:.5}%", percentage));
        assert_eq!("0.00000%", format!("{:.5}%", calculate_percentage(0, 0)));
    }

    #[test]
    fn percent__zero_numerator() {
        let percentage = calculate_percentage(0, 2_598_960);

        assert_eq!("0.00000%", format!("{:.5}%", percentage));
    }

    #[test]
    fn percent__zero_denominator() {
        let percentage = calculate_percentage(48, 0);

        assert_eq!("0.00000%", format!("{:.5}%", percentage));
    }
}

//endregion
