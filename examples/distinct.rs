use pkcore::arrays::matchups::masked::{
    Masked, MASKED_UNIQUE_TYPE_FIVE, MASKED_UNIQUE_TYPE_FOUR, MASKED_UNIQUE_TYPE_ONE,
    MASKED_UNIQUE_TYPE_SEVEN, MASKED_UNIQUE_TYPE_SIX, MASKED_UNIQUE_TYPE_THREE,
    MASKED_UNIQUE_TYPE_TWO,
};
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
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
/// 8580 type one hands with 4 suit sigs
/// 133848 type two hands with 24 suit sigs
/// 36504 type three hands with 12 suit sigs
/// 158184 type four hands with 24 suit sigs
/// 316368 type five hands with 24 suit sigs
/// 73008 type six hands with 6 suit sigs
/// 85683 type seven hands with 6 suit sigs
/// ```
/// //
//     // let total = unique.len();
//     //
//     // let one = do_it(&mut unique, SortedHeadsUp::is_type_one, "one");
//     // let two = do_it(&mut unique, SortedHeadsUp::is_type_two, "two");
//     // let three = do_it(&mut unique, SortedHeadsUp::is_type_three, "three");
//     // let four = do_it(&mut unique, SortedHeadsUp::is_type_four, "four");
//     // let five = do_it(&mut unique, SortedHeadsUp::is_type_five, "five");
//     // let six = do_it(&mut unique, SortedHeadsUp::is_type_six, "six");
//     // let seven = do_it(&mut unique, SortedHeadsUp::is_type_seven, "seven");
//     //
//     // let sum =
//     //     one.len() + two.len() + three.len() + four.len() + five.len() + six.len() + seven.len();
//     // assert_eq!(0, unique.len());
//     // assert_eq!(total, unique.len() + sum);
//     // check(&unique, &one);
//     // check(&unique, &two);
//     // check(&unique, &three);
//     // check(&unique, &four);
//     // check(&unique, &five);
//     // check(&unique, &six);
//     //
//     // // assert_eq!(total, one + two + three + four + five + six);
//     // SortedHeadsUp::generate_csv("generated/unique_type_remaining.csv", unique)
//     //     .expect("TODO: panic message");
fn main() -> Result<(), PKError> {
    println!(
        "{} type one has {} suit masks",
        MASKED_UNIQUE_TYPE_ONE.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_ONE, Masked::is_type_one).len()
    );
    println!(
        "{} type two has {} suit masks",
        MASKED_UNIQUE_TYPE_TWO.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_TWO, Masked::is_type_two).len()
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
        "{} type five has {} suit masks",
        MASKED_UNIQUE_TYPE_FIVE.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_FIVE, Masked::is_type_five).len()
    );
    println!(
        "{} type six has {} suit masks",
        MASKED_UNIQUE_TYPE_SIX.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_SIX, Masked::is_type_six).len()
    );
    println!(
        "{} type seven has {} suit masks",
        MASKED_UNIQUE_TYPE_SEVEN.len(),
        Masked::suit_masks(&MASKED_UNIQUE_TYPE_SEVEN, Masked::is_type_seven).len()
    );

    let distinct = Masked::distinct();
    println!("{}", distinct.len());

    SortedHeadsUp::generate_csv(
        "generated/distinct_masked_shus.csv",
        Masked::into_shus(&distinct),
    )
    .expect("TODO: panic message");

    Ok(())
}
