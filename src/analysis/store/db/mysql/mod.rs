use crate::analysis::store::db::headsup_preflop_result::HUPResult;
use crate::arrays::hole_cards::twos::Twos;
use crate::arrays::matchups::masked::Masked;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::{PKError, Pile};
use dotenv::dotenv;
use mockall::automock;
use mysql::prelude::Queryable;
use mysql::{Pool, PooledConn};
use std::env;
use std::env::VarError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub struct MySqlDB;

#[automock]
pub trait DbConnectOps {
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
    fn connection_string() -> mysql::Result<String, VarError>;

    /// # `CoPilot` bringing the snark:
    ///
    /// "This function is a simple wrapper around the mysql `Pool::get_conn` method. It's a little
    /// redundant, but it's a good way to keep the connection logic in one place."
    ///
    /// # Errors
    ///
    /// Returns an error if the connection string is not set properly.
    fn get_connection() -> Result<PooledConn, Box<dyn std::error::Error>>;

    fn version_string() -> Option<String>;
}

impl DbConnectOps for MySqlDB {
    fn connection_string() -> mysql::Result<String, VarError> {
        dotenv().ok();

        let user = env::var("MYSQL_PKDB_USER")?;
        let pwd = env::var("MYSQL_PKDB_PWD")?;
        let host = env::var("MYSQL_PKDB_HOST")?;
        let port = env::var("MYSQL_PKDB_PORT")?;
        let database = env::var("MYSQL_PKDB_DB")?;

        Ok(format!("mysql://{user}:{pwd}@{host}:{port}/{database}"))
    }

    fn get_connection() -> Result<PooledConn, Box<dyn std::error::Error>> {
        let connection_string = MySqlDB::connection_string()?;
        let pool = Pool::new(connection_string.as_str())?;
        Ok(pool.get_conn()?)
    }

    fn version_string() -> Option<String> {
        match MySqlDB::get_connection() {
            Ok(mut conn) => conn.query_first("SELECT VERSION()").unwrap_or(None),
            Err(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeadsUpRawResult {
    pub higher: u64,
    pub lower: u64,
    pub higher_wins: u64,
    pub lower_wins: u64,
    pub ties: u64,
}

impl HeadsUpRawResult {
    /// # Errors
    ///
    /// Throws a `PKError` if unable to query the database.
    pub fn all(conn: &mut PooledConn) -> Result<Vec<Self>, PKError> {
        match conn.query_map(
            "SELECT higher, lower, higher_wins, lower_wins, ties FROM nlh_headsup_result",
            |(higher, lower, higher_wins, lower_wins, ties)| HeadsUpRawResult {
                higher,
                lower,
                higher_wins,
                lower_wins,
                ties,
            },
        ) {
            Ok(q) => Ok(q),
            Err(_) => Err(PKError::SqlError),
        }
    }

    /// # Errors
    ///
    /// Thows a `PKError` if unable to query the database.
    pub fn all_as_hup_results(conn: &mut PooledConn) -> Result<Vec<HUPResult>, PKError> {
        let raw_results = HeadsUpRawResult::all(conn)?;
        Ok(raw_results.into_iter().map(HUPResult::from).collect())
    }
}

pub trait DbHeadsUpRawResultOps {
    /// # Errors
    ///
    /// Throws `PKError` if unable to insert the result.
    fn insert(&self, conn: &mut PooledConn) -> Result<(), PKError>;
}

impl DbHeadsUpRawResultOps for HeadsUpRawResult {
    fn insert(&self, conn: &mut PooledConn) -> Result<(), PKError> {
        match conn.exec_drop(
            "INSERT INTO nlh_headsup_result (higher, lower, higher_wins, lower_wins, ties) VALUES (?, ?, ?, ?, ?)",
            (self.higher, self.lower, self.higher_wins, self.lower_wins, self.ties),
        ) {
            Ok(()) => Ok(()),
            Err(_) => Err(PKError::SqlError),
        }
    }
}

impl Display for HeadsUpRawResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "INSERT INTO `nlh_headsup_result` (`higher`, `lower`, `higher_wins`, `lower_wins`, `ties`) VALUES({},{},{},{},{});", self.higher, self.lower, self.higher_wins, self.lower_wins, self.ties)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeadsUpQuery {
    pub higher: u64,
    pub lower: u64,
}

impl HeadsUpQuery {
    pub fn exists(&self, conn: &mut PooledConn) -> bool {
        self.query(conn).is_ok()
    }

    /// TODO: How do I test this without a database connection?
    ///
    /// # Errors
    ///
    /// Throws `PKError` if unable to insert the result.
    pub fn insert(&self, conn: &mut PooledConn, result: &HUPResult) -> Result<(), PKError> {
        match conn.exec_drop(
            "INSERT INTO nlh_headsup_result (higher, lower, higher_wins, lower_wins, ties) VALUES (?, ?, ?, ?, ?)",
            (
                self.higher,
                self.lower,
                result.higher_wins,
                result.lower_wins,
                result.ties,
            ),
        ) {
            Ok(()) => Ok(()),
            Err(_) => Err(PKError::SqlError),
        }
    }

    /// # Errors
    ///
    /// Throws `PKError::SqlError` if unable to query the database.
    /// Throws `PKError::Fubar` if the match statement somehow gets messed up.
    pub fn query(&self, conn: &mut PooledConn) -> Result<HUPResult, PKError> {
        let Ok(query) = conn.exec_map(
                        "SELECT higher, lower, higher_wins, lower_wins, ties FROM nlh_headsup_result WHERE higher = ? AND lower = ?",
                     (self.higher, self.lower),
                     |(higher, lower, higher_wins, lower_wins, ties)| { HeadsUpRawResult {higher, lower, higher_wins, lower_wins, ties}},
                ) else { return Err(PKError::SqlError) };

        match query.len() {
            0 => Err(PKError::SqlEmptyResult),
            1 => match query.into_iter().next() {
                Some(raw) => Ok(HUPResult::from(raw)),
                None => Err(PKError::Fubar),
            },
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

impl From<Masked> for HeadsUpQuery {
    fn from(masked: Masked) -> Self {
        HeadsUpQuery {
            higher: masked.shu.higher().bard().as_u64(),
            lower: masked.shu.lower().bard().as_u64(),
        }
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
            Err(e) => Err(e),
            Ok(sorted) => Ok(HeadsUpQuery::from(sorted)),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis_store_db_mysql_tests {
    use super::*;

    use mockall::predicate::*;
    use mockall::*;

    mock! {

        DbOpsMock {
            fn insert(&self, conn: &mut PooledConn) -> Result<(), PKError>;
        }
    }

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
    fn test_heads_up_query__insert() {
        let mut mock = MockDbOpsMock::new();
        // let dummy_conn = &mut PooledConn::
        let dummy_result = HeadsUpRawResult {
            higher: 17592186052608,
            lower: 549822922752,
            higher_wins: 523851,
            lower_wins: 1118235,
            ties: 70218,
        }; // Create a dummy HUPResult

        let huq = HeadsUpQuery {
            higher: 17592186052608,
            lower: 549822922752,
        };

        let actual = HeadsUpQuery::from(Masked::from_str("J♦ 9♠ 3♥ 2♠").unwrap());

        // assert_eq!(expected, actual);
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
        let twos = Twos::from_str("J♦ 9♠ 3♥ 2♠").unwrap();
        let huq = HeadsUpQuery::from_str("J♦ 9♠ 3♥ 2♠").unwrap();
    }
}
