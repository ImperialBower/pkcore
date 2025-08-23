use pkcore::PKError;
use pkcore::arrays::hole_cards::twos::StartingHands;
use pkcore::util::terminal::Terminal;
use pkcore::util::wincounter::results::Results;

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
    loop {
        read_input();
    }
}

fn read_input() {
    match Terminal::receive_cards_in_twos("hole cards> ") {
        Ok(twos) => match work(twos) {
            Ok(_) => {}
            Err(e) => println!("{:?}", e),
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

fn work(hands: StartingHands) -> Result<(), PKError> {
    let now = std::time::Instant::now();

    // let case_events = hands.bcm_case_evals()?;
    // let case_events = hands.bcm_case_evals()?;
    let case_events = hands.bcm_rayon_case_evals()?;
    let wins = case_events.wins();
    let results = Results::from_wins(&wins, hands.len());
    println!("{results}");

    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}
