use csv::Reader;
use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::Shifty;
use rusqlite::Connection;
use std::fs::File;

fn main() {
    let conn = Connection::open("data/hups_07_31_2025.db").unwrap();
    HUPResult::create_table(&conn).unwrap();
    let mut shus = distinct();
    // db_dump(&conn);
    ff(&conn, &mut shus);
}

fn _db_dump(conn: &Connection) {
    for (i, hup) in HUPResult::select_all(conn).into_iter().enumerate() {
        println!("{i} {hup}");
    }
}

fn ff(conn: &Connection, shus: &mut Vec<SortedHeadsUp>) {
    for shu in shus.clone() {
        for shift in shu.shifts() {
            match HUPResult::exists(conn, &shift) {
                true => {
                    println!("{shift} is in DB");
                }
                false => {
                    println!("{shift} is not in DB");
                    return;
                }
            }
        }
        shus.pop();
    }
}

fn distinct() -> Vec<SortedHeadsUp> {
    let file = File::open("../generated/old/distinct_shu.csv").unwrap();
    let mut rdr = Reader::from_reader(file);
    let mut shus: Vec<SortedHeadsUp> = Vec::new();
    for deserialized_shu in rdr.deserialize::<SortedHeadsUp>() {
        shus.push(deserialized_shu.unwrap())
    }
    shus
}
