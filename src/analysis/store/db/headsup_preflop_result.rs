use crate::bard::Bard;

pub struct HUPResult {
    higher: Bard,
    lower: Bard,
    higher_wins: u64,
    lower_wins: u64,
    ties_wins: u64,
}