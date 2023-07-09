use std::error::Error;
use crate::analysis::hand_rank::HandRankValue;
use crate::bard::Bard;
use serde::{Deserialize, Serialize};
use crate::arrays::five::Five;
use crate::card::Card;
use crate::cards::Cards;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct BinaryCardMap {
    pub bc: Bard,
    pub best: Bard,
    pub rank: HandRankValue,
}

impl BinaryCardMap {
    pub fn generate(path: &str) -> Result<(), Box<dyn Error>> {
        let mut wtr = csv::Writer::from_path(path)?;
        let deck = Cards::deck();

        // for b in deck.combinations(5) {
        //     wtr.serialize(BinaryCardMap::from(b))?;
        // }
        //
        // for b in deck.combinations(7) {
        //     wtr.serialize(BinaryCardMap::from(b))?;
        // }

        wtr.flush()?;

        Ok(())
    }
}

impl From<Vec<&Card>> for BinaryCardMap {
    fn from(v: Vec<&Card>) -> Self {
        todo!()
    }
}

