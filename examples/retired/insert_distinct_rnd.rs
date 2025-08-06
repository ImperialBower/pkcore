use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::Masked;
use pkcore::util::terminal::receive_usize;
use rand::prelude::IteratorRandom;
use rusqlite::Connection;
use pkcore::util::csv::distinct_shus_from_csv_as_masked_vec;

/// `cargo run --example insert_distinct_rnd`
fn main() {
    env_logger::init();

    let mut distinct = distinct_shus_from_csv_as_masked_vec();
    let conn = get_connection();

    loop {
        read_input(&conn, &mut distinct);
    }
}
fn read_input(conn: &Connection, distinct: &mut Vec<Masked>) {
    let mut x = 0usize;
    let i = receive_usize("How many runs? ");
    println!("Processing {i} hands.");

    let mut rng = rand::thread_rng();

    while x < i {
        let Some(masked) = distinct.into_iter().choose(&mut rng) else {
            println!("None remaining.");
            return;
        };
        if HUPResult::exists(&conn, &masked.shu) {
            println!("{} exists!", masked.shu);
            continue;
        } else {
            println!("Calculating {}", masked.shu);
            let hupr = HUPResult::from(&masked.shu);
            if HUPResult::exists(&conn, &masked.shu) {
                println!("... already inserted");
            } else {
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
        }
        x = x + 1;
    }
}

fn get_connection() -> Connection {
    let conn = Connection::open("../../generated/hups_07_31_2025.db").unwrap();
    HUPResult::create_table(&conn).expect("TODO: panic message");
    conn
}
