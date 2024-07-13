use std::str::FromStr;
use clap::Parser;
use pkcore::bard::Bard;
use pkcore::cards::Cards;
use pkcore::{Pile, PKError};

/// `cargo run --example bard -- -t 1`
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'f', long)]
    from: Option<String>,

    #[clap(short = 't', long)]
    to: Option<u64>,
}

fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let args = Args::parse();

    if args.from.is_some() {
        let cards = Cards::from_str(args.from.unwrap().as_str()).unwrap();
        let bard = cards.bard();
        println!("from: {cards} = {}: 0b{bard}", bard.as_u64());
    }

    if args.to.is_some() {
        let cards = Cards::from(Bard::from(args.to.unwrap()));
        println!("from: {cards}");
    }

    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}