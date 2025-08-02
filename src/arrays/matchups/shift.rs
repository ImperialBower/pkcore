use crate::Shifty;
use crate::arrays::matchups::masked::Masked;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use std::fmt::{Display, Formatter};
use crate::analysis::store::db::headsup_preflop_result::HUPResult;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Shifter {
    pub masked: Masked,
    pub shifts: Vec<SortedHeadsUp>,
}

impl Shifter {
    pub fn shifts(&self, hupr: &HUPResult) -> Vec<HUPResult> {
        todo!()
    }
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

impl From<HUPResult> for Shifter {
    fn from(hupr: HUPResult) -> Self {
        let masked = Masked::from(hupr);
        let shifts: Vec<SortedHeadsUp> = masked.shifts().iter().map(|s| (*s).into()).collect();
        Shifter { masked, shifts }
    }
}

impl From<&HUPResult> for Shifter {
    fn from(hupr: &HUPResult) -> Self {
        let masked = Masked::from(hupr);
        let shifts: Vec<SortedHeadsUp> = masked.shifts().iter().map(|s| (*s).into()).collect();
        Shifter { masked, shifts }
    }
}

impl From<&Masked> for Shifter {
    fn from(masked: &Masked) -> Self {
        let shifts: Vec<SortedHeadsUp> = masked.shifts().iter().map(|s| (*s).into()).collect();
        Shifter { masked: *masked, shifts }
    }
}

impl From<SortedHeadsUp> for Shifter {
    fn from(shu: SortedHeadsUp) -> Self {
        let masked = Masked::from(shu);
        let shifts: Vec<SortedHeadsUp> = masked.shifts().iter().map(|s| (*s).into()).collect();
        Shifter { masked, shifts }
    }
}

impl From<&SortedHeadsUp> for Shifter {
    fn from(shu: &SortedHeadsUp) -> Self {
        let masked = Masked::from(*shu);
        let shifts: Vec<SortedHeadsUp> = masked.shifts().iter().map(|s| (*s).into()).collect();
        Shifter { masked, shifts }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__masks__shift_tests {
    use std::str::FromStr;
    use crate::arrays::matchups::masks::rank_mask::RankMask;
    use crate::arrays::matchups::masks::suit_mask::SuitMask;
    use crate::arrays::two::Two;
    use crate::bard::Bard;
    use super::*;

    fn hupr() -> HUPResult {
        HUPResult {
            higher: Bard::from(Two::HAND_AD_TD),
            lower: Bard::from(Two::HAND_5H_4S),
            higher_wins: 1108295,
            lower_wins: 595903,
            ties: 8106,
        }
    }

    #[test]
    fn shifts() {

        // A♦ T♦ (1108295) 5♥ 4♠ (595903) ties: (8106)
        // Shifter { masked: A♦ T♦ - 5♥ 4♠ Type1123 0010,1100 1000100000000,0000000001100, shifts:
        // ...A♠ T♠ - 5♦ 4♥
        // ...A♣ T♦ - 5♥ 4♥
        // ...A♦ T♦ - 5♥ 4♣ }
        let hupr = hupr();

        let shifter: Shifter = hupr.into();

        // let shu = SortedHeadsUp::from_str("A♠ A♥ T♥ 4♦").unwrap();
        // let shifter: Shifter = shu.into();
        //
        // let shifts = shifter.shifts(&HUPResult::from(shu));
        // assert_eq!(shifts.len(), 3);
        // assert_eq!(shifts[0].to_string(), "A♠ A♥ - T♥ 4♣");
        // assert_eq!(shifts[1].to_string(), "A♠ A♣ - T♣ 4♠");
        // assert_eq!(shifts[2].to_string(), "A♦ A♣ - T♣ 4♠");
    }

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

    #[test]
    fn from_hup_result() {
        let hupr = hupr();
        let expected = Shifter {
            masked: Masked {
                shu: SortedHeadsUp::new(Two::HAND_AD_TD, Two::HAND_5H_4S),
                texture: crate::arrays::matchups::masks::suit_texture::SuitTexture::Type1123,
                suit_mask: SuitMask {
                    higher: 0b0010,
                    lower: 0b1100,
                },
                rank_mask: RankMask {
                    higher: 0b1000100000000,
                    lower: 0b0000000001100,
                },
            },
            shifts: vec![
                SortedHeadsUp::from_str("A♦ T♣ 5♥ 4♥").unwrap(), /// This shift looks sus af
                SortedHeadsUp::from_str("A♦ T♦ 5♥ 4♣").unwrap(),
                SortedHeadsUp::from_str("A♣ T♦ 5♥ 4♥").unwrap(),
            ],
        };

        let actual: Shifter = Shifter::from(&hupr);

        println!("{actual}");

        assert_eq!(actual, expected);
    }
}