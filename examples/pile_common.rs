use pkcore::Pile;
use pkcore::arrays::four::Four;
use pkcore::card::Card;
use pkcore::games::omaha::OmahaHigh;

const ROBL_HAND: [Card; 4] = [
    Card::ACE_SPADES,
    Card::QUEEN_SPADES,
    Card::QUEEN_DIAMONDS,
    Card::JACK_CLUBS,
];

const _BOARD: [Card; 5] = [
    Card::FOUR_DIAMONDS,
    Card::ACE_DIAMONDS,
    Card::SEVEN_SPADES,
    Card::JACK_DIAMONDS,
    Card::ACE_CLUBS,
];

fn main() {
    let hand = OmahaHigh::from(Four::from(ROBL_HAND));
    let expected = vec![
        Card::ACE_SPADES,
        Card::QUEEN_SPADES,
        Card::QUEEN_DIAMONDS,
        Card::JACK_CLUBS,
    ];

    let actual = hand.to_vec();

    assert_eq!(expected, actual);
}
