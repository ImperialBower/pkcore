use dotenv::dotenv;
use mysql::{Pool, PooledConn};
use std::env;
use std::env::VarError;
use crate::arrays::hole_cards::twos::Twos;
use crate::{Pile, PKError};

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
pub struct HeadsUpResult {
    higher: u64,
    lower: u64,
    higher_wins: u64,
    lower_wins: u64,
    ties: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeadsUpQuery {
    higher: u64,
    lower: u64,
}

impl TryFrom<Twos> for HeadsUpQuery {
    type Error = PKError;

    fn try_from(twos: Twos) -> Result<Self, Self::Error> {
        match twos.len() {
            1 => Err(PKError::NotEnoughHands),
            2 => Ok(HeadsUpQuery {
                higher: twos.0[0].bard().as_u64(),
                lower: twos.0[1].bard().as_u64(),
            }),
            _ => Err(PKError::TooManyHands),
        }
    }
}
