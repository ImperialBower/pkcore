use mysql::*;
use pkcore::analysis::store::db::mysql::{DbConnectOps, HeadsUpRawResult};
use pkcore::analysis::store::db::mysql::MySqlDB;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::util::csv::distinct_shus_from_csv_as_masked_vec;

/// `cargo run --example mysql`
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut distinct = distinct_shus_from_csv_as_masked_vec();

    let mut conn = MySqlDB::get_connection()?;

    let hups = HeadsUpRawResult::all(&mut conn).unwrap();

    for (i, hup) in hups.into_iter().enumerate() {
        println!("#{i} {}", SortedHeadsUp::try_from(&hup)?);
    }

    Ok(())
}
