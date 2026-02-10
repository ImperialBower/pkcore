use std::str::FromStr;
use pkcore::PKError;
use pkcore::analysis::store::nubibus::pluribus::Pluribus;

/// `cargo run --example plurry`
fn main() -> Result<(), PKError> {
    let plur =
        "STATE:77:ffr200ffc/cc/r537c/cr1099f:3c7h|TdQd|Qc3h|8hJh|Ad2d|8cKd/5h4c4s/7d/3d:-50|-537|0|0|587|0:Pluribus|MrWhite|Gogo|Budd|Eddie|Bill";

    Pluribus::from_str(plur)?.play_hand()?;
    Ok(())
}
