/// I want to get the tests right for this macro since it's going to be the foundation
/// for all of the range analysis work.
///
/// And the testing already caught an error with the `ACE_JACK_OFFSUIT` constant.
///
/// ## Resources
///
/// * [Poker Ranges & Range Reading](https://www.splitsuit.com/poker-ranges-reading)
/// * [POKER RANGES: POKER RANGE CHARTS](https://www.tightpoker.com/poker-ranges/)
#[macro_export(local_inner_macros)]
#[rustfmt::skip]
macro_rules! range {
    (AA) => { Twos::from($crate::arrays::combos::AA.to_vec()) };
    (KK) => { Twos::from($crate::arrays::combos::KK.to_vec()) };
    (QQ) => { Twos::from($crate::arrays::combos::QQ.to_vec()) };
    (JJ) => { Twos::from($crate::arrays::combos::JJ.to_vec()) };
    (TT) => { Twos::from($crate::arrays::combos::TENS.to_vec()) };
    (99) => { Twos::from($crate::arrays::combos::NINES.to_vec()) };
    (88) => { Twos::from($crate::arrays::combos::EIGHTS.to_vec()) };
    (77) => { Twos::from($crate::arrays::combos::SEVENS.to_vec()) };
    (66) => { Twos::from($crate::arrays::combos::SIXES.to_vec()) };
    (55) => { Twos::from($crate::arrays::combos::FIVES.to_vec()) };
    (44) => { Twos::from($crate::arrays::combos::FOURS.to_vec()) };
    (33) => { Twos::from($crate::arrays::combos::TREYS.to_vec()) };
    (22) => { Twos::from($crate::arrays::combos::DEUCES.to_vec()) };

    (KK+) => {
        Twos::from($crate::arrays::combos::KK.to_vec()).extend(
            &Twos::from($crate::arrays::combos::AA.to_vec())
        )
    };
    (QQ+) => {
        Twos::from($crate::arrays::combos::QQ.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KK.to_vec()).extend(
                &Twos::from($crate::arrays::combos::AA.to_vec())
            )
        )
    };
    (JJ+) => {
        Twos::from($crate::arrays::combos::JJ.to_vec()).extend(
            &Twos::from($crate::arrays::combos::QQ.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KK.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::AA.to_vec())
                )
            )
        )
    };
    (TT+) => {
        Twos::from($crate::arrays::combos::TENS.to_vec()).extend(
            &Twos::from($crate::arrays::combos::JJ.to_vec()).extend(
                &Twos::from($crate::arrays::combos::QQ.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KK.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::AA.to_vec())
                    )
                )
            )
        )
    };
    (99+) => {
        Twos::from($crate::arrays::combos::NINES.to_vec()).extend(
            &Twos::from($crate::arrays::combos::TENS.to_vec()).extend(
                &Twos::from($crate::arrays::combos::JJ.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::QQ.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KK.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::AA.to_vec())
                        )
                    )
                )
            )
        )
    };
    (88+) => {
        Twos::from($crate::arrays::combos::EIGHTS.to_vec()).extend(
            &Twos::from($crate::arrays::combos::NINES.to_vec()).extend(
                &Twos::from($crate::arrays::combos::TENS.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::JJ.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::QQ.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KK.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::AA.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (77+) => {
        Twos::from($crate::arrays::combos::SEVENS.to_vec()).extend(
            &Twos::from($crate::arrays::combos::EIGHTS.to_vec()).extend(
                &Twos::from($crate::arrays::combos::NINES.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::TENS.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::JJ.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::QQ.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::KK.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::AA.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (66+) => {
        Twos::from($crate::arrays::combos::SIXES.to_vec()).extend(
            &Twos::from($crate::arrays::combos::SEVENS.to_vec()).extend(
                &Twos::from($crate::arrays::combos::EIGHTS.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::NINES.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::TENS.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::JJ.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::QQ.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::KK.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::AA.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (55+) => {
        Twos::from($crate::arrays::combos::FIVES.to_vec()).extend(
            &Twos::from($crate::arrays::combos::SIXES.to_vec()).extend(
                &Twos::from($crate::arrays::combos::SEVENS.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::EIGHTS.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::NINES.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::TENS.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::JJ.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::QQ.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::KK.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::AA.to_vec())
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (44+) => {
        Twos::from($crate::arrays::combos::FOURS.to_vec()).extend(
            &Twos::from($crate::arrays::combos::FIVES.to_vec()).extend(
                &Twos::from($crate::arrays::combos::SIXES.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::SEVENS.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::EIGHTS.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::NINES.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::TENS.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::JJ.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::QQ.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::KK.to_vec()).extend(
                                                &Twos::from($crate::arrays::combos::AA.to_vec())
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (33+) => {
        Twos::from($crate::arrays::combos::TREYS.to_vec()).extend(
            &Twos::from($crate::arrays::combos::FOURS.to_vec()).extend(
                &Twos::from($crate::arrays::combos::FIVES.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::SIXES.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::SEVENS.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::EIGHTS.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::NINES.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::TENS.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::JJ.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::QQ.to_vec()).extend(
                                                &Twos::from($crate::arrays::combos::KK.to_vec()).extend(
                                                    &Twos::from($crate::arrays::combos::AA.to_vec())
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (22+) => {
        Twos::from($crate::arrays::combos::DEUCES.to_vec()).extend(
            &Twos::from($crate::arrays::combos::TREYS.to_vec()).extend(
                &Twos::from($crate::arrays::combos::FOURS.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::FIVES.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::SIXES.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::SEVENS.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::EIGHTS.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::NINES.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::TENS.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::JJ.to_vec()).extend(
                                                &Twos::from($crate::arrays::combos::QQ.to_vec()).extend(
                                                    &Twos::from($crate::arrays::combos::KK.to_vec()).extend(
                                                        &Twos::from($crate::arrays::combos::AA.to_vec())
                                                    )
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };

    (AKs) => { Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec()) };
    (AKo) => { Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec()) };
    (AK) => { Twos::from($crate::arrays::combos::ACE_KING.to_vec()) };
    (AQs) => { Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()) };
    (AQo) => { Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()) };
    (AQ) => { Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()) };
    (AJs) => { Twos::from($crate::arrays::combos::ACE_JACK_SUITED.to_vec()) };
    (AJo) => { Twos::from($crate::arrays::combos::ACE_JACK_OFFSUIT.to_vec()) };
    (AJ) => { Twos::from($crate::arrays::combos::ACE_JACK.to_vec()) };
    (ATs) => { Twos::from($crate::arrays::combos::ACE_TEN_SUITED.to_vec()) };
    (ATo) => { Twos::from($crate::arrays::combos::ACE_TEN_OFFSUIT.to_vec()) };
    (AT) => { Twos::from($crate::arrays::combos::ACE_TEN.to_vec()) };
    (A9s) => { Twos::from($crate::arrays::combos::ACE_NINE_SUITED.to_vec()) };
    (A9o) => { Twos::from($crate::arrays::combos::ACE_NINE_OFFSUIT.to_vec()) };
    (A9) => { Twos::from($crate::arrays::combos::ACE_NINE.to_vec()) };
    (A8s) => { Twos::from($crate::arrays::combos::ACE_EIGHT_SUITED.to_vec()) };
    (A8o) => { Twos::from($crate::arrays::combos::ACE_EIGHT_OFFSUIT.to_vec()) };
    (A8) => { Twos::from($crate::arrays::combos::ACE_EIGHT.to_vec()) };
    (A7s) => { Twos::from($crate::arrays::combos::ACE_SEVEN_SUITED.to_vec()) };
    (A7o) => { Twos::from($crate::arrays::combos::ACE_SEVEN_OFFSUIT.to_vec()) };
    (A7) => { Twos::from($crate::arrays::combos::ACE_SEVEN.to_vec()) };
    (A6s) => { Twos::from($crate::arrays::combos::ACE_SIX_SUITED.to_vec()) };
    (A6o) => { Twos::from($crate::arrays::combos::ACE_SIX_OFFSUIT.to_vec()) };
    (A6) => { Twos::from($crate::arrays::combos::ACE_SIX.to_vec()) };
    (A5s) => { Twos::from($crate::arrays::combos::ACE_FIVE_SUITED.to_vec()) };
    (A5o) => { Twos::from($crate::arrays::combos::ACE_FIVE_OFFSUIT.to_vec()) };
    (A5) => { Twos::from($crate::arrays::combos::ACE_FIVE.to_vec()) };
    (A4s) => { Twos::from($crate::arrays::combos::ACE_FOUR_SUITED.to_vec()) };
    (A4o) => { Twos::from($crate::arrays::combos::ACE_FOUR_OFFSUIT.to_vec()) };
    (A4) => { Twos::from($crate::arrays::combos::ACE_FOUR.to_vec()) };
    (A3s) => { Twos::from($crate::arrays::combos::ACE_TREY_SUITED.to_vec()) };
    (A3o) => { Twos::from($crate::arrays::combos::ACE_TREY_OFFSUIT.to_vec()) };
    (A3) => { Twos::from($crate::arrays::combos::ACE_TREY.to_vec()) };
    (A2s) => { Twos::from($crate::arrays::combos::ACE_DEUCE_SUITED.to_vec()) };
    (A2o) => { Twos::from($crate::arrays::combos::ACE_DEUCE_OFFSUIT.to_vec()) };
    (A2) => { Twos::from($crate::arrays::combos::ACE_DEUCE.to_vec()) };

    (KQs) => { Twos::from($crate::arrays::combos::KING_QUEEN_SUITED.to_vec()) };
    (KQo) => { Twos::from($crate::arrays::combos::KING_QUEEN_OFFSUIT.to_vec()) };
    (KQ) => { Twos::from($crate::arrays::combos::KING_QUEEN.to_vec()) };
    (KJs) => { Twos::from($crate::arrays::combos::KING_JACK_SUITED.to_vec()) };
    (KJo) => { Twos::from($crate::arrays::combos::KING_JACK_OFFSUIT.to_vec()) };
    (KJ) => { Twos::from($crate::arrays::combos::KING_JACK.to_vec()) };
    (KTs) => { Twos::from($crate::arrays::combos::KING_TEN_SUITED.to_vec()) };
    (KTo) => { Twos::from($crate::arrays::combos::KING_TEN_OFFSUIT.to_vec()) };
    (KT) => { Twos::from($crate::arrays::combos::KING_TEN.to_vec()) };
    (K9s) => { Twos::from($crate::arrays::combos::KING_NINE_SUITED.to_vec()) };
    (K9o) => { Twos::from($crate::arrays::combos::KING_NINE_OFFSUIT.to_vec()) };
    (K9) => { Twos::from($crate::arrays::combos::KING_NINE.to_vec()) };
    (K8s) => { Twos::from($crate::arrays::combos::KING_EIGHT_SUITED.to_vec()) };
    (K8o) => { Twos::from($crate::arrays::combos::KING_EIGHT_OFFSUIT.to_vec()) };
    (K8) => { Twos::from($crate::arrays::combos::KING_EIGHT.to_vec()) };
    (K7s) => { Twos::from($crate::arrays::combos::KING_SEVEN_SUITED.to_vec()) };
    (K7o) => { Twos::from($crate::arrays::combos::KING_SEVEN_OFFSUIT.to_vec()) };
    (K7) => { Twos::from($crate::arrays::combos::KING_SEVEN.to_vec()) };
    (K6s) => { Twos::from($crate::arrays::combos::KING_SIX_SUITED.to_vec()) };
    (K6o) => { Twos::from($crate::arrays::combos::KING_SIX_OFFSUIT.to_vec()) };
    (K6) => { Twos::from($crate::arrays::combos::KING_SIX.to_vec()) };
    (K5s) => { Twos::from($crate::arrays::combos::KING_FIVE_SUITED.to_vec()) };
    (K5o) => { Twos::from($crate::arrays::combos::KING_FIVE_OFFSUIT.to_vec()) };
    (K5) => { Twos::from($crate::arrays::combos::KING_FIVE.to_vec()) };
    (K4s) => { Twos::from($crate::arrays::combos::KING_FOUR_SUITED.to_vec()) };
    (K4o) => { Twos::from($crate::arrays::combos::KING_FOUR_OFFSUIT.to_vec()) };
    (K4) => { Twos::from($crate::arrays::combos::KING_FOUR.to_vec()) };
    (K3s) => { Twos::from($crate::arrays::combos::KING_TREY_SUITED.to_vec()) };
    (K3o) => { Twos::from($crate::arrays::combos::KING_TREY_OFFSUIT.to_vec()) };
    (K3) => { Twos::from($crate::arrays::combos::KING_TREY.to_vec()) };
    (K2s) => { Twos::from($crate::arrays::combos::KING_DEUCE_SUITED.to_vec()) };
    (K2o) => { Twos::from($crate::arrays::combos::KING_DEUCE_OFFSUIT.to_vec()) };
    (K2) => { Twos::from($crate::arrays::combos::KING_DEUCE.to_vec()) };

    (QJs) => { Twos::from($crate::arrays::combos::QUEEN_JACK_SUITED.to_vec()) };
    (QJo) => { Twos::from($crate::arrays::combos::QUEEN_JACK_OFFSUIT.to_vec()) };
    (QJ) => { Twos::from($crate::arrays::combos::QUEEN_JACK.to_vec()) };
    (QTs) => { Twos::from($crate::arrays::combos::QUEEN_TEN_SUITED.to_vec()) };
    (QTo) => { Twos::from($crate::arrays::combos::QUEEN_TEN_OFFSUIT.to_vec()) };
    (QT) => { Twos::from($crate::arrays::combos::QUEEN_TEN.to_vec()) };
    (Q9s) => { Twos::from($crate::arrays::combos::QUEEN_NINE_SUITED.to_vec()) };
    (Q9o) => { Twos::from($crate::arrays::combos::QUEEN_NINE_OFFSUIT.to_vec()) };
    (Q9) => { Twos::from($crate::arrays::combos::QUEEN_NINE.to_vec()) };
    (Q8s) => { Twos::from($crate::arrays::combos::QUEEN_EIGHT_SUITED.to_vec()) };
    (Q8o) => { Twos::from($crate::arrays::combos::QUEEN_EIGHT_OFFSUIT.to_vec()) };
    (Q8) => { Twos::from($crate::arrays::combos::QUEEN_EIGHT.to_vec()) };
    (Q7s) => { Twos::from($crate::arrays::combos::QUEEN_SEVEN_SUITED.to_vec()) };
    (Q7o) => { Twos::from($crate::arrays::combos::QUEEN_SEVEN_OFFSUIT.to_vec()) };
    (Q7) => { Twos::from($crate::arrays::combos::QUEEN_SEVEN.to_vec()) };
    (Q6s) => { Twos::from($crate::arrays::combos::QUEEN_SIX_SUITED.to_vec()) };
    (Q6o) => { Twos::from($crate::arrays::combos::QUEEN_SIX_OFFSUIT.to_vec()) };
    (Q6) => { Twos::from($crate::arrays::combos::QUEEN_SIX.to_vec()) };
    (Q5s) => { Twos::from($crate::arrays::combos::QUEEN_FIVE_SUITED.to_vec()) };
    (Q5o) => { Twos::from($crate::arrays::combos::QUEEN_FIVE_OFFSUIT.to_vec()) };
    (Q5) => { Twos::from($crate::arrays::combos::QUEEN_FIVE.to_vec()) };
    (Q4s) => { Twos::from($crate::arrays::combos::QUEEN_FOUR_SUITED.to_vec()) };
    (Q4o) => { Twos::from($crate::arrays::combos::QUEEN_FOUR_OFFSUIT.to_vec()) };
    (Q4) => { Twos::from($crate::arrays::combos::QUEEN_FOUR.to_vec()) };
    (Q3s) => { Twos::from($crate::arrays::combos::QUEEN_TREY_SUITED.to_vec()) };
    (Q3o) => { Twos::from($crate::arrays::combos::QUEEN_TREY_OFFSUIT.to_vec()) };
    (Q3) => { Twos::from($crate::arrays::combos::QUEEN_TREY.to_vec()) };
    (Q2s) => { Twos::from($crate::arrays::combos::QUEEN_DEUCE_SUITED.to_vec()) };
    (Q2o) => { Twos::from($crate::arrays::combos::QUEEN_DEUCE_OFFSUIT.to_vec()) };
    (Q2) => { Twos::from($crate::arrays::combos::QUEEN_DEUCE.to_vec()) };

    (JTs) => { Twos::from($crate::arrays::combos::JACK_TEN_SUITED.to_vec()) };
    (JTo) => { Twos::from($crate::arrays::combos::JACK_TEN_OFFSUIT.to_vec()) };
    (JT) => { Twos::from($crate::arrays::combos::JACK_TEN.to_vec()) };
    (J9s) => { Twos::from($crate::arrays::combos::JACK_NINE_SUITED.to_vec()) };
    (J9o) => { Twos::from($crate::arrays::combos::JACK_NINE_OFFSUIT.to_vec()) };
    (J9) => { Twos::from($crate::arrays::combos::JACK_NINE.to_vec()) };
    (J8s) => { Twos::from($crate::arrays::combos::JACK_EIGHT_SUITED.to_vec()) };
    (J8o) => { Twos::from($crate::arrays::combos::JACK_EIGHT_OFFSUIT.to_vec()) };
    (J8) => { Twos::from($crate::arrays::combos::JACK_EIGHT.to_vec()) };
    (J7s) => { Twos::from($crate::arrays::combos::JACK_SEVEN_SUITED.to_vec()) };
    (J7o) => { Twos::from($crate::arrays::combos::JACK_SEVEN_OFFSUIT.to_vec()) };
    (J7) => { Twos::from($crate::arrays::combos::JACK_SEVEN.to_vec()) };
    (J6s) => { Twos::from($crate::arrays::combos::JACK_SIX_SUITED.to_vec()) };
    (J6o) => { Twos::from($crate::arrays::combos::JACK_SIX_OFFSUIT.to_vec()) };
    (J6) => { Twos::from($crate::arrays::combos::JACK_SIX.to_vec()) };
    (J5s) => { Twos::from($crate::arrays::combos::JACK_FIVE_SUITED.to_vec()) };
    (J5o) => { Twos::from($crate::arrays::combos::JACK_FIVE_OFFSUIT.to_vec()) };
    (J5) => { Twos::from($crate::arrays::combos::JACK_FIVE.to_vec()) };
    (J4s) => { Twos::from($crate::arrays::combos::JACK_FOUR_SUITED.to_vec()) };
    (J4o) => { Twos::from($crate::arrays::combos::JACK_FOUR_OFFSUIT.to_vec()) };
    (J4) => { Twos::from($crate::arrays::combos::JACK_FOUR.to_vec()) };
    (J3s) => { Twos::from($crate::arrays::combos::JACK_TREY_SUITED.to_vec()) };
    (J3o) => { Twos::from($crate::arrays::combos::JACK_TREY_OFFSUIT.to_vec()) };
    (J3) => { Twos::from($crate::arrays::combos::JACK_TREY.to_vec()) };
    (J2s) => { Twos::from($crate::arrays::combos::JACK_DEUCE_SUITED.to_vec()) };
    (J2o) => { Twos::from($crate::arrays::combos::JACK_DEUCE_OFFSUIT.to_vec()) };
    (J2) => { Twos::from($crate::arrays::combos::JACK_DEUCE.to_vec()) };

    (T9s) => { Twos::from($crate::arrays::combos::TEN_NINE_SUITED.to_vec()) };
    (T9o) => { Twos::from($crate::arrays::combos::TEN_NINE_OFFSUIT.to_vec()) };
    (T9) => { Twos::from($crate::arrays::combos::TEN_NINE.to_vec()) };
    (T8s) => { Twos::from($crate::arrays::combos::TEN_EIGHT_SUITED.to_vec()) };
    (T8o) => { Twos::from($crate::arrays::combos::TEN_EIGHT_OFFSUIT.to_vec()) };
    (T8) => { Twos::from($crate::arrays::combos::TEN_EIGHT.to_vec()) };
    (T7s) => { Twos::from($crate::arrays::combos::TEN_SEVEN_SUITED.to_vec()) };
    (T7o) => { Twos::from($crate::arrays::combos::TEN_SEVEN_OFFSUIT.to_vec()) };
    (T7) => { Twos::from($crate::arrays::combos::TEN_SEVEN.to_vec()) };
    (T6s) => { Twos::from($crate::arrays::combos::TEN_SIX_SUITED.to_vec()) };
    (T6o) => { Twos::from($crate::arrays::combos::TEN_SIX_OFFSUIT.to_vec()) };
    (T6) => { Twos::from($crate::arrays::combos::TEN_SIX.to_vec()) };
    (T5s) => { Twos::from($crate::arrays::combos::TEN_FIVE_SUITED.to_vec()) };
    (T5o) => { Twos::from($crate::arrays::combos::TEN_FIVE_OFFSUIT.to_vec()) };
    (T5) => { Twos::from($crate::arrays::combos::TEN_FIVE.to_vec()) };
    (T4s) => { Twos::from($crate::arrays::combos::TEN_FOUR_SUITED.to_vec()) };
    (T4o) => { Twos::from($crate::arrays::combos::TEN_FOUR_OFFSUIT.to_vec()) };
    (T4) => { Twos::from($crate::arrays::combos::TEN_FOUR.to_vec()) };
    (T3s) => { Twos::from($crate::arrays::combos::TEN_TREY_SUITED.to_vec()) };
    (T3o) => { Twos::from($crate::arrays::combos::TEN_TREY_OFFSUIT.to_vec()) };
    (T3) => { Twos::from($crate::arrays::combos::TEN_TREY.to_vec()) };
    (T2s) => { Twos::from($crate::arrays::combos::TEN_DEUCE_SUITED.to_vec()) };
    (T2o) => { Twos::from($crate::arrays::combos::TEN_DEUCE_OFFSUIT.to_vec()) };
    (T2) => { Twos::from($crate::arrays::combos::TEN_DEUCE.to_vec()) };

    (98s) => { Twos::from($crate::arrays::combos::NINE_EIGHT_SUITED.to_vec()) };
    (98o) => { Twos::from($crate::arrays::combos::NINE_EIGHT_OFFSUIT.to_vec()) };
    (98) => { Twos::from($crate::arrays::combos::NINE_EIGHT.to_vec()) };
    (97s) => { Twos::from($crate::arrays::combos::NINE_SEVEN_SUITED.to_vec()) };
    (97o) => { Twos::from($crate::arrays::combos::NINE_SEVEN_OFFSUIT.to_vec()) };
    (97) => { Twos::from($crate::arrays::combos::NINE_SEVEN.to_vec()) };
    (96s) => { Twos::from($crate::arrays::combos::NINE_SIX_SUITED.to_vec()) };
    (96o) => { Twos::from($crate::arrays::combos::NINE_SIX_OFFSUIT.to_vec()) };
    (96) => { Twos::from($crate::arrays::combos::NINE_SIX.to_vec()) };
    (95s) => { Twos::from($crate::arrays::combos::NINE_FIVE_SUITED.to_vec()) };
    (95o) => { Twos::from($crate::arrays::combos::NINE_FIVE_OFFSUIT.to_vec()) };
    (95) => { Twos::from($crate::arrays::combos::NINE_FIVE.to_vec()) };
    (94s) => { Twos::from($crate::arrays::combos::NINE_FOUR_SUITED.to_vec()) };
    (94o) => { Twos::from($crate::arrays::combos::NINE_FOUR_OFFSUIT.to_vec()) };
    (94) => { Twos::from($crate::arrays::combos::NINE_FOUR.to_vec()) };
    (93s) => { Twos::from($crate::arrays::combos::NINE_TREY_SUITED.to_vec()) };
    (93o) => { Twos::from($crate::arrays::combos::NINE_TREY_OFFSUIT.to_vec()) };
    (93) => { Twos::from($crate::arrays::combos::NINE_TREY.to_vec()) };
    (92s) => { Twos::from($crate::arrays::combos::NINE_DEUCE_SUITED.to_vec()) };
    (92o) => { Twos::from($crate::arrays::combos::NINE_DEUCE_OFFSUIT.to_vec()) };
    (92) => { Twos::from($crate::arrays::combos::NINE_DEUCE.to_vec()) };

    (87s) => { Twos::from($crate::arrays::combos::EIGHT_SEVEN_SUITED.to_vec()) };
    (87o) => { Twos::from($crate::arrays::combos::EIGHT_SEVEN_OFFSUIT.to_vec()) };
    (87) => { Twos::from($crate::arrays::combos::EIGHT_SEVEN.to_vec()) };
    (86s) => { Twos::from($crate::arrays::combos::EIGHT_SIX_SUITED.to_vec()) };
    (86o) => { Twos::from($crate::arrays::combos::EIGHT_SIX_OFFSUIT.to_vec()) };
    (86) => { Twos::from($crate::arrays::combos::EIGHT_SIX.to_vec()) };
    (85s) => { Twos::from($crate::arrays::combos::EIGHT_FIVE_SUITED.to_vec()) };
    (85o) => { Twos::from($crate::arrays::combos::EIGHT_FIVE_OFFSUIT.to_vec()) };
    (85) => { Twos::from($crate::arrays::combos::EIGHT_FIVE.to_vec()) };
    (84s) => { Twos::from($crate::arrays::combos::EIGHT_FOUR_SUITED.to_vec()) };
    (84o) => { Twos::from($crate::arrays::combos::EIGHT_FOUR_OFFSUIT.to_vec()) };
    (84) => { Twos::from($crate::arrays::combos::EIGHT_FOUR.to_vec()) };
    (83s) => { Twos::from($crate::arrays::combos::EIGHT_TREY_SUITED.to_vec()) };
    (83o) => { Twos::from($crate::arrays::combos::EIGHT_TREY_OFFSUIT.to_vec()) };
    (83) => { Twos::from($crate::arrays::combos::EIGHT_TREY.to_vec()) };
    (82s) => { Twos::from($crate::arrays::combos::EIGHT_DEUCE_SUITED.to_vec()) };
    (82o) => { Twos::from($crate::arrays::combos::EIGHT_DEUCE_OFFSUIT.to_vec()) };
    (82) => { Twos::from($crate::arrays::combos::EIGHT_DEUCE.to_vec()) };

    (76s) => { Twos::from($crate::arrays::combos::SEVEN_SIX_SUITED.to_vec()) };
    (76o) => { Twos::from($crate::arrays::combos::SEVEN_SIX_OFFSUIT.to_vec()) };
    (76) => { Twos::from($crate::arrays::combos::SEVEN_SIX.to_vec()) };
    (75s) => { Twos::from($crate::arrays::combos::SEVEN_FIVE_SUITED.to_vec()) };
    (75o) => { Twos::from($crate::arrays::combos::SEVEN_FIVE_OFFSUIT.to_vec()) };
    (75) => { Twos::from($crate::arrays::combos::SEVEN_FIVE.to_vec()) };
    (74s) => { Twos::from($crate::arrays::combos::SEVEN_FOUR_SUITED.to_vec()) };
    (74o) => { Twos::from($crate::arrays::combos::SEVEN_FOUR_OFFSUIT.to_vec()) };
    (74) => { Twos::from($crate::arrays::combos::SEVEN_FOUR.to_vec()) };
    (73s) => { Twos::from($crate::arrays::combos::SEVEN_TREY_SUITED.to_vec()) };
    (73o) => { Twos::from($crate::arrays::combos::SEVEN_TREY_OFFSUIT.to_vec()) };
    (73) => { Twos::from($crate::arrays::combos::SEVEN_TREY.to_vec()) };
    (72s) => { Twos::from($crate::arrays::combos::SEVEN_DEUCE_SUITED.to_vec()) };
    (72o) => { Twos::from($crate::arrays::combos::SEVEN_DEUCE_OFFSUIT.to_vec()) };
    (72) => { Twos::from($crate::arrays::combos::SEVEN_DEUCE.to_vec()) };

    (65s) => { Twos::from($crate::arrays::combos::SIX_FIVE_SUITED.to_vec()) };
    (65o) => { Twos::from($crate::arrays::combos::SIX_FIVE_OFFSUIT.to_vec()) };
    (65) => { Twos::from($crate::arrays::combos::SIX_FIVE.to_vec()) };
    (64s) => { Twos::from($crate::arrays::combos::SIX_FOUR_SUITED.to_vec()) };
    (64o) => { Twos::from($crate::arrays::combos::SIX_FOUR_OFFSUIT.to_vec()) };
    (64) => { Twos::from($crate::arrays::combos::SIX_FOUR.to_vec()) };
    (63s) => { Twos::from($crate::arrays::combos::SIX_TREY_SUITED.to_vec()) };
    (63o) => { Twos::from($crate::arrays::combos::SIX_TREY_OFFSUIT.to_vec()) };
    (63) => { Twos::from($crate::arrays::combos::SIX_TREY.to_vec()) };
    (62s) => { Twos::from($crate::arrays::combos::SIX_DEUCE_SUITED.to_vec()) };
    (62o) => { Twos::from($crate::arrays::combos::SIX_DEUCE_OFFSUIT.to_vec()) };
    (62) => { Twos::from($crate::arrays::combos::SIX_DEUCE.to_vec()) };

    (54s) => { Twos::from($crate::arrays::combos::FIVE_FOUR_SUITED.to_vec()) };
    (54o) => { Twos::from($crate::arrays::combos::FIVE_FOUR_OFFSUIT.to_vec()) };
    (54) => { Twos::from($crate::arrays::combos::FIVE_FOUR.to_vec()) };
    (53s) => { Twos::from($crate::arrays::combos::FIVE_TREY_SUITED.to_vec()) };
    (53o) => { Twos::from($crate::arrays::combos::FIVE_TREY_OFFSUIT.to_vec()) };
    (53) => { Twos::from($crate::arrays::combos::FIVE_TREY.to_vec()) };
    (52s) => { Twos::from($crate::arrays::combos::FIVE_DEUCE_SUITED.to_vec()) };
    (52o) => { Twos::from($crate::arrays::combos::FIVE_DEUCE_OFFSUIT.to_vec()) };
    (52) => { Twos::from($crate::arrays::combos::FIVE_DEUCE.to_vec()) };

    (43s) => { Twos::from($crate::arrays::combos::FOUR_TREY_SUITED.to_vec()) };
    (43o) => { Twos::from($crate::arrays::combos::FOUR_TREY_OFFSUIT.to_vec()) };
    (43) => { Twos::from($crate::arrays::combos::FOUR_TREY.to_vec()) };
    (42s) => { Twos::from($crate::arrays::combos::FOUR_DEUCE_SUITED.to_vec()) };
    (42o) => { Twos::from($crate::arrays::combos::FOUR_DEUCE_OFFSUIT.to_vec()) };
    (42) => { Twos::from($crate::arrays::combos::FOUR_DEUCE.to_vec()) };

    (32s) => { Twos::from($crate::arrays::combos::TREY_DEUCE_SUITED.to_vec()) };
    (32o) => { Twos::from($crate::arrays::combos::TREY_DEUCE_OFFSUIT.to_vec()) };
    (32) => { Twos::from($crate::arrays::combos::TREY_DEUCE.to_vec()) };

    (AQ+) => {
        Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_KING.to_vec())
        )
    };
    (AJ+) => {
        Twos::from($crate::arrays::combos::ACE_JACK.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_KING.to_vec())
            )
        )
    };
    (AT+) => {
        Twos::from($crate::arrays::combos::ACE_TEN.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_JACK.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_KING.to_vec())
                )
            )
        )
    };
    (A9+) => {
        Twos::from($crate::arrays::combos::ACE_NINE.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_TEN.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_JACK.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_KING.to_vec())
                    )
                )
            )
        )
    };
    (A8+) => {
        Twos::from($crate::arrays::combos::ACE_EIGHT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_NINE.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_TEN.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_JACK.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_KING.to_vec())
                        )
                    )
                )
            )
        )
    };
    (A7+) => {
        Twos::from($crate::arrays::combos::ACE_SEVEN.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_EIGHT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_NINE.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_TEN.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_JACK.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_KING.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (A6+) => {
        Twos::from($crate::arrays::combos::ACE_SIX.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_SEVEN.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_EIGHT.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_NINE.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_TEN.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_JACK.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_KING.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A5+) => {
        Twos::from($crate::arrays::combos::ACE_FIVE.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_SIX.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_SEVEN.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_EIGHT.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_NINE.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_TEN.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_JACK.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_KING.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A4+) => {
        Twos::from($crate::arrays::combos::ACE_FOUR.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_FIVE.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_SIX.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_SEVEN.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_EIGHT.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_NINE.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_TEN.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_JACK.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::ACE_KING.to_vec())
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A3+) => {
        Twos::from($crate::arrays::combos::ACE_TREY.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_FOUR.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_FIVE.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_SIX.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_SEVEN.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_EIGHT.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_NINE.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_TEN.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_JACK.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()).extend(
                                                &Twos::from($crate::arrays::combos::ACE_KING.to_vec())
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (Ax) => {
        Twos::from($crate::arrays::combos::ACE_DEUCE.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_TREY.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_FOUR.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_FIVE.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_SIX.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_SEVEN.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_EIGHT.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_NINE.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_TEN.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::ACE_JACK.to_vec()).extend(
                                                &Twos::from($crate::arrays::combos::ACE_QUEEN.to_vec()).extend(
                                                    &Twos::from($crate::arrays::combos::ACE_KING.to_vec())
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };

    (AQs+) => {
        Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec())
        )
    };
    (AJs+) => {
        Twos::from($crate::arrays::combos::ACE_JACK_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec())
            )
        )
    };
    (ATs+) => {
        Twos::from($crate::arrays::combos::ACE_TEN_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_JACK_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec())
                )
            )
        )
    };
    (A9s+) => {
        Twos::from($crate::arrays::combos::ACE_NINE_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_TEN_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_JACK_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec())
                    )
                )
            )
        )
    };
    (A8s+) => {
        Twos::from($crate::arrays::combos::ACE_EIGHT_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_NINE_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_TEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_JACK_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec())
                        )
                    )
                )
            )
        )
    };
    (A7s+) => {
        Twos::from($crate::arrays::combos::ACE_SEVEN_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_EIGHT_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_NINE_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_TEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_JACK_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (A6s+) => {
        Twos::from($crate::arrays::combos::ACE_SIX_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_SEVEN_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_EIGHT_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_NINE_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_TEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_JACK_SUITED.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A5s+) => {
        Twos::from($crate::arrays::combos::ACE_FIVE_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_SIX_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_SEVEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_EIGHT_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_NINE_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_TEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_JACK_SUITED.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A4s+) => {
        Twos::from($crate::arrays::combos::ACE_FOUR_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_FIVE_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_SIX_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_SEVEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_EIGHT_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_NINE_SUITED.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_TEN_SUITED.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_JACK_SUITED.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec())
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A3s+) => {
        Twos::from($crate::arrays::combos::ACE_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_FOUR_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_FIVE_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_SIX_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_SEVEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_EIGHT_SUITED.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_NINE_SUITED.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_TEN_SUITED.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_JACK_SUITED.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()).extend(
                                                &Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec())
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A2s+) => {
        Twos::from($crate::arrays::combos::ACE_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_FOUR_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_FIVE_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_SIX_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_SEVEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_EIGHT_SUITED.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_NINE_SUITED.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_TEN_SUITED.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::ACE_JACK_SUITED.to_vec()).extend(
                                                &Twos::from($crate::arrays::combos::ACE_QUEEN_SUITED.to_vec()).extend(
                                                    &Twos::from($crate::arrays::combos::ACE_KING_SUITED.to_vec())
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };

    (AQo+) => {
        Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec())
        )
    };
    (AJo+) => {
        Twos::from($crate::arrays::combos::ACE_JACK_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec())
            )
        )
    };
    (ATo+) => {
        Twos::from($crate::arrays::combos::ACE_TEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_JACK_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec())
                )
            )
        )
    };
    (A9o+) => {
        Twos::from($crate::arrays::combos::ACE_NINE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_TEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_JACK_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec())
                    )
                )
            )
        )
    };
    (A8o+) => {
        Twos::from($crate::arrays::combos::ACE_EIGHT_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_NINE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_TEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_JACK_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec())
                        )
                    )
                )
            )
        )
    };
    (A7o+) => {
        Twos::from($crate::arrays::combos::ACE_SEVEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_NINE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_TEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_JACK_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (A6o+) => {
        Twos::from($crate::arrays::combos::ACE_SIX_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_SEVEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_NINE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_TEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_JACK_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A5o+) => {
        Twos::from($crate::arrays::combos::ACE_FIVE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_SIX_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_SEVEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_NINE_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_TEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_JACK_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A4o+) => {
        Twos::from($crate::arrays::combos::ACE_FOUR_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_FIVE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_SIX_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_SEVEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_NINE_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_TEN_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_JACK_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec())
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A3o+) => {
        Twos::from($crate::arrays::combos::ACE_TREY_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_FOUR_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_FIVE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_SIX_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_SEVEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_NINE_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_TEN_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_JACK_OFFSUIT.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                                &Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec())
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A2o+) => {
        Twos::from($crate::arrays::combos::ACE_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::ACE_TREY_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::ACE_FOUR_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::ACE_FIVE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::ACE_SIX_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::ACE_SEVEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::ACE_NINE_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::ACE_TEN_OFFSUIT.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::ACE_JACK_OFFSUIT.to_vec()).extend(
                                                &Twos::from($crate::arrays::combos::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                                    &Twos::from($crate::arrays::combos::ACE_KING_OFFSUIT.to_vec())
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };

    (KJ+) => {
        Twos::from($crate::arrays::combos::KING_JACK.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_QUEEN.to_vec()))
    };
    (KT+) => {
        Twos::from($crate::arrays::combos::KING_TEN.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_JACK.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_QUEEN.to_vec())
            )
        )
    };
    (K9+) => {
        Twos::from($crate::arrays::combos::KING_NINE.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_TEN.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_JACK.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_QUEEN.to_vec())
                )
            )
        )
    };
    (K8+) => {
        Twos::from($crate::arrays::combos::KING_EIGHT.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_NINE.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_TEN.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_JACK.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_QUEEN.to_vec())
                    )
                )
            )
        )
    };
    (K7+) => {
        Twos::from($crate::arrays::combos::KING_SEVEN.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_EIGHT.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_NINE.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_TEN.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_JACK.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_QUEEN.to_vec())
                        )
                    )
                )
            )
        )
    };
    (K6+) => {
        Twos::from($crate::arrays::combos::KING_SIX.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_SEVEN.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_EIGHT.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_NINE.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_TEN.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_JACK.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::KING_QUEEN.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (K5+) => {
        Twos::from($crate::arrays::combos::KING_FIVE.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_SIX.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_SEVEN.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_EIGHT.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_NINE.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_TEN.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::KING_JACK.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::KING_QUEEN.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (K4+) => {
        Twos::from($crate::arrays::combos::KING_FOUR.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_FIVE.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_SIX.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_SEVEN.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_EIGHT.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_NINE.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::KING_TEN.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::KING_JACK.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::KING_QUEEN.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (K3+) => {
        Twos::from($crate::arrays::combos::KING_TREY.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_FOUR.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_FIVE.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_SIX.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_SEVEN.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_EIGHT.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::KING_NINE.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::KING_TEN.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::KING_JACK.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::KING_QUEEN.to_vec())
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (Kx) => {
        Twos::from($crate::arrays::combos::KING_DEUCE.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_TREY.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_FOUR.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_FIVE.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_SIX.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_SEVEN.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::KING_EIGHT.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::KING_NINE.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::KING_TEN.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::KING_JACK.to_vec()).extend(
                                                &Twos::from($crate::arrays::combos::KING_QUEEN.to_vec())
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };

    (KJs+) => {
        Twos::from($crate::arrays::combos::KING_JACK_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_QUEEN_SUITED.to_vec())
        )
    };
    (KTs+) => {
        Twos::from($crate::arrays::combos::KING_TEN_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_JACK_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_QUEEN_SUITED.to_vec())
            )
        )
    };
    (K9s+) => {
        Twos::from($crate::arrays::combos::KING_NINE_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_TEN_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_JACK_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_QUEEN_SUITED.to_vec())
                )
            )
        )
    };
    (K8s+) => {
        Twos::from($crate::arrays::combos::KING_EIGHT_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_NINE_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_TEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_JACK_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_QUEEN_SUITED.to_vec())
                    )
                )
            )
        )
    };
    (K7s+) => {
        Twos::from($crate::arrays::combos::KING_SEVEN_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_EIGHT_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_NINE_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_TEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_JACK_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_QUEEN_SUITED.to_vec())
                        )
                    )
                )
            )
        )
    };
    (K6s+) => {
        Twos::from($crate::arrays::combos::KING_SIX_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_SEVEN_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_EIGHT_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_NINE_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_TEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_JACK_SUITED.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::KING_QUEEN_SUITED.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (K5s+) => {
        Twos::from($crate::arrays::combos::KING_FIVE_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_SIX_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_SEVEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_EIGHT_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_NINE_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_TEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::KING_JACK_SUITED.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::KING_QUEEN_SUITED.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (K4s+) => {
        Twos::from($crate::arrays::combos::KING_FOUR_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_FIVE_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_SIX_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_SEVEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_EIGHT_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_NINE_SUITED.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::KING_TEN_SUITED.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::KING_JACK_SUITED.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::KING_QUEEN_SUITED.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (K3s+) => {
        Twos::from($crate::arrays::combos::KING_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_FOUR_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_FIVE_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_SIX_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_SEVEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_EIGHT_SUITED.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::KING_NINE_SUITED.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::KING_TEN_SUITED.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::KING_JACK_SUITED.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::KING_QUEEN_SUITED.to_vec())
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (K2s+) => {
        Twos::from($crate::arrays::combos::KING_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::arrays::combos::KING_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::arrays::combos::KING_FOUR_SUITED.to_vec()).extend(
                    &Twos::from($crate::arrays::combos::KING_FIVE_SUITED.to_vec()).extend(
                        &Twos::from($crate::arrays::combos::KING_SIX_SUITED.to_vec()).extend(
                            &Twos::from($crate::arrays::combos::KING_SEVEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::arrays::combos::KING_EIGHT_SUITED.to_vec()).extend(
                                    &Twos::from($crate::arrays::combos::KING_NINE_SUITED.to_vec()).extend(
                                        &Twos::from($crate::arrays::combos::KING_TEN_SUITED.to_vec()).extend(
                                            &Twos::from($crate::arrays::combos::KING_JACK_SUITED.to_vec()).extend(
                                                &Twos::from($crate::arrays::combos::KING_QUEEN_SUITED.to_vec())
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };



    // TODO: Work in Progress
    // [$($x:tt),* $(,)?] => {
    // [$($x:tt),* ] => {
    //     {
    //         let mut v = Twos::default();
    //         $(
    //             let extended = range!($x);
    //             v.extend($x);
    //         )*
    //         v
    //     }
    // };
}

#[cfg(test)]
mod tests {
    use crate::arrays::combos::twos::Twos;
    use crate::rank::Rank;

    fn assert_on_pair(range: Twos, rank: Rank) {
        assert_eq!(
            range.hashset(),
            Twos::unique().filter_on_rank(rank).filter_is_paired().hashset()
        )
    }

    #[test]
    fn poker_pairs() {
        assert_on_pair(range!(AA), Rank::ACE);
        assert_on_pair(range!(KK), Rank::KING);
        assert_on_pair(range!(QQ), Rank::QUEEN);
        assert_on_pair(range!(JJ), Rank::JACK);
        assert_on_pair(range!(TT), Rank::TEN);
        assert_on_pair(range!(99), Rank::NINE);
        assert_on_pair(range!(88), Rank::EIGHT);
        assert_on_pair(range!(77), Rank::SEVEN);
        assert_on_pair(range!(66), Rank::SIX);
        assert_on_pair(range!(55), Rank::FIVE);
        assert_on_pair(range!(44), Rank::FOUR);
        assert_on_pair(range!(33), Rank::TREY);
        assert_on_pair(range!(22), Rank::DEUCE);
    }

    fn assert_on_suited_non_pairs(range: Twos, top: Rank, bottom: Rank) {
        let twos = Twos::unique()
            .filter_on_rank(top)
            .filter_on_rank(bottom)
            .filter_is_suited()
            .hashset();
        assert_eq!(range.hashset(), twos)
    }

    #[test]
    fn test_suited_non_pairs() {
        assert_on_suited_non_pairs(range!(AKs), Rank::ACE, Rank::KING);
        assert_on_suited_non_pairs(range!(AQs), Rank::ACE, Rank::QUEEN);
        assert_on_suited_non_pairs(range!(AJs), Rank::ACE, Rank::JACK);
        assert_on_suited_non_pairs(range!(ATs), Rank::ACE, Rank::TEN);
        assert_on_suited_non_pairs(range!(A9s), Rank::ACE, Rank::NINE);
        assert_on_suited_non_pairs(range!(A8s), Rank::ACE, Rank::EIGHT);
        assert_on_suited_non_pairs(range!(A7s), Rank::ACE, Rank::SEVEN);
        assert_on_suited_non_pairs(range!(A6s), Rank::ACE, Rank::SIX);
        assert_on_suited_non_pairs(range!(A5s), Rank::ACE, Rank::FIVE);
        assert_on_suited_non_pairs(range!(A4s), Rank::ACE, Rank::FOUR);
        assert_on_suited_non_pairs(range!(A3s), Rank::ACE, Rank::TREY);
        assert_on_suited_non_pairs(range!(A2s), Rank::ACE, Rank::DEUCE);

        assert_on_suited_non_pairs(range!(KQs), Rank::KING, Rank::QUEEN);
        assert_on_suited_non_pairs(range!(KJs), Rank::KING, Rank::JACK);
        assert_on_suited_non_pairs(range!(KTs), Rank::KING, Rank::TEN);
        assert_on_suited_non_pairs(range!(K9s), Rank::KING, Rank::NINE);
        assert_on_suited_non_pairs(range!(K8s), Rank::KING, Rank::EIGHT);
        assert_on_suited_non_pairs(range!(K7s), Rank::KING, Rank::SEVEN);
        assert_on_suited_non_pairs(range!(K6s), Rank::KING, Rank::SIX);
        assert_on_suited_non_pairs(range!(K5s), Rank::KING, Rank::FIVE);
        assert_on_suited_non_pairs(range!(K4s), Rank::KING, Rank::FOUR);
        assert_on_suited_non_pairs(range!(K3s), Rank::KING, Rank::TREY);
        assert_on_suited_non_pairs(range!(K2s), Rank::KING, Rank::DEUCE);

        assert_on_suited_non_pairs(range!(QJs), Rank::QUEEN, Rank::JACK);
        assert_on_suited_non_pairs(range!(QTs), Rank::QUEEN, Rank::TEN);
        assert_on_suited_non_pairs(range!(Q9s), Rank::QUEEN, Rank::NINE);
        assert_on_suited_non_pairs(range!(Q8s), Rank::QUEEN, Rank::EIGHT);
        assert_on_suited_non_pairs(range!(Q7s), Rank::QUEEN, Rank::SEVEN);
        assert_on_suited_non_pairs(range!(Q6s), Rank::QUEEN, Rank::SIX);
        assert_on_suited_non_pairs(range!(Q5s), Rank::QUEEN, Rank::FIVE);
        assert_on_suited_non_pairs(range!(Q4s), Rank::QUEEN, Rank::FOUR);
    }

    fn assert_on_non_suited_non_pairs(range: Twos, top: Rank, bottom: Rank) {
        let twos = Twos::unique()
            .filter_on_rank(top)
            .filter_on_rank(bottom)
            .filter_is_not_suited()
            .hashset();
        assert_eq!(range.hashset(), twos)
    }

    #[test]
    fn test_non_suited_non_pairs() {
        assert_on_non_suited_non_pairs(range!(AKo), Rank::ACE, Rank::KING);
        assert_on_non_suited_non_pairs(range!(AQo), Rank::ACE, Rank::QUEEN);
        assert_on_non_suited_non_pairs(range!(AJo), Rank::ACE, Rank::JACK);
        assert_on_non_suited_non_pairs(range!(ATo), Rank::ACE, Rank::TEN);
        assert_on_non_suited_non_pairs(range!(A9o), Rank::ACE, Rank::NINE);
        assert_on_non_suited_non_pairs(range!(A8o), Rank::ACE, Rank::EIGHT);
        assert_on_non_suited_non_pairs(range!(A7o), Rank::ACE, Rank::SEVEN);
        assert_on_non_suited_non_pairs(range!(A6o), Rank::ACE, Rank::SIX);
        assert_on_non_suited_non_pairs(range!(A5o), Rank::ACE, Rank::FIVE);
        assert_on_non_suited_non_pairs(range!(A4o), Rank::ACE, Rank::FOUR);
        assert_on_non_suited_non_pairs(range!(A3o), Rank::ACE, Rank::TREY);
        assert_on_non_suited_non_pairs(range!(A2o), Rank::ACE, Rank::DEUCE);

        assert_on_non_suited_non_pairs(range!(KQo), Rank::KING, Rank::QUEEN);
        assert_on_non_suited_non_pairs(range!(KJo), Rank::KING, Rank::JACK);
        assert_on_non_suited_non_pairs(range!(KTo), Rank::KING, Rank::TEN);
        assert_on_non_suited_non_pairs(range!(K9o), Rank::KING, Rank::NINE);
        assert_on_non_suited_non_pairs(range!(K8o), Rank::KING, Rank::EIGHT);
        assert_on_non_suited_non_pairs(range!(K7o), Rank::KING, Rank::SEVEN);
        assert_on_non_suited_non_pairs(range!(K6o), Rank::KING, Rank::SIX);
        assert_on_non_suited_non_pairs(range!(K5o), Rank::KING, Rank::FIVE);
        assert_on_non_suited_non_pairs(range!(K4o), Rank::KING, Rank::FOUR);
        assert_on_non_suited_non_pairs(range!(K3o), Rank::KING, Rank::TREY);
        assert_on_non_suited_non_pairs(range!(K2o), Rank::KING, Rank::DEUCE);

        assert_on_non_suited_non_pairs(range!(QJo), Rank::QUEEN, Rank::JACK);
        assert_on_non_suited_non_pairs(range!(QTo), Rank::QUEEN, Rank::TEN);
        assert_on_non_suited_non_pairs(range!(Q9o), Rank::QUEEN, Rank::NINE);
        assert_on_non_suited_non_pairs(range!(Q8o), Rank::QUEEN, Rank::EIGHT);
        assert_on_non_suited_non_pairs(range!(Q7o), Rank::QUEEN, Rank::SEVEN);
        assert_on_non_suited_non_pairs(range!(Q6o), Rank::QUEEN, Rank::SIX);
        assert_on_non_suited_non_pairs(range!(Q5o), Rank::QUEEN, Rank::FIVE);
        assert_on_non_suited_non_pairs(range!(Q4o), Rank::QUEEN, Rank::FOUR);
        assert_on_non_suited_non_pairs(range!(Q3o), Rank::QUEEN, Rank::TREY);
        assert_on_non_suited_non_pairs(range!(Q2o), Rank::QUEEN, Rank::DEUCE);

        assert_on_non_suited_non_pairs(range!(JTo), Rank::JACK, Rank::TEN);
        assert_on_non_suited_non_pairs(range!(J9o), Rank::JACK, Rank::NINE);
        assert_on_non_suited_non_pairs(range!(J8o), Rank::JACK, Rank::EIGHT);
        assert_on_non_suited_non_pairs(range!(J7o), Rank::JACK, Rank::SEVEN);
        assert_on_non_suited_non_pairs(range!(J6o), Rank::JACK, Rank::SIX);
        assert_on_non_suited_non_pairs(range!(J5o), Rank::JACK, Rank::FIVE);
        assert_on_non_suited_non_pairs(range!(J4o), Rank::JACK, Rank::FOUR);
        assert_on_non_suited_non_pairs(range!(J3o), Rank::JACK, Rank::TREY);
        assert_on_non_suited_non_pairs(range!(J2o), Rank::JACK, Rank::DEUCE);

        assert_on_non_suited_non_pairs(range!(T9o), Rank::TEN, Rank::NINE);
        assert_on_non_suited_non_pairs(range!(T8o), Rank::TEN, Rank::EIGHT);
        assert_on_non_suited_non_pairs(range!(T7o), Rank::TEN, Rank::SEVEN);
        assert_on_non_suited_non_pairs(range!(T6o), Rank::TEN, Rank::SIX);
        assert_on_non_suited_non_pairs(range!(T5o), Rank::TEN, Rank::FIVE);
        assert_on_non_suited_non_pairs(range!(T4o), Rank::TEN, Rank::FOUR);
        assert_on_non_suited_non_pairs(range!(T3o), Rank::TEN, Rank::TREY);
        assert_on_non_suited_non_pairs(range!(T2o), Rank::TEN, Rank::DEUCE);

        assert_on_non_suited_non_pairs(range!(98o), Rank::NINE, Rank::EIGHT);
        assert_on_non_suited_non_pairs(range!(97o), Rank::NINE, Rank::SEVEN);
        assert_on_non_suited_non_pairs(range!(96o), Rank::NINE, Rank::SIX);
        assert_on_non_suited_non_pairs(range!(95o), Rank::NINE, Rank::FIVE);
        assert_on_non_suited_non_pairs(range!(94o), Rank::NINE, Rank::FOUR);
        assert_on_non_suited_non_pairs(range!(93o), Rank::NINE, Rank::TREY);
        assert_on_non_suited_non_pairs(range!(92o), Rank::NINE, Rank::DEUCE);

        assert_on_non_suited_non_pairs(range!(87o), Rank::EIGHT, Rank::SEVEN);
        assert_on_non_suited_non_pairs(range!(86o), Rank::EIGHT, Rank::SIX);
        assert_on_non_suited_non_pairs(range!(85o), Rank::EIGHT, Rank::FIVE);
        assert_on_non_suited_non_pairs(range!(84o), Rank::EIGHT, Rank::FOUR);
        assert_on_non_suited_non_pairs(range!(83o), Rank::EIGHT, Rank::TREY);
        assert_on_non_suited_non_pairs(range!(82o), Rank::EIGHT, Rank::DEUCE);

        assert_on_non_suited_non_pairs(range!(76o), Rank::SEVEN, Rank::SIX);
        assert_on_non_suited_non_pairs(range!(75o), Rank::SEVEN, Rank::FIVE);
        assert_on_non_suited_non_pairs(range!(74o), Rank::SEVEN, Rank::FOUR);
        assert_on_non_suited_non_pairs(range!(73o), Rank::SEVEN, Rank::TREY);
        assert_on_non_suited_non_pairs(range!(72o), Rank::SEVEN, Rank::DEUCE);

        assert_on_non_suited_non_pairs(range!(65o), Rank::SIX, Rank::FIVE);
        assert_on_non_suited_non_pairs(range!(64o), Rank::SIX, Rank::FOUR);
        assert_on_non_suited_non_pairs(range!(63o), Rank::SIX, Rank::TREY);
        assert_on_non_suited_non_pairs(range!(62o), Rank::SIX, Rank::DEUCE);

        assert_on_non_suited_non_pairs(range!(54o), Rank::FIVE, Rank::FOUR);
        assert_on_non_suited_non_pairs(range!(53o), Rank::FIVE, Rank::TREY);
        assert_on_non_suited_non_pairs(range!(52o), Rank::FIVE, Rank::DEUCE);

        assert_on_non_suited_non_pairs(range!(43o), Rank::FOUR, Rank::TREY);
        assert_on_non_suited_non_pairs(range!(42o), Rank::FOUR, Rank::DEUCE);

        assert_on_non_suited_non_pairs(range!(32o), Rank::TREY, Rank::DEUCE);
    }

    fn assert_on_non_pairs(range: Twos, top: Rank, bottom: Rank) {
        let twos = Twos::unique().filter_on_rank(top).filter_on_rank(bottom).hashset();
        assert_eq!(range.hashset(), twos)
    }

    #[test]
    fn test_non_pairs() {
        assert_on_non_pairs(range!(AK), Rank::ACE, Rank::KING);
        assert_on_non_pairs(range!(AQ), Rank::ACE, Rank::QUEEN);
        assert_on_non_pairs(range!(AJ), Rank::ACE, Rank::JACK);
        assert_on_non_pairs(range!(AT), Rank::ACE, Rank::TEN);
        assert_on_non_pairs(range!(A9), Rank::ACE, Rank::NINE);
        assert_on_non_pairs(range!(A8), Rank::ACE, Rank::EIGHT);
        assert_on_non_pairs(range!(A7), Rank::ACE, Rank::SEVEN);
        assert_on_non_pairs(range!(A6), Rank::ACE, Rank::SIX);
        assert_on_non_pairs(range!(A5), Rank::ACE, Rank::FIVE);
        assert_on_non_pairs(range!(A4), Rank::ACE, Rank::FOUR);
        assert_on_non_pairs(range!(A3), Rank::ACE, Rank::TREY);
        assert_on_non_pairs(range!(A2), Rank::ACE, Rank::DEUCE);

        assert_on_non_pairs(range!(KQ), Rank::KING, Rank::QUEEN);
        assert_on_non_pairs(range!(KJ), Rank::KING, Rank::JACK);
        assert_on_non_pairs(range!(KT), Rank::KING, Rank::TEN);
        assert_on_non_pairs(range!(K9), Rank::KING, Rank::NINE);
        assert_on_non_pairs(range!(K8), Rank::KING, Rank::EIGHT);
        assert_on_non_pairs(range!(K7), Rank::KING, Rank::SEVEN);
        assert_on_non_pairs(range!(K6), Rank::KING, Rank::SIX);
        assert_on_non_pairs(range!(K5), Rank::KING, Rank::FIVE);
        assert_on_non_pairs(range!(K4), Rank::KING, Rank::FOUR);
        assert_on_non_pairs(range!(K3), Rank::KING, Rank::TREY);
        assert_on_non_pairs(range!(K2), Rank::KING, Rank::DEUCE);

        assert_on_non_pairs(range!(QJ), Rank::QUEEN, Rank::JACK);
        assert_on_non_pairs(range!(QT), Rank::QUEEN, Rank::TEN);
        assert_on_non_pairs(range!(Q9), Rank::QUEEN, Rank::NINE);
        assert_on_non_pairs(range!(Q8), Rank::QUEEN, Rank::EIGHT);
        assert_on_non_pairs(range!(Q7), Rank::QUEEN, Rank::SEVEN);
        assert_on_non_pairs(range!(Q6), Rank::QUEEN, Rank::SIX);
        assert_on_non_pairs(range!(Q5), Rank::QUEEN, Rank::FIVE);
        assert_on_non_pairs(range!(Q4), Rank::QUEEN, Rank::FOUR);
        assert_on_non_pairs(range!(Q3), Rank::QUEEN, Rank::TREY);
        assert_on_non_pairs(range!(Q2), Rank::QUEEN, Rank::DEUCE);

        assert_on_non_pairs(range!(JT), Rank::JACK, Rank::TEN);
        assert_on_non_pairs(range!(J9), Rank::JACK, Rank::NINE);
        assert_on_non_pairs(range!(J8), Rank::JACK, Rank::EIGHT);
        assert_on_non_pairs(range!(J7), Rank::JACK, Rank::SEVEN);
        assert_on_non_pairs(range!(J6), Rank::JACK, Rank::SIX);
        assert_on_non_pairs(range!(J5), Rank::JACK, Rank::FIVE);
        assert_on_non_pairs(range!(J4), Rank::JACK, Rank::FOUR);
        assert_on_non_pairs(range!(J3), Rank::JACK, Rank::TREY);
        assert_on_non_pairs(range!(J2), Rank::JACK, Rank::DEUCE);

        assert_on_non_pairs(range!(T9), Rank::TEN, Rank::NINE);
        assert_on_non_pairs(range!(T8), Rank::TEN, Rank::EIGHT);
        assert_on_non_pairs(range!(T7), Rank::TEN, Rank::SEVEN);
        assert_on_non_pairs(range!(T6), Rank::TEN, Rank::SIX);
        assert_on_non_pairs(range!(T5), Rank::TEN, Rank::FIVE);
        assert_on_non_pairs(range!(T4), Rank::TEN, Rank::FOUR);
        assert_on_non_pairs(range!(T3), Rank::TEN, Rank::TREY);
        assert_on_non_pairs(range!(T2), Rank::TEN, Rank::DEUCE);

        assert_on_non_pairs(range!(98), Rank::NINE, Rank::EIGHT);
        assert_on_non_pairs(range!(97), Rank::NINE, Rank::SEVEN);
        assert_on_non_pairs(range!(96), Rank::NINE, Rank::SIX);
        assert_on_non_pairs(range!(95), Rank::NINE, Rank::FIVE);
        assert_on_non_pairs(range!(94), Rank::NINE, Rank::FOUR);
        assert_on_non_pairs(range!(93), Rank::NINE, Rank::TREY);
        assert_on_non_pairs(range!(92), Rank::NINE, Rank::DEUCE);

        assert_on_non_pairs(range!(87), Rank::EIGHT, Rank::SEVEN);
        assert_on_non_pairs(range!(86), Rank::EIGHT, Rank::SIX);
        assert_on_non_pairs(range!(85), Rank::EIGHT, Rank::FIVE);
        assert_on_non_pairs(range!(84), Rank::EIGHT, Rank::FOUR);
        assert_on_non_pairs(range!(83), Rank::EIGHT, Rank::TREY);
        assert_on_non_pairs(range!(82), Rank::EIGHT, Rank::DEUCE);

        assert_on_non_pairs(range!(76), Rank::SEVEN, Rank::SIX);
        assert_on_non_pairs(range!(75), Rank::SEVEN, Rank::FIVE);
        assert_on_non_pairs(range!(74), Rank::SEVEN, Rank::FOUR);
        assert_on_non_pairs(range!(73), Rank::SEVEN, Rank::TREY);
        assert_on_non_pairs(range!(72), Rank::SEVEN, Rank::DEUCE);

        assert_on_non_pairs(range!(65), Rank::SIX, Rank::FIVE);
        assert_on_non_pairs(range!(64), Rank::SIX, Rank::FOUR);
        assert_on_non_pairs(range!(63), Rank::SIX, Rank::TREY);
        assert_on_non_pairs(range!(62), Rank::SIX, Rank::DEUCE);

        assert_on_non_pairs(range!(54), Rank::FIVE, Rank::FOUR);
        assert_on_non_pairs(range!(53), Rank::FIVE, Rank::TREY);
        assert_on_non_pairs(range!(52), Rank::FIVE, Rank::DEUCE);

        assert_on_non_pairs(range!(43), Rank::FOUR, Rank::TREY);
        assert_on_non_pairs(range!(42), Rank::FOUR, Rank::DEUCE);

        assert_on_non_pairs(range!(32), Rank::TREY, Rank::DEUCE);
    }

    #[test]
    fn kk_plus() {
        let expected = range!(AA).extend(&range!(KK));

        let actual = range!(KK+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn qq_plus() {
        let expected = range!(AA).extend(&range!(KK)).extend(&range!(QQ));

        let actual = range!(QQ+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn jj_plus() {
        let expected = range!(AA).extend(&range!(KK)).extend(&range!(QQ)).extend(&range!(JJ));

        let actual = range!(JJ+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn tt_plus() {
        let expected = range!(AA)
            .extend(&range!(KK))
            .extend(&range!(QQ))
            .extend(&range!(JJ))
            .extend(&range!(TT));

        let actual = range!(TT+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn nine_plus() {
        let expected = range!(AA)
            .extend(&range!(KK))
            .extend(&range!(QQ))
            .extend(&range!(JJ))
            .extend(&range!(TT))
            .extend(&range!(99));

        let actual = range!(99+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn eight_plus() {
        let expected = range!(AA)
            .extend(&range!(KK))
            .extend(&range!(QQ))
            .extend(&range!(JJ))
            .extend(&range!(TT))
            .extend(&range!(99))
            .extend(&range!(88));

        let actual = range!(88+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn seven_plus() {
        let expected = range!(AA)
            .extend(&range!(KK))
            .extend(&range!(QQ))
            .extend(&range!(JJ))
            .extend(&range!(TT))
            .extend(&range!(99))
            .extend(&range!(88))
            .extend(&range!(77));

        let actual = range!(77+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn six_plus() {
        let expected = range!(AA)
            .extend(&range!(KK))
            .extend(&range!(QQ))
            .extend(&range!(JJ))
            .extend(&range!(TT))
            .extend(&range!(99))
            .extend(&range!(88))
            .extend(&range!(77))
            .extend(&range!(66));

        let actual = range!(66+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn five_plus() {
        let expected = range!(AA)
            .extend(&range!(KK))
            .extend(&range!(QQ))
            .extend(&range!(JJ))
            .extend(&range!(TT))
            .extend(&range!(99))
            .extend(&range!(88))
            .extend(&range!(77))
            .extend(&range!(66))
            .extend(&range!(55));

        let actual = range!(55+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn four_plus() {
        let expected = range!(AA)
            .extend(&range!(KK))
            .extend(&range!(QQ))
            .extend(&range!(JJ))
            .extend(&range!(TT))
            .extend(&range!(99))
            .extend(&range!(88))
            .extend(&range!(77))
            .extend(&range!(66))
            .extend(&range!(55))
            .extend(&range!(44));

        let actual = range!(44+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn three_plus() {
        let expected = range!(AA)
            .extend(&range!(KK))
            .extend(&range!(QQ))
            .extend(&range!(JJ))
            .extend(&range!(TT))
            .extend(&range!(99))
            .extend(&range!(88))
            .extend(&range!(77))
            .extend(&range!(66))
            .extend(&range!(55))
            .extend(&range!(44))
            .extend(&range!(33));

        let actual = range!(33+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn two_plus() {
        let expected = range!(AA)
            .extend(&range!(KK))
            .extend(&range!(QQ))
            .extend(&range!(JJ))
            .extend(&range!(TT))
            .extend(&range!(99))
            .extend(&range!(88))
            .extend(&range!(77))
            .extend(&range!(66))
            .extend(&range!(55))
            .extend(&range!(44))
            .extend(&range!(33))
            .extend(&range!(22));

        let actual = range!(22+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn aq_plus() {
        let expected = range!(AQ).extend(&range!(AK));

        let actual = range!(AQ+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn aj_plus() {
        let expected = range!(AJ).extend(&range!(AQ)).extend(&range!(AK));

        let actual = range!(AJ+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn at_plus() {
        let expected = range!(AT).extend(&range!(AJ)).extend(&range!(AQ)).extend(&range!(AK));

        let actual = range!(AT+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn a9_plus() {
        let expected = range!(A9)
            .extend(&range!(AT))
            .extend(&range!(AJ))
            .extend(&range!(AQ))
            .extend(&range!(AK));

        let actual = range!(A9+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn a8_plus() {
        let expected = range!(A8)
            .extend(&range!(A9))
            .extend(&range!(AT))
            .extend(&range!(AJ))
            .extend(&range!(AQ))
            .extend(&range!(AK));

        let actual = range!(A8+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn a7_plus() {
        let expected = range!(A7)
            .extend(&range!(A8))
            .extend(&range!(A9))
            .extend(&range!(AT))
            .extend(&range!(AJ))
            .extend(&range!(AQ))
            .extend(&range!(AK));

        let actual = range!(A7+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn a6_plus() {
        let expected = range!(A6)
            .extend(&range!(A7))
            .extend(&range!(A8))
            .extend(&range!(A9))
            .extend(&range!(AT))
            .extend(&range!(AJ))
            .extend(&range!(AQ))
            .extend(&range!(AK));

        let actual = range!(A6+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn a5_plus() {
        let expected = range!(A5)
            .extend(&range!(A6))
            .extend(&range!(A7))
            .extend(&range!(A8))
            .extend(&range!(A9))
            .extend(&range!(AT))
            .extend(&range!(AJ))
            .extend(&range!(AQ))
            .extend(&range!(AK));

        let actual = range!(A5+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn a4_plus() {
        let expected = range!(A4)
            .extend(&range!(A5))
            .extend(&range!(A6))
            .extend(&range!(A7))
            .extend(&range!(A8))
            .extend(&range!(A9))
            .extend(&range!(AT))
            .extend(&range!(AJ))
            .extend(&range!(AQ))
            .extend(&range!(AK));

        let actual = range!(A4+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn a3_plus() {
        let expected = range!(A3)
            .extend(&range!(A4))
            .extend(&range!(A5))
            .extend(&range!(A6))
            .extend(&range!(A7))
            .extend(&range!(A8))
            .extend(&range!(A9))
            .extend(&range!(AT))
            .extend(&range!(AJ))
            .extend(&range!(AQ))
            .extend(&range!(AK));

        let actual = range!(A3+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn ax() {
        let expected = range!(A2)
            .extend(&range!(A3))
            .extend(&range!(A4))
            .extend(&range!(A5))
            .extend(&range!(A6))
            .extend(&range!(A7))
            .extend(&range!(A8))
            .extend(&range!(A9))
            .extend(&range!(AT))
            .extend(&range!(AJ))
            .extend(&range!(AQ))
            .extend(&range!(AK));

        let actual = range!(Ax);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn a2s_plus() {
        let expected = range!(A2s)
            .extend(&range!(A3s))
            .extend(&range!(A4s))
            .extend(&range!(A5s))
            .extend(&range!(A6s))
            .extend(&range!(A7s))
            .extend(&range!(A8s))
            .extend(&range!(A9s))
            .extend(&range!(ATs))
            .extend(&range!(AJs))
            .extend(&range!(AQs))
            .extend(&range!(AKs));

        let actual = range!(A2s+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn a2o_plus() {
        let expected = range!(A2o)
            .extend(&range!(A3o))
            .extend(&range!(A4o))
            .extend(&range!(A5o))
            .extend(&range!(A6o))
            .extend(&range!(A7o))
            .extend(&range!(A8o))
            .extend(&range!(A9o))
            .extend(&range!(ATo))
            .extend(&range!(AJo))
            .extend(&range!(AQo))
            .extend(&range!(AKo));

        let actual = range!(A2o+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn kj_plus() {
        let expected = range!(KJ).extend(&range!(KQ));

        let actual = range!(KJ+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn kt_plus() {
        let expected = range!(KT).extend(&range!(KJ)).extend(&range!(KQ));

        let actual = range!(KT+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k9_plus() {
        let expected = range!(K9).extend(&range!(KT)).extend(&range!(KJ)).extend(&range!(KQ));

        let actual = range!(K9+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k8_plus() {
        let expected = range!(K8)
            .extend(&range!(K9))
            .extend(&range!(KT))
            .extend(&range!(KJ))
            .extend(&range!(KQ));

        let actual = range!(K8+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k7_plus() {
        let expected = range!(K7)
            .extend(&range!(K8))
            .extend(&range!(K9))
            .extend(&range!(KT))
            .extend(&range!(KJ))
            .extend(&range!(KQ));

        let actual = range!(K7+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k6_plus() {
        let expected = range!(K6)
            .extend(&range!(K7))
            .extend(&range!(K8))
            .extend(&range!(K9))
            .extend(&range!(KT))
            .extend(&range!(KJ))
            .extend(&range!(KQ));

        let actual = range!(K6+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k5_plus() {
        let expected = range!(K5)
            .extend(&range!(K6))
            .extend(&range!(K7))
            .extend(&range!(K8))
            .extend(&range!(K9))
            .extend(&range!(KT))
            .extend(&range!(KJ))
            .extend(&range!(KQ));

        let actual = range!(K5+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k4_plus() {
        let expected = range!(K4)
            .extend(&range!(K5))
            .extend(&range!(K6))
            .extend(&range!(K7))
            .extend(&range!(K8))
            .extend(&range!(K9))
            .extend(&range!(KT))
            .extend(&range!(KJ))
            .extend(&range!(KQ));

        let actual = range!(K4+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k3_plus() {
        let expected = range!(K3)
            .extend(&range!(K4))
            .extend(&range!(K5))
            .extend(&range!(K6))
            .extend(&range!(K7))
            .extend(&range!(K8))
            .extend(&range!(K9))
            .extend(&range!(KT))
            .extend(&range!(KJ))
            .extend(&range!(KQ));

        let actual = range!(K3+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn kx() {
        let expected = range!(K2)
            .extend(&range!(K3))
            .extend(&range!(K4))
            .extend(&range!(K5))
            .extend(&range!(K6))
            .extend(&range!(K7))
            .extend(&range!(K8))
            .extend(&range!(K9))
            .extend(&range!(KT))
            .extend(&range!(KJ))
            .extend(&range!(KQ));

        let actual = range!(Kx);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn kjs_plus() {
        let expected = range!(KJs).extend(&range!(KQs));

        let actual = range!(KJs+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn kts_plus() {
        let expected = range!(KTs).extend(&range!(KJs)).extend(&range!(KQs));

        let actual = range!(KTs+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k9s_plus() {
        let expected = range!(K9s)
            .extend(&range!(KTs))
            .extend(&range!(KJs))
            .extend(&range!(KQs));

        let actual = range!(K9s+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k8s_plus() {
        let expected = range!(K8s)
            .extend(&range!(K9s))
            .extend(&range!(KTs))
            .extend(&range!(KJs))
            .extend(&range!(KQs));

        let actual = range!(K8s+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k7s_plus() {
        let expected = range!(K7s)
            .extend(&range!(K8s))
            .extend(&range!(K9s))
            .extend(&range!(KTs))
            .extend(&range!(KJs))
            .extend(&range!(KQs));

        let actual = range!(K7s+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k6s_plus() {
        let expected = range!(K6s)
            .extend(&range!(K7s))
            .extend(&range!(K8s))
            .extend(&range!(K9s))
            .extend(&range!(KTs))
            .extend(&range!(KJs))
            .extend(&range!(KQs));

        let actual = range!(K6s+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k5s_plus() {
        let expected = range!(K5s)
            .extend(&range!(K6s))
            .extend(&range!(K7s))
            .extend(&range!(K8s))
            .extend(&range!(K9s))
            .extend(&range!(KTs))
            .extend(&range!(KJs))
            .extend(&range!(KQs));

        let actual = range!(K5s+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k4s_plus() {
        let expected = range!(K4s)
            .extend(&range!(K5s))
            .extend(&range!(K6s))
            .extend(&range!(K7s))
            .extend(&range!(K8s))
            .extend(&range!(K9s))
            .extend(&range!(KTs))
            .extend(&range!(KJs))
            .extend(&range!(KQs));

        let actual = range!(K4s+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k3s_plus() {
        let expected = range!(K3s)
            .extend(&range!(K4s))
            .extend(&range!(K5s))
            .extend(&range!(K6s))
            .extend(&range!(K7s))
            .extend(&range!(K8s))
            .extend(&range!(K9s))
            .extend(&range!(KTs))
            .extend(&range!(KJs))
            .extend(&range!(KQs));

        let actual = range!(K3s+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }

    #[test]
    fn k2s_plus() {
        let expected = range!(K2s)
            .extend(&range!(K3s))
            .extend(&range!(K4s))
            .extend(&range!(K5s))
            .extend(&range!(K6s))
            .extend(&range!(K7s))
            .extend(&range!(K8s))
            .extend(&range!(K9s))
            .extend(&range!(KTs))
            .extend(&range!(KJs))
            .extend(&range!(KQs));

        let actual = range!(K2s+);

        assert_eq!(expected, actual);
        assert!(actual.is_aligned());
    }
}
