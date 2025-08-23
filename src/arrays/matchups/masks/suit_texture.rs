use crate::Pile;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::suit::Suit;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub enum SuitTexture {
    #[default]
    TypeUnknown,
    Type1111,  // 1. suited, suited, same suit
    Type1112a, // 2a. suited, off suit, sharing suit | One of the hands is a pocket pair
    Type1112b, // 2b. suited, off suit, sharing suit | Top suited - Bottom lower same suit
    Type1112c, // 2c. suited, off suit, sharing suit | Top suited - Bottom higher same suit
    Type1112d, // 2d. suited, off suit, sharing suit | Bottom suited - Top higher same suit
    Type1112e, // 2e. suited, off suit, sharing suit | Bottom suited - Top lower same suit
    Type1122,  // 3. suited, suited, different suits
    Type1123,  // 4. suited, off suit, different suits
    Type1223a, // 5a. off suit, off suit, sharing one suit TODO: Defect watch
    Type1223b, // 5b. off suit, off suit, sharing one suit TODO: Defect watch
    Type1223c, // 5c. off suit, off suit, sharing one suit TODO: Defect watch
    Type1223d, // 5d. off suit, off suit, sharing one suit TODO: Defect watch
    Type1212a, // 6a. off suit, off suit, sharing both suits, higher first shares suit with lower first
    Type1212b, // 6b. off suit, off suit, sharing both suits, higher first shares suit with lower second
    Type1234,  // 7. off suit, off suit, sharing no suits
    Type1233,  // 8. off suit, suited, sharing no suits
}

impl From<SortedHeadsUp> for SuitTexture {
    fn from(shu: SortedHeadsUp) -> Self {
        SuitTexture::from(&shu)
    }
}

