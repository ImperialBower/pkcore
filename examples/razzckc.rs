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

    #[rstest]
    #[case("2♠ 2♥ 5♥ 4♠ 3♠", CaliforniaHandRank::HIGH_22543, 420)]
    #[case("2♠ 2♦ 5♥ 4♠ 3♠", CaliforniaHandRank::HIGH_22543, 420)]
    #[case("2♠ 2♦ 6♣ 4♠ 3♠", CaliforniaHandRank::HIGH_22643, 660)]
    #[case("2♠ 2♦ 6♣ 5♠ 3♠", CaliforniaHandRank::HIGH_22653, 924)]
    #[case("2♠ 2♦ 6♣ 5♠ 4♠", CaliforniaHandRank::HIGH_22654, 1540)]
    #[case("2♠ 2♦ 7♣ 4♠ 3♠", CaliforniaHandRank::HIGH_22743, 780)]
    #[case("2♠ 2♦ 7♣ 5♠ 3♠", CaliforniaHandRank::HIGH_22753, 1092)]
    #[case("2♠ 2♦ 7♣ 5♠ 4♠", CaliforniaHandRank::HIGH_22754, 1820)]
    #[case("2♠ 2♦ 7♣ 6♠ 3♠", CaliforniaHandRank::HIGH_22763, 1716)]
    #[case("2♠ 2♦ 7♣ 6♠ 4♠", CaliforniaHandRank::HIGH_22764, 2860)]
    #[case("2♠ 2♦ 7♣ 6♠ 5♠", CaliforniaHandRank::HIGH_22765, 4004)]
    #[case("2♠ 2♦ 8♣ 4♠ 3♠", CaliforniaHandRank::HIGH_22843, 1020)]
    #[case("2♠ 2♦ 8♣ 5♠ 3♠", CaliforniaHandRank::HIGH_22853, 1428)]
    #[case("2♠ 2♦ 8♣ 5♠ 4♠", CaliforniaHandRank::HIGH_22854, 2380)]
    #[case("2♠ 2♦ 8♣ 6♠ 3♠", CaliforniaHandRank::HIGH_22863, 2244)]
    #[case("2♠ 2♦ 8♣ 6♠ 4♠", CaliforniaHandRank::HIGH_22864, 3740)]
    #[case("2♠ 2♦ 8♣ 6♠ 5♠", CaliforniaHandRank::HIGH_22865, 5236)]
    #[case("2♠ 2♦ 8♣ 7♠ 3♠", CaliforniaHandRank::HIGH_22873, 2652)]
    #[case("2♠ 2♦ 8♣ 7♠ 4♠", CaliforniaHandRank::HIGH_22874, 4420)]
    #[case("2♠ 2♦ 8♣ 7♠ 5♠", CaliforniaHandRank::HIGH_22875, 6188)]
    #[case("2♠ 2♦ 8♣ 7♠ 6♠", CaliforniaHandRank::HIGH_22876, 9724)]
    #[case("2♠ 2♦ 9♣ 4♠ 3♠", CaliforniaHandRank::HIGH_22943, 1140)]
    #[case("2♠ 2♦ 9♣ 5♠ 3♠", CaliforniaHandRank::HIGH_22953, 1596)]
    #[case("2♠ 2♦ 9♣ 5♠ 4♠", CaliforniaHandRank::HIGH_22954, 2660)]
    #[case("2♠ 2♦ 9♣ 6♠ 3♠", CaliforniaHandRank::HIGH_22963, 2508)]

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
