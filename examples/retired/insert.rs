use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::{MASKED_DISTINCT, Masked};
use pkcore::util::terminal::Terminal;
use rand::prelude::IteratorRandom;
use rusqlite::Connection;
use std::collections::HashSet;

/// `cargo run --example insert`
fn main() {
    let mut distinct = MASKED_DISTINCT.clone();
    let conn = Connection::open("../../generated/hups.db").unwrap();
    loop {
        read_input(&conn, &mut distinct);
    }
}

fn read_input(conn: &Connection, distinct: &mut HashSet<Masked>) {
    let mut x = 0usize;
    let i = Terminal::receive_usize("How many runs? ");
    println!("Processing {i} hands.");

    let mut rng = rand::rng();

    while x < i {
        let Some(masked) = distinct.clone().into_iter().choose(&mut rng) else {
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
                        let remaining = HUPResult::distinct_remaining(&conn).len();
                        println!("... inserted... {remaining} remaining");
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
