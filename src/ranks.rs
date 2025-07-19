use crate::PKError;
use crate::rank::Rank;
use std::str::FromStr;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Ranks(Vec<Rank>);

impl Ranks {
    #[must_use]
    pub fn count_ones(&self) -> u32 {
        self.sum_or().count_ones()
    }

    /// Returns the OR sum of all the `Rank` bit flags.
    ///
    /// Originally created to be used for `Razz` hand evaluations.
    #[must_use]
    pub fn sum_or(&self) -> u16 {
        self.0.iter().fold(0, |acc, rank| acc | rank.rank_bit_flag())
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

impl FromStr for Ranks {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ranks = Vec::new();
        for s in s.split_whitespace() {
            let c = Rank::from_str(s)?;
            if c.is_blank() {
                return Err(PKError::InvalidRankIndex);
            }
            ranks.push(c);
        }
        if ranks.is_empty() {
            Err(PKError::InvalidRankIndex)
        } else {
            Ok(Ranks::from(ranks))
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod ranks_tests {
    use super::*;
    use crate::Pile;
    use crate::cards::Cards;
    use std::str::FromStr;

    #[test]
    fn sum_or() {
        let ranks = Ranks::from(vec![Rank::ACE, Rank::ACE, Rank::KING, Rank::QUEEN]);

        assert_eq!(ranks.sum_or(), 0b1110000000000);
    }

    #[test]
    fn pile__ranks() {
        let wheel = Cards::from_str("AS 2C 3D 4h 5c").unwrap().ranks();

        assert_eq!(wheel.sum_or(), 0b1000000001111);
    }
}
