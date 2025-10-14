#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct ForcedBets {
    pub small_blind: usize,
    pub big_blind: usize,
    pub ante: usize,
}

impl ForcedBets {
    #[must_use]
    pub fn new(small_blind: usize, big_blind: usize) -> Self {
        ForcedBets {
            small_blind,
            big_blind,
            ante: 0,
        }
    }

    #[must_use]
    pub fn new_with_ante(small_blind: usize, big_blind: usize, ante: usize) -> Self {
        ForcedBets {
            small_blind,
            big_blind,
            ante,
        }
    }
}

impl std::fmt::Display for ForcedBets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ante == 0 {
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
