use pkcore::PKError;
use pkcore::casino::table::Table;
use pkcore::util::terminal::Terminal;

/// `cargo run --example table`
fn main() {
    env_logger::init();
    // loop {
    //     read_input();
    // }
    let table = Table::default();

    println!("{table}");
    Terminal::pause("deal the cards> ").expect("TODO: panic message");
}

fn _read_input() {
    loop {
        Terminal::pause("step> ").expect("TODO: panic message");
        let _ = _work();
    }
}

fn _work() -> Result<(), PKError> {
    let now = std::time::Instant::now();

    println!("{}", now.elapsed().as_secs());
    Ok(())
}
