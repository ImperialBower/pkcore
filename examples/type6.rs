use pkcore::arrays::matchups::masked::Masked;
use pkcore::Shifty;
use std::collections::HashSet;
use std::str::FromStr;

fn main() {
    let masked = Masked::from_str("A♠ K♥ Q♠ J♥").unwrap();
    let ranks: HashSet<Masked> = masked.shifts();
    // let ranks = Masked::filter(&masked.my_types(), )

    println!("{}", ranks.len());

    for m in ranks {
        println!("{m}");
    }
}
