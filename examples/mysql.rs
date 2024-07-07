use mysql::prelude::*;
use mysql::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pkcore::analysis::store::db::mysql::get_connection()?;

    let row: Option<String> = conn.query_first("SELECT VERSION()")?;
    println!("MySQL version: {:?}", row.unwrap());

    Ok(())
}
