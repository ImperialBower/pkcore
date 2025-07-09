use std::str::FromStr;
use pkcore::arrays::five::Five;
use pkcore::arrays::HandRanker;

fn main() {
    let wheel = Five::from_str("4♠ 3♠ 2♠ A♠ 5♥").unwrap().sort();

    println!("{}", wheel);
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use pkcore::games::razz::low_a5_hand_rank::LowA7HandRank;
    use super::*;

    fn get_wheel() -> Five {
        Five::from_str("4♠ 3♠ 2♠ A♠ 5♥").unwrap().sort()
    }

    #[test]
    fn wheel() {
        let wheel = get_wheel();
        let bits = wheel.or_rank_bits();
        let rank = LowA7HandRank::from(wheel);

        println!("wheel: {:b}", bits);
        println!("rank: {}", rank.get_hand_rank() as u8);
    }

    #[test]
    fn ducks() {
        let hand = Five::from_str("4♠ 3♠ 2♠ 2♥ 5♥").unwrap().sort();
        let bits = hand.or_rank_bits();
        let primes = hand.multiply_primes();
        let products = hand.find_in_products();
        let hrv = hand.not_unique();


        println!("ducks: {:b}", bits);
        println!("primes: {}", primes);
        println!("products: {}", products);
        println!("hrv: {}", hrv);
    }

}