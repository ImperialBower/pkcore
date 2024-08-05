use std::collections::{HashMap, HashSet};
use crate::analysis::store::db::headsup_preflop_result::HUPResult;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::PKError;

/// I have a problem, and this struct is designed to help with the solution. My reported
/// results are turning out to be inconsistent. The `Alignment` struct will store every
/// variant (shoutout to the `TVA`) so that we can easily isolate any issues. I'm betting
/// that the source of the issue is me trying to run too many instances calculating preflop
/// results.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Aligner(HashMap<SortedHeadsUp, HashSet<HUPResult>>);

impl Aligner {
    #[must_use]
    pub fn exists(&self, hupr: &HUPResult) -> bool {
        if let Ok(shu) = SortedHeadsUp::try_from(hupr) {
            if self.0.contains_key(&shu) {
                match self.0.get(&shu) {
                    Some(set) => set.contains(hupr),
                    None => false,
                }
            } else {
                false
            }
        } else {
            log::warn!("Processing invalid HUPResult: {hupr}");
            false
        }
    }

    /// Insert a `SortedHeadsUp` and `HUPResult` into the `Aligner`, returning the
    /// number of `HUPResult` in the `HashSet` for the `SortedHeadsUp`.
    ///
    /// # Errors
    ///
    /// Throws a `PKError` if the `SortedHeadsUp` from the `HUPResult` is invalid.
    pub fn insert(&mut self, hupr: HUPResult) -> Result<Option<usize>, PKError> {
        let shu = match SortedHeadsUp::try_from(hupr) {
            Ok(shu) => shu,
            Err(e) => return Err(e),
        };

        //     Ok(shu) => {
        //         match self.0.get(&shu) {
        //             Some(set) => Some(set),
        //             None => None,
        //         }
        //     }
        //     Err(_) => None,
        // };


        todo!("Insert a SortedHeadsUp and HUPResult into the Aligner");
    }

    /// Get the `HashSet` of `HUPResult` for a given `SortedHeadsUp`.
    #[must_use]
    pub fn get(&self, shu: &SortedHeadsUp) -> Option<&HashSet<HUPResult>> {
        self.0.get(shu)
    }

    /// Get the `HashMap` of `SortedHeadsUp` and `HashSet` of `HUPResult`.
    #[must_use]
    pub fn get_all(&self) -> &HashMap<SortedHeadsUp, HashSet<HUPResult>> {
        &self.0
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of `SortedHeadsUp` in the `Aligner`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get the number of `HUPResult` in the `Aligner`.
    #[must_use]
    pub fn sum(&self) -> usize {
        self.0.values().map(HashSet::len).sum()
    }
}

#[cfg(test)]
mod analysis__store__alignment_tests {
    use crate::arrays::two::Two;
    use crate::Pile;
    use super::*;

    #[test]
    fn default() {
        let mut aligner = Aligner::default();
        let hupr = HUPResult::default();
        let shu = SortedHeadsUp::try_from(&hupr).unwrap();
        assert_eq!(aligner.exists(&hupr), false);
        assert_eq!(aligner.get(&shu), None);
        assert_eq!(aligner.is_empty(), true);
        assert_eq!(aligner.len(), 0);
        assert_eq!(aligner.sum(), 0);
    }

    #[test]
    fn insert() {
        let hupr = HUPResult {
            higher: Two::HAND_TS_2H.bard(),
            lower: Two::HAND_TH_TD.bard(),
            higher_wins: 73_828,
            lower_wins: 1_580_550,
            ties: 57_926,
        };

        let mut aligner = Aligner::default();

        let shu = SortedHeadsUp::try_from(&hupr).unwrap();
        assert_eq!(aligner.insert(hupr.clone()).unwrap(), None);
        assert_eq!(aligner.exists(&hupr), true);
        assert_eq!(aligner.get(&shu).unwrap().contains(&hupr), true);
        assert_eq!(aligner.is_empty(), false);
        assert_eq!(aligner.len(), 1);
        assert_eq!(aligner.sum(), 1);
    }

    #[test]
    fn insert_default_invalid() {
        let hupr = HUPResult::default();
        let mut aligner = Aligner::default();
        assert_eq!(aligner.insert(hupr).is_err(), true);
    }
}