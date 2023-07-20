use crate::bard::Bard;

pub struct HUPResult {
    pub higher: Bard,
    pub lower: Bard,
    pub higher_wins: u64,
    pub lower_wins: u64,
    pub ties_wins: u64,
}

impl HUPResult {}
