#![warn(clippy::pedantic)]

use crate::util::wincounter::result::HeadsUp;

pub mod result;

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
