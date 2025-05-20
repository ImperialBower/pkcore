use crate::arrays::two::Two;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Row is a data format designed to store a specific Heads Up preflop analysis. The struct sorts
/// the hands so that the higher one in sort order is first. Since performing preflop calculations
/// is so intensive this is to avoid doing duplicate work.
///
/// "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠" HSP THE hand Negreanu/Hansen
/// First:  79.73% (1365284) || 79.71% (1364802) // stats from <https://tools.timodenk.com/poker-odds-pre-flop/>
/// Second: 18.39% (314904)  || 18.39% (314904)
/// Ties:    1.88% (32116)   ||  1.90% (32598)
/// Elapsed: 678.51s
///
/// Naked
/// ```txt
/// A♠ A♥ 7♦ 7♣, 79.69% (1364608), 20.05% (343300), 0.26% (4396) || 79.75% (1365570). 19.99% (342340), 0.26% (4394)
/// A♠ A♥ 6♦ 6♣, 79.66% (1363968), 20.05% (343394), 0.29% (4942) || 80.16% (1372658),
/// A♠ A♥ 5♦ 5♣, 80.06% (1370808), 19.60% (335688), 0.34% (5808)
/// A♠ A♥ 4♦ 4♣, 80.47% (1377896), 19.15% (327870), 0.38% (6538)
/// A♠ A♥ 3♦ 3♣, 80.88% (1384984), 18.68% (319884), 0.43% (7436)
/// A♠ A♥ 2♦ 2♣, 81.30% (1392072), 18.20% (311672), 0.50% (8560)
/// ```
///
/// A♠ A♥ A♦ A♣, 2.17% (37210), 2.17% (37210), 95.65% (1637884)
/// A♠ A♥ K♦ K♣, 81.06% (1388072), 18.55% (317694), 0.38% (6538)
/// A♠ A♥ K♠ K♣, 81.71% (1399204), 17.82% (305177), 0.46% (7923)
/// A♠ A♥ K♠ K♥, 82.36% (1410336), 17.09% (292660), 0.54% (9308)
/// A♠ A♦ K♥ Q♥, 82.13% (1406263), 17.50% (299588), 0.38% (6453)
/// A♠ A♥ Q♦ Q♣, 80.69% (1381624), 18.96% (324624), 0.35% (6056)
/// A♠ A♥ Q♠ Q♣, 81.33% (1392614), 18.24% (312251), 0.43% (7439)
/// A♠ A♥ Q♠ Q♥, 81.97% (1403604), 17.51% (299878), 0.52% (8822)
/// A♠ A♦ Q♥ J♥, 80.12% (1371916), 19.52% (334283), 0.36% (6105)
/// A♠ A♥ J♦ J♣, 80.31% (1375176), 19.36% (331554), 0.33% (5574)
/// A♠ A♥ J♠ J♣, 80.94% (1386024), 18.65% (319325), 0.41% (6955)
/// A♠ A♥ J♠ J♥, 81.58% (1396872), 17.93% (307096), 0.49% (8336)
/// A♠ A♦ J♥ T♥, 78.12% (1337569), 21.55% (368978), 0.34% (5757)
/// A♠ A♦ J♠ T♥, 82.56% (1413685), 17.08% (292487), 0.36% (6132)
/// A♠ A♦ J♥ T♠, 82.56% (1413643), 17.08% (292529), 0.36% (6132)
/// A♠ A♦ J♠ T♦, 83.22% (1424984), 16.34% (279821), 0.44% (7499)
/// A♠ A♦ J♦ T♠, 83.22% (1424984), 16.34% (279821), 0.44% (7499)
/// A♠ A♥ T♦ T♣, 79.93% (1368728), 19.77% (338484), 0.30% (5092)
/// A♠ A♥ T♠ T♣, 80.56% (1379434), 19.06% (326399), 0.38% (6471)
/// A♠ A♥ T♠ T♥, 81.19% (1390140), 18.36% (314314), 0.46% (7850)
/// A♠ A♥ 9♦ 9♣, 80.05% (1370736), 19.66% (336724), 0.28% (4844)
/// A♠ A♥ 9♠ 9♣, 80.68% (1381464), 18.96% (324610), 0.36% (6230)
/// A♠ A♥ 9♠ 9♥, 81.31% (1392192), 18.25% (312496), 0.44% (7616)
/// A♠ A♥ 8♦ 8♣, 79.68% (1364288), 20.07% (343646), 0.26% (4370)
/// A♠ A♥ 8♠ 8♣, 80.29% (1374874), 19.37% (331680), 0.34% (5750)
/// A♠ A♥ 8♠ 8♥, 80.91% (1385460), 18.67% (319714), 0.42% (7130)
/// A♠ A♥ 7♦ 7♣, 79.69% (1364608), 20.05% (343300), 0.26% (4396)
/// A♠ A♥ 7♠ 7♣, 80.31% (1375194), 19.35% (331347), 0.34% (5763)
/// A♠ A♥ 7♠ 7♥, 80.93% (1385780), 18.65% (319394), 0.42% (7130)
/// A♠ A♥ 6♦ 6♣, 79.66% (1363968), 20.05% (343394), 0.29% (4942)
/// A♠ A♥ 6♠ 6♣, 80.27% (1374538), 19.36% (331487), 0.37% (6279)
/// A♠ A♥ 6♦ 6♥, 80.27% (1374538), 19.36% (331487), 0.37% (6279)
/// A♠ A♥ 6♠ 6♥, 80.89% (1385108), 18.66% (319580), 0.44% (7616)
///
/// A♠ A♥ 5♦ 5♣, 80.06% (1370808), 19.60% (335688), 0.34% (5808)
/// A♠ A♥ 5♠ 5♣, 80.68% (1381517), 18.90% (323705), 0.41% (7082)
/// A♠ A♥ 5♠ 5♥, 81.31% (1392226), 18.20% (311722), 0.49% (8356)
/// A♠ A♥ 4♦ 4♣, 80.47% (1377896), 19.15% (327870), 0.38% (6538)
/// A♠ A♥ 4♠ 4♣, 81.10% (1388747), 18.45% (315867), 0.45% (7690)
/// A♠ A♥ 4♠ 4♥, 81.74% (1399598), 17.75% (303864), 0.52% (8842)
/// A♠ A♥ 3♦ 3♣, 80.88% (1384984), 18.68% (319884), 0.43% (7436)
/// A♠ A♥ 3♠ 3♣, 81.53% (1395977), 17.98% (307945), 0.49% (8382)
/// A♠ A♥ 3♠ 3♥, 82.17% (1406970), 17.29% (296006), 0.54% (9328)
/// A♠ A♥ 2♦ 2♣, 81.30% (1392072), 18.20% (311672), 0.50% (8560)
/// A♠ A♥ 2♠ 2♣, 81.95% (1403207), 17.51% (299910), 0.54% (9187)
/// A♠ A♥ 2♠ 2♥, 82.60% (1414342), 16.83% (288148), 0.57% (9814)
///
/// K♠ K♥ 3♦ 3♣, 80.49% (1378184), 19.08% (326652), 0.44% (7468)
/// K♠ K♥ 3♠ 3♣, 81.12% (1389038), 18.39% (314845), 0.49% (8421)
/// K♠ K♥ 3♠ 3♥, 81.75% (1399892), 17.70% (303038), 0.55% (9374)
/// K♠ K♥ 2♦ 2♣, 80.90% (1385272), 18.60% (318440), 0.50% (8592)
/// K♠ K♥ 2♠ 2♣, 81.54% (1396268), 17.92% (306810), 0.54% (9226)
/// K♠ K♥ 2♠ 2♥, 82.19% (1407264), 17.24% (295180), 0.58% (9860)
///
/// 6♠ 6♥ 5♦ 5♣, 79.73% (1365284), 18.39% (314904), 1.88% (32116)
/// 6♠ 6♥ 5♠ 5♣, 80.39% (1376436), 17.67% (302502), 1.95% (33366)
/// 6♠ 6♥ 5♠ 5♥, 81.04% (1387588), 16.94% (290100), 2.02% (34616)
///
/// 6♠ 6♥ 4♦ 4♣, 79.77% (1365852), 18.33% (313854), 1.90% (32598)
/// 6♠ 6♥ 4♠ 4♣, 80.42% (1377007), 17.61% (301564), 1.97% (33733)
/// 6♠ 6♥ 4♠ 4♥, 81.07% (1388162), 16.89% (289274), 2.04% (34868)
///
/// A♠ K♠ Q♠ J♠, 65.65% (1124180), 33.74% (577797), 0.60% (10327)
/// A♠ K♠ T♠ 9♠, 63.95% (1095005), 35.48% (607520), 0.57% (9779)
/// A♠ K♠ 8♠ 7♠, 63.25% (1083026), 36.21% (620020), 0.54% (9258)
/// A♠ K♠ J♦ J♣, 45.94% (786618), 53.68% (919198), 0.38% (6488)
/// K♠ Q♠ J♦ J♣
/// A♠ K♠ 7♣ 6♣, 60.14% (1029832), 39.42% (674947), 0.44% (7525)
///
/// 3♣ 2♦ 3♦ 2♣, 0.71% (12216), 0.71% (12216), 98.57% (1687872)
///
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct PreflopRow {
    pub higher: Two,
    pub lower: Two,
    pub higher_wins: usize,
    pub lower_wins: usize,
    pub ties: usize,
}

