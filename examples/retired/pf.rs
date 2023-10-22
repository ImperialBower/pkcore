use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::{Masked, MASKED_DISTINCT};
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::util::csv::distinct_shus_from_csv_as_masked_vec;
use pkcore::Shifty;
use rusqlite::Connection;
use std::thread;

/// This is a `MARK III` version of inserting distincts into
///
/// `cargo run --example pf`
fn main() {
    env_logger::init();

    let conn = Connection::open("generated/clean_hups.db").unwrap();
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

        let conn = Connection::open("generated/clean_hups.db").unwrap();

        let hupr = get_result(&conn, &masked);
        for shift in hupr.shifts() {
            let shift_shu = SortedHeadsUp::try_from(&shift).unwrap();
            if HUPResult::exists(&conn, &shift_shu) {
                println!("#{n} SHIFT {} exists!", shift_shu);
                continue;
            } else {
                match HUPResult::insert(&conn, &shift) {
                    Ok(_) => {
                        println!("#{n} ... {shift} inserted");
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

fn get_result(conn: &Connection, masked: &Masked) -> HUPResult {
    match HUPResult::select_from_shifts(&conn, &masked) {
        None => {
            println!("Calculating {}", masked.shu);
            HUPResult::from(&masked.shu)
        }
        Some(hupr) => hupr,
    }
}
