//! Generates `generated/hups.bin` — a compact postcard-serialized binary file containing all
//! precomputed heads-up preflop matchup results from the embedded SQLite database.
//!
//! This binary file is embedded into all builds via `include_bytes!` so that WASM targets
//! can look up HUP odds without needing SQLite.
//!
//! Run with:
//! ```sh
//! cargo run --example export_hups_bin
//! ```

use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use rusqlite::Connection;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("generated/hups.db")?;
    let all = HUPResult::select_all(&conn);

    println!("Loaded {} HUP records from embedded db", all.len());

    let records: Vec<(u64, u64, u64, u64, u64)> = all
        .iter()
        .map(|h| {
            (
                h.higher.as_u64(),
                h.lower.as_u64(),
                h.odds.wins,
                h.odds.losses,
                h.odds.draws,
            )
        })
        .collect();

    let bytes = postcard::to_stdvec(&records)?;
    println!("Serialized {} bytes", bytes.len());

    fs::create_dir_all("generated")?;
    fs::write("generated/hups.bin", &bytes)?;
    println!("Written to generated/hups.bin");

    Ok(())
}