impl PreflopRow {
    #[must_use]
    pub fn new(first: Two, second: Two, first_wins: usize, second_wins: usize, ties: usize) -> PreflopRow {
        if first > second {
            PreflopRow {
                higher: first,
                lower: second,
                higher_wins: first_wins,
                lower_wins: second_wins,
                ties,
            }
        } else {
            PreflopRow {
                higher: second,
                lower: first,
                higher_wins: second_wins,
                lower_wins: first_wins,
                ties,
            }
        }
    }

    #[must_use]
    pub fn get_wins(&self, hand: Two) -> Option<usize> {
        if hand == self.higher {
            Some(self.higher_wins)
        } else if hand == self.lower {
            Some(self.lower_wins)
        } else {
            None
        }
    }

    #[must_use]
    pub fn to_index(&self) -> String {
        HUP::two_to_index(self.higher, self.lower)
    }
}

/// I need a way to store heads up calculations locally in memory for now
/// so that I can see if a calculation has already been done. Since the `Twos`
/// are sorted and the `Cards` in the `Twos` are sorted, we can get rid of a lot of
/// duplicate calculations.
///
/// TODO: Write tests!!!
///
///
#[derive(Clone, Debug, Default)]
pub struct PreflopRowHash(pub HashMap<String, PreflopRow>);

