use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::mysql::{DbConnectOps, HeadsUpRawResult, MySqlDB};
use pkcore::analysis::store::db::sqlite::Sqlitable;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::error::Error;

/// # unique_transfer
///
/// Goal of this example is to create a sqlite db with unique hands from
/// the data in the distinct hands. The big problem is that we are seeing
/// a difference in the sum of the results in different runs.
///
/// `cargo run --example ut`
fn main() {
    env_logger::init();

    let distinct = get_distinct_hups().unwrap();
    let conn = get_connection();

    for (i, hurr) in distinct.iter().enumerate() {
        // println!("{i} {hurr}");
        process_distinct(&conn, hurr);
    }
}

fn process_distinct(conn: &Connection, hurr: &HeadsUpRawResult) {
    let mut mappie: HashMap<SortedHeadsUp, HashSet<HUPResult>> = HashMap::new();

    let shu = SortedHeadsUp::try_from(hurr).unwrap();
    let r = HUPResult::select(conn, &shu);
    match r {
        None => {
            // println!("{} not found", shu);
        }
        Some(result) => {
            let huprhurr = HUPResult::from(*hurr);
            if result != huprhurr {
                let shu = SortedHeadsUp::try_from(hurr).unwrap();
                println!("{} {}", shu.higher, shu.lower);
                println!("    sqlite: {result} !=",);
                println!("    mysql:  {huprhurr}",);
                println!("    sum:    {}", result.sum());
                println!("    sum:    {}", huprhurr.sum());
            } else {
                // println!("{} found", result);
            }
            // assert_eq!(result, HUPResult::from(*hurr));
            // println!("{} found", result);
        }
    }
}

fn get_connection() -> Connection {
    let conn = Connection::open("generated/hups.db").unwrap();
    HUPResult::create_table_unless_exists(&conn).expect("TODO: panic message");
    conn
}

fn get_distinct_hups() -> Result<HashSet<HeadsUpRawResult>, Box<dyn Error>> {
    let mut conn = MySqlDB::get_connection()?;
    Ok(HeadsUpRawResult::all_as_hashset(&mut conn)?)
}
