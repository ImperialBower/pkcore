use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::Masked;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::util::terminal::receive_usize;
use pkcore::{Pile, Shifty};
use rusqlite::Connection;

/// `cargo run --example inc`
fn main() {
    env_logger::init();

    let mut distinct = get_distinct();
    let conn = get_connection();

    loop {
        read_input(&conn, &mut distinct);
    }
}

fn get_distinct() -> Vec<Masked> {
    println!("Loading distinct entries...");

    let shus = SortedHeadsUp::read_csv("data/csv/shus/distinct_masked_shus.csv").unwrap();
    let mut distinct = Masked::parse_as_vectors(&*shus);

    distinct.reverse();
    distinct
}

fn get_connection() -> Connection {
    let conn = Connection::open("../../generated/hups_07_31_2025.db").unwrap();
    HUPResult::create_table(&conn).expect("TODO: panic message");
    conn
}

fn read_input(conn: &Connection, distinct: &mut Vec<Masked>) {
    let mut x = 0usize;
    let i = receive_usize("How many runs? ");
    println!("Processing {i} hands.");

    while x < i {
        let Some(masked) = distinct.pop() else {
            println!("None remaining.");
            return;
        };
        if check_exists(&conn, &masked.shu) {
            println!("{} exists!", masked.shu);
            continue;
        }
        println!("{} checking...", masked.shu);
        match check_shifts(&conn, &masked) {
            None => {
                println!("   no entry for {}", masked.shu);
                x = x + 1;
            }
            Some(hup) => {
                println!("   {} exists as {hup}", masked.shu);
                insert_distinct(&conn, &masked.shu, &hup);
            }
        }
    }
}

fn check_exists(conn: &Connection, shu: &SortedHeadsUp) -> bool {
    HUPResult::exists(&conn, &shu)
}

fn check_shifts(conn: &Connection, masked: &Masked) -> Option<HUPResult> {
    for mask in masked.shifts() {
        match HUPResult::select(&conn, &mask.shu) {
            None => continue,
            Some(hup) => return Some(hup),
        }
    }
    None
}

fn insert_distinct(conn: &Connection, shu: &SortedHeadsUp, hup: &HUPResult) {
    let distinct = HUPResult {
        higher: shu.higher.bard(),
        lower: shu.lower.bard(),
        higher_wins: hup.higher_wins,
        lower_wins: hup.lower_wins,
        ties: hup.ties,
    };
    match HUPResult::insert(&conn, &distinct) {
        Ok(_) => {}
        Err(e) => {
            println!("Unable to insert {distinct}");
            println!("Error: {:?}", e);
        }
    }
}
