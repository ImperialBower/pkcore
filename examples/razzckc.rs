use pkcore::arrays::HandRanker;
use pkcore::arrays::five::Five;
use std::str::FromStr;

fn main() {
    let wheel = Five::from_str("4♠ 3♠ 2♠ A♠ 5♥").unwrap().sort();

    println!("{}", wheel);
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use pkcore::analysis::class::Class;
    use pkcore::analysis::hand_rank::HandRankValue;
    use pkcore::analysis::name::Name;
    use pkcore::games::razz::california::{CaliforniaHandRank, CaliforniaHandRankValue};
    use rstest::rstest;

    fn get_wheel() -> Five {
        Five::from_str("4♠ 3♠ 2♠ A♠ 5♥").unwrap().sort()
    }

    #[test]
    fn wheel() {
        let wheel = get_wheel();
        let bits = wheel.or_rank_bits();
        let rank = CaliforniaHandRank::from(wheel);

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

    #[rstest]
    #[case("A♠ K♠ Q♠ J♠ T♠", 1, 31367009, 1)]
    #[case("A♥ K♥ Q♥ J♥ T♥", 1, 31367009, 1)]
    #[case("4♠ 3♠ 2♠ 2♥ 5♥", 1, 420, 1)]
    #[case("4♠ 3♠ 2♠ 2♦ 5♥", 1, 420, 1)]
    #[case("4♣ 3♠ 2♠ 2♦ 5♥", 1, 420, 1)]
    fn hand_ranker__hand_rank(
        #[case] index: &'static str,
        #[case] expected_value: CaliforniaHandRankValue,
        #[case] expected_prime: usize,
        #[case] expected_hrv: u16,
    ) {
        let hand = Five::from_str(index).unwrap();

        assert_eq!(hand.multiply_primes(), expected_prime);
    }
}
