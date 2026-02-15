use pkcore::PKError;
use pkcore::analysis::store::nubibus::pluribus::Pluribus;
use std::str::FromStr;

/// `cargo run --example plurry`
fn main() -> Result<(), PKError> {
    let plur = "STATE:193:r225fcffc/ccc/ccc/ccc:2s7d|7c9c|KcQs|2dQh|9h9s|Ac8d/5cKhJh/As/7s:-50|-225|500|0|-225|0:Eddie|MrOrange|Bill|MrBlue|Pluribus|MrPink";

    Pluribus::from_str(plur)?.play_hand()?;
    Ok(())
}
