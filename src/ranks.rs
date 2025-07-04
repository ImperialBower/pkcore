use crate::rank::Rank;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Ranks(Vec<Rank>);

impl Ranks {
    // pub fn into_u16(&self) -> u16 {
    //     self.iter.sum()
    // }
    // pub fn iter(&self) -> impl Iterator<Item = &Rank> {
    //     self.0.iter()
    // }

    #[must_use]
    pub fn sum_or(&self) -> u16 {
        let mut sum = 0;
        for rank in &self.0 {
            sum |= rank.rank_bit_flag();
        }
        sum
    }

    #[must_use]
    pub fn vec(&self) -> &Vec<Rank> {
        &self.0
    }
}

impl From<Vec<Rank>> for Ranks {
    fn from(ranks: Vec<Rank>) -> Self {
        Ranks(ranks)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod ranks_tests {
    use super::*;

    #[test]
    fn sum_or() {
        let ranks = Ranks::from(vec![Rank::ACE, Rank::ACE, Rank::KING, Rank::QUEEN]);

        assert_eq!(ranks.sum_or(), 0b1110000000000);
    }
}
