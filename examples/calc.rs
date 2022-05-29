use clap::Parser;
use pkcore::cards::Cards;
use pkcore::PKError;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'd', long)]
    dealt: String,

    #[clap(short = 'b', long)]
    board: String,
}

/// cargo run --example calc -- -d `6♠ 6♥ 5♦ 5♣` -b "9♣ 6♦ 5♥ 5♠ 8♠" THE HAND
fn main() -> Result<(), PKError> {
    let args = Args::parse();

    let cards_dealt = Cards::from_str(&*args.dealt)?;
    let cards_board = Cards::from_str(&*args.board)?;

    println!("DEALT: {} BOARD: {}", cards_dealt, cards_board);

    Ok(())
}
