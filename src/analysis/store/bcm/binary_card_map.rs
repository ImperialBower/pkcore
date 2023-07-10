use crate::analysis::hand_rank::HandRankValue;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::HandRanker;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use crate::{PKError, Pile};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct BinaryCardMap {
    pub bc: Bard,
    pub best: Bard,
    pub rank: HandRankValue,
}

impl BinaryCardMap {
    /// # Errors
    ///
    /// Trips if the Card combinations are off, which shouldn't be possible.
    pub fn generate(path: &str) -> Result<(), Box<dyn Error>> {
        let mut wtr = csv::Writer::from_path(path)?;
        let deck = Cards::deck();

        for b in deck.combinations(5) {
            wtr.serialize(BinaryCardMap::try_from(b))?;
        }

        for b in deck.combinations(7) {
            wtr.serialize(BinaryCardMap::try_from(b))?;
        }

        wtr.flush()?;

        Ok(())
    }
}

impl TryFrom<Five> for BinaryCardMap {
    type Error = PKError;

    fn try_from(five: Five) -> Result<Self, Self::Error> {
        let bard = five.bard();
        let rank = five.hand_rank().value;
        let bcm = BinaryCardMap {
            bc: bard,
            best: bard,
            rank,
        };
        Ok(bcm)
    }
}

impl TryFrom<Seven> for BinaryCardMap {
    type Error = PKError;

    fn try_from(seven: Seven) -> Result<Self, Self::Error> {
        let (rank, five) = seven.hand_rank_value_and_hand();
        let bcm = BinaryCardMap {
            bc: seven.bard(),
            best: five.bard(),
            rank,
        };
        Ok(bcm)
    }
}

impl TryFrom<Vec<Card>> for BinaryCardMap {
    type Error = PKError;

    fn try_from(v: Vec<Card>) -> Result<Self, Self::Error> {
        match v.len() {
            5 => Ok(BinaryCardMap::try_from(Five::try_from(v)?)?),
            7 => Ok(BinaryCardMap::try_from(Seven::try_from(v)?)?),
            _ => Ok(BinaryCardMap::default()),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__store__bcm__binary_card_map_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn try_from__five() {
        let five = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();

        let sut = BinaryCardMap::try_from(five).unwrap();

        assert_eq!(sut.rank, 1);
        assert_eq!(sut.bc, Bard(4_362_862_139_015_168));
        assert_eq!(sut.best, Bard(4_362_862_139_015_168));
    }

    #[test]
    fn try_from__seven() {
        let seven = Seven::from_str("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠").unwrap();
        let five = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();

        let sut = BinaryCardMap::try_from(seven).unwrap();

        assert_eq!(sut.rank, 1);
        assert_eq!(seven.cards(), Cards::from(sut.bc));
        assert_eq!(five.cards(), Cards::from(sut.best));
        assert_eq!(sut.bc, Bard(4_468_415_255_281_664));
        assert_eq!(sut.best, Bard(4_362_862_139_015_168));
    }

    /// This test actually surprises me.
    #[test]
    fn from_five__default() {
        let bcm = BinaryCardMap::try_from(Five::default());
        assert!(bcm.is_ok());
        assert_eq!(BinaryCardMap::default(), bcm.unwrap());
    }
}
