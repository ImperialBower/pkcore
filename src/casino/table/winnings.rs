use crate::prelude::{Eval, SeatEquity};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Winnings {
    pub equity: SeatEquity,
    pub eval: Eval,
}

impl Display for Winnings {
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
    use crate::casino::table::seats::seat_equity::SeatEquity;
    use crate::casino::table::seats::seatbit::Seatbit;
    use std::str::FromStr;

    #[test]
    fn winnings_display_contains_equity_and_eval() {
        let equity = SeatEquity::new(150, Seatbit::SEAT_0 | Seatbit::SEAT_1);
        let five = Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap();
        let eval = Eval::from(five);

        let w = Winnings { equity, eval };

        let s = w.to_string();
        assert!(s.contains("Winnings("));
        assert!(s.contains("chips=150"));
        assert!(s.contains("Royal") || s.contains("HandRank") || s.contains("A"));
    }
}
