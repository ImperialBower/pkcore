use crate::arrays::combos::combo::Combo;
use crate::arrays::combos::combos::Combos;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ComboRangeForm {
    PocketPairs,
    SuitedConnectors,
    OffsuitConnectors,
    Connectors,
    AceXSuited,
    AceXOffsuit,
    AceX,
    #[default]
    Unsupported,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ComboRange {
    pub higher: Combo,
    pub lower: Combo,
}

impl ComboRange {
    #[must_use]
    pub fn new(higher: Combo, lower: Combo) -> Self {
        let cr = match higher < lower {
            true => ComboRange {
                higher: lower,
                lower: higher,
            },
            false => ComboRange { higher, lower },
        };

        cr
    }

    pub fn filter_collection(self, collection: &[Combo]) -> Combos {
        Combos::from(
            collection
                .iter()
                .copied()
                .filter(|combo| self.contains(*combo))
                .collect::<Vec<Combo>>(),
        )
    }

    #[must_use]
    pub fn contains(&self, combo: Combo) -> bool {
        combo >= self.lower && combo <= self.higher
    }

    #[must_use]
    pub fn is_aligned(&self) -> bool {
        !self.is_empty() && self.higher.is_aligned_with(&self.lower) && self.lower.is_aligned_with(&self.higher)
    }

    #[must_use]
    pub fn is_ace_x(&self) -> bool {
        !self.is_empty() && self.higher.is_ace_x() && self.lower.is_ace_x()
    }

    #[must_use]
    pub fn is_ace_x_suited(&self) -> bool {
        !self.is_empty() && self.higher.is_ace_x_suited() && self.lower.is_ace_x_suited()
    }

    #[must_use]
    pub fn is_ace_x_offsuit(&self) -> bool {
        !self.is_empty() && self.higher.is_ace_x_offsuit() && self.lower.is_ace_x_offsuit()
    }

    #[must_use]
    pub fn is_connector(&self) -> bool {
        !self.is_empty() && self.is_aligned() && self.higher.is_connector()
    }

    #[must_use]
    pub fn is_suited_connector(&self) -> bool {
        !self.is_empty() && self.is_aligned() && self.higher.is_suited_connector() && self.lower.is_suited_connector()
    }

    #[must_use]
    pub fn is_offsuit_connector(&self) -> bool {
        !self.is_empty() && self.is_aligned() && self.higher.is_offsuit_connector() && self.lower.is_offsuit_connector()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.higher == self.lower
    }

    #[must_use]
    pub fn are_pocket_pairs(&self) -> bool {
        !self.is_empty() && self.higher.is_pocket_pair() && self.lower.is_pocket_pair()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__ranges__combos__combo_range_tests {
    use super::*;

    #[test]
    fn new() {
        let range = ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QJ);
        assert_eq!(Combo::COMBO_AK, range.higher);
        assert_eq!(Combo::COMBO_QJ, range.lower);

        let range = ComboRange::new(Combo::COMBO_QJ, Combo::COMBO_AK);
        assert_eq!(Combo::COMBO_AK, range.higher);
        assert_eq!(Combo::COMBO_QJ, range.lower);
    }

    #[test]
    fn contains() {
        let range = ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QJ);
        assert!(range.contains(Combo::COMBO_AK));
        assert!(range.contains(Combo::COMBO_KQ));
        assert!(range.contains(Combo::COMBO_QJ));
        assert!(!range.contains(Combo::COMBO_JT));
        assert!(!range.contains(Combo::COMBO_T9));

        let range = ComboRange::new(Combo::COMBO_QJ, Combo::COMBO_AK);
        assert!(range.contains(Combo::COMBO_AK));
        assert!(range.contains(Combo::COMBO_QJ));
        assert!(!range.contains(Combo::COMBO_JT));
        assert!(!range.contains(Combo::COMBO_T9));
    }

    #[test]
    fn filter_collection() {
        let range = ComboRange::new(Combo::COMBO_AK, Combo::COMBO_T9);
        let collection = vec![
            Combo::COMBO_AK,
            Combo::COMBO_KQ,
            Combo::COMBO_QJ,
            Combo::COMBO_JT,
            Combo::COMBO_T9,
        ];
        let filtered = range.filter_collection(&collection);
        assert_eq!(Combos::from(collection), filtered);
    }

    #[test]
    fn is_aligned() {
        assert!(ComboRange::new(Combo::COMBO_AA, Combo::COMBO_22).is_aligned());
        assert!(ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QJ).is_aligned());
        assert!(ComboRange::new(Combo::COMBO_AK, Combo::COMBO_A4).is_aligned());
        assert!(ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_QJs).is_aligned());
        assert!(ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_QJo).is_aligned());

        assert!(!ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_QJo).is_aligned());

        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QJo).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AA, Combo::COMBO_KQ).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QT).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_AK).is_aligned());
    }

    #[test]
    fn is_aligned_ace_x_suited() {
        let ace_xs = ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_A2s);

        assert!(Combo::COMBO_AKs.is_connector());
        assert!(Combo::COMBO_AKs.is_suited_connector());
        assert!(Combo::COMBO_A2s.is_ace_x_suited());

        assert!(!ace_xs.is_empty());
        assert!(ace_xs.higher.is_aligned_with(&ace_xs.lower));
        assert!(ace_xs.is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_A2s).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_A2).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_A2o).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_A2s).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_A2o).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_A2).is_aligned());
    }

    #[test]
    fn is_aligned_ace_x_offsuit() {
        assert!(ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_A4o).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_A4s).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_A4).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_A4o).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_A4o).is_aligned());
    }

    #[test]
    fn is_aligned_ace_x() {
        assert!(Combo::COMBO_AK.is_aligned_with(&Combo::COMBO_A4));
        assert!(Combo::COMBO_A4.is_aligned_with(&Combo::COMBO_AK));
        assert!(ComboRange::new(Combo::COMBO_AK, Combo::COMBO_A4).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_A4s).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_A4).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_A4o).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_A4o).is_aligned());
    }

    #[test]
    fn is_pocket_pairs() {
        assert!(ComboRange::new(Combo::COMBO_AA, Combo::COMBO_22).are_pocket_pairs());
        assert!(ComboRange::new(Combo::COMBO_33, Combo::COMBO_44).are_pocket_pairs());
        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QJ).are_pocket_pairs());
    }

    #[test]
    fn is_ace_x_suited() {
        assert!(ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_A2s).is_aligned());
        assert!(ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_A2s).is_ace_x_suited());

        assert!(ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_A3s).is_aligned());
        assert!(ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_A3s).is_ace_x_suited());

        assert!(ComboRange::new(Combo::COMBO_AQs, Combo::COMBO_A3s).is_ace_x_suited());
        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QJ).is_ace_x_suited());
    }
}
