use crate::Shifty;
use crate::arrays::matchups::masked::Masked;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Shifter {
    pub masked: Masked,
    pub shifts: Vec<SortedHeadsUp>,
}

impl Display for Shifter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Shifter {{ masked: {}, shifts:\n...{}\n...{}\n...{} }}",
               self.masked,
               self.shifts.get(0).map_or("None".to_string(), |s| s.to_string()),
               self.shifts.get(1).map_or("None".to_string(), |s| s.to_string()),
               self.shifts.get(2).map_or("None".to_string(), |s| s.to_string())
        )
    }
}

impl From<SortedHeadsUp> for Shifter {
    fn from(shu: SortedHeadsUp) -> Self {
        let masked = Masked::from(shu);
        let shifts: Vec<SortedHeadsUp> = masked.shifts().iter().map(|s| (*s).into()).collect();
        Shifter { masked, shifts }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__masks__shift_tests {
    use std::str::FromStr;
    use super::*;

    /// - AS AH - TH 4D
    ///   - AS AH - TH 4C
    /// - AS AH - TD 4C
    /// - AS AD - TS 4C
    ///   - AS AD - TS 4H
    /// - AH AC - TC 4D
    ///   - AH AD - TC 4D
    ///
    #[test]
    fn display() {
        let shu = SortedHeadsUp::from_str("A♠ A♥ T♥ 4♦").unwrap();
        let shifter: Shifter = shu.into();
        assert_eq!(shifter.to_string(), "Shifter { masked: A♠ A♥ - T♥ 4♦ Type1223a 1100,0110 1000000000000,0000100000100, shifts:\n...A♥ A♦ - T♥ 4♣\n...A♥ A♣ - T♣ 4♠\n...A♦ A♣ - T♣ 4♠ }");
    }
}