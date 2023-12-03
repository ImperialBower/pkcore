use pkcore::arrays::matchups::masked::{
    Masked, MASKED_UNIQUE_TYPE_FIVE_A, MASKED_UNIQUE_TYPE_FIVE_B, MASKED_UNIQUE_TYPE_FIVE_C, MASKED_UNIQUE_TYPE_FIVE_D,
    MASKED_UNIQUE_TYPE_FOUR, MASKED_UNIQUE_TYPE_ONE, MASKED_UNIQUE_TYPE_SEVEN, MASKED_UNIQUE_TYPE_SIX_A,
    MASKED_UNIQUE_TYPE_SIX_B, MASKED_UNIQUE_TYPE_THREE, MASKED_UNIQUE_TYPE_TWO_A, MASKED_UNIQUE_TYPE_TWO_B,
    MASKED_UNIQUE_TYPE_TWO_C, MASKED_UNIQUE_TYPE_TWO_D, MASKED_UNIQUE_TYPE_TWO_E,
};
use pkcore::PKError;

/// ```txt
/// ♠ ♥ ♦ ♣
/// 1111 - suited, suited, same suit
/// 1112 - suited, off suit, sharing suit
/// 1122 - suited, suited, different suits
/// 1123 - suited, off suit, different suits
/// 1223 - off suit, off suit, sharing one suit
/// 1212 - off suit, off suit, sharing both suits
/// 1234 - off suit, off suit, sharing no suits
///
/// 8580 type one has 4 suit masks
/// 10296 type two A has 24 suit masks
/// 32604 type two B has 12 suit masks
/// 29172 type two C has 12 suit masks
/// 32604 type two D has 12 suit masks
/// 29172 type two E has 12 suit masks
/// 36504 type three has 12 suit masks
/// 158184 type four has 24 suit masks
/// 77064 type five A has 24 suit masks
/// 85176 type five B has 24 suit masks
/// 77064 type five C has 24 suit masks
/// 77064 type five D has 24 suit masks
/// 34788 type six A has 6 suit masks
/// 38220 type six B has 6 suit masks
/// 85683 type seven has 6 suit masks
/// 49933 distinct
///
/// 8580 type one has 4 suit masks
/// 10296 type two A has 24 suit masks
/// 32604 type two B has 12 suit masks
/// 29172 type two C has 12 suit masks
/// 32604 type two D has 12 suit masks
/// 29172 type two E has 12 suit masks
/// 36504 type three has 12 suit masks
/// 158184 type four has 24 suit masks
/// 77064 type five A has 24 suit masks
/// 85176 type five B has 24 suit masks
/// 77064 type five C has 24 suit masks
/// 77064 type five D has 24 suit masks
/// 34788 type six A has 6 suit masks
/// 38220 type six B has 6 suit masks
/// 85683 type seven has 6 suit masks
/// 47125
/// ```
///
/// ```
/// 8580 type one has 4 suit masks
/// 10296 type two A has 24 suit masks
/// 32604 type two B has 12 suit masks
/// 29172 type two C has 12 suit masks
/// 32604 type two D has 12 suit masks
/// 29172 type two E has 12 suit masks
/// 36504 type three has 12 suit masks
/// 158184 type four has 24 suit masks
/// 85176 type five A has 24 suit masks
/// 77064 type five B has 24 suit masks
/// 84864 type five C has 24 suit masks
/// 69264 type five D has 24 suit masks
/// 34788 type six A has 6 suit masks
/// 38220 type six B has 6 suit masks
/// 85683 type seven has 6 suit masks
/// 45747
/// ```
///
/// ```txt
/// 8580 type one has 4 suit masks
/// 10296 type two A has 24 suit masks
/// 32604 type two B has 12 suit masks
/// 29172 type two C has 12 suit masks
/// 32604 type two D has 12 suit masks
/// 29172 type two E has 12 suit masks
/// 36504 type three has 12 suit masks
/// 158184 type four has 24 suit masks
/// 88608 type five A has 24 suit masks
/// 73008 type five B has 24 suit masks
/// 89544 type five C has 24 suit masks
/// 65208 type five D has 24 suit masks
/// 34788 type six A has 6 suit masks
/// 38220 type six B has 6 suit masks
/// 85683 type seven has 6 suit masks
/// 45019
/// Elapsed: 698.83s
///
/// 8580 type one has 4 suit masks
/// 10296 type two A has 24 suit masks
/// 32604 type two B has 12 suit masks
/// 29172 type two C has 12 suit masks
/// 32604 type two D has 12 suit masks
/// 29172 type two E has 12 suit masks
/// 36504 type three has 12 suit masks
/// 158184 type four has 24 suit masks
/// 88608 type five A has 24 suit masks
/// 73008 type five B has 24 suit masks
/// 89544 type five C has 24 suit masks
/// 65208 type five D has 24 suit masks
/// 36504 type six A has 6 suit masks
/// 36504 type six B has 6 suit masks
/// 85683 type seven has 6 suit masks
/// 44733
/// Elapsed: 298.31s
///
/// 8580 type one has 4 suit masks
/// 10296 type two A has 24 suit masks
/// 32604 type two B has 12 suit masks
/// 29172 type two C has 12 suit masks
/// 32604 type two D has 12 suit masks
/// 29172 type two E has 12 suit masks
/// 36504 type three has 12 suit masks
/// 158184 type four has 24 suit masks
/// 88608 type five A has 24 suit masks
/// 73008 type five B has 24 suit masks
/// 89544 type five C has 24 suit masks
/// 65208 type five D has 24 suit masks
/// 39936 type six A has 6 suit masks
/// 33072 type six B has 6 suit masks
/// 85683 type seven has 6 suit masks
/// 44161
/// Elapsed: 300.81s
/// ```
///
/// TARGET: 47,008
/// `cargo run --example distinct`
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    println!(
        "{} type one has {} suit masks",
        MASKED_UNIQUE_TYPE_ONE.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_ONE, Masked::is_type_one).len()
    );
    println!(
        "{} type two A has {} suit masks",
        MASKED_UNIQUE_TYPE_TWO_A.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_TWO_A, Masked::is_type_two_a).len()
    );
    println!(
        "{} type two B has {} suit masks",
        MASKED_UNIQUE_TYPE_TWO_B.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_TWO_B, Masked::is_type_two_b).len()
    );
    println!(
        "{} type two C has {} suit masks",
        MASKED_UNIQUE_TYPE_TWO_C.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_TWO_C, Masked::is_type_two_c).len()
    );
    println!(
        "{} type two D has {} suit masks",
        MASKED_UNIQUE_TYPE_TWO_D.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_TWO_D, Masked::is_type_two_d).len()
    );
    println!(
        "{} type two E has {} suit masks",
        MASKED_UNIQUE_TYPE_TWO_E.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_TWO_E, Masked::is_type_two_e).len()
    );
    println!(
        "{} type three has {} suit masks",
        MASKED_UNIQUE_TYPE_THREE.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_THREE, Masked::is_type_three).len()
    );
    println!(
        "{} type four has {} suit masks",
        MASKED_UNIQUE_TYPE_FOUR.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_FOUR, Masked::is_type_four).len()
    );
    println!(
        "{} type five A has {} suit masks",
        MASKED_UNIQUE_TYPE_FIVE_A.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_FIVE_A, Masked::is_type_five_a).len()
    );
    println!(
        "{} type five B has {} suit masks",
        MASKED_UNIQUE_TYPE_FIVE_B.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_FIVE_B, Masked::is_type_five_b).len()
    );
    println!(
        "{} type five C has {} suit masks",
        MASKED_UNIQUE_TYPE_FIVE_C.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_FIVE_C, Masked::is_type_five_c).len()
    );
    println!(
        "{} type five D has {} suit masks",
        MASKED_UNIQUE_TYPE_FIVE_D.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_FIVE_D, Masked::is_type_five_d).len()
    );
    println!(
        "{} type six A has {} suit masks",
        MASKED_UNIQUE_TYPE_SIX_A.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_SIX_A, Masked::is_type_six_a).len()
    );
    println!(
        "{} type six B has {} suit masks",
        MASKED_UNIQUE_TYPE_SIX_B.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_SIX_B, Masked::is_type_six_b).len()
    );
    println!(
        "{} type seven has {} suit masks",
        MASKED_UNIQUE_TYPE_SEVEN.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_SEVEN, Masked::is_type_seven).len()
    );

    let distinct = Masked::distinct();
    println!("{}", distinct.len());

    //
    // SortedHeadsUp::generate_csv(
    //     "generated/unique_masked_type5a_shus.csv",
    //     Masked::into_shus(&MASKED_UNIQUE_TYPE_FIVE_A),
    // )
    // .expect("TODO: panic message");
    // SortedHeadsUp::generate_csv(
    //     "generated/unique_masked_type5b_shus.csv",
    //     Masked::into_shus(&MASKED_UNIQUE_TYPE_FIVE_B),
    // )
    // .expect("TODO: panic message");
    // SortedHeadsUp::generate_csv(
    //     "generated/unique_masked_type5c_shus.csv",
    //     Masked::into_shus(&MASKED_UNIQUE_TYPE_FIVE_C),
    // )
    // .expect("TODO: panic message");
    // SortedHeadsUp::generate_csv(
    //     "generated/unique_masked_type5d_shus.csv",
    //     Masked::into_shus(&MASKED_UNIQUE_TYPE_FIVE_D),
    // )
    // .expect("TODO: panic message");
    // SortedHeadsUp::generate_csv(
    //     "generated/unique_masked_type6a_shus.csv",
    //     Masked::into_shus(&MASKED_UNIQUE_TYPE_SIX_A),
    // )
    // .expect("TODO: panic message");
    // SortedHeadsUp::generate_csv(
    //     "generated/unique_masked_type6b_shus.csv",
    //     Masked::into_shus(&MASKED_UNIQUE_TYPE_SIX_B),
    // )
    // .expect("TODO: panic message");

    // SortedHeadsUp::generate_csv(
    //     "generated/distinct_masked_shus.csv",
    //     Masked::into_shus(&distinct),
    // )
    // .expect("TODO: panic message");

    println!("Elapsed: {:.2?}", now.elapsed());

    Ok(())
}
