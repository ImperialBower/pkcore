use pkcore::analysis::store::nubibus::Nubibus;
use pkcore::analysis::store::nubibus::pluribus::Pluribus;
use std::str::FromStr;

fn main() {
    let row = "STATE:52:fr200fffr1100c/r2225c/cc/r5600f:8h3c|7d6d|5dTd|7c7s|Ah4c|Ts9d/3s6c8c/Ad/8s:-50|2275|0|-2225|0|0:Bill|Pluribus|MrWhite|Gogo|Budd|Eddie";

    let plur = Pluribus::from_str(row);
    let nub = Nubibus::from_pluribus(&plur.as_ref().unwrap());
    assert!(plur.is_ok());
    println!("{nub}");
}
