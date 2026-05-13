/// Forced bets for a hand: blinds, antes, and (EPIC-29 Phase 6) bring-in.
///
/// Hold'em and Omaha use `small_blind` + `big_blind`; antes are optional.
/// Stud and Razz (EPIC-32 / EPIC-33) leave the blinds at 0 and use
/// `ante` + `bring_in` instead. The `bring_in` field defaults to 0 and is
/// ignored by NLHE / FLHE / PLO; no existing caller is required to pass
/// it.
///
/// # Examples
///
/// ```
/// use pkcore::casino::game::ForcedBets;
///
/// let nlhe = ForcedBets::new(50, 100);
/// assert_eq!(0, nlhe.bring_in);
///
/// let stud = ForcedBets::new_with_ante_and_bring_in(0, 0, 10, 30);
/// assert_eq!(10, stud.ante);
/// assert_eq!(30, stud.bring_in);
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct ForcedBets {
    pub small_blind: usize,
    pub big_blind: usize,
    pub ante: usize,
    /// Bring-in amount used by stud-family variants (EPIC-32 / EPIC-33).
    /// 0 for Hold'em / Omaha; non-zero only when the variant deals
    /// per-seat upcards on 3rd street.
    pub bring_in: usize,
}

impl ForcedBets {
    /// Constructs blinds-only `ForcedBets`. Ante and bring-in default to 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let fb = ForcedBets::new(50, 100);
    /// assert_eq!(50, fb.small_blind);
    /// assert_eq!(100, fb.big_blind);
    /// assert_eq!(0, fb.ante);
    /// assert_eq!(0, fb.bring_in);
    /// ```
    #[must_use]
    pub fn new(small_blind: usize, big_blind: usize) -> Self {
        ForcedBets {
            small_blind,
            big_blind,
            ante: 0,
            bring_in: 0,
        }
    }

    /// Constructs `ForcedBets` with blinds + ante (no bring-in).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let fb = ForcedBets::new_with_ante(25, 50, 5);
    /// assert_eq!(5, fb.ante);
    /// assert_eq!(0, fb.bring_in);
    /// ```
    #[must_use]
    pub fn new_with_ante(small_blind: usize, big_blind: usize, ante: usize) -> Self {
        ForcedBets {
            small_blind,
            big_blind,
            ante,
            bring_in: 0,
        }
    }

    /// Constructs `ForcedBets` for stud-family variants: zero blinds, an
    /// ante per player, and a bring-in for the seat showing the lowest
    /// (Stud Hi) or highest (Razz) upcard on 3rd street.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let fb = ForcedBets::new_with_ante_and_bring_in(0, 0, 10, 30);
    /// assert_eq!(10, fb.ante);
    /// assert_eq!(30, fb.bring_in);
    /// ```
    #[must_use]
    pub fn new_with_ante_and_bring_in(small_blind: usize, big_blind: usize, ante: usize, bring_in: usize) -> Self {
        ForcedBets {
            small_blind,
            big_blind,
            ante,
            bring_in,
        }
    }
}

impl std::fmt::Display for ForcedBets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.ante, self.bring_in) {
            (0, 0) => write!(f, "SB: {}, BB: {}", self.small_blind, self.big_blind),
            (_, 0) => write!(
                f,
                "SB: {}, BB: {}, Ante: {}",
                self.small_blind, self.big_blind, self.ante
            ),
            _ => write!(f, "Ante: {}, Bring-in: {}", self.ante, self.bring_in),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__game__tests {
    use super::*;

    #[test]
    fn new_defaults_ante_and_bring_in_to_zero() {
        let fb = ForcedBets::new(50, 100);
        assert_eq!(50, fb.small_blind);
        assert_eq!(100, fb.big_blind);
        assert_eq!(0, fb.ante);
        assert_eq!(0, fb.bring_in);
    }

    #[test]
    fn new_with_ante_leaves_bring_in_at_zero() {
        let fb = ForcedBets::new_with_ante(25, 50, 5);
        assert_eq!(5, fb.ante);
        assert_eq!(0, fb.bring_in);
    }

    #[test]
    fn new_with_ante_and_bring_in() {
        let fb = ForcedBets::new_with_ante_and_bring_in(0, 0, 10, 30);
        assert_eq!(0, fb.small_blind);
        assert_eq!(0, fb.big_blind);
        assert_eq!(10, fb.ante);
        assert_eq!(30, fb.bring_in);
    }

    #[test]
    fn default_is_all_zero() {
        let fb = ForcedBets::default();
        assert_eq!(0, fb.small_blind);
        assert_eq!(0, fb.big_blind);
        assert_eq!(0, fb.ante);
        assert_eq!(0, fb.bring_in);
    }

    #[test]
    fn display_blinds_only() {
        let fb = ForcedBets::new(50, 100);
        assert_eq!("SB: 50, BB: 100", fb.to_string());
    }

    #[test]
    fn display_with_ante() {
        let fb = ForcedBets::new_with_ante(25, 50, 5);
        assert_eq!("SB: 25, BB: 50, Ante: 5", fb.to_string());
    }

    #[test]
    fn display_stud_shape() {
        let fb = ForcedBets::new_with_ante_and_bring_in(0, 0, 10, 30);
        assert_eq!("Ante: 10, Bring-in: 30", fb.to_string());
    }
}
