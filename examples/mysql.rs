use mysql::*;
use pkcore::analysis::store::db::mysql::{DbConnectOps, HeadsUpRawResult};
use pkcore::analysis::store::db::mysql::MySqlDB;
use pkcore::util::csv::distinct_shus_from_csv_as_masked_vec;

/// `cargo run --example mysql`
fn main() -> Result<(), Box<dyn std::error::Error>> {


    println!("MySQL version: {:?}", MySqlDB::version_string().unwrap());

    let mut conn = MySqlDB::get_connection()?;
    let distinct = distinct_shus_from_csv_as_masked_vec().len();
    let hups = HeadsUpRawResult::all(&mut conn).unwrap().len();
    let remaining = distinct - hups;

    println!("{hups} done {remaining} hands remaining");

    Ok(())
}
