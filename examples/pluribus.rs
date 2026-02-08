use pkcore::analysis::store::nubibus::pluribus::Pluribus;
use std::io;
use pkcore::prelude::Table;

/// `cargo run --example pluribus`
fn main() {
    for game in Pluribus::read_in_log("data/pluribus/raw/sample_game_30.log").unwrap() {
        println!("{}", game);

        let table = Table::try_from(&game).unwrap();
        println!("{table}");

        println!("\nPress Enter to continue to the next game...");
        let _ = io::stdin().read_line(&mut String::new());
    }
}
