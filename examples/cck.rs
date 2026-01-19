use clap::Parser;
use itertools::Itertools;
use pkcore::PKError;
use pkcore::arrays::HandRanker;
use pkcore::arrays::five::Five;
use pkcore::arrays::seven::Seven;
use pkcore::arrays::six::Six;
use pkcore::cards::Cards;
use std::fmt::Display;
use std::str::FromStr;

/// ```
/// ❯ cargo run --example cck -- -c "AS KS QS JS TS"
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

    let cards = Cards::from_str(&*args.card)?;

    // TODO NOTE: This incarnation eats errors in card indexes
    // For example: `❯ cargo run --example repl -- -c "AS KS QS JS TS 9S 9s"`
    match cards.len() {
        5 => show(Five::try_from(cards)?),
        6 => show(Six::try_from(cards)?),
        7 => show(Seven::try_from(cards)?),
        _ => println!("{}", cards), // https://stackoverflow.com/a/23977218/1245251
    };

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}

// https://stackoverflow.com/questions/51247690/how-can-i-define-a-function-with-a-parameter-that-can-be-multiple-kinds-of-trait
fn show<T>(cards: T)
where
    T: Display + HandRanker,
{
    let (hand_rank, hand) = cards.hand_rank_and_hand();
    println!(
        "CARDS: {} - BEST HAND: {} - {}: {:?}",
        cards.to_string(),
        hand.to_string(),
        hand_rank.value,
        hand_rank.class,
    );

    let sorts = hand.iter().counts_by(|card| card.get_rank());

    println!("{:?}", sorts);
}
