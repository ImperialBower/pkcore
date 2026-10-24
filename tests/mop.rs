/// Worked numbers from Bill Chen and Jerrod Ankenman, *The Mathematics of
/// Poker* (ConJelCo, 2006), reproduced with pkcore.
///
/// Page numbers are those of the print edition. Each test names its chapter
/// and page, and the reading notes in `docs/mop/` link back here. Every
/// probability is checked as an exact ratio of whole-number counts, so a
/// wrong count cannot hide inside a float tolerance.
///
/// If pkcore ever disagrees with the book, the expected value is not bent to
/// match: the note in `docs/mop/` records the disagreement and who was wrong.
#[cfg(test)]
#[allow(non_snake_case)]
mod mop__ch01_tests {
    use pkcore::prelude::*;

    /// `hits / total == num / den`, compared by cross-multiplying.
    fn is_ratio(hits: usize, total: usize, num: usize, den: usize) -> bool {
        hits * den == num * total
    }

    fn count_deck(predicate: impl Fn(&Card) -> bool) -> usize {
        Cards::deck().iter().filter(|card| predicate(card)).count()
    }

    // p16: one hold'em hand in 221 is a pair of aces.
    #[test]
    fn ch01_p016_pocket_aces_are_one_in_221() {
        let hands: Vec<Vec<Card>> = Cards::deck().combinations(2).collect();
        let aces = hands
            .iter()
            .filter(|hand| hand.iter().all(|card| card.get_rank() == Rank::ACE))
            .count();

        assert_eq!(hands.len(), 1326);
        assert_eq!(aces, 6);
        assert!(is_ratio(aces, hands.len(), 1, 221));
    }

    // p16: one card in 13 is an ace.
    #[test]
    fn ch01_p016_any_single_card_is_an_ace_one_in_13() {
        let aces = count_deck(|card| card.get_rank() == Rank::ACE);

        assert_eq!(Cards::deck().len(), 52);
        assert_eq!(aces, 4);
        assert!(is_ratio(aces, Cards::deck().len(), 1, 13));
    }

    // p17: heart and ace are independent, so both together is 1/4 × 1/13 = 1/52.
    #[test]
    fn ch01_p017_card_is_heart_and_ace_one_in_52() {
        let both = count_deck(|card| card.get_suit() == Suit::HEARTS && card.get_rank() == Rank::ACE);

        assert_eq!(both, 1);
        assert!(is_ratio(both, 52, 1, 52));
    }

    // p17: heart or ace is 1/4 + 1/13 − 1/52 = 4/13.
    #[test]
    fn ch01_p017_card_is_heart_or_ace_four_in_13() {
        let hearts = count_deck(|card| card.get_suit() == Suit::HEARTS);
        let either = count_deck(|card| card.get_suit() == Suit::HEARTS || card.get_rank() == Rank::ACE);

        assert_eq!(hearts, 13);
        assert_eq!(either, 16);
        assert!(is_ratio(either, 52, 4, 13));
    }

    // p18: after one ace is gone, 3 of the 51 cards left are aces.
    #[test]
    fn ch01_p018_second_ace_after_first_is_three_in_51() {
        let rest = Cards::deck_minus(&Cards::from(vec![Card::ACE_SPADES]));
        let aces = rest.iter().filter(|card| card.get_rank() == Rank::ACE).count();

        assert_eq!(rest.len(), 51);
        assert_eq!(aces, 3);
        assert!(is_ratio(aces, rest.len(), 1, 17));
        // First ace × second ace, over ordered pairs of cards: 1/221 again.
        assert!(is_ratio(4 * aces, 52 * rest.len(), 1, 221));
    }

    fn range_hands(range: &str) -> usize {
        let combos: Combos = range.parse().unwrap();
        Twos::from(&combos).len()
    }

    // p19: aces, kings or queens is 3 × 1/221 = 3/221.
    #[test]
    fn ch01_p019_aces_kings_or_queens_is_three_in_221() {
        let hands = range_hands("AA, KK, QQ");

        assert_eq!(hands, 18);
        assert_eq!(range_hands("QQ+"), hands);
        assert!(is_ratio(hands, 1326, 3, 221));
    }

    // p19: with two spades in hand, 11 spades remain in 50 cards; all three
    // flop cards are spades (11/50)(10/49)(9/48) = 33/3920 of the time.
    #[test]
    fn ch01_p019_suited_hand_flops_a_flush_33_in_3920() {
        let rest = Cards::deck_minus(&Cards::from(vec![Card::ACE_SPADES, Card::KING_SPADES]));
        let flops: Vec<Vec<Card>> = rest.combinations(3).collect();
        let flushes = flops
            .iter()
            .filter(|flop| flop.iter().all(|card| card.get_suit() == Suit::SPADES))
            .count();

        assert_eq!(flops.len(), 19_600);
        assert_eq!(flushes, 165);
        assert!(is_ratio(flushes, flops.len(), 33, 3920));
    }

    fn bet(wins: u64, losses: u64, win_amount: u64, loss_amount: u64) -> Ev {
        let odds = WinLoseDraw { wins, losses, draws: 0 };
        Ev::new(odds, PotOdds::new(win_amount, loss_amount))
    }

    // p21: a fair $10 coin flip is worth nothing.
    #[test]
    fn ch01_p021_fair_coin_flip_is_worth_zero() {
        let ev = bet(1, 1, 10, 10);

        assert_eq!(ev.numerator(), 0);
        assert!(!ev.is_positive());
    }

    // p21: win $11, lose $10 on a coin flip: EV = (11 − 10) / 2 = +$0.50.
    #[test]
    fn ch01_p021_eleven_to_ten_coin_flip_is_worth_fifty_cents() {
        let ev = bet(1, 1, 11, 10);

        assert_eq!((ev.numerator(), ev.total()), (1, 2));
        assert!(ev.is_positive());
    }

    // p21: win $30 on double sixes, else lose $1: EV = (30 − 35) / 36 = −5/36.
    #[test]
    fn ch01_p021_double_sixes_bet_loses_five_in_36() {
        let ev = bet(1, 35, 30, 1);

        assert_eq!((ev.numerator(), ev.total()), (-5, 36));
        assert!(!ev.is_positive());
    }

    // p22: the tight raiser's range {AA, KK, QQ, AKs, AKo}. The book lists it
    // without a count; pkcore counts 6 + 6 + 6 + 4 + 12 = 34 hands.
    #[test]
    fn ch01_p022_tight_raising_range_holds_34_hands() {
        assert_eq!(range_hands("AA, KK, QQ, AKs, AKo"), 34);
    }
}
