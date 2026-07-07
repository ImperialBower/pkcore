use crate::prelude::{Eval, SeatEquity};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub struct Winnings(Vec<PotWin>);

impl Winnings {
    #[must_use]
    pub fn first(&self) -> PotWin {
        let binding = PotWin::default();
        let f = self.0.first().unwrap_or(&binding);
        *f
    }

    #[must_use]
    pub fn second(&self) -> PotWin {
        let binding = PotWin::default();
        let f = self.0.get(1).unwrap_or(&binding);
        *f
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
    pub fn vec(&self) -> &Vec<PotWin> {
        &self.0
    }
}

impl Display for Winnings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Winnings({})",
            self.0.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
        )
    }
}

impl From<Vec<PotWin>> for Winnings {
    fn from(winnings: Vec<PotWin>) -> Self {
        let mut winnings = winnings;
        winnings.sort();
        Winnings(winnings)
    }
}

impl From<PotWin> for Winnings {
    fn from(win: PotWin) -> Self {
        Winnings(vec![win])
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub struct PotWin {
    pub equity: SeatEquity,
    pub eval: Eval,
}

impl Display for PotWin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Compose using the component Display implementations so the output is
        // consistent with other types in the crate.
        write!(f, "Winnings(equity={}, eval={})", self.equity, self.eval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::eval::Eval;
    use crate::arrays::five::Five;
    use crate::casino::equity::seat_equity::SeatEquity;
    use crate::casino::equity::seatbit::Seatbit;
    use std::str::FromStr;

    #[test]
    fn win_display_contains_equity_and_eval() {
        let equity = SeatEquity::new(150, Seatbit::SEAT_0 | Seatbit::SEAT_1);
        let five = Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap();
        let eval = Eval::from(five);

        let w = PotWin { equity, eval };

        let s = w.to_string();
        assert!(s.contains("Winnings("));
        assert!(s.contains("chips=150"));
        assert!(s.contains("Royal") || s.contains("HandRank") || s.contains("A"));
    }
}
