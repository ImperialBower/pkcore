use pkcore::PKError;
use pkcore::analysis::store::nubibus::pluribus::Pluribus;
use std::str::FromStr;

/// `cargo run --example plurry`
fn main() -> Result<(), PKError> {
    let plur = "STATE:55:ffr200r700fcr2250ff:Kc7s|8s9s|Jc3d|5d9d|AhAc|JdTd:-50|-700|0|0|1450|-700:MrPink|MrBlue|Joe|Bill|Pluribus|MrOrange";

    Pluribus::from_str(plur)?.play_hand()?;
    Ok(())
}
