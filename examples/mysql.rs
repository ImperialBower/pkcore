use mysql::prelude::*;
use mysql::*;
use pkcore::analysis::store::db::mysql::connection_string;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection_string = connection_string()?;
    let pool = Pool::new(connection_string.as_str())?;
    let mut conn = pool.get_conn()?;

    let row: Option<String> = conn.query_first("SELECT VERSION()")?;
    println!("MySQL version: {:?}", row.unwrap());

    Ok(())
}
