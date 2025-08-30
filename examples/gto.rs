use clap::Parser;
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::vs::Versus;
use pkcore::arrays::two::Two;
use pkcore::{GTO, PKError};
use rusqlite::Connection;
use std::str::FromStr;
use pkcore::analysis::store::db::hup::HUPResult;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'p', long)]
    player: String,

    #[clap(short = 'v', long)]
    villain: String,

    #[clap(short = 'n', long)]
    nuts: bool,
}

/// `cargo run --example gto -- -p "K♠ K♥" -v "66+,AJs+,KQs,AJo+,KQo"`
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let args = Args::parse();

    let solver = Versus::new(Two::from_str(&*args.player)?, Combos::from_str(&*args.villain)?);

    println!("{}", solver);
    println!();
    println!("{}", solver.villain.combo_pairs());

    println!();

    println!("{}", solver.combo_pairs());
    println!();
    println!("¹⁄₁₆");

    println!();
    println!();

    let conn = Connection::open("generated/hups.db").unwrap();

    let hups = solver.hups(&conn);

    for key in hups.keys() {
        println!("{}", hups.get(key).unwrap());
    }

    let results = Versus::combined_odds(hups.values().collect::<Vec<&HUPResult>>());
    println!();
    println!("{}", results);


    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}
