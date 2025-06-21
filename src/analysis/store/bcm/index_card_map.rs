use crate::PKError;
use crate::analysis::hand_rank::HandRankValue;
use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::card::Card;
use crate::cards::Cards;
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// # V1 - Cards
///
/// At first I tried using the `Cards` struct for my cards and best fields, but `IndexSet` doesn't
/// support `Serde`. Thus, I needed to use a vector. The problem is that then I need to turn off
/// csv headers because, as rust-csv is so kind to tell me, "cannot serialize sequence container
/// inside struct when writing headers from structs." I don't want to do that. Still, let's see what
/// happens...
///
/// ```txt
/// A♠,K♠,Q♠,J♠,T♠,A♠,K♠,Q♠,J♠,T♠,1
/// A♠,K♠,Q♠,J♠,9♠,A♠,K♠,Q♠,J♠,9♠,323
/// A♠,K♠,Q♠,J♠,8♠,A♠,K♠,Q♠,J♠,8♠,324
/// A♠,K♠,Q♠,J♠,7♠,A♠,K♠,Q♠,J♠,7♠,325
/// ```
///
/// Oh no, that's not going to do. This is a comma separated data structure. Sure, I could us a
/// different delimiter, or figure out how to get quotes around a vector, which I'm feeling would
/// cause a whole mess of confusion. Let us try `String`.
///
/// # v2 - Strings
///
/// The nice thing about this is the amount of groundwork we did on Card index strings. One of my
/// big obsessions is that I want my logs to be as readable as possible. I shouldn't have to squint
/// and dig to figure out what's going on.
///
///
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct IndexCardMap {
    pub cards: String,
    pub best: String,
    pub rank: HandRankValue,
}

impl IndexCardMap {
    /// OK, this is the old school way of generating serialized data. Next step
    /// is to try to do the same with an embedded DB like
    /// [sled](https://github.com/spacejam/sled).
    ///
    /// # Errors
    ///
    /// Trips if the Card combinations are off, which shouldn't be possible.
    pub fn generate_csv(path: &str) -> Result<(), Box<dyn Error>> {
        let mut wtr = WriterBuilder::new().has_headers(false).from_path(path)?;

        let deck = Cards::deck();

        for b in deck.combinations(5) {
            if let Ok(bcm) = IndexCardMap::try_from(b) {
                wtr.serialize(bcm)?;
            }
        }

        for b in deck.combinations(7) {
            if let Ok(bcm) = IndexCardMap::try_from(b) {
                wtr.serialize(bcm)?;
            }
        }

        wtr.flush()?;

        Ok(())
    }
}

impl TryFrom<Five> for IndexCardMap {
    type Error = PKError;

    fn try_from(five: Five) -> Result<Self, Self::Error> {
        let cards = five.to_string();
        let rank = five.hand_rank().value;
        let icm = IndexCardMap {
            cards: cards.clone(),
            best: cards,
            rank,
        };
        Ok(icm)
    }
}

impl TryFrom<Seven> for IndexCardMap {
    type Error = PKError;

    fn try_from(seven: Seven) -> Result<Self, Self::Error> {
        let (rank, five) = seven.hand_rank_value_and_hand();
        let icm = IndexCardMap {
            cards: seven.to_string(),
            best: five.to_string(),
            rank,
        };
        Ok(icm)
    }
}

impl TryFrom<Vec<Card>> for IndexCardMap {
    type Error = PKError;

    fn try_from(v: Vec<Card>) -> Result<Self, Self::Error> {
        match v.len() {
            5 => Ok(IndexCardMap::try_from(Five::try_from(v)?)?),
            7 => Ok(IndexCardMap::try_from(Seven::try_from(v)?)?),
            _ => Ok(IndexCardMap::default()),
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

        let sut = IndexCardMap::try_from(five).unwrap();

        assert_eq!(sut.rank, 1);
        assert_eq!(five, Five::from_str(sut.cards.as_str()).unwrap());
        assert_eq!(five, Five::from_str(sut.best.as_str()).unwrap());
    }

    #[test]
    fn try_from__seven() {
        let seven = Seven::from_str("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠").unwrap();
        let five = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();

        let sut = IndexCardMap::try_from(seven).unwrap();

        assert_eq!(sut.rank, 1);
        assert_eq!(seven, Seven::from_str(sut.cards.as_str()).unwrap());
        assert_eq!(five, Five::from_str(sut.best.as_str()).unwrap());
    }

    /// I don't care about this.
    ///
    /// UPDATE: Why am I ignoring this?
    #[test]
    fn from_five__default() {
        let sut = IndexCardMap::try_from(Five::default());
        assert!(sut.is_ok());
        assert_eq!(IndexCardMap::default(), sut.unwrap());
    }
}
