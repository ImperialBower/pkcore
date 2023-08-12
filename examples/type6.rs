use std::collections::HashSet;
use std::str::FromStr;
use pkcore::arrays::matchups::masked::Masked;

fn main() {
    let masked = Masked::from_str("A♠ K♥ Q♠ J♥").unwrap();
    let ranks: HashSet<Masked> = masked.my_shifts();
    // let ranks = Masked::filter(&masked.my_types(), )

    for m in ranks {
        println!("{m}");
    }

}