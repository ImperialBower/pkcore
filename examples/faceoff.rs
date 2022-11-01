use pkcore::arrays::matchups::two_by_2::TwoBy2;
use pkcore::arrays::two::Two;

pub const HAND: TwoBy2 = TwoBy2 {
    first: Two::HAND_JC_4H,
    second: Two::HAND_8C_7C,
};

fn main() {
    let actual_wins = HAND.to_wins().unwrap();

    println!("{:?}", actual_wins);
}
