use crate::Shifty;
use crate::analysis::store::db::hup::HUPResult;
use crate::arrays::matchups::masked::Masked;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Shifter {
    pub masked: Masked,
    pub shifts: Vec<SortedHeadsUp>,
}

impl Shifter {
    #[must_use]
    pub fn shifts(&self, _hupr: &HUPResult) -> Vec<HUPResult> {
        todo!()
    }
}

impl Display for Shifter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let shifts_str = self
            .shifts
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n...");

        write!(f, "Shifter {{ masked: {}, shifts:\n...{}}}", self.masked, shifts_str)
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
        Shifter {
            masked: *masked,
            shifts,
        }
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
    use super::*;
    use crate::analysis::gto::odds::WinLoseDraw;
    use crate::arrays::matchups::masks::rank_mask::RankMask;
    use crate::arrays::matchups::masks::suit_mask::SuitMask;
    use crate::arrays::matchups::masks::suit_texture::SuitTexture;
    use crate::arrays::two::Two;
    use crate::bard::Bard;
    use std::str::FromStr;

    fn hupr() -> HUPResult {
        HUPResult {
            higher: Bard::from(Two::HAND_AD_TD),
            lower: Bard::from(Two::HAND_5H_4S),
            odds: WinLoseDraw {
                wins: 1108295,
                losses: 595903,
                draws: 8106,
            },
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

        let _shifter: Shifter = hupr.into();

        // let shu = SortedHeadsUp::from_str("A♠ A♥ T♥ 4♦").unwrap();
        // let shifter: Shifter = shu.into();
        //
        // let shifts = shifter.shifts(&HUPResult::from(shu));
        // assert_eq!(shifts.len(), 3);
        // assert_eq!(shifts[0].to_string(), "A♠ A♥ - T♥ 4♣");
        // assert_eq!(shifts[1].to_string(), "A♠ A♣ - T♣ 4♠");
        // assert_eq!(shifts[2].to_string(), "A♦ A♣ - T♣ 4♠");
    }

    #[test]
    #[ignore]
    fn display() {
        let shu = SortedHeadsUp::from_str("A♠ A♥ T♥ 4♦").unwrap();
        let shifter: Shifter = shu.into();
        assert_eq!(
            shifter.to_string(),
            "Shifter { masked: A♠ A♥ - T♥ 4♦ Type1223a 1100,0110 1000000000000,0000100000100, shifts:\n...A♠ A♦ - T♦ 4♥\n...A♦ A♣ - T♦ 4♥\n...A♥ A♦ - T♦ 4♣\n...A♠ A♣ - T♣ 4♥\n...A♠ A♣ - T♠ 4♥\n...A♠ A♣ - T♣ 4♦\n...A♠ A♣ - T♠ 4♦\n...A♠ A♥ - T♠ 4♦\n...A♥ A♦ - T♦ 4♠\n...A♦ A♣ - T♦ 4♠\n...A♥ A♦ - T♥ 4♠\n...A♠ A♦ - T♠ 4♥\n...A♥ A♣ - T♥ 4♠\n...A♥ A♦ - T♥ 4♣\n...A♠ A♦ - T♦ 4♣\n...A♠ A♦ - T♠ 4♣\n...A♦ A♣ - T♣ 4♥\n...A♠ A♥ - T♠ 4♣\n...A♥ A♣ - T♥ 4♦\n...A♥ A♣ - T♣ 4♦\n...A♦ A♣ - T♣ 4♠\n...A♥ A♣ - T♣ 4♠\n...A♠ A♥ - T♥ 4♦\n...A♠ A♥ - T♥ 4♣}"
        );
    }

    /// This shift looked sus af, which it was, which is why I wrote this test.
    #[test]
    fn from_hup_result() {
        let hupr = hupr();
        let expected = Shifter {
            masked: Masked {
                shu: SortedHeadsUp::new(Two::HAND_AD_TD, Two::HAND_5H_4S),
                texture: SuitTexture::Type1123,
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
                SortedHeadsUp::new(Two::HAND_AS_TS, Two::HAND_5H_4D),
                SortedHeadsUp::new(Two::HAND_AS_TS, Two::HAND_5H_4C),
                SortedHeadsUp::new(Two::HAND_AS_TS, Two::HAND_5D_4H),
                SortedHeadsUp::new(Two::HAND_AS_TS, Two::HAND_5D_4C),
                SortedHeadsUp::new(Two::HAND_AS_TS, Two::HAND_5C_4H),
                SortedHeadsUp::new(Two::HAND_AS_TS, Two::HAND_5C_4D),
                SortedHeadsUp::new(Two::HAND_AH_TH, Two::HAND_5S_4D),
                SortedHeadsUp::new(Two::HAND_AH_TH, Two::HAND_5S_4C),
                SortedHeadsUp::new(Two::HAND_AH_TH, Two::HAND_5D_4S),
                SortedHeadsUp::new(Two::HAND_AH_TH, Two::HAND_5D_4C),
                SortedHeadsUp::new(Two::HAND_AH_TH, Two::HAND_5C_4S),
                SortedHeadsUp::new(Two::HAND_AH_TH, Two::HAND_5C_4D),
                SortedHeadsUp::new(Two::HAND_AD_TD, Two::HAND_5S_4H),
                SortedHeadsUp::new(Two::HAND_AD_TD, Two::HAND_5S_4C),
                SortedHeadsUp::new(Two::HAND_AD_TD, Two::HAND_5H_4S),
                SortedHeadsUp::new(Two::HAND_AD_TD, Two::HAND_5H_4C),
                SortedHeadsUp::new(Two::HAND_AD_TD, Two::HAND_5C_4S),
                SortedHeadsUp::new(Two::HAND_AD_TD, Two::HAND_5C_4H),
                SortedHeadsUp::new(Two::HAND_AC_TC, Two::HAND_5S_4H),
                SortedHeadsUp::new(Two::HAND_AC_TC, Two::HAND_5S_4D),
                SortedHeadsUp::new(Two::HAND_AC_TC, Two::HAND_5H_4S),
                SortedHeadsUp::new(Two::HAND_AC_TC, Two::HAND_5H_4D),
                SortedHeadsUp::new(Two::HAND_AC_TC, Two::HAND_5D_4S),
                SortedHeadsUp::new(Two::HAND_AC_TC, Two::HAND_5D_4H),
            ],
        };

        let mut actual: Shifter = Shifter::from(&hupr);
        actual.shifts.sort();
        actual.shifts.reverse();

        println!("{actual}");

        assert_eq!(actual.shifts.len(), 24);
        assert_eq!(actual, expected);
    }
}
