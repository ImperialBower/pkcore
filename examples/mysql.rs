use mysql::prelude::*;
use mysql::*;
use pkcore::analysis::store::db::mysql::DB;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = DB::get_connection()?;

    let row: Option<String> = conn.query_first("SELECT VERSION()")?;
    println!("MySQL version: {:?}", row.unwrap());

    Ok(())
}
