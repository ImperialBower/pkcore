use pkcore::arrays::five::hands::UNIQUE_HANDS;
use pkcore::games::razz::california::CaliforniaHandRank;
use pkcore::Pile;

fn main() {
    for hand in UNIQUE_HANDS.iter() {

        let hr = CaliforniaHandRank::get_hand_rank_from_rank_bit_flags(hand.get_rank_bits());

        if hr != CaliforniaHandRank::Unknown {
            continue; // Skip unknown hands
        }

        let index = hand.ranks_index();

        let hr = hand.multiply_primes();

        // println!("{hr} => CaliforniaHandRank::HIGH_{},", index);
        println!("#[case(\"{hand}\", CaliforniaHandRank::HIGH_{index}, {hr})]");
    }
}
