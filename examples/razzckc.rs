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
    #[case("2♠ 2♥ 5♥ 4♠ 3♠", CaliforniaHandRank::PAIR_22543, 420)]
    #[case("2♠ 2♦ 5♥ 4♠ 3♠", CaliforniaHandRank::PAIR_22543, 420)]
    #[case("2♠ 2♦ 6♣ 4♠ 3♠", CaliforniaHandRank::PAIR_22643, 660)]
    #[case("2♠ 2♦ 6♣ 5♠ 3♠", CaliforniaHandRank::PAIR_22653, 924)]
    #[case("2♠ 2♦ 6♣ 5♠ 4♠", CaliforniaHandRank::PAIR_22654, 1540)]
    #[case("2♠ 2♦ 7♣ 4♠ 3♠", CaliforniaHandRank::PAIR_22743, 780)]
    #[case("2♠ 2♦ 7♣ 5♠ 3♠", CaliforniaHandRank::PAIR_22753, 1092)]
    #[case("2♠ 2♦ 7♣ 5♠ 4♠", CaliforniaHandRank::PAIR_22754, 1820)]
    #[case("2♠ 2♦ 7♣ 6♠ 3♠", CaliforniaHandRank::PAIR_22763, 1716)]
    #[case("2♠ 2♦ 7♣ 6♠ 4♠", CaliforniaHandRank::PAIR_22764, 2860)]
    #[case("2♠ 2♦ 7♣ 6♠ 5♠", CaliforniaHandRank::PAIR_22765, 4004)]

    #[case("2♠ 2♦ 8♣ 4♠ 3♠", CaliforniaHandRank::PAIR_22843, 1020)]
    #[case("2♠ 2♦ 8♣ 5♠ 3♠", CaliforniaHandRank::PAIR_22853, 1428)]
    #[case("2♠ 2♦ 8♣ 5♠ 4♠", CaliforniaHandRank::PAIR_22854, 2380)]
    #[case("2♠ 2♦ 8♣ 6♠ 3♠", CaliforniaHandRank::PAIR_22863, 2244)]

    fn hand_ranker__hand_rank(
        #[case] index: &'static str,
        #[case] expected_hr: CaliforniaHandRank,
        #[case] expected_prime: usize,

    ) {
        let hand = Five::from_str(index).unwrap();

        assert_eq!(hand.multiply_primes(), expected_prime);
        assert_eq!(CaliforniaHandRank::from(hand), expected_hr);
    }
}
