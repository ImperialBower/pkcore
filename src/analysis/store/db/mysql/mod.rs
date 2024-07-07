use dotenv::dotenv;
use mysql::{Pool, PooledConn};
use std::env;
use std::env::VarError;

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

pub fn get_connection() -> Result<PooledConn, Box<dyn std::error::Error>> {
    let connection_string = connection_string()?;
    let pool = Pool::new(connection_string.as_str())?;
    Ok(pool.get_conn()?)
}
