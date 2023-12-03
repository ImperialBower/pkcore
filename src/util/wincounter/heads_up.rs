use crate::util::Util;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct HeadsUp {
    pub first_wins: usize,
    pub second_wins: usize,
    pub ties: usize,
}

impl HeadsUp {
    #[must_use]
    pub fn new(first_wins: usize, second_wins: usize, ties: usize) -> Self {
        Self {
            first_wins,
            second_wins,
            ties,
        }
    }

    #[must_use]
    pub fn percentage_first(&self) -> f32 {
        Util::calculate_percentage(self.first_wins, self.total())
    }

    #[must_use]
    pub fn percentage_first_cumulative(&self) -> f32 {
        Util::calculate_percentage(self.first_wins + self.ties, self.total())
    }

    #[must_use]
    pub fn percentage_second(&self) -> f32 {
        Util::calculate_percentage(self.second_wins, self.total())
    }

    #[must_use]
    pub fn percentage_second_cumulative(&self) -> f32 {
        Util::calculate_percentage(self.second_wins + self.ties, self.total())
    }

    #[must_use]
    pub fn percentage_ties(&self) -> f32 {
        Util::calculate_percentage(self.ties, self.total())
    }

    #[must_use]
    pub fn percentage_total(&self) -> f32 {
        let x = self.percentage_first() + self.percentage_second() + self.percentage_ties();
        (x * 100.0).round() / 100.0
    }

    #[must_use]
    pub fn total(&self) -> usize {
        self.first_wins + self.second_wins + self.ties
    }
}

impl fmt::Display for HeadsUp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.2}% ({}), {:.2}% ({}), {:.2}% ({})",
            self.percentage_first(),
            self.first_wins,
            self.percentage_second(),
            self.second_wins,
            self.percentage_ties(),
            self.ties
        )
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__wincounter__heads_up__tests {
    use super::*;

    // "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠" HSP THE hand Negreanu/Hansen
    // First:  78.3% (1365284) || 79.71% (1364802) // stats from https://tools.timodenk.com/poker-odds-pre-flop
    // Second: 18.1% (314904)  || 18.39% (314904)
    // Ties:    3.7% (32116)   ||  1.90% (32598)
    fn the_hand() -> HeadsUp {
        HeadsUp::new(1365284, 314904, 32116)
    }

    fn simple() -> HeadsUp {
        HeadsUp::new(40, 40, 20)
    }

    #[test]
    fn percentage_first() {
        assert_eq!(79.73374, the_hand().percentage_first());
        assert_eq!(40.0, simple().percentage_first());
    }

    #[test]
    fn percentage_first_cumulative() {
        assert_eq!(81.60934, the_hand().percentage_first_cumulative());
        assert_eq!(60.0, simple().percentage_first_cumulative());
    }

    #[test]
    fn percentage_second() {
        assert_eq!(18.39066, the_hand().percentage_second());
        assert_eq!(40.0, simple().percentage_second());
    }

    #[test]
    fn percentage_second_cumulative() {
        assert_eq!(20.266262, the_hand().percentage_second_cumulative());
        assert_eq!(60.0, simple().percentage_second_cumulative());
    }

    #[test]
    fn percentage_ties() {
        assert_eq!(1.8756015, the_hand().percentage_ties());
        assert_eq!(20.0, simple().percentage_ties());
    }

    #[test]
    fn percentage_total() {
        assert_eq!(100.0, the_hand().percentage_total());
        assert_eq!(100.0, simple().percentage_total());
    }

    #[test]
    fn total() {
        let hup = HeadsUp::new(1365284, 314904, 32116);

        assert_eq!(1_712_304, hup.total())
    }

    #[test]
    fn remote_test() {
        // stats from https://tools.timodenk.com/poker-odds-pre-flop
        let hup = HeadsUp::new(1364802, 314904, 32598);

        assert_eq!(1_712_304, hup.total());
        assert_eq!(79.70558, hup.percentage_first());
        assert_eq!(18.39066, hup.percentage_second());
        assert_eq!(1.9037508, hup.percentage_ties());
        assert_eq!(100.0, hup.percentage_total());
    }

    #[test]
    fn display() {
        assert_eq!(
            "79.73% (1365284), 18.39% (314904), 1.88% (32116)",
            the_hand().to_string()
        );
    }
}
