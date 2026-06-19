use pkcore::PKError;
use pkcore::play::stages::deal_eval::DealEval;
use pkcore::prelude::HoleCards;
use pkcore::util::terminal::Terminal;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

/// OK, this makes me sad. My new shiny pkcore library takes over twice as long to run a single calc
///
/// ```txt
/// ❯ cargo run --example bcrepl
/// ...
/// hole cards> A♠ A♥ 6♦ 6♣
/// Elapsed: 8.27s
// A♠ A♥ 6♦ 6♣, 79.66% (1363968), 20.05% (343394), 0.29% (4942)
/// ```
///
/// ```
/// pkcore❯ cargo run --example bcrepl
/// ...
/// hole cards> A♠ A♥ 6♦ 6♣
/// Elapsed: 22.00s
/// A♠ A♥ 6♦ 6♣, 79.66% (1363968), 20.05% (343394), 0.29% (4942)
/// ```
///
/// This is going to need some investigation.
///
/// `cargo run --example bcrepl`
/// `A♠ A♥ A♦ A♣`
fn main() {
    env_logger::init();
    let mut cache: HashMap<HoleCards, DealEval> = HashMap::new();
    loop {
        read_input(&mut cache);
    }
}

fn read_input(cache: &mut HashMap<HoleCards, DealEval>) {
    match Terminal::receive_cards_in_twos("hole cards> ") {
        Ok(twos) => match work(twos, cache) {
            Ok(_) => {}
            Err(e) => println!("{:?}", e),
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

fn work(hands: HoleCards, cache: &mut HashMap<HoleCards, DealEval>) -> Result<(), PKError> {
    let now = std::time::Instant::now();

    let results = match cache.entry(hands) {
        Entry::Occupied(e) => e.into_mut(),
        Entry::Vacant(e) => {
            let eval = DealEval::new(e.key().clone())?;
            e.insert(eval)
        }
    };

    println!("{results}");
    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}
