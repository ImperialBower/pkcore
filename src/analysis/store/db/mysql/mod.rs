use crate::analysis::store::db::headsup_preflop_result::HUPResult;
use crate::arrays::hole_cards::twos::Twos;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::{PKError, Pile};
use dotenv::dotenv;
use mysql::prelude::Queryable;
use mysql::{Pool, PooledConn};
use std::env;
use std::env::VarError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub struct DB;

impl DB {
    /// There are several ways that we can create this function. One is to use unwrap or else and
    /// provide default values. Another is to return a Result type and error out if the env vars aren't
    /// set properly. While this will make the work a little harder, by forcing the user to set things
    /// up correctly, there's much less of a risk that some rogue database is running around being
    /// hit from all angles.
    ///
    /// TODO: Work on tightening up the configuration for [pkdb](https://github.com/ImperialBower/pkdb).
    /// Right now it's pretty open ended.
    ///
    /// TODO: This could generate a much better error message for each other the fields.
    ///
    /// # Errors
    ///
    /// This function will return an error if the environment variables are not set.
    pub fn connection_string() -> mysql::Result<String, VarError> {
        dotenv().ok();

        let user = env::var("MYSQL_PKDB_USER")?;
        let pwd = env::var("MYSQL_PKDB_PWD")?;
        let host = env::var("MYSQL_PKDB_HOST")?;
        let port = env::var("MYSQL_PKDB_PORT")?;
        let database = env::var("MYSQL_PKDB_DB")?;

        Ok(format!("mysql://{user}:{pwd}@{host}:{port}/{database}"))
    }

    /// # `CoPilot` bringing the snark:
    ///
    /// This function is a simple wrapper around the mysql `Pool::get_conn` method. It's a little
    /// redundant, but it's a good way to keep the connection logic in one place.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection string is not set properly.
    pub fn get_connection() -> Result<PooledConn, Box<dyn std::error::Error>> {
        let connection_string = DB::connection_string()?;
        let pool = Pool::new(connection_string.as_str())?;
        Ok(pool.get_conn()?)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeadsUpRawResult {
    pub higher: u64,
    pub lower: u64,
    pub higher_wins: u64,
    pub lower_wins: u64,
    pub ties: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeadsUpQuery {
    pub higher: u64,
    pub lower: u64,
}

impl HeadsUpQuery {
    pub fn query(&self, conn: &mut PooledConn) -> Result<HUPResult, PKError> {
        let query = match conn.exec_map(
            "SELECT higher, lower, higher_wins, lower_wins, ties FROM nlh_headsup_result WHERE higher = ? AND lower = ?",
            (self.higher, self.lower),
            |(higher, lower, higher_wins, lower_wins, ties)| { HeadsUpRawResult {higher, lower, higher_wins, lower_wins, ties}},
        ) {
            Ok(q) => q,
            Err(_) => return Err(PKError::from(PKError::SqlError)),
        };

        match query.len() {
            0 => Err(PKError::SqlEmptyResult),
            1 => Ok(HUPResult::from(query.into_iter().next().unwrap())),
            _ => Err(PKError::SqlDuplicateResult),
        }
    }
}

impl Display for HeadsUpQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let high = Two::try_from(Bard::from(self.higher)).unwrap_or(Two::default());
        let low = Two::try_from(Bard::from(self.lower)).unwrap_or(Two::default());
        write!(f, "{}", SortedHeadsUp::new(high, low))
    }
}

impl From<SortedHeadsUp> for HeadsUpQuery {
    fn from(sorted: SortedHeadsUp) -> Self {
        HeadsUpQuery {
            higher: sorted.higher().bard().as_u64(),
            lower: sorted.lower().bard().as_u64(),
        }
    }
}

impl FromStr for HeadsUpQuery {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SortedHeadsUp::from_str(s).map(HeadsUpQuery::from)
    }
}

impl TryFrom<Twos> for HeadsUpQuery {
    type Error = PKError;

    fn try_from(twos: Twos) -> Result<Self, Self::Error> {
        match SortedHeadsUp::try_from(twos) {
            Err(e) => return Err(e),
            Ok(sorted) => Ok(HeadsUpQuery::from(sorted)),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis_store_db_mysql_tests {
    use super::*;

    #[test]
    fn from_str() {
        let expected = HeadsUpQuery {
            higher: 70368748371968,
            lower: 549890031616,
        };

        let actual = HeadsUpQuery::from_str("J♦ 9♠ 3♥ 2♠").unwrap();

        assert_eq!(expected, actual);
        assert_ne!(expected, HeadsUpQuery::from_str("J♦ 9♠ 3♥ 3♠").unwrap());
    }

    #[test]
    fn test_heads_up_query_from_sorted_heads_up() {
        let expected = HeadsUpQuery {
            higher: 70368748371968,
            lower: 549890031616,
        };

        let actual = HeadsUpQuery::from(SortedHeadsUp::new(
            Two::try_from(Bard::from(70368748371968)).unwrap(),
            Two::try_from(Bard::from(549890031616)).unwrap(),
        ));

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_heads_up_query_from_twos() {
        let huq = HeadsUpQuery::from_str("J♦ 9♠ 3♥ 2♠").unwrap();
    }
}
