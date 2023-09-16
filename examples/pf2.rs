use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::{Masked, MASKED_DISTINCT};
use pkcore::util::csv::distinct_shus_from_csv_as_masked_vec;
use rusqlite::Connection;
use std::thread;

/// This is a `MARK III` version of inserting distincts into
///
/// `cargo run --example pf2`
fn main() {
    env_logger::init();

    let conn = Connection::open("generated/dhups.db").unwrap();
    HUPResult::create_table(&conn).expect("TODO: panic message");
    conn.close().unwrap();

    let handle1 = thread::spawn(|| {
        run_it(1, &mut distinct_shus_from_csv_as_masked_vec());
    });
    let handle2 = thread::spawn(|| {
        let mut distinct = distinct_shus_from_csv_as_masked_vec();
        distinct.reverse();
        run_it(2, &mut distinct);
    });

    let _ = Vec::from_iter(MASKED_DISTINCT.clone());

    let handle3 = thread::spawn(|| {
        let mut distinct = Vec::from_iter(MASKED_DISTINCT.clone());
        run_it(3, &mut distinct);
    });

    let handle4 = thread::spawn(|| {
        let mut distinct = Vec::from_iter(MASKED_DISTINCT.clone());
        distinct.reverse();
        run_it(4, &mut distinct);
    });
    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();
    handle4.join().unwrap();
}

fn run_it(n: usize, distinct: &mut Vec<Masked>) {
    println!("Thread #{n} init");
    loop {
        let Some(masked) = distinct.pop() else {
            println!("#{n} none remaining.");
            return;
        };

        let conn = Connection::open("generated/dhups.db").unwrap();

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
                        let remaining = HUPResult::distinct_remaining(&conn).len();
                        println!("... inserted {hupr}");
                        println!("... {remaining} remaining");
                    }
                    Err(e) => {
                        println!("Unable to insert {hupr}");
                        println!("Error: {:?}", e);
                    }
                }
            }
        }
    }
}
