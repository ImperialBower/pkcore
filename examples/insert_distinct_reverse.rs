use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::Masked;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::util::terminal::receive_usize;
use rusqlite::Connection;

/// `cargo run --example insert_distinct_reverse`
fn main() {
    env_logger::init();

    let mut distinct = get_distinct();
    let conn = get_connection();

    loop {
        read_input(&conn, &mut distinct);
    }
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

fn get_distinct() -> Vec<Masked> {
    println!("Loading distinct entries...");

    let shus = SortedHeadsUp::read_csv("data/csv/shus/distinct_masked_shus.csv").unwrap();
    Masked::parse_as_vectors(&*shus)
}

fn get_connection() -> Connection {
    let conn = Connection::open("generated/hups.db").unwrap();
    HUPResult::create_table(&conn).expect("TODO: panic message");
    conn
}
