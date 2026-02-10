use pkcore::analysis::store::nubibus::pluribus::{Pluribus};
use pkcore::PKError;

/// `cargo run --example pluribus`
fn main() -> Result<(), PKError> {
    for plur in Pluribus::read_in_log("data/pluribus/raw/sample_game_30.log").unwrap() {
        plur.play_hand()?;
    }
    Ok(())
}
