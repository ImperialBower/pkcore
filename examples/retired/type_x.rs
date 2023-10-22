use pkcore::arrays::matchups::masked::Masked;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::Shifty;
use std::str::FromStr;

// #2 ...
// #2 ...
//
// #2 ... T♠ T♥ (73828) T♣ 2♠ (1580550) ties: (57926) inserted
//
// T♠ 2♥ T♥ T♦

fn main() {
    let case1 = Masked::from_str("T♠ T♣ T♦ 2♣").unwrap();

    for shift in case1.shifts() {
        println!("{shift}");
    }

    let case2: SortedHeadsUp = case1.into();
    for shift in case2.shifts() {
        println!("{shift}");
    }

    // let case3 = HUPResult {
    //     higher: Two::from_str("T♠ 2♥").unwrap().bard(),
    //     lower: Two::from_str("T♥ T♦").unwrap().bard(),
    //     higher_wins: 73_828,
    //     lower_wins: 1_580_550,
    //     ties: 57_926,
    // };
    //
    // for shift in case2.shifts() {
    //     println!("{shift}");
    // }
    //
    // let case3 = SortedHeadsUp::from_str("T♠ 2♥ T♥ T♦").unwrap();
    // for shift in case3.shifts() {
    //     println!("{shift}");
    // }

    // let conn = Connection::open("generated/dhups.db").unwrap();
    // HUPResult::create_table(&conn).expect("TODO: panic message");

    // // T♠ 2♥ (73828) T♥ T♦ (1580550) ties: (57926) inserted
    // let shu = SortedHeadsUp::from_str("T♠ 2♥ T♥ T♦").unwrap();
    // let hr = HUPResult::select(&conn, &shu).unwrap();
    // println!("{shu}");
    // println!("{hr}");
    //
    // // T♥ 2♦ (73828) T♦ T♣ (1580550) ties: (57926) inserted
    // let shu = SortedHeadsUp::from_str("T♥ 2♦ T♦ T♣").unwrap();
    // let hr = HUPResult::select(&conn, &shu).unwrap();
    // println!("{shu}");
    // println!("{hr}");

    // T♠ T♣ (73828) T♦ 2♣ (1580550) ties: (57926) inserted
    // let shu = SortedHeadsUp::from_str("T♠ T♣ T♦ 2♣").unwrap();
    // let hr = HUPResult::select(&conn, &shu).unwrap();
    // println!("{shu}");
    // println!("{hr}");
}
