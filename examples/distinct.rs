use pkcore::PKError;
use pkcore::arrays::matchups::masked::{
    MASKED_UNIQUE, MASKED_UNIQUE_TYPE_EIGHT, MASKED_UNIQUE_TYPE_FIVE_A, MASKED_UNIQUE_TYPE_FIVE_B,
    MASKED_UNIQUE_TYPE_FIVE_C, MASKED_UNIQUE_TYPE_FIVE_D, MASKED_UNIQUE_TYPE_FOUR, MASKED_UNIQUE_TYPE_ONE,
    MASKED_UNIQUE_TYPE_SEVEN, MASKED_UNIQUE_TYPE_SIX_A, MASKED_UNIQUE_TYPE_SIX_B, MASKED_UNIQUE_TYPE_THREE,
    MASKED_UNIQUE_TYPE_TWO_A, MASKED_UNIQUE_TYPE_TWO_B, MASKED_UNIQUE_TYPE_TWO_C, MASKED_UNIQUE_TYPE_TWO_D,
    MASKED_UNIQUE_TYPE_TWO_E, Masked,
};
use pkcore::arrays::matchups::sorted_heads_up::{
    SORTED_HEADS_UP_UNIQUE_TYPE_EIGHT, SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_A, SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_B,
    SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_C, SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_D, SORTED_HEADS_UP_UNIQUE_TYPE_FOUR,
    SORTED_HEADS_UP_UNIQUE_TYPE_ONE, SORTED_HEADS_UP_UNIQUE_TYPE_SEVEN, SORTED_HEADS_UP_UNIQUE_TYPE_SIX_A,
    SORTED_HEADS_UP_UNIQUE_TYPE_SIX_B, SORTED_HEADS_UP_UNIQUE_TYPE_THREE, SORTED_HEADS_UP_UNIQUE_TYPE_TWO_A,
    SORTED_HEADS_UP_UNIQUE_TYPE_TWO_B, SORTED_HEADS_UP_UNIQUE_TYPE_TWO_C, SORTED_HEADS_UP_UNIQUE_TYPE_TWO_D,
    SORTED_HEADS_UP_UNIQUE_TYPE_TWO_E, SortedHeadsUp,
};

/// ```txt
/// 8580 type one has 4 suit masks
/// 10296 type two A has 24 suit masks
/// 32604 type two B has 12 suit masks
/// 29172 type two C has 12 suit masks
/// 32604 type two D has 12 suit masks
/// 29172 type two E has 12 suit masks
/// 36504 type three has 12 suit masks
/// 81120 type four has 12 suit masks
/// 88608 type five A has 24 suit masks
/// 73008 type five B has 24 suit masks
/// 89544 type five C has 24 suit masks
/// 65208 type five D has 24 suit masks
/// 39936 type six A has 6 suit masks
/// 33072 type six B has 6 suit masks
/// 85683 type seven has 6 suit masks
/// 77064 type eight has 12 suit masks
/// distinct: 47320
/// unique: 812175
/// Elapsed: 234.72s
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
    println!(
        "{} type eight has {} suit masks",
        MASKED_UNIQUE_TYPE_EIGHT.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_EIGHT, Masked::is_type_eight).len()
    );

    println!("distinct: {}", Masked::distinct().len());
    println!("unique: {}", MASKED_UNIQUE.len());

    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type1.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_ONE.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type2a.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_TWO_A.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type2b.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_TWO_B.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type2c.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_TWO_C.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type2d.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_TWO_D.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type2e.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_TWO_E.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type3.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_THREE.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type4.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_FOUR.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type5a.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_A.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type5b.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_B.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type5c.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_C.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type5d.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_D.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type6a.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_SIX_A.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type6b.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_SIX_B.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type7.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_SEVEN.clone(),
    )
    .expect("TODO: panic message");
    SortedHeadsUp::generate_csv(
        "generated/unique_shus_type8.csv",
        SORTED_HEADS_UP_UNIQUE_TYPE_EIGHT.clone(),
    )
    .expect("TODO: panic message");
    // SortedHeadsUp::generate_csv(
    //     "generated/distinct_shus.csv",
    //     Masked::into_shus(&distinct),
    // )
    // .expect("TODO: panic message");

    println!("Elapsed: {:.2?}", now.elapsed());

    Ok(())
}
