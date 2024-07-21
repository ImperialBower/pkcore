use mysql::*;
use pkcore::analysis::store::db::mysql::MySqlDB;
use pkcore::analysis::store::db::mysql::{DbConnectOps, HeadsUpRawResult};
use pkcore::arrays::matchups::masked::Masked;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;

/// `cargo run --example mysql`
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MySQL version: {:?}", MySqlDB::version_string().unwrap());

    let mut conn = MySqlDB::get_connection()?;
    let distinct = SortedHeadsUp::distinct_shus_from_csv()?;
    let all = HeadsUpRawResult::all(&mut conn).unwrap();
    let mut unique = HeadsUpRawResult::all_as_hashset(&mut conn)?;

    let mut mappie = HeadsUpRawResult::all_as_shu_hashmap(&mut conn)?;

    println!(
        "DISTINCT: {} ALL: {} UNIQUE: {} hands remaining",
        distinct.len(),
        all.len(),
        unique.len()
    );

    let mut remains: Vec<SortedHeadsUp> = Vec::new();

    for hup in distinct.iter() {
        if mappie.contains_key(&hup) {
            mappie.remove(&hup);
        } else {
            remains.push(*hup);
        }
    }

    println!("remaining {} spillover: {}", mappie.keys().len(), remains.len());

    // for (i, hup) in distinct.into_iter().enumerate() {
    //
    //     println!("#{i} {}", hup);
    // }

    Ok(())
}
