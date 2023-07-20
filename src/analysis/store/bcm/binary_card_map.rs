use crate::analysis::hand_rank::HandRankValue;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::HandRanker;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use crate::{PKError, Pile};
use csv::WriterBuilder;
use rusqlite::{named_params, Connection};
use serde::{Deserialize, Serialize};
use std::error::Error;

// TODO: Implement display trait.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct BinaryCardMap {
    pub bc: Bard,
    pub best: Bard,
    pub rank: HandRankValue,
}

impl BinaryCardMap {
    /// OK, this is the old school way of generating serialized data. Next step
    /// is to try to do the same with an embedded DB like
    /// [sled](https://github.com/spacejam/sled).
    ///
    /// # Errors
    ///
    /// Trips if the Card combinations are off, which shouldn't be possible.
    pub fn generate_csv(path: &str) -> Result<(), Box<dyn Error>> {
        let mut wtr = WriterBuilder::new().has_headers(true).from_path(path)?;

        let deck = Cards::deck();

        for b in deck.combinations(5) {
            if let Ok(bcm) = BinaryCardMap::try_from(b) {
                wtr.serialize(bcm)?;
            }
        }

        for b in deck.combinations(7) {
            if let Ok(bcm) = BinaryCardMap::try_from(b) {
                wtr.serialize(bcm)?;
            }
        }

        wtr.flush()?;

        Ok(())
    }

    /// Now that we got it working with an example, let's codify it inside of our struct. We'll
    /// use this to write some unit tests validating that our sqlite work. It's always better to
    /// have your work codified into automated unit tests so that your CI server will scream if
    /// you start breaking things. _Back in the olden times, we would have these things called
    /// manual regression tests, where armies of talented QA engineers would painstakingly verify
    /// that us stupid coders didn't break something with all our messing about. Now, thanks
    /// to unit testing we get all that for free, and they can focus on exploratory testing, we're
    /// all the really fun bugs are. If they're busy doing the simple things, they won't have time
    /// for the really creative destruction that QA engineers excel at. It's taken companies a very
    /// long time to realize that they just can't hire enough people to test every possible
    /// combination of things given how complex our systems are growing._
    ///
    /// # Errors
    ///
    /// Throws an error if rusqlite isn't able to create the table.
    pub fn sqlite_create_table(conn: &Connection) -> rusqlite::Result<usize> {
        conn.execute(
            "create table if not exists bcm (
            bc integer primary key,
            best integer not null,
            rank integer not null
         )",
            [],
        )
    }

    /// # Errors
    ///
    /// Throws an error if rusqlite isn't able to insert the record into the table. Should not
    /// throw if the record is already there.
    pub fn sqlite_insert_bcm(conn: &Connection, bcm: &BinaryCardMap) -> rusqlite::Result<usize> {
        let mut stmt =
            conn.prepare("INSERT INTO bcm (bc, best, rank) VALUES (:bc, :best, :rank)")?;
        stmt.execute(named_params! {
            ":bc": bcm.bc.as_u64(),
            ":best": bcm.best.as_u64(),
            ":rank": u64::from(bcm.rank)
        })
    }

    pub fn sqlite_select_bcm(conn: &Connection, bc: &Bard) -> Option<BinaryCardMap> {
        let mut stmt = conn
            .prepare("SELECT bc, best, rank FROM bcm WHERE bc=:bc")
            .ok()?;

        let mut rows = stmt
            .query_map(named_params! {":bc": bc.as_u64()}, |row| {
                let bc: u64 = row.get(0)?;
                let best: u64 = row.get(1)?;
                let rank: u16 = row.get(2)?;

                let bcm = BinaryCardMap {
                    bc: Bard::from(bc),
                    best: Bard::from(best),
                    rank,
                };
                Ok(bcm)
            })
            .ok()?;

        let result = rows.next().ok_or(rusqlite::Error::InvalidQuery).ok()?;
        let bcm = result.ok()?;

        Some(bcm)
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
    use crate::analysis::store::db::sqlite::Connect;
    use crate::util::data::TestData;
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

    /// I'm just going to throw everything into one unit test for now. Yes, I am being lazy,
    /// but as the Larry Wall, the inventor of Perl says, laziness is a virtue in a programmer.
    #[test]
    fn sqlite() {
        let conn = Connect::in_memory_connection().unwrap().connection;
        BinaryCardMap::sqlite_create_table(&conn).unwrap();
        let i =
            BinaryCardMap::sqlite_insert_bcm(&conn, &TestData::spades_royal_flush_bcm()).unwrap();

        assert!(
            BinaryCardMap::sqlite_select_bcm(&conn, &TestData::spades_royal_flush_bcm().bc)
                .is_some()
        );
        assert!(BinaryCardMap::sqlite_select_bcm(
            &conn,
            &TestData::spades_king_high_flush_bcm().bc
        )
        .is_none());
    }
}