/// TODO: HARRANGE - This code is an abomination. No wonder there are so many gaps in it.
impl From<&SortedHeadsUp> for SuitTexture {
    #[allow(clippy::if_same_then_else, clippy::collapsible_else_if)]
    fn from(shu: &SortedHeadsUp) -> Self {
        let suits: HashSet<Suit> = shu.higher.suits().union(&shu.lower.suits()).copied().collect();

        match suits.len() {
            1 => SuitTexture::Type1111,
            2 => {
                if shu.higher.is_suited() && shu.lower.is_suited() {
                    SuitTexture::Type1122
                } else if !shu.higher.is_suited() && !shu.lower.is_suited() {
                    // 4♠ 4♥ 3♥ 2♠ Type1212b 1100,1100 0000000000100,0000000000011 4♠ 4♥ (1477762) 3♥ 2♠ (169696) ties: (64846)
                    // 4♠ 4♥ 3♠ 2♥ Type1212a 1100,1100 0000000000100,0000000000011 4♠ 4♥ (1477762) 3♠ 2♥ (169696) ties: (64846)
                    if shu.higher.is_pair() || shu.lower.is_pair() {
                        SuitTexture::Type1212a
                    } else if shu.higher.first().get_suit() == shu.lower.first().get_suit() {
                        SuitTexture::Type1212a
                    } else {
                        SuitTexture::Type1212b
                    }
                } else if shu.higher.is_pair() || shu.lower.is_pair() {
                    SuitTexture::Type1112a
                } else if shu.higher.is_suited() {
                    if shu.higher.first().get_suit() == shu.lower.second().get_suit() {
                        SuitTexture::Type1112b
                    } else {
                        SuitTexture::Type1112c
                    }
                } else {
                    // Bottom must be suited
                    if shu.higher.first().get_suit() == shu.lower.second().get_suit() {
                        SuitTexture::Type1112d
                    } else {
                        SuitTexture::Type1112e
                    }
                }
            }
            3 => {
                // #[case("A♠ A♣ A♥ 2♥", "A♠ A♥ A♦ 2♦", Type1123)]
                if shu.higher.is_suited() {
                    if shu.lower.is_suited() {
                        panic!("This is impossible since ")
                    } else {
                        // #[case("8♣ 2♣ 3♠ 2♥", "8♠ 2♠ 3♥ 2♦", Type1233)]
                        SuitTexture::Type1123
                    }
                } else {
                    if shu.lower.is_suited() {
                        SuitTexture::Type1233
                    } else {
                        if shu.higher.is_pair() {
                            if (shu.lower.first().get_suit() == shu.higher.first().get_suit())
                                || (shu.lower.first().get_suit() == shu.higher.second().get_suit())
                            {
                                // A♠ A♥ T♥ 4♦
                                SuitTexture::Type1223a
                            } else {
                                // A♥ A♦ T♣ 4♥
                                SuitTexture::Type1223c
                            }
                        } else if shu.lower.is_pair() {
                            if (shu.higher.first().get_suit() == shu.lower.first().get_suit())
                                || (shu.higher.first().get_suit() == shu.lower.second().get_suit())
                            {
                                SuitTexture::Type1223a
                            } else {
                                SuitTexture::Type1223c
                            }
                        } else {
                            if shu.higher.first().get_suit() == shu.lower.first().get_suit() {
                                SuitTexture::Type1223a
                            } else if shu.higher.first().get_suit() == shu.lower.second().get_suit() {
                                SuitTexture::Type1223b
                            } else if shu.higher.second().get_suit() == shu.lower.first().get_suit() {
                                SuitTexture::Type1223c
                            } else {
                                SuitTexture::Type1223d
                            }
                        }
                    }
                }
            }
            4 => SuitTexture::Type1234,
            _ => SuitTexture::TypeUnknown,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__masks__suit_texture_tests {
    use crate::arrays::matchups::masked::Masked;
    use crate::arrays::matchups::masks::suit_texture::SuitTexture;
    use std::str::FromStr;

    use crate::Shifty;
    use crate::arrays::matchups::masks::suit_texture::SuitTexture::{
        Type1111, Type1112b, Type1112d, Type1112e, Type1123, Type1212a, Type1212b, Type1223a, Type1223b, Type1223c,
        Type1223d, Type1233, Type1234,
    };
    use rstest::rstest;

    #[rstest]
    #[allow(clippy::duplicated_attributes)]
    #[case("7♥ 5♥ 6♥ 2♥", "7♠ 5♠ 6♠ 2♠", Type1111)]
    #[case("A♦ Q♦ K♣ 5♦", "A♠ Q♠ K♥ 5♠", Type1112b)]
    #[case("8♥ 2♦ 7♥ 5♥", "8♠ 2♥ 7♠ 5♠", Type1112d)]
    #[case("7♣ 2♠ 5♠ 3♠", "7♠ 2♥ 5♥ 3♥", Type1112e)]
    #[case("6♦ 3♣ 5♣ 4♣", "6♠ 3♥ 5♥ 4♥", Type1112e)]
    #[case("4♠ 4♥ 3♥ 2♠", "4♠ 4♥ 3♠ 2♥", Type1212a)]
    #[case("7♦ 7♣ 6♣ 5♦", "7♠ 7♥ 6♠ 5♥", Type1212a)]
    #[case("T♠ 8♥ 4♠ 4♥", "T♥ 8♠ 4♠ 4♥", Type1212a)]
    #[case("8♥ 2♠ 4♠ 4♥", "8♠ 2♥ 4♠ 4♥", Type1212a)]
    #[case("A♥ Q♦ J♦ 6♥", "A♠ Q♥ J♥ 6♠", Type1212b)]
    #[case("7♦ 5♣ 4♣ 3♦", "7♠ 5♥ 4♥ 3♠", Type1212b)]
    #[case("A♦ A♣ 9♦ 6♠", "A♠ A♥ 9♥ 6♦", Type1223a)]
    #[case("A♥ K♦ 7♥ 5♣", "A♠ K♥ 7♠ 5♦", Type1223a)]
    #[case("9♣ 7♠ 4♣ 3♥", "9♦ 7♠ 4♦ 3♥", Type1223a)]
    #[case("6♠ 3♥ 4♠ 4♣", "6♠ 3♥ 4♠ 4♦", Type1223a)]
    #[case("6♠ 3♥ 4♠ 4♣", "6♠ 3♥ 4♠ 4♦", Type1223a)]
    #[case("3♣ 2♠ 2♦ 2♣", "3♠ 2♥ 2♠ 2♦", Type1223a)]
    #[case("9♣ 7♠ 4♥ 3♣", "9♠ 7♥ 4♦ 3♠", Type1223b)]
    #[case("A♠ A♥ A♦ K♠", "A♠ A♥ A♦ K♥", Type1223c)]
    #[case("A♠ A♥ A♦ K♠", "A♥ A♦ A♣ K♥", Type1223c)]
    #[case("8♠ 2♦ 3♥ 3♦", "8♣ 2♠ 3♠ 3♦", Type1223c)]
    #[case("A♥ A♦ T♣ 4♥", "A♠ A♥ T♦ 4♥", Type1223c)]
    #[case("5♠ 3♥ 5♥ 5♦", "5♠ 3♦ 5♥ 5♦", Type1223c)]
    #[case("3♠ 3♥ 3♦ 2♥", "3♠ 3♥ 3♦ 2♠", Type1223c)]
    #[case("T♣ 2♠ 5♠ 4♥", "T♠ 2♥ 5♥ 4♦", Type1223c)]
    #[case("6♠ 2♣ 6♦ 6♣", "6♠ 6♥ 6♦ 2♠", Type1223c)]
    #[case("3♠ 3♥ 3♣ 2♦", "3♠ 3♥ 3♦ 2♣", Type1234)]
    #[case("A♠ A♣ A♥ 2♥", "A♠ A♥ A♦ 2♦", Type1233)]
    #[case("8♣ 2♣ 3♠ 2♥", "8♠ 2♠ 3♥ 2♦", Type1123)]
    #[case("8♠ 7♣ 4♥ 3♦", "8♣ 7♠ 4♥ 3♦", Type1234)]
    #[case("7♣ 5♠ 6♥ 3♦", "7♠ 5♥ 6♦ 3♣", Type1234)]
    fn inverse_many(#[case] case1: &str, #[case] case2: &str, #[case] texture: SuitTexture) {
        let masked1 = Masked::from_str(case1).unwrap();
        let masked2 = Masked::from_str(case2).unwrap();

        base(masked1, masked2, texture);
    }

    fn base(masked1: Masked, masked2: Masked, texture: SuitTexture) {
        assert_ne!(masked1, masked2);
        assert_eq!(masked1.texture, texture);
        assert_eq!(masked2.texture, texture);
        assert_eq!(masked1.shu.shifts(), masked2.shu.shifts());
        assert!(masked1.shu.shifts().contains(&masked2.shu));
        assert!(masked2.shu.shifts().contains(&masked1.shu));
    }

    #[test]
    #[ignore]
    fn paired_1212() {
        // #[case(, , Type1212a)]
        let masked1 = Masked::from_str("T♠ 8♥ 4♠ 4♥").unwrap();
        let masked2 = Masked::from_str("T♥ 8♠ 4♠ 4♥").unwrap();

        base(masked1, masked2, Type1212a);
    }

    #[test]
    fn paired_1223_not() {
        // Q♦ J♣ 7♠ 6♣ Type1223d 0011,1001 0011000000000,0000000110000 Q♦ J♣ (1105763) 7♠ 6♣ (591787) ties: (14754)
        // Q♠ J♥ 7♠ 6♦ Type1223a 1100,1010 0011000000000,0000000110000 Q♠ J♥ (1105763) 7♠ 6♦ (591754) ties: (14787)
        let masked1 = Masked::from_str("Q♦ J♣ 7♠ 6♣").unwrap();
        let masked2 = Masked::from_str("Q♠ J♥ 7♠ 6♦").unwrap();

        assert_eq!(masked1.texture, Type1223d);
        assert_eq!(masked2.texture, Type1223a);

        none(masked1, masked2);
    }

    fn none(masked1: Masked, masked2: Masked) -> bool {
        let s1 = masked1.shifts();
        let s2 = masked2.shifts();
        for m in &s1 {
            if s2.contains(&m) {
                return false;
            }
        }
        for m in &s2 {
            if s1.contains(&m) {
                return false;
            }
        }
        return true;
    }
}
