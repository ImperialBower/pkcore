use crate::play::stages::flop_eval::FlopEval;
use crate::util::Percentage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct WinLoseDraw {
    pub wins: u64,
    pub losses: u64,
    pub draws: u64,
}

impl WinLoseDraw {
    #[must_use]
    pub fn total(&self) -> u64 {
        self.wins + self.losses + self.draws
    }

    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn win_percentage(&self) -> f32 {
        Percentage::new(self.wins as usize, self.total() as usize).calculate()
    }

    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn loss_percentage(&self) -> f32 {
        Percentage::new(self.losses as usize, self.total() as usize).calculate()
    }

    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn draw_percentage(&self) -> f32 {
        Percentage::new(self.draws as usize, self.total() as usize).calculate()
    }
}

impl std::ops::Add for WinLoseDraw {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            wins: self.wins + other.wins,
            losses: self.losses + other.losses,
            draws: self.draws + other.draws,
        }
    }
}

impl std::fmt::Display for WinLoseDraw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.2}% ({}), {:.2}% ({}), {:.2}% ({})",
            self.win_percentage(),
            self.wins,
            self.loss_percentage(),
            self.losses,
            self.draw_percentage(),
            self.draws,
        )
    }
}

impl From<FlopEval> for WinLoseDraw {
    fn from(fe: FlopEval) -> Self {
        let (wins, draws) = fe.results.wins_and_ties(0);
        let (loses, _) = fe.results.wins_and_ties(1);
        WinLoseDraw {
            wins: wins as u64,
            losses: loses as u64,
            draws: draws as u64,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__gto__odds_tests {
    use super::*;

    #[test]
    fn add() {
        let a = WinLoseDraw {
            wins: 1,
            losses: 2,
            draws: 3,
        };
        let b = WinLoseDraw {
            wins: 4,
            losses: 5,
            draws: 6,
        };
        let c = a + b;
        assert_eq!(
            c,
            WinLoseDraw {
                wins: 5,
                losses: 7,
                draws: 9
            }
        );
    }

    #[test]
    fn display() {
        let a = WinLoseDraw {
            wins: 1,
            losses: 2,
            draws: 3,
        };
        assert_eq!(format!("{}", a), "16.67% (1), 33.33% (2), 50.00% (3)");
    }
}
