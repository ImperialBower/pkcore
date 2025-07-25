use crate::PKError;
use crate::rank::Rank;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// The `ranges` module is an attempt to create a progromatic representation of poker ranges.
///
/// - [Poker Ranges & Range Reading](https://www.splitsuit.com/poker-ranges-reading)
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Qualifier {
    #[default]
    ALL,
    SUITED,
    OFFSUIT,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Combo {
    first: Rank,
    second: Rank,
    qualifier: Qualifier,
    higher: Option<bool>,
}

impl Combo {}

impl FromStr for Combo {
    type Err = PKError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.len() {
            0..1 => Err(PKError::InvalidComboIndex),

            _ => Ok(Combo::default()),
        }
    }
}
