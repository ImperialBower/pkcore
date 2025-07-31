use pkcore::Pile;
use pkcore::card::Card;
use pkcore::cards::Cards;
use std::str::FromStr;

#[allow(dead_code)]
const ROBL_HAND: [Card; 4] = [
    Card::ACE_SPADES,
    Card::QUEEN_SPADES,
    Card::QUEEN_DIAMONDS,
    Card::JACK_CLUBS,
];

const _ANTONIUS_HAND: [Card; 4] = [
    Card::NINE_HEARTS,
    Card::EIGHT_DIAMONDS,
    Card::SIX_DIAMONDS,
    Card::FIVE_DIAMONDS,
];

const BOARD: [Card; 5] = [
    Card::FOUR_DIAMONDS,
    Card::ACE_DIAMONDS,
    Card::SEVEN_SPADES,
    Card::JACK_DIAMONDS,
    Card::ACE_CLUBS,
];

const PERMUTATION: [Card; 5] = [
    Card::ACE_SPADES,
    Card::ACE_DIAMONDS,
    Card::QUEEN_SPADES,
    Card::SEVEN_SPADES,
    Card::FOUR_DIAMONDS,
];

// A♠ A♦ Q♠ 7♠ 4♦

fn main() {
    let robl_hand = Cards::from(ROBL_HAND);
    let board = Cards::from(BOARD);
    let permutation = Cards::from(PERMUTATION);

    let common_board_perm = Cards::from_str("AD 7S 4D").unwrap();

    //                              A♠ A♦ Q♠ 7♠ 4♦
    assert_eq!(Cards::from_str("A♠ A♦ Q♠ 7♠ 4♦").unwrap(), permutation);
    //                              A♠ Q♠ Q♦ J♣ -
    assert_eq!(Cards::from_str("A♠ Q♠ Q♦ J♣").unwrap(), robl_hand);

    assert_eq!(3, board.how_many(&permutation));
    assert_eq!(board.common(&permutation), common_board_perm);

    let common_hand_perm = Cards::from_str("AS QS").unwrap();
    assert_eq!(2, robl_hand.how_many(&permutation));
    assert_eq!(robl_hand.common(&permutation), common_hand_perm);
}
