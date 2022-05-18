use clap::Parser;
use pkcore::card::Card;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'c', long)]
    card: String,
}
fn main() {
    let now = std::time::Instant::now();

    let args = Args::parse();

    // https://stackoverflow.com/a/23977218/1245251
    println!("{}", Card::from_str(&*args.card).unwrap());

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
