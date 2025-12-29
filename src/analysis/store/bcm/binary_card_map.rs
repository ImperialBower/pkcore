use crate::analysis::hand_rank::HandRankValue;
use crate::analysis::store::db::sqlite::Sqlable;
use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use crate::{PKError, Pile};
use csv::Reader;
use csv::WriterBuilder;
use rusqlite::{Connection, named_params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

/// This code is brutal, heavy, and wonderful. It is an optimization that makes things much slower
/// in the short term, and MUCH faster in the long term. Eventually, we will want containers that
/// have all this stuff loaded for bear. We're not there yet.
///
/// TODO TD: Add logging
#[allow(clippy::unwrap_used)]
pub static BC_RANK_HASHMAP: std::sync::LazyLock<HashMap<Bard, FiveBCM>> = std::sync::LazyLock::new(|| {
    let mut m = HashMap::new();
    let file = File::open(SevenFiveBCM::get_csv_filepath()).unwrap();
    let mut rdr = Reader::from_reader(file);

    for result in rdr.deserialize() {
        let bcm: SevenFiveBCM = result.unwrap();
        m.insert(bcm.bc, FiveBCM::from(bcm));
    }
    m
});

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FiveBCM {
    pub bc: Bard,
    pub rank: HandRankValue,
}

impl FiveBCM {
    #[must_use]
    pub fn new(bc: Bard, rank: HandRankValue) -> FiveBCM {
        FiveBCM { bc, rank }
    }
}

impl From<SevenFiveBCM> for FiveBCM {
    fn from(bcm: SevenFiveBCM) -> Self {
        FiveBCM::new(bcm.best, bcm.rank)
    }
}

/// Way of easily storing `Card` `HandRanks` using bitflags.
///
/// ## From inside the struct, now refactored
///
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
///
/// This code is all moving to the `SQLable` trait.
/// ```txt
/// pub fn sqlite_create_table(conn: &Connection) -> rusqlite::Result<usize> {
///     conn.execute(
///         "create table if not exists bcm (
///         bc integer primary key,
///         best integer not null,
///         rank integer not null
///      )",
///         [],
///     )
/// }
/// ```
///
/// ```txt
/// pub fn sqlite_insert_bcm(conn: &Connection, bcm: &BinaryCardMap) -> rusqlite::Result<usize> {
///     let mut stmt =
///         conn.prepare("INSERT INTO bcm (bc, best, rank) VALUES (:bc, :best, :rank)")?;
///     stmt.execute(named_params! {
///         ":bc": bcm.bc.as_u64(),
///         ":best": bcm.best.as_u64(),
///         ":rank": u64::from(bcm.rank)
///     })
/// }
///
/// pub fn sqlite_select_bcm(conn: &Connection, bc: &Bard) -> Option<BinaryCardMap> {
///     let mut stmt = conn
///         .prepare("SELECT bc, best, rank FROM bcm WHERE bc=:bc")
///         .ok()?;
///
///     let mut rows = stmt
///         .query_map(named_params! {":bc": bc.as_u64()}, |row| {
///             let bc: u64 = row.get(0)?;
///             let best: u64 = row.get(1)?;
///             let rank: u16 = row.get(2)?;
///
///             let bcm = BinaryCardMap {
///                 bc: Bard::from(bc),
///                 best: Bard::from(best),
///                 rank,
///             };
///             Ok(bcm)
///         })
///         .ok()?;
///
///     let result = rows.next().ok_or(rusqlite::Error::InvalidQuery).ok()?;
///     let bcm = result.ok()?;
///
///     Some(bcm)
/// }
/// ```
/// TODO: Implement display trait.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct SevenFiveBCM {
    pub bc: Bard,
    pub best: Bard,
    pub rank: HandRankValue,
}

impl SevenFiveBCM {
    /// This file is a little under 5GB in size. Please ask the author for a link
    /// if you don't want to generate it yourself with the `examples/generate_bcm.rs` utility.
    /// TBH, I don't remember how long it took to generate.
    pub const DEFAULT_PKCORE_75BCM_CSV_PATH: &'static str = "generated/bcm.original.csv";

    #[must_use]
    pub fn get_csv_filepath() -> String {
        std::env::var("PKCORE_75BCM_CSV_PATH")
            .unwrap_or_else(|_| SevenFiveBCM::DEFAULT_PKCORE_75BCM_CSV_PATH.to_string())
    }

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
            if let Ok(bcm) = SevenFiveBCM::try_from(b) {
                wtr.serialize(bcm)?;
            }
        }

        for b in deck.combinations(7) {
            if let Ok(bcm) = SevenFiveBCM::try_from(b) {
                wtr.serialize(bcm)?;
            }
        }

        wtr.flush()?;

        Ok(())
    }
}

impl Sqlable<SevenFiveBCM, Bard> for SevenFiveBCM {
    fn create_table(conn: &Connection) -> rusqlite::Result<usize> {
        conn.execute(
            "create table if not exists bcm (
            bc integer primary key,
            best integer not null,
            rank integer not null
         )",
            [],
        )
    }

    fn exists(_conn: &Connection, _record: &Bard) -> bool {
        todo!()
    }

    fn insert(conn: &Connection, bcm: &SevenFiveBCM) -> rusqlite::Result<bool> {
        let mut stmt = conn.prepare("INSERT INTO bcm (bc, best, rank) VALUES (:bc, :best, :rank)")?;
        stmt.execute(named_params! {
            ":bc": bcm.bc.as_u64(),
            ":best": bcm.best.as_u64(),
            ":rank": u64::from(bcm.rank)
        })?;
        Ok(true)
    }

    fn insert_many(_conn: &Connection, _records: Vec<&SevenFiveBCM>) -> rusqlite::Result<usize> {
        todo!()
    }

    fn select(conn: &Connection, bc: &Bard) -> Option<SevenFiveBCM> {
        let mut stmt = conn.prepare("SELECT bc, best, rank FROM bcm WHERE bc=:bc").ok()?;

        let bcm = stmt
            .query_row(named_params! {":bc": bc.as_u64()}, |row| {
                let bc: u64 = row.get(0)?;
                let best: u64 = row.get(1)?;
                let rank: u16 = row.get(2)?;

                let bcm = SevenFiveBCM {
                    bc: Bard::from(bc),
                    best: Bard::from(best),
                    rank,
                };
                Ok(bcm)
            })
            .ok()?;

        Some(bcm)
    }

    fn select_all(_conn: &Connection) -> Vec<SevenFiveBCM> {
        todo!()
    }
}

impl TryFrom<Five> for SevenFiveBCM {
    type Error = PKError;

    fn try_from(five: Five) -> Result<Self, Self::Error> {
        let bard = five.bard();
        let rank = five.hand_rank().value;
        let bcm = SevenFiveBCM {
            bc: bard,
            best: bard,
            rank,
        };
        Ok(bcm)
    }
}

impl TryFrom<Seven> for SevenFiveBCM {
    type Error = PKError;

    fn try_from(seven: Seven) -> Result<Self, Self::Error> {
        let (rank, five) = seven.hand_rank_value_and_hand();
        let bcm = SevenFiveBCM {
            bc: seven.bard(),
            best: five.bard(),
            rank,
        };
        Ok(bcm)
    }
}

impl TryFrom<Vec<Card>> for SevenFiveBCM {
    type Error = PKError;

    fn try_from(v: Vec<Card>) -> Result<Self, Self::Error> {
        match v.len() {
            5 => Ok(SevenFiveBCM::try_from(Five::try_from(v)?)?),
            7 => Ok(SevenFiveBCM::try_from(Seven::try_from(v)?)?),
            _ => Ok(SevenFiveBCM::default()),
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

        let sut = SevenFiveBCM::try_from(five).unwrap();

        assert_eq!(sut.rank, 1);
        assert_eq!(sut.bc, Bard(4_362_862_139_015_168));
        assert_eq!(sut.best, Bard(4_362_862_139_015_168));
    }

    #[test]
    fn try_from__seven() {
        let seven = Seven::from_str("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠").unwrap();
        let five = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();

        let sut = SevenFiveBCM::try_from(seven).unwrap();

        assert_eq!(sut.rank, 1);
        assert_eq!(seven.cards(), Cards::from(sut.bc));
        assert_eq!(five.cards(), Cards::from(sut.best));
        assert_eq!(sut.bc, Bard(4_468_415_255_281_664));
        assert_eq!(sut.best, Bard(4_362_862_139_015_168));
    }

    /// This test actually surprises me.
    #[test]
    fn from_five__default() {
        let bcm = SevenFiveBCM::try_from(Five::default());
        assert!(bcm.is_ok());
        assert_eq!(SevenFiveBCM::default(), bcm.unwrap());
    }

    /// I'm just going to throw everything into one unit test for now. Yes, I am being lazy,
    /// but as the Larry Wall, the inventor of Perl says, laziness is a virtue in a programmer.
    #[test]
    fn sqlite() {
        let conn = Connect::in_memory_connection().unwrap().connection;
        SevenFiveBCM::create_table(&conn).unwrap();
        SevenFiveBCM::insert(&conn, &TestData::spades_royal_flush_bcm()).unwrap();

        assert!(SevenFiveBCM::select(&conn, &TestData::spades_royal_flush_bcm().bc).is_some());
        assert!(SevenFiveBCM::select(&conn, &TestData::spades_king_high_flush_bcm().bc).is_none());
    }
}
