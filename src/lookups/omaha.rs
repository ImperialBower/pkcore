use fudd::games::holdem::hand::Hand;

pub struct EightOrBetter;

impl EightOrBetter {
    pub fn from(collapsed: u32) -> Self {
        let mut ranks = hand.ranks();
        ranks.sort_unstable();
        let mut unique_ranks = ranks.clone();
        unique_ranks.dedup();
        if unique_ranks.len() < 5 {
            EightOrBetter::Invalid
        } else if unique_ranks[0] < 8 {
            EightOrBetter::Invalid
        } else {
            EightOrBetter::Valid
        }
    }
}