use mysql::*;
use pkcore::analysis::store::db::mysql::{DbConnectOps, HeadsUpRawResult};
use pkcore::analysis::store::db::mysql::MySqlDB;
use pkcore::util::csv::distinct_shus_from_csv_as_masked_vec;

/// `cargo run --example mysql`
fn main() -> Result<(), Box<dyn std::error::Error>> {


    println!("MySQL version: {:?}", MySqlDB::version_string().unwrap());

    let mut conn = MySqlDB::get_connection()?;
    let distinct = distinct_shus_from_csv_as_masked_vec();
    let all = HeadsUpRawResult::all(&mut conn).unwrap();
    let mut unique = HeadsUpRawResult::all_as_hashset(&mut conn).unwrap();

    println!("DISTINCT: {} ALL: {} UNIQUE: {} hands remaining", distinct.len(), all.len(), unique.len());

    for (i, hup) in distinct.into_iter().enumerate() {

        println!("#{i} {}", hup);
    }

    Ok(())
}
