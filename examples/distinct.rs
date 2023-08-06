use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::arrays::two::Two;
use pkcore::Shifty;

// A♦ A♣ - K♠ K♥
// A♠ A♣ - K♥ K♦
// A♥ A♦ - K♠ K♣
// A♠ A♥ - K♦ K♣
// A♠ A♦ - K♥ K♣
// A♥ A♣ - K♠ K♦
fn main() {
    // ♠ ♥ ♦ ♣

    let aces = Two::HAND_AS_AH;
    let kings = Two::HAND_KD_KC;
    let hup = SortedHeadsUp::new(aces, kings);
    let distinct = hup.shifts();

    for m in distinct {
        println!("{m}");
    }
}
