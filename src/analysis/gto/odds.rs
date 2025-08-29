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
            "{}-{}-{} (W:{:.2}% L:{:.2}% D:{:.2}%)",
            self.wins,
            self.losses,
            self.draws,
            self.win_percentage(),
            self.loss_percentage(),
            self.draw_percentage()
        )
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
}
