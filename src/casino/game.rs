use crate::casino::cashier::chips::Chips;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForcedBets {
    pub small_blind: Chips,
    pub big_blind: Chips,
    pub ante: Chips,
}

impl ForcedBets {
    #[must_use]
    pub fn new(small_blind: usize, big_blind: usize) -> Self {
        ForcedBets {
            small_blind: Chips::new(small_blind),
            big_blind: Chips::new(big_blind),
            ante: Chips::new(0),
        }
    }

    #[must_use]
    pub fn new_with_ante(small_blind: usize, big_blind: usize, ante: usize) -> Self {
        ForcedBets {
            small_blind: Chips::new(small_blind),
            big_blind: Chips::new(big_blind),
            ante: Chips::new(ante),
        }
    }
}

impl std::fmt::Display for ForcedBets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ante.is_empty() {
            write!(f, "SB: {}, BB: {}", self.small_blind, self.big_blind)
        } else {
            write!(
                f,
                "SB: {}, BB: {}, Ante: {}",
                self.small_blind, self.big_blind, self.ante
            )
        }
    }
}
