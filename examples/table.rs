use pkcore::PKError;
use pkcore::arrays::hole_cards::twos::StartingHands;
use pkcore::casino::table::Table;
use pkcore::util::terminal::Terminal;
use pkcore::util::wincounter::results::Results;

/// `cargo run --example table`
fn main() {
    env_logger::init();
    // loop {
    //     read_input();
    // }
    let mut table = Table::default();

    println!("{table}");
    Terminal::pause("deal the cards> ");


}

fn read_input() {
    loop {
        Terminal::pause("step> ");
        work();
    }
}

fn work() -> Result<(), PKError> {
    let now = std::time::Instant::now();

    println!("{}", now.elapsed().as_secs());
    Ok(())
}
