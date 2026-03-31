use crate::analysis::gto::combo::Combo;
use crate::analysis::gto::twos::Twos;
use crate::arrays::two::Two;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ComboPairs(HashMap<Combo, Twos>);

impl ComboPairs {
    /// Adds a single [`Two`] hand to the [`Twos`] collection for the given [`Combo`].
    ///
    /// Creates the entry if it does not yet exist.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::combo_pairs::ComboPairs;
    /// use pkcore::arrays::two::Two;
    ///
    /// let mut cp = ComboPairs::default();
    /// cp.add(Combo::COMBO_AA, Two::HAND_AS_AH);
    /// assert!(cp.twos_for_combo(&Combo::COMBO_AA).is_some());
    /// ```
    pub fn add(&mut self, combo: Combo, two: Two) {
        let twos = self.0.entry(combo).or_default();
        twos.insert(two);
    }

    /// Returns a reference to the underlying [`HashMap`] mapping each [`Combo`] to its [`Twos`].
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo_pairs::ComboPairs;
    ///
    /// let cp = ComboPairs::default();
    /// assert!(cp.hash_map().is_empty());
    /// ```
    #[must_use]
    pub fn hash_map(&self) -> &HashMap<Combo, Twos> {
        &self.0
    }

    /// Inserts a full [`Twos`] collection for the given [`Combo`], replacing any existing entry.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::combo_pairs::ComboPairs;
    /// use pkcore::analysis::gto::twos::Twos;
    ///
    /// let mut cp = ComboPairs::default();
    /// cp.insert(Combo::COMBO_KK, Twos::default());
    /// assert!(cp.twos_for_combo(&Combo::COMBO_KK).is_some());
    /// ```
    pub fn insert(&mut self, combo: Combo, twos: Twos) {
        self.0.insert(combo, twos);
    }

    /// Returns all [`Combo`] keys sorted in descending order (strongest hand first).
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::combo_pairs::ComboPairs;
    /// use pkcore::arrays::two::Two;
    ///
    /// let mut cp = ComboPairs::default();
    /// cp.add(Combo::COMBO_AA, Two::HAND_AS_AH);
    /// cp.add(Combo::COMBO_KK, Two::HAND_KS_KH);
    /// let keys = cp.key_vec();
    /// assert_eq!(keys[0], Combo::COMBO_AA);
    /// ```
    #[must_use]
    pub fn key_vec(&self) -> Vec<Combo> {
        let mut v: Vec<Combo> = self.0.keys().copied().collect();
        v.sort();
        v.reverse();
        v
    }

    /// Returns an iterator over the [`Combo`] keys in arbitrary order.
    ///
    /// Use [`key_vec`](Self::key_vec) when a sorted order is required.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo_pairs::ComboPairs;
    ///
    /// let cp = ComboPairs::default();
    /// assert_eq!(cp.keys().count(), 0);
    /// ```
    pub fn keys(&self) -> impl Iterator<Item = &Combo> {
        self.0.keys()
    }

    /// Returns the [`Twos`] for the given [`Combo`], or `None` if not present.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::combo_pairs::ComboPairs;
    ///
    /// let cp = ComboPairs::default();
    /// assert!(cp.twos_for_combo(&Combo::COMBO_AA).is_none());
    /// ```
    #[must_use]
    pub fn twos_for_combo(&self, combo: &Combo) -> Option<&Twos> {
        self.0.get(combo)
    }
}

impl Display for ComboPairs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for combo in self.key_vec() {
            match self.twos_for_combo(&combo) {
                Some(twos) => {
                    write!(f, "{:>03}", combo.to_string())?;
                    write!(f, " {:>2} of {:>2}", twos.len(), combo.total_pairs())?;
                    writeln!(f, ": {twos}")?;
                }
                None => {
                    write!(f, "{:>03}:", combo.to_string())?;
                }
            }
        }
        Ok(())
    }
}

impl From<HashMap<Combo, Twos>> for ComboPairs {
    fn from(hash_map: HashMap<Combo, Twos>) -> Self {
        ComboPairs(hash_map)
    }
}
