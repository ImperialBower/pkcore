use crate::arrays::two::Two;
use serde::{Deserialize, Serialize};

/// Row is a data format designed to store a specific Heads Up preflop analysis. The struct sorts
/// the hands so that the higher one in sort order is first. Since performing preflop calculations
/// is so intensive this is to avoid doing duplicate work.
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
    pub fn new(
        first: Two,
        second: Two,
        first_wins: usize,
        second_wins: usize,
        ties: usize,
    ) -> PreflopRow {
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
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__store__heads_up_row_test {
    use super::*;
    use crate::util::data::TestData;

    fn row() -> PreflopRow {
        let wins = TestData::wins_the_hand().results_heads_up();
        PreflopRow::new(
            Two::HAND_6S_6H,
            Two::HAND_5D_5C,
            wins.first_wins,
            wins.second_wins,
            wins.ties,
        )
    }

    fn row_inverted() -> PreflopRow {
        let wins = TestData::wins_the_hand().results_heads_up();
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
        let wins = TestData::wins_the_hand().results_heads_up();
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
        let wins = TestData::wins_the_hand().results_heads_up();
        let row = row();
        let row_inverted = row_inverted();

        assert_eq!(row.get_wins(Two::HAND_6S_6H).unwrap(), wins.first_wins);
        assert_eq!(row.get_wins(Two::HAND_5D_5C).unwrap(), wins.second_wins);
        assert!(row.get_wins(Two::HAND_AC_KC).is_none());

        assert_eq!(
            row_inverted.get_wins(Two::HAND_6S_6H).unwrap(),
            wins.first_wins
        );
        assert_eq!(
            row_inverted.get_wins(Two::HAND_5D_5C).unwrap(),
            wins.second_wins
        );
        assert!(row_inverted.get_wins(Two::HAND_AC_KS).is_none());
    }
}
