use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::Masked;
use pkcore::util::csv::distinct_shus_from_csv_as_masked_vec;
use pkcore::util::terminal::Terminal;
use rusqlite::Connection;

/// `cargo run --example insert_distinct_reverse`
fn main() {
    env_logger::init();

    let mut distinct = distinct_shus_from_csv_as_masked_vec();
    distinct.reverse();
    let conn = get_connection();

    loop {
        read_input(&conn, &mut distinct);
    }
}
fn read_input(conn: &Connection, distinct: &mut Vec<Masked>) {
    let mut x = 0usize;
    let i = Terminal::receive_usize("How many runs? ");
    println!("Processing {i} hands.");

    while x < i {
        let Some(masked) = distinct.pop() else {
            println!("None remaining.");
            return;
        };
        if HUPResult::exists(&conn, &masked.shu) {
            println!("{} exists!", masked.shu);
            continue;
        } else {
            println!("Calculating {}", masked.shu);
            let hupr = HUPResult::from(&masked.shu);
            match HUPResult::insert(&conn, &hupr) {
                Ok(_) => {
                    println!("... inserted");
                }
                Err(e) => {
                    println!("Unable to insert {hupr}");
                    println!("Error: {:?}", e);
                }
            }
        }
        x = x + 1;
    }
}

fn get_connection() -> Connection {
    let conn = Connection::open("../../generated/hups.db").unwrap();
    HUPResult::create_table(&conn).expect("TODO: panic message");
    conn
}
