use rusqlite::Connection;
use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::MASKED_DISTINCT;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::util::terminal::receive_usize;

fn main() {
    env_logger::init();
    loop {
        read_input();
    }
}

fn read_input() {
    let mut distinct = Vec::from_iter(MASKED_DISTINCT.clone());
    let mut distinct = MASKED_DISTINCT.clone();

    let conn = Connection::open("generated/hups.db").unwrap();
    HUPResult::create_table(&conn).expect("TODO: panic message");

    let i = receive_usize("How many runs? ");
    println!("Processing {i} hands.");

    let mut x = 0usize;

    while x < i {
        let shu = distinct.nex
        if !check_exists(&conn, )
    }


}

fn check_exists(conn: &Connection, shu: &SortedHeadsUp) -> bool {
    HUPResult::exists(&conn, &shu)
}