use clap::Parser;
use pkcore::arrays::five::Five;
use pkcore::cards::Cards;
use pkcore::PKError;
use std::str::FromStr;
use pkcore::arrays::six::Six;

/// ```
/// ❯ cargo run --example repl -- -c "AS KS QS JS TS"
///     Finished dev [unoptimized + debuginfo] target(s) in 0.04s
///      Running `target/debug/examples/repl -c 'AS KS QS JS TS'`
/// A♠ K♠ Q♠ J♠ T♠
/// Elapsed: 325.18µs
/// ```
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'c', long)]
    card: String,
}
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();

    let args = Args::parse();

    let index = &*args.card;

    let cards = Cards::from_str(index).unwrap();

    match cards.len() {
        5 => println!("Five: {}", Five::try_from(cards)?),
        6 => println!("Six: {}", Six::try_from(cards)?),
        _ => println!("{}", cards), // https://stackoverflow.com/a/23977218/1245251
    };

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}