impl PreflopRowHash {
    pub fn add(&mut self, value: PreflopRow) -> Option<PreflopRow> {
        let key = HUP::two_to_index(value.higher, value.lower);
        self.0.insert(key, value)
    }

    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
}

/// Empty HUP struct. Using this for utility methods. Hacky, but I am a hack :-P
pub struct HUP;

impl HUP {
    #[must_use]
    pub fn two_to_index(a: Two, b: Two) -> String {
        format!("{a} {b}")
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__store__heads_up__hup_test {
    use super::*;

    #[test]
    fn two_to_index() {
        assert_eq!("5♦ 5♣ 6♠ 6♥", HUP::two_to_index(Two::HAND_5D_5C, Two::HAND_6S_6H));
        assert_eq!("5♦ 5♣ 6♠ 6♥", HUP::two_to_index(Two::HAND_5D_5C, Two::HAND_6S_6H));
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__store__heads_up_row_test {
    use super::*;
    use crate::util::data::TestData;

    fn row() -> PreflopRow {
        let wins = TestData::the_hand_as_wins().results_heads_up();
        PreflopRow::new(
            Two::HAND_6S_6H,
            Two::HAND_5D_5C,
            wins.first_wins,
            wins.second_wins,
            wins.ties,
        )
    }

    fn row_inverted() -> PreflopRow {
        let wins = TestData::the_hand_as_wins().results_heads_up();
        PreflopRow::new(
            Two::HAND_5D_5C,
            Two::HAND_6S_6H,
            wins.second_wins,
            wins.first_wins,
            wins.ties,
        )
    }

    #[test]
    fn new() {
        let wins = TestData::the_hand_as_wins().results_heads_up();
        let row = row();
        let row_inverted = row_inverted();

        assert_eq!(row.higher, Two::HAND_6S_6H);
        assert_eq!(row.lower, Two::HAND_5D_5C);
        assert_eq!(row.higher_wins, wins.first_wins);
        assert_eq!(row.lower_wins, wins.second_wins);
        assert_eq!(row.ties, wins.ties);

        assert_eq!(row_inverted.higher, Two::HAND_6S_6H);
        assert_eq!(row_inverted.lower, Two::HAND_5D_5C);
        assert_eq!(row_inverted.higher_wins, wins.first_wins);
        assert_eq!(row_inverted.lower_wins, wins.second_wins);
        assert_eq!(row_inverted.ties, wins.ties);

        assert_eq!(row, row_inverted);
    }

    #[test]
    fn get_wins() {
        let wins = TestData::the_hand_as_wins().results_heads_up();
        let row = row();
        let row_inverted = row_inverted();

        assert_eq!(row.get_wins(Two::HAND_6S_6H).unwrap(), wins.first_wins);
        assert_eq!(row.get_wins(Two::HAND_5D_5C).unwrap(), wins.second_wins);
        assert!(row.get_wins(Two::HAND_AC_KC).is_none());

        assert_eq!(row_inverted.get_wins(Two::HAND_6S_6H).unwrap(), wins.first_wins);
        assert_eq!(row_inverted.get_wins(Two::HAND_5D_5C).unwrap(), wins.second_wins);
        assert!(row_inverted.get_wins(Two::HAND_AC_KS).is_none());
    }

    #[test]
    fn to_index() {
        assert_eq!("6♠ 6♥ 5♦ 5♣", row().to_index());
        assert_eq!("6♠ 6♥ 5♦ 5♣", row_inverted().to_index());
    }
}
