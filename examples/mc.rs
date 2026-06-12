use std::str::FromStr;
use clap::Parser;
use pkcore::PKError;
use pkcore::prelude::{Board, Game, HoleCards};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'd', long)]
    dealt: String,

    #[clap(short = 'b', long)]
    board: String,
}

/// # Megacalc!
///
/// `cargo run --example mc -- -d "AC JC KD QD" -b "4S AH KH 8D 6H"`
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let args = Args::parse();
    let game = Game::new(HoleCards::from_str(&args.dealt)?, Board::from_str(&args.board)?);
    println!("{}", game);

    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}