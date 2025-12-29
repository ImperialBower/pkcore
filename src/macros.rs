/// ```
/// use pkcore::prelude::*;
///
/// assert_eq!(BoxedCards::blanks(3), boxed!("__ __ __"));
/// ```
#[macro_export]
#[allow(clippy::pedantic)]
macro_rules! boxed {
    ($card_str:expr) => {
        BoxedCards::forgiving_from_str($card_str)
    };
}

#[macro_export]
#[allow(clippy::pedantic)]
macro_rules! cards {
    ($card_str:expr) => {
        Cards::forgiving_from_str($card_str)
    };
}

#[macro_export]
macro_rules! deck {
    () => {
        Cards::deck()
    };
}

#[macro_export]
#[allow(clippy::pedantic)]
macro_rules! cc {
    ($card_str:expr) => {
        CardsCell::from(Cards::forgiving_from_str($card_str))
    };
}

#[macro_export]
macro_rules! deck_cell {
    () => {
        CardsCell::deck()
    };
}

/// I want to get the tests right for this macro since it's going to be the foundation
/// for all of the range analysis work.
///
/// And the testing already caught an error with the `ACE_JACK_OFFSUIT` constant.
///
/// ## Resources
///
/// * [Poker Ranges & Range Reading](https://www.splitsuit.com/poker-ranges-reading)
/// * [POKER RANGES: POKER RANGE CHARTS](https://www.tightpoker.com/poker-ranges/)
#[macro_export]
#[rustfmt::skip]
macro_rules! range {
    (AA) => { Twos::from($crate::analysis::gto::AA.to_vec()) };
    (KK) => { Twos::from($crate::analysis::gto::KK.to_vec()) };
    (QQ) => { Twos::from($crate::analysis::gto::QQ.to_vec()) };
    (JJ) => { Twos::from($crate::analysis::gto::JJ.to_vec()) };
    (TT) => { Twos::from($crate::analysis::gto::TENS.to_vec()) };
    (99) => { Twos::from($crate::analysis::gto::NINES.to_vec()) };
    (88) => { Twos::from($crate::analysis::gto::EIGHTS.to_vec()) };
    (77) => { Twos::from($crate::analysis::gto::SEVENS.to_vec()) };
    (66) => { Twos::from($crate::analysis::gto::SIXES.to_vec()) };
    (55) => { Twos::from($crate::analysis::gto::FIVES.to_vec()) };
    (44) => { Twos::from($crate::analysis::gto::FOURS.to_vec()) };
    (33) => { Twos::from($crate::analysis::gto::TREYS.to_vec()) };
    (22) => { Twos::from($crate::analysis::gto::DEUCES.to_vec()) };

    (KK+) => {
        Twos::from($crate::analysis::gto::KK.to_vec()).extend(
            &Twos::from($crate::analysis::gto::AA.to_vec())
        )
    };
    (QQ+) => {
        Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                &Twos::from($crate::analysis::gto::AA.to_vec())
            )
        )
    };
    (JJ+) => {
        Twos::from($crate::analysis::gto::JJ.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::AA.to_vec())
                )
            )
        )
    };
    (TT+) => {
        Twos::from($crate::analysis::gto::TENS.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JJ.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::AA.to_vec())
                    )
                )
            )
        )
    };
    (99+) => {
        Twos::from($crate::analysis::gto::NINES.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TENS.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JJ.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::AA.to_vec())
                        )
                    )
                )
            )
        )
    };
    (88+) => {
        Twos::from($crate::analysis::gto::EIGHTS.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINES.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TENS.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JJ.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::AA.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (77+) => {
        Twos::from($crate::analysis::gto::SEVENS.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHTS.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINES.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TENS.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JJ.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::AA.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (66+) => {
        Twos::from($crate::analysis::gto::SIXES.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVENS.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHTS.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINES.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TENS.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JJ.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::AA.to_vec())
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
        Twos::from($crate::analysis::gto::FIVES.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SIXES.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SEVENS.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::EIGHTS.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::NINES.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::TENS.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::JJ.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::AA.to_vec())
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
        Twos::from($crate::analysis::gto::FOURS.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FIVES.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SIXES.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SEVENS.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::EIGHTS.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::NINES.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::TENS.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::JJ.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::AA.to_vec())
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
        Twos::from($crate::analysis::gto::TREYS.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FOURS.to_vec()).extend(
                &Twos::from($crate::analysis::gto::FIVES.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SIXES.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::SEVENS.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::EIGHTS.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::NINES.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::TENS.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::JJ.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                                                    &Twos::from($crate::analysis::gto::AA.to_vec())
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
        Twos::from($crate::analysis::gto::DEUCES.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TREYS.to_vec()).extend(
                &Twos::from($crate::analysis::gto::FOURS.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::FIVES.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::SIXES.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::SEVENS.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::EIGHTS.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::NINES.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::TENS.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::JJ.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
                                                    &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                                                        &Twos::from($crate::analysis::gto::AA.to_vec())
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

    (AKs) => { Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec()) };
    (AKo) => { Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec()) };
    (AK) => { Twos::from($crate::analysis::gto::ACE_KING.to_vec()) };
    (AQs) => { Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()) };
    (AQo) => { Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()) };
    (AQ) => { Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()) };
    (AJs) => { Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()) };
    (AJo) => { Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()) };
    (AJ) => { Twos::from($crate::analysis::gto::ACE_JACK.to_vec()) };
    (ATs) => { Twos::from($crate::analysis::gto::ACE_TEN_SUITED.to_vec()) };
    (ATo) => { Twos::from($crate::analysis::gto::ACE_TEN_OFFSUIT.to_vec()) };
    (AT) => { Twos::from($crate::analysis::gto::ACE_TEN.to_vec()) };
    (A9s) => { Twos::from($crate::analysis::gto::ACE_NINE_SUITED.to_vec()) };
    (A9o) => { Twos::from($crate::analysis::gto::ACE_NINE_OFFSUIT.to_vec()) };
    (A9) => { Twos::from($crate::analysis::gto::ACE_NINE.to_vec()) };
    (A8s) => { Twos::from($crate::analysis::gto::ACE_EIGHT_SUITED.to_vec()) };
    (A8o) => { Twos::from($crate::analysis::gto::ACE_EIGHT_OFFSUIT.to_vec()) };
    (A8) => { Twos::from($crate::analysis::gto::ACE_EIGHT.to_vec()) };
    (A7s) => { Twos::from($crate::analysis::gto::ACE_SEVEN_SUITED.to_vec()) };
    (A7o) => { Twos::from($crate::analysis::gto::ACE_SEVEN_OFFSUIT.to_vec()) };
    (A7) => { Twos::from($crate::analysis::gto::ACE_SEVEN.to_vec()) };
    (A6s) => { Twos::from($crate::analysis::gto::ACE_SIX_SUITED.to_vec()) };
    (A6o) => { Twos::from($crate::analysis::gto::ACE_SIX_OFFSUIT.to_vec()) };
    (A6) => { Twos::from($crate::analysis::gto::ACE_SIX.to_vec()) };
    (A5s) => { Twos::from($crate::analysis::gto::ACE_FIVE_SUITED.to_vec()) };
    (A5o) => { Twos::from($crate::analysis::gto::ACE_FIVE_OFFSUIT.to_vec()) };
    (A5) => { Twos::from($crate::analysis::gto::ACE_FIVE.to_vec()) };
    (A4s) => { Twos::from($crate::analysis::gto::ACE_FOUR_SUITED.to_vec()) };
    (A4o) => { Twos::from($crate::analysis::gto::ACE_FOUR_OFFSUIT.to_vec()) };
    (A4) => { Twos::from($crate::analysis::gto::ACE_FOUR.to_vec()) };
    (A3s) => { Twos::from($crate::analysis::gto::ACE_TREY_SUITED.to_vec()) };
    (A3o) => { Twos::from($crate::analysis::gto::ACE_TREY_OFFSUIT.to_vec()) };
    (A3) => { Twos::from($crate::analysis::gto::ACE_TREY.to_vec()) };
    (A2s) => { Twos::from($crate::analysis::gto::ACE_DEUCE_SUITED.to_vec()) };
    (A2o) => { Twos::from($crate::analysis::gto::ACE_DEUCE_OFFSUIT.to_vec()) };
    (A2) => { Twos::from($crate::analysis::gto::ACE_DEUCE.to_vec()) };

    (KQs) => { Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec()) };
    (KQo) => { Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec()) };
    (KQ) => { Twos::from($crate::analysis::gto::KING_QUEEN.to_vec()) };
    (KJs) => { Twos::from($crate::analysis::gto::KING_JACK_SUITED.to_vec()) };
    (KJo) => { Twos::from($crate::analysis::gto::KING_JACK_OFFSUIT.to_vec()) };
    (KJ) => { Twos::from($crate::analysis::gto::KING_JACK.to_vec()) };
    (KTs) => { Twos::from($crate::analysis::gto::KING_TEN_SUITED.to_vec()) };
    (KTo) => { Twos::from($crate::analysis::gto::KING_TEN_OFFSUIT.to_vec()) };
    (KT) => { Twos::from($crate::analysis::gto::KING_TEN.to_vec()) };
    (K9s) => { Twos::from($crate::analysis::gto::KING_NINE_SUITED.to_vec()) };
    (K9o) => { Twos::from($crate::analysis::gto::KING_NINE_OFFSUIT.to_vec()) };
    (K9) => { Twos::from($crate::analysis::gto::KING_NINE.to_vec()) };
    (K8s) => { Twos::from($crate::analysis::gto::KING_EIGHT_SUITED.to_vec()) };
    (K8o) => { Twos::from($crate::analysis::gto::KING_EIGHT_OFFSUIT.to_vec()) };
    (K8) => { Twos::from($crate::analysis::gto::KING_EIGHT.to_vec()) };
    (K7s) => { Twos::from($crate::analysis::gto::KING_SEVEN_SUITED.to_vec()) };
    (K7o) => { Twos::from($crate::analysis::gto::KING_SEVEN_OFFSUIT.to_vec()) };
    (K7) => { Twos::from($crate::analysis::gto::KING_SEVEN.to_vec()) };
    (K6s) => { Twos::from($crate::analysis::gto::KING_SIX_SUITED.to_vec()) };
    (K6o) => { Twos::from($crate::analysis::gto::KING_SIX_OFFSUIT.to_vec()) };
    (K6) => { Twos::from($crate::analysis::gto::KING_SIX.to_vec()) };
    (K5s) => { Twos::from($crate::analysis::gto::KING_FIVE_SUITED.to_vec()) };
    (K5o) => { Twos::from($crate::analysis::gto::KING_FIVE_OFFSUIT.to_vec()) };
    (K5) => { Twos::from($crate::analysis::gto::KING_FIVE.to_vec()) };
    (K4s) => { Twos::from($crate::analysis::gto::KING_FOUR_SUITED.to_vec()) };
    (K4o) => { Twos::from($crate::analysis::gto::KING_FOUR_OFFSUIT.to_vec()) };
    (K4) => { Twos::from($crate::analysis::gto::KING_FOUR.to_vec()) };
    (K3s) => { Twos::from($crate::analysis::gto::KING_TREY_SUITED.to_vec()) };
    (K3o) => { Twos::from($crate::analysis::gto::KING_TREY_OFFSUIT.to_vec()) };
    (K3) => { Twos::from($crate::analysis::gto::KING_TREY.to_vec()) };
    (K2s) => { Twos::from($crate::analysis::gto::KING_DEUCE_SUITED.to_vec()) };
    (K2o) => { Twos::from($crate::analysis::gto::KING_DEUCE_OFFSUIT.to_vec()) };
    (K2) => { Twos::from($crate::analysis::gto::KING_DEUCE.to_vec()) };

    (QJs) => { Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec()) };
    (QJo) => { Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec()) };
    (QJ) => { Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec()) };
    (QTs) => { Twos::from($crate::analysis::gto::QUEEN_TEN_SUITED.to_vec()) };
    (QTo) => { Twos::from($crate::analysis::gto::QUEEN_TEN_OFFSUIT.to_vec()) };
    (QT) => { Twos::from($crate::analysis::gto::QUEEN_TEN.to_vec()) };
    (Q9s) => { Twos::from($crate::analysis::gto::QUEEN_NINE_SUITED.to_vec()) };
    (Q9o) => { Twos::from($crate::analysis::gto::QUEEN_NINE_OFFSUIT.to_vec()) };
    (Q9) => { Twos::from($crate::analysis::gto::QUEEN_NINE.to_vec()) };
    (Q8s) => { Twos::from($crate::analysis::gto::QUEEN_EIGHT_SUITED.to_vec()) };
    (Q8o) => { Twos::from($crate::analysis::gto::QUEEN_EIGHT_OFFSUIT.to_vec()) };
    (Q8) => { Twos::from($crate::analysis::gto::QUEEN_EIGHT.to_vec()) };
    (Q7s) => { Twos::from($crate::analysis::gto::QUEEN_SEVEN_SUITED.to_vec()) };
    (Q7o) => { Twos::from($crate::analysis::gto::QUEEN_SEVEN_OFFSUIT.to_vec()) };
    (Q7) => { Twos::from($crate::analysis::gto::QUEEN_SEVEN.to_vec()) };
    (Q6s) => { Twos::from($crate::analysis::gto::QUEEN_SIX_SUITED.to_vec()) };
    (Q6o) => { Twos::from($crate::analysis::gto::QUEEN_SIX_OFFSUIT.to_vec()) };
    (Q6) => { Twos::from($crate::analysis::gto::QUEEN_SIX.to_vec()) };
    (Q5s) => { Twos::from($crate::analysis::gto::QUEEN_FIVE_SUITED.to_vec()) };
    (Q5o) => { Twos::from($crate::analysis::gto::QUEEN_FIVE_OFFSUIT.to_vec()) };
    (Q5) => { Twos::from($crate::analysis::gto::QUEEN_FIVE.to_vec()) };
    (Q4s) => { Twos::from($crate::analysis::gto::QUEEN_FOUR_SUITED.to_vec()) };
    (Q4o) => { Twos::from($crate::analysis::gto::QUEEN_FOUR_OFFSUIT.to_vec()) };
    (Q4) => { Twos::from($crate::analysis::gto::QUEEN_FOUR.to_vec()) };
    (Q3s) => { Twos::from($crate::analysis::gto::QUEEN_TREY_SUITED.to_vec()) };
    (Q3o) => { Twos::from($crate::analysis::gto::QUEEN_TREY_OFFSUIT.to_vec()) };
    (Q3) => { Twos::from($crate::analysis::gto::QUEEN_TREY.to_vec()) };
    (Q2s) => { Twos::from($crate::analysis::gto::QUEEN_DEUCE_SUITED.to_vec()) };
    (Q2o) => { Twos::from($crate::analysis::gto::QUEEN_DEUCE_OFFSUIT.to_vec()) };
    (Q2) => { Twos::from($crate::analysis::gto::QUEEN_DEUCE.to_vec()) };

    (JTs) => { Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec()) };
    (JTo) => { Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec()) };
    (JT) => { Twos::from($crate::analysis::gto::JACK_TEN.to_vec()) };
    (J9s) => { Twos::from($crate::analysis::gto::JACK_NINE_SUITED.to_vec()) };
    (J9o) => { Twos::from($crate::analysis::gto::JACK_NINE_OFFSUIT.to_vec()) };
    (J9) => { Twos::from($crate::analysis::gto::JACK_NINE.to_vec()) };
    (J8s) => { Twos::from($crate::analysis::gto::JACK_EIGHT_SUITED.to_vec()) };
    (J8o) => { Twos::from($crate::analysis::gto::JACK_EIGHT_OFFSUIT.to_vec()) };
    (J8) => { Twos::from($crate::analysis::gto::JACK_EIGHT.to_vec()) };
    (J7s) => { Twos::from($crate::analysis::gto::JACK_SEVEN_SUITED.to_vec()) };
    (J7o) => { Twos::from($crate::analysis::gto::JACK_SEVEN_OFFSUIT.to_vec()) };
    (J7) => { Twos::from($crate::analysis::gto::JACK_SEVEN.to_vec()) };
    (J6s) => { Twos::from($crate::analysis::gto::JACK_SIX_SUITED.to_vec()) };
    (J6o) => { Twos::from($crate::analysis::gto::JACK_SIX_OFFSUIT.to_vec()) };
    (J6) => { Twos::from($crate::analysis::gto::JACK_SIX.to_vec()) };
    (J5s) => { Twos::from($crate::analysis::gto::JACK_FIVE_SUITED.to_vec()) };
    (J5o) => { Twos::from($crate::analysis::gto::JACK_FIVE_OFFSUIT.to_vec()) };
    (J5) => { Twos::from($crate::analysis::gto::JACK_FIVE.to_vec()) };
    (J4s) => { Twos::from($crate::analysis::gto::JACK_FOUR_SUITED.to_vec()) };
    (J4o) => { Twos::from($crate::analysis::gto::JACK_FOUR_OFFSUIT.to_vec()) };
    (J4) => { Twos::from($crate::analysis::gto::JACK_FOUR.to_vec()) };
    (J3s) => { Twos::from($crate::analysis::gto::JACK_TREY_SUITED.to_vec()) };
    (J3o) => { Twos::from($crate::analysis::gto::JACK_TREY_OFFSUIT.to_vec()) };
    (J3) => { Twos::from($crate::analysis::gto::JACK_TREY.to_vec()) };
    (J2s) => { Twos::from($crate::analysis::gto::JACK_DEUCE_SUITED.to_vec()) };
    (J2o) => { Twos::from($crate::analysis::gto::JACK_DEUCE_OFFSUIT.to_vec()) };
    (J2) => { Twos::from($crate::analysis::gto::JACK_DEUCE.to_vec()) };

    (T9s) => { Twos::from($crate::analysis::gto::TEN_NINE_SUITED.to_vec()) };
    (T9o) => { Twos::from($crate::analysis::gto::TEN_NINE_OFFSUIT.to_vec()) };
    (T9) => { Twos::from($crate::analysis::gto::TEN_NINE.to_vec()) };
    (T8s) => { Twos::from($crate::analysis::gto::TEN_EIGHT_SUITED.to_vec()) };
    (T8o) => { Twos::from($crate::analysis::gto::TEN_EIGHT_OFFSUIT.to_vec()) };
    (T8) => { Twos::from($crate::analysis::gto::TEN_EIGHT.to_vec()) };
    (T7s) => { Twos::from($crate::analysis::gto::TEN_SEVEN_SUITED.to_vec()) };
    (T7o) => { Twos::from($crate::analysis::gto::TEN_SEVEN_OFFSUIT.to_vec()) };
    (T7) => { Twos::from($crate::analysis::gto::TEN_SEVEN.to_vec()) };
    (T6s) => { Twos::from($crate::analysis::gto::TEN_SIX_SUITED.to_vec()) };
    (T6o) => { Twos::from($crate::analysis::gto::TEN_SIX_OFFSUIT.to_vec()) };
    (T6) => { Twos::from($crate::analysis::gto::TEN_SIX.to_vec()) };
    (T5s) => { Twos::from($crate::analysis::gto::TEN_FIVE_SUITED.to_vec()) };
    (T5o) => { Twos::from($crate::analysis::gto::TEN_FIVE_OFFSUIT.to_vec()) };
    (T5) => { Twos::from($crate::analysis::gto::TEN_FIVE.to_vec()) };
    (T4s) => { Twos::from($crate::analysis::gto::TEN_FOUR_SUITED.to_vec()) };
    (T4o) => { Twos::from($crate::analysis::gto::TEN_FOUR_OFFSUIT.to_vec()) };
    (T4) => { Twos::from($crate::analysis::gto::TEN_FOUR.to_vec()) };
    (T3s) => { Twos::from($crate::analysis::gto::TEN_TREY_SUITED.to_vec()) };
    (T3o) => { Twos::from($crate::analysis::gto::TEN_TREY_OFFSUIT.to_vec()) };
    (T3) => { Twos::from($crate::analysis::gto::TEN_TREY.to_vec()) };
    (T2s) => { Twos::from($crate::analysis::gto::TEN_DEUCE_SUITED.to_vec()) };
    (T2o) => { Twos::from($crate::analysis::gto::TEN_DEUCE_OFFSUIT.to_vec()) };
    (T2) => { Twos::from($crate::analysis::gto::TEN_DEUCE.to_vec()) };

    (98s) => { Twos::from($crate::analysis::gto::NINE_EIGHT_SUITED.to_vec()) };
    (98o) => { Twos::from($crate::analysis::gto::NINE_EIGHT_OFFSUIT.to_vec()) };
    (98) => { Twos::from($crate::analysis::gto::NINE_EIGHT.to_vec()) };
    (97s) => { Twos::from($crate::analysis::gto::NINE_SEVEN_SUITED.to_vec()) };
    (97o) => { Twos::from($crate::analysis::gto::NINE_SEVEN_OFFSUIT.to_vec()) };
    (97) => { Twos::from($crate::analysis::gto::NINE_SEVEN.to_vec()) };
    (96s) => { Twos::from($crate::analysis::gto::NINE_SIX_SUITED.to_vec()) };
    (96o) => { Twos::from($crate::analysis::gto::NINE_SIX_OFFSUIT.to_vec()) };
    (96) => { Twos::from($crate::analysis::gto::NINE_SIX.to_vec()) };
    (95s) => { Twos::from($crate::analysis::gto::NINE_FIVE_SUITED.to_vec()) };
    (95o) => { Twos::from($crate::analysis::gto::NINE_FIVE_OFFSUIT.to_vec()) };
    (95) => { Twos::from($crate::analysis::gto::NINE_FIVE.to_vec()) };
    (94s) => { Twos::from($crate::analysis::gto::NINE_FOUR_SUITED.to_vec()) };
    (94o) => { Twos::from($crate::analysis::gto::NINE_FOUR_OFFSUIT.to_vec()) };
    (94) => { Twos::from($crate::analysis::gto::NINE_FOUR.to_vec()) };
    (93s) => { Twos::from($crate::analysis::gto::NINE_TREY_SUITED.to_vec()) };
    (93o) => { Twos::from($crate::analysis::gto::NINE_TREY_OFFSUIT.to_vec()) };
    (93) => { Twos::from($crate::analysis::gto::NINE_TREY.to_vec()) };
    (92s) => { Twos::from($crate::analysis::gto::NINE_DEUCE_SUITED.to_vec()) };
    (92o) => { Twos::from($crate::analysis::gto::NINE_DEUCE_OFFSUIT.to_vec()) };
    (92) => { Twos::from($crate::analysis::gto::NINE_DEUCE.to_vec()) };

    (87s) => { Twos::from($crate::analysis::gto::EIGHT_SEVEN_SUITED.to_vec()) };
    (87o) => { Twos::from($crate::analysis::gto::EIGHT_SEVEN_OFFSUIT.to_vec()) };
    (87) => { Twos::from($crate::analysis::gto::EIGHT_SEVEN.to_vec()) };
    (86s) => { Twos::from($crate::analysis::gto::EIGHT_SIX_SUITED.to_vec()) };
    (86o) => { Twos::from($crate::analysis::gto::EIGHT_SIX_OFFSUIT.to_vec()) };
    (86) => { Twos::from($crate::analysis::gto::EIGHT_SIX.to_vec()) };
    (85s) => { Twos::from($crate::analysis::gto::EIGHT_FIVE_SUITED.to_vec()) };
    (85o) => { Twos::from($crate::analysis::gto::EIGHT_FIVE_OFFSUIT.to_vec()) };
    (85) => { Twos::from($crate::analysis::gto::EIGHT_FIVE.to_vec()) };
    (84s) => { Twos::from($crate::analysis::gto::EIGHT_FOUR_SUITED.to_vec()) };
    (84o) => { Twos::from($crate::analysis::gto::EIGHT_FOUR_OFFSUIT.to_vec()) };
    (84) => { Twos::from($crate::analysis::gto::EIGHT_FOUR.to_vec()) };
    (83s) => { Twos::from($crate::analysis::gto::EIGHT_TREY_SUITED.to_vec()) };
    (83o) => { Twos::from($crate::analysis::gto::EIGHT_TREY_OFFSUIT.to_vec()) };
    (83) => { Twos::from($crate::analysis::gto::EIGHT_TREY.to_vec()) };
    (82s) => { Twos::from($crate::analysis::gto::EIGHT_DEUCE_SUITED.to_vec()) };
    (82o) => { Twos::from($crate::analysis::gto::EIGHT_DEUCE_OFFSUIT.to_vec()) };
    (82) => { Twos::from($crate::analysis::gto::EIGHT_DEUCE.to_vec()) };

    (76s) => { Twos::from($crate::analysis::gto::SEVEN_SIX_SUITED.to_vec()) };
    (76o) => { Twos::from($crate::analysis::gto::SEVEN_SIX_OFFSUIT.to_vec()) };
    (76) => { Twos::from($crate::analysis::gto::SEVEN_SIX.to_vec()) };
    (75s) => { Twos::from($crate::analysis::gto::SEVEN_FIVE_SUITED.to_vec()) };
    (75o) => { Twos::from($crate::analysis::gto::SEVEN_FIVE_OFFSUIT.to_vec()) };
    (75) => { Twos::from($crate::analysis::gto::SEVEN_FIVE.to_vec()) };
    (74s) => { Twos::from($crate::analysis::gto::SEVEN_FOUR_SUITED.to_vec()) };
    (74o) => { Twos::from($crate::analysis::gto::SEVEN_FOUR_OFFSUIT.to_vec()) };
    (74) => { Twos::from($crate::analysis::gto::SEVEN_FOUR.to_vec()) };
    (73s) => { Twos::from($crate::analysis::gto::SEVEN_TREY_SUITED.to_vec()) };
    (73o) => { Twos::from($crate::analysis::gto::SEVEN_TREY_OFFSUIT.to_vec()) };
    (73) => { Twos::from($crate::analysis::gto::SEVEN_TREY.to_vec()) };
    (72s) => { Twos::from($crate::analysis::gto::SEVEN_DEUCE_SUITED.to_vec()) };
    (72o) => { Twos::from($crate::analysis::gto::SEVEN_DEUCE_OFFSUIT.to_vec()) };
    (72) => { Twos::from($crate::analysis::gto::SEVEN_DEUCE.to_vec()) };

    (65s) => { Twos::from($crate::analysis::gto::SIX_FIVE_SUITED.to_vec()) };
    (65o) => { Twos::from($crate::analysis::gto::SIX_FIVE_OFFSUIT.to_vec()) };
    (65) => { Twos::from($crate::analysis::gto::SIX_FIVE.to_vec()) };
    (64s) => { Twos::from($crate::analysis::gto::SIX_FOUR_SUITED.to_vec()) };
    (64o) => { Twos::from($crate::analysis::gto::SIX_FOUR_OFFSUIT.to_vec()) };
    (64) => { Twos::from($crate::analysis::gto::SIX_FOUR.to_vec()) };
    (63s) => { Twos::from($crate::analysis::gto::SIX_TREY_SUITED.to_vec()) };
    (63o) => { Twos::from($crate::analysis::gto::SIX_TREY_OFFSUIT.to_vec()) };
    (63) => { Twos::from($crate::analysis::gto::SIX_TREY.to_vec()) };
    (62s) => { Twos::from($crate::analysis::gto::SIX_DEUCE_SUITED.to_vec()) };
    (62o) => { Twos::from($crate::analysis::gto::SIX_DEUCE_OFFSUIT.to_vec()) };
    (62) => { Twos::from($crate::analysis::gto::SIX_DEUCE.to_vec()) };

    (54s) => { Twos::from($crate::analysis::gto::FIVE_FOUR_SUITED.to_vec()) };
    (54o) => { Twos::from($crate::analysis::gto::FIVE_FOUR_OFFSUIT.to_vec()) };
    (54) => { Twos::from($crate::analysis::gto::FIVE_FOUR.to_vec()) };
    (53s) => { Twos::from($crate::analysis::gto::FIVE_TREY_SUITED.to_vec()) };
    (53o) => { Twos::from($crate::analysis::gto::FIVE_TREY_OFFSUIT.to_vec()) };
    (53) => { Twos::from($crate::analysis::gto::FIVE_TREY.to_vec()) };
    (52s) => { Twos::from($crate::analysis::gto::FIVE_DEUCE_SUITED.to_vec()) };
    (52o) => { Twos::from($crate::analysis::gto::FIVE_DEUCE_OFFSUIT.to_vec()) };
    (52) => { Twos::from($crate::analysis::gto::FIVE_DEUCE.to_vec()) };

    (43s) => { Twos::from($crate::analysis::gto::FOUR_TREY_SUITED.to_vec()) };
    (43o) => { Twos::from($crate::analysis::gto::FOUR_TREY_OFFSUIT.to_vec()) };
    (43) => { Twos::from($crate::analysis::gto::FOUR_TREY.to_vec()) };
    (42s) => { Twos::from($crate::analysis::gto::FOUR_DEUCE_SUITED.to_vec()) };
    (42o) => { Twos::from($crate::analysis::gto::FOUR_DEUCE_OFFSUIT.to_vec()) };
    (42) => { Twos::from($crate::analysis::gto::FOUR_DEUCE.to_vec()) };

    (32s) => { Twos::from($crate::analysis::gto::TREY_DEUCE_SUITED.to_vec()) };
    (32o) => { Twos::from($crate::analysis::gto::TREY_DEUCE_OFFSUIT.to_vec()) };
    (32) => { Twos::from($crate::analysis::gto::TREY_DEUCE.to_vec()) };

    (AQ+) => {
        Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_KING.to_vec())
        )
    };
    (AJ+) => {
        Twos::from($crate::analysis::gto::ACE_JACK.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_KING.to_vec())
            )
        )
    };
    (AT+) => {
        Twos::from($crate::analysis::gto::ACE_TEN.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_JACK.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_KING.to_vec())
                )
            )
        )
    };
    (A9+) => {
        Twos::from($crate::analysis::gto::ACE_NINE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_TEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_JACK.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_KING.to_vec())
                    )
                )
            )
        )
    };
    (A8+) => {
        Twos::from($crate::analysis::gto::ACE_EIGHT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_NINE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_TEN.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_JACK.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_KING.to_vec())
                        )
                    )
                )
            )
        )
    };
    (A7+) => {
        Twos::from($crate::analysis::gto::ACE_SEVEN.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_EIGHT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_NINE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_TEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_JACK.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_KING.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (A6+) => {
        Twos::from($crate::analysis::gto::ACE_SIX.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_SEVEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_EIGHT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_NINE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_TEN.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_JACK.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_KING.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A5+) => {
        Twos::from($crate::analysis::gto::ACE_FIVE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_SIX.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_SEVEN.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_EIGHT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_NINE.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_TEN.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_JACK.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_KING.to_vec())
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
        Twos::from($crate::analysis::gto::ACE_FOUR.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_FIVE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_SIX.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_SEVEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_EIGHT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_NINE.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_TEN.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_JACK.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::ACE_KING.to_vec())
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
        Twos::from($crate::analysis::gto::ACE_TREY.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_FOUR.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_FIVE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_SIX.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_SEVEN.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_EIGHT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_NINE.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_TEN.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_JACK.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::ACE_KING.to_vec())
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
        Twos::from($crate::analysis::gto::ACE_DEUCE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_TREY.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_FOUR.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_FIVE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_SIX.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_SEVEN.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_EIGHT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_NINE.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_TEN.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::ACE_JACK.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::ACE_QUEEN.to_vec()).extend(
                                                    &Twos::from($crate::analysis::gto::ACE_KING.to_vec())
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
        Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec())
        )
    };
    (AJs+) => {
        Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec())
            )
        )
    };
    (ATs+) => {
        Twos::from($crate::analysis::gto::ACE_TEN_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec())
                )
            )
        )
    };
    (A9s+) => {
        Twos::from($crate::analysis::gto::ACE_NINE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_TEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec())
                    )
                )
            )
        )
    };
    (A8s+) => {
        Twos::from($crate::analysis::gto::ACE_EIGHT_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_NINE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_TEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec())
                        )
                    )
                )
            )
        )
    };
    (A7s+) => {
        Twos::from($crate::analysis::gto::ACE_SEVEN_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_EIGHT_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_NINE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_TEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (A6s+) => {
        Twos::from($crate::analysis::gto::ACE_SIX_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_SEVEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_EIGHT_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_NINE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_TEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A5s+) => {
        Twos::from($crate::analysis::gto::ACE_FIVE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_SIX_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_SEVEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_EIGHT_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_NINE_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_TEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec())
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
        Twos::from($crate::analysis::gto::ACE_FOUR_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_FIVE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_SIX_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_SEVEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_EIGHT_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_NINE_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_TEN_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec())
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
        Twos::from($crate::analysis::gto::ACE_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_FOUR_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_FIVE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_SIX_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_SEVEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_EIGHT_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_NINE_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_TEN_SUITED.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec())
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
        Twos::from($crate::analysis::gto::ACE_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_FOUR_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_FIVE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_SIX_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_SEVEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_EIGHT_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_NINE_SUITED.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_TEN_SUITED.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::ACE_QUEEN_SUITED.to_vec()).extend(
                                                    &Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec())
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
        Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec())
        )
    };
    (AJo+) => {
        Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec())
            )
        )
    };
    (ATo+) => {
        Twos::from($crate::analysis::gto::ACE_TEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec())
                )
            )
        )
    };
    (A9o+) => {
        Twos::from($crate::analysis::gto::ACE_NINE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_TEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec())
                    )
                )
            )
        )
    };
    (A8o+) => {
        Twos::from($crate::analysis::gto::ACE_EIGHT_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_NINE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_TEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec())
                        )
                    )
                )
            )
        )
    };
    (A7o+) => {
        Twos::from($crate::analysis::gto::ACE_SEVEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_NINE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_TEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (A6o+) => {
        Twos::from($crate::analysis::gto::ACE_SIX_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_SEVEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_NINE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_TEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (A5o+) => {
        Twos::from($crate::analysis::gto::ACE_FIVE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_SIX_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_SEVEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_NINE_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_TEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec())
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
        Twos::from($crate::analysis::gto::ACE_FOUR_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_FIVE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_SIX_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_SEVEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_NINE_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_TEN_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec())
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
        Twos::from($crate::analysis::gto::ACE_TREY_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_FOUR_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_FIVE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_SIX_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_SEVEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_NINE_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_TEN_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec())
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
        Twos::from($crate::analysis::gto::ACE_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::ACE_TREY_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::ACE_FOUR_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::ACE_FIVE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::ACE_SIX_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::ACE_SEVEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::ACE_EIGHT_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::ACE_NINE_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::ACE_TEN_OFFSUIT.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::ACE_QUEEN_OFFSUIT.to_vec()).extend(
                                                    &Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec())
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

    (KQ+) => {
        Twos::from($crate::analysis::gto::ACE_KING.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec())
        )
    };
    (KJ+) => {
        Twos::from($crate::analysis::gto::KING_JACK.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec()))
    };
    (KT+) => {
        Twos::from($crate::analysis::gto::KING_TEN.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_JACK.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec())
            )
        )
    };
    (K9+) => {
        Twos::from($crate::analysis::gto::KING_NINE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_TEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_JACK.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec())
                )
            )
        )
    };
    (K8+) => {
        Twos::from($crate::analysis::gto::KING_EIGHT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_NINE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_TEN.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_JACK.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec())
                    )
                )
            )
        )
    };
    (K7+) => {
        Twos::from($crate::analysis::gto::KING_SEVEN.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_EIGHT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_NINE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_TEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_JACK.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec())
                        )
                    )
                )
            )
        )
    };
    (K6+) => {
        Twos::from($crate::analysis::gto::KING_SIX.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_SEVEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_EIGHT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_NINE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_TEN.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_JACK.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (K5+) => {
        Twos::from($crate::analysis::gto::KING_FIVE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_SIX.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_SEVEN.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_EIGHT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_NINE.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_TEN.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_JACK.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (K4+) => {
        Twos::from($crate::analysis::gto::KING_FOUR.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_FIVE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_SIX.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_SEVEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_EIGHT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_NINE.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_TEN.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_JACK.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec())
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
        Twos::from($crate::analysis::gto::KING_TREY.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_FOUR.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_FIVE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_SIX.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_SEVEN.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_EIGHT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_NINE.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_TEN.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::KING_JACK.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec())
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
        Twos::from($crate::analysis::gto::KING_DEUCE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_TREY.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_FOUR.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_FIVE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_SIX.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_SEVEN.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_EIGHT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_NINE.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::KING_TEN.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::KING_JACK.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec())
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

    (KQs+) => {
        Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec())
        )
    };
    (KJs+) => {
        Twos::from($crate::analysis::gto::KING_JACK_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec())
        )
    };
    (KTs+) => {
        Twos::from($crate::analysis::gto::KING_TEN_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_JACK_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec())
            )
        )
    };
    (K9s+) => {
        Twos::from($crate::analysis::gto::KING_NINE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_TEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_JACK_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec())
                )
            )
        )
    };
    (K8s+) => {
        Twos::from($crate::analysis::gto::KING_EIGHT_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_NINE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_TEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_JACK_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec())
                    )
                )
            )
        )
    };
    (K7s+) => {
        Twos::from($crate::analysis::gto::KING_SEVEN_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_EIGHT_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_NINE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_TEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_JACK_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec())
                        )
                    )
                )
            )
        )
    };
    (K6s+) => {
        Twos::from($crate::analysis::gto::KING_SIX_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_SEVEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_EIGHT_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_NINE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_TEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_JACK_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (K5s+) => {
        Twos::from($crate::analysis::gto::KING_FIVE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_SIX_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_SEVEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_EIGHT_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_NINE_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_TEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_JACK_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (K4s+) => {
        Twos::from($crate::analysis::gto::KING_FOUR_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_FIVE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_SIX_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_SEVEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_EIGHT_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_NINE_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_TEN_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_JACK_SUITED.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec())
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
        Twos::from($crate::analysis::gto::KING_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_FOUR_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_FIVE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_SIX_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_SEVEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_EIGHT_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_NINE_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_TEN_SUITED.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::KING_JACK_SUITED.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec())
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
        Twos::from($crate::analysis::gto::KING_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_FOUR_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_FIVE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_SIX_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_SEVEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_EIGHT_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_NINE_SUITED.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::KING_TEN_SUITED.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::KING_JACK_SUITED.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec())
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

    (KQo+) => {
        Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec())
        )
    };
    (KJo+) => {
        Twos::from($crate::analysis::gto::KING_JACK_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec())
        )
    };
    (KTo+) => {
        Twos::from($crate::analysis::gto::KING_TEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_JACK_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec())
            )
        )
    };
    (K9o+) => {
        Twos::from($crate::analysis::gto::KING_NINE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_TEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_JACK_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec())
                )
            )
        )
    };
    (K8o+) => {
        Twos::from($crate::analysis::gto::KING_EIGHT_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_NINE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_TEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_JACK_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec())
                    )
                )
            )
        )
    };
    (K7o+) => {
        Twos::from($crate::analysis::gto::KING_SEVEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_EIGHT_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_NINE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_TEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_JACK_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec())
                        )
                    )
                )
            )
        )
    };
    (K6o+) => {
        Twos::from($crate::analysis::gto::KING_SIX_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_SEVEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_EIGHT_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_NINE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_TEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_JACK_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (K5o+) => {
        Twos::from($crate::analysis::gto::KING_FIVE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_SIX_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_SEVEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_EIGHT_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_NINE_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_TEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_JACK_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (K4o+) => {
        Twos::from($crate::analysis::gto::KING_FOUR_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_FIVE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_SIX_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_SEVEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_EIGHT_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_NINE_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_TEN_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_JACK_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (K3o+) => {
        Twos::from($crate::analysis::gto::KING_TREY_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_FOUR_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_FIVE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_SIX_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_SEVEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_EIGHT_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_NINE_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_TEN_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::KING_JACK_OFFSUIT.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec())
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
    (K2o+) => {
        Twos::from($crate::analysis::gto::KING_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_TREY_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::KING_FOUR_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::KING_FIVE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::KING_SIX_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::KING_SEVEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::KING_EIGHT_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::KING_NINE_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::KING_TEN_OFFSUIT.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::KING_JACK_OFFSUIT.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec())
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

    (QJ+) => {
        Twos::from($crate::analysis::gto::ACE_JACK.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec())
            )
        )
    };
    (QT+) => {
        Twos::from($crate::analysis::gto::QUEEN_TEN.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec())
        )
    };
    (Q9+) => {
        Twos::from($crate::analysis::gto::QUEEN_NINE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_TEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec())
            )
        )
    };
    (Q8+) => {
        Twos::from($crate::analysis::gto::QUEEN_EIGHT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_NINE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_TEN.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec())
                )
            )
        )
    };
    (Q7+) => {
        Twos::from($crate::analysis::gto::QUEEN_SEVEN.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_EIGHT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_NINE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_TEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec())
                    )
                )
            )
        )
    };
    (Q6+) => {
        Twos::from($crate::analysis::gto::QUEEN_SIX.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_SEVEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_EIGHT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_NINE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_TEN.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec())
                        )
                    )
                )
            )
        )
    };
    (Q5+) => {
        Twos::from($crate::analysis::gto::QUEEN_FIVE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_SIX.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_SEVEN.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_EIGHT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_NINE.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_TEN.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (Q4+) => {
        Twos::from($crate::analysis::gto::QUEEN_FOUR.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_FIVE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_SIX.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_SEVEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_EIGHT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_NINE.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_TEN.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (Q3+) => {
        Twos::from($crate::analysis::gto::QUEEN_TREY.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_FOUR.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_FIVE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_SIX.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_SEVEN.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_EIGHT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_NINE.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::QUEEN_TEN.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (Qx) => {
        Twos::from($crate::analysis::gto::QUEEN_DEUCE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_TREY.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_FOUR.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_FIVE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_SIX.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_SEVEN.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_EIGHT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::QUEEN_NINE.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::QUEEN_TEN.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec())
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

    (QJs+) => {
        Twos::from($crate::analysis::gto::ACE_JACK_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec())
            )
        )
    };
    (QTs+) => {
        Twos::from($crate::analysis::gto::QUEEN_TEN_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec())
        )
    };
    (Q9s+) => {
        Twos::from($crate::analysis::gto::QUEEN_NINE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_TEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec())
            )
        )
    };
    (Q8s+) => {
        Twos::from($crate::analysis::gto::QUEEN_EIGHT_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_NINE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_TEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec())
                )
            )
        )
    };
    (Q7s+) => {
        Twos::from($crate::analysis::gto::QUEEN_SEVEN_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_EIGHT_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_NINE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_TEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec())
                    )
                )
            )
        )
    };
    (Q6s+) => {
        Twos::from($crate::analysis::gto::QUEEN_SIX_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_SEVEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_EIGHT_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_NINE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_TEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec())
                        )
                    )
                )
            )
        )
    };
    (Q5s+) => {
        Twos::from($crate::analysis::gto::QUEEN_FIVE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_SIX_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_SEVEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_EIGHT_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_NINE_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_TEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (Q4s+) => {
        Twos::from($crate::analysis::gto::QUEEN_FOUR_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_FIVE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_SIX_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_SEVEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_EIGHT_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_NINE_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_TEN_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (Q3s+) => {
        Twos::from($crate::analysis::gto::QUEEN_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_FOUR_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_FIVE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_SIX_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_SEVEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_EIGHT_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_NINE_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::QUEEN_TEN_SUITED.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (Q2s+) => {
        Twos::from($crate::analysis::gto::QUEEN_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_FOUR_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_FIVE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_SIX_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_SEVEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_EIGHT_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::QUEEN_NINE_SUITED.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::QUEEN_TEN_SUITED.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec())
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

    (QJo+) => {
        Twos::from($crate::analysis::gto::ACE_JACK_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec())
            )
        )
    };
    (QTo+) => {
        Twos::from($crate::analysis::gto::QUEEN_TEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec())
        )
    };
    (Q9o+) => {
        Twos::from($crate::analysis::gto::QUEEN_NINE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_TEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec())
            )
        )
    };
    (Q8o+) => {
        Twos::from($crate::analysis::gto::QUEEN_EIGHT_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_NINE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_TEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec())
                )
            )
        )
    };
    (Q7o+) => {
        Twos::from($crate::analysis::gto::QUEEN_SEVEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_EIGHT_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_NINE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_TEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec())
                    )
                )
            )
        )
    };
    (Q6o+) => {
        Twos::from($crate::analysis::gto::QUEEN_SIX_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_SEVEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_EIGHT_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_NINE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_TEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec())
                        )
                    )
                )
            )
        )
    };
    (Q5o+) => {
        Twos::from($crate::analysis::gto::QUEEN_FIVE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_SIX_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_SEVEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_EIGHT_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_NINE_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_TEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (Q4o+) => {
        Twos::from($crate::analysis::gto::QUEEN_FOUR_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_FIVE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_SIX_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_SEVEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_EIGHT_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_NINE_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_TEN_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (Q3o+) => {
        Twos::from($crate::analysis::gto::QUEEN_TREY_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_FOUR_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_FIVE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_SIX_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_SEVEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_EIGHT_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_NINE_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::QUEEN_TEN_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (Q2o+) => {
        Twos::from($crate::analysis::gto::QUEEN_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::QUEEN_TREY_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_FOUR_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::QUEEN_FIVE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::QUEEN_SIX_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::QUEEN_SEVEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::QUEEN_EIGHT_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::QUEEN_NINE_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::QUEEN_TEN_OFFSUIT.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec())
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

    (JT+) => {
        Twos::from($crate::analysis::gto::ACE_KING.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN.to_vec())
                )
            )
        )
    };
    (J9+) => {
        Twos::from($crate::analysis::gto::JACK_NINE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_TEN.to_vec())
        )
    };
    (J8+) => {
        Twos::from($crate::analysis::gto::JACK_EIGHT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_NINE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_TEN.to_vec())
            )
        )
    };
    (J7+) => {
        Twos::from($crate::analysis::gto::JACK_SEVEN.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_EIGHT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_NINE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN.to_vec())
                )
            )
        )
    };
    (J6+) => {
        Twos::from($crate::analysis::gto::JACK_SIX.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_SEVEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_EIGHT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_NINE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_TEN.to_vec())
                    )
                )
            )
        )
    };
    (J5+) => {
        Twos::from($crate::analysis::gto::JACK_FIVE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_SIX.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_SEVEN.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_EIGHT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_NINE.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_TEN.to_vec())
                        )
                    )
                )
            )
        )
    };
    (J4+) => {
        Twos::from($crate::analysis::gto::JACK_FOUR.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_FIVE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_SIX.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_SEVEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_EIGHT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_NINE.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::JACK_TEN.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (J3+) => {
        Twos::from($crate::analysis::gto::JACK_TREY.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_FOUR.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_FIVE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_SIX.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_SEVEN.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_EIGHT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::JACK_NINE.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::JACK_TEN.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (Jx) => {
        Twos::from($crate::analysis::gto::JACK_DEUCE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_TREY.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_FOUR.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_FIVE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_SIX.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_SEVEN.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::JACK_EIGHT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::JACK_NINE.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::JACK_TEN.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };

    (JTs+) => {
        Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec())
                )
            )
        )
    };
    (J9s+) => {
        Twos::from($crate::analysis::gto::JACK_NINE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec())
        )
    };
    (J8s+) => {
        Twos::from($crate::analysis::gto::JACK_EIGHT_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_NINE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec())
            )
        )
    };
    (J7s+) => {
        Twos::from($crate::analysis::gto::JACK_SEVEN_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_EIGHT_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_NINE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec())
                )
            )
        )
    };
    (J6s+) => {
        Twos::from($crate::analysis::gto::JACK_SIX_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_SEVEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_EIGHT_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_NINE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec())
                    )
                )
            )
        )
    };
    (J5s+) => {
        Twos::from($crate::analysis::gto::JACK_FIVE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_SIX_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_SEVEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_EIGHT_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_NINE_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec())
                        )
                    )
                )
            )
        )
    };
    (J4s+) => {
        Twos::from($crate::analysis::gto::JACK_FOUR_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_FIVE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_SIX_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_SEVEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_EIGHT_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_NINE_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (J3s+) => {
        Twos::from($crate::analysis::gto::JACK_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_FOUR_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_FIVE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_SIX_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_SEVEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_EIGHT_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::JACK_NINE_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (J2s+) => {
        Twos::from($crate::analysis::gto::JACK_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_FOUR_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_FIVE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_SIX_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_SEVEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::JACK_EIGHT_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::JACK_NINE_SUITED.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };

    (JTo+) => {
        Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec())
                )
            )
        )
    };
    (J9o+) => {
        Twos::from($crate::analysis::gto::JACK_NINE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec())
        )
    };
    (J8o+) => {
        Twos::from($crate::analysis::gto::JACK_EIGHT_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_NINE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec())
            )
        )
    };
    (J7o+) => {
        Twos::from($crate::analysis::gto::JACK_SEVEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_EIGHT_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_NINE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec())
                )
            )
        )
    };
    (J6o+) => {
        Twos::from($crate::analysis::gto::JACK_SIX_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_SEVEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_EIGHT_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_NINE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec())
                    )
                )
            )
        )
    };
    (J5o+) => {
        Twos::from($crate::analysis::gto::JACK_FIVE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_SIX_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_SEVEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_EIGHT_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_NINE_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec())
                        )
                    )
                )
            )
        )
    };
    (J4o+) => {
        Twos::from($crate::analysis::gto::JACK_FOUR_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_FIVE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_SIX_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_SEVEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_EIGHT_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_NINE_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (J3o+) => {
        Twos::from($crate::analysis::gto::JACK_TREY_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_FOUR_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_FIVE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_SIX_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_SEVEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_EIGHT_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::JACK_NINE_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };
    (J2o+) => {
        Twos::from($crate::analysis::gto::JACK_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::JACK_TREY_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::JACK_FOUR_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_FIVE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::JACK_SIX_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::JACK_SEVEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::JACK_EIGHT_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::JACK_NINE_OFFSUIT.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec())
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    };

    (T9+) => {
        Twos::from($crate::analysis::gto::ACE_KING.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_NINE.to_vec())
                    )
                )
            )
        )
    };
    (T8+) => {
        Twos::from($crate::analysis::gto::TEN_EIGHT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_NINE.to_vec())
        )
    };
    (T7+) => {
        Twos::from($crate::analysis::gto::TEN_SEVEN.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_EIGHT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_NINE.to_vec())
            )
        )
    };
    (T6+) => {
        Twos::from($crate::analysis::gto::TEN_SIX.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_SEVEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_EIGHT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_NINE.to_vec())
                )
            )
        )
    };
    (T5+) => {
        Twos::from($crate::analysis::gto::TEN_FIVE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_SIX.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_SEVEN.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_EIGHT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_NINE.to_vec())
                    )
                )
            )
        )
    };
    (T4+) => {
        Twos::from($crate::analysis::gto::TEN_FOUR.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_FIVE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_SIX.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_SEVEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_EIGHT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::TEN_NINE.to_vec())
                        )
                    )
                )
            )
        )
    };
    (T3+) => {
        Twos::from($crate::analysis::gto::TEN_TREY.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_FOUR.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_FIVE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_SIX.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_SEVEN.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::TEN_EIGHT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::TEN_NINE.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (Tx) => {
        Twos::from($crate::analysis::gto::TEN_DEUCE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_TREY.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_FOUR.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_FIVE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_SIX.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::TEN_SEVEN.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::TEN_EIGHT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::TEN_NINE.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };

    (T9s+) => {
        Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_NINE_SUITED.to_vec())
                    )
                )
            )
        )
    };
    (T8s+) => {
        Twos::from($crate::analysis::gto::TEN_EIGHT_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_NINE_SUITED.to_vec())
        )
    };
    (T7s+) => {
        Twos::from($crate::analysis::gto::TEN_SEVEN_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_EIGHT_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_NINE_SUITED.to_vec())
            )
        )
    };
    (T6s+) => {
        Twos::from($crate::analysis::gto::TEN_SIX_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_SEVEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_EIGHT_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_NINE_SUITED.to_vec())
                )
            )
        )
    };
    (T5s+) => {
        Twos::from($crate::analysis::gto::TEN_FIVE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_SIX_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_SEVEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_EIGHT_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_NINE_SUITED.to_vec())
                    )
                )
            )
        )
    };
    (T4s+) => {
        Twos::from($crate::analysis::gto::TEN_FOUR_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_FIVE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_SIX_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_SEVEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_EIGHT_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::TEN_NINE_SUITED.to_vec())
                        )
                    )
                )
            )
        )
    };
    (T3s+) => {
        Twos::from($crate::analysis::gto::TEN_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_FOUR_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_FIVE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_SIX_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_SEVEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::TEN_EIGHT_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::TEN_NINE_SUITED.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (T2s+) => {
        Twos::from($crate::analysis::gto::TEN_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_FOUR_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_FIVE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_SIX_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::TEN_SEVEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::TEN_EIGHT_SUITED.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::TEN_NINE_SUITED.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };

    (T9o+) => {
        Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_NINE_OFFSUIT.to_vec())
                    )
                )
            )
        )
    };
    (T8o+) => {
        Twos::from($crate::analysis::gto::TEN_EIGHT_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_NINE_OFFSUIT.to_vec())
        )
    };
    (T7o+) => {
        Twos::from($crate::analysis::gto::TEN_SEVEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_EIGHT_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_NINE_OFFSUIT.to_vec())
            )
        )
    };
    (T6o+) => {
        Twos::from($crate::analysis::gto::TEN_SIX_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_SEVEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_EIGHT_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_NINE_OFFSUIT.to_vec())
                )
            )
        )
    };
    (T5o+) => {
        Twos::from($crate::analysis::gto::TEN_FIVE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_SIX_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_SEVEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_EIGHT_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_NINE_OFFSUIT.to_vec())
                    )
                )
            )
        )
    };
    (T4o+) => {
        Twos::from($crate::analysis::gto::TEN_FOUR_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_FIVE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_SIX_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_SEVEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_EIGHT_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::TEN_NINE_OFFSUIT.to_vec())
                        )
                    )
                )
            )
        )
    };
    (T3o+) => {
        Twos::from($crate::analysis::gto::TEN_TREY_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_FOUR_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_FIVE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_SIX_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_SEVEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::TEN_EIGHT_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::TEN_NINE_OFFSUIT.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };
    (T2o+) => {
        Twos::from($crate::analysis::gto::TEN_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::TEN_TREY_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::TEN_FOUR_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::TEN_FIVE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_SIX_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::TEN_SEVEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::TEN_EIGHT_OFFSUIT.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::TEN_NINE_OFFSUIT.to_vec())
                                )
                            )
                        )
                    )
                )
            )
        )
    };

    (98+) => {
        Twos::from($crate::analysis::gto::ACE_KING.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_NINE.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::NINE_EIGHT.to_vec())
                        )
                    )
                )
            )
        )
    };
    (97+) => {
        Twos::from($crate::analysis::gto::NINE_SEVEN.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_EIGHT.to_vec())
        )
    };
    (96+) => {
        Twos::from($crate::analysis::gto::NINE_SIX.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_SEVEN.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_EIGHT.to_vec())
            )
        )
    };
    (95+) => {
        Twos::from($crate::analysis::gto::NINE_FIVE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_SIX.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_SEVEN.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_EIGHT.to_vec())
                )
            )
        )
    };
    (94+) => {
        Twos::from($crate::analysis::gto::NINE_FOUR.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_FIVE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_SIX.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_SEVEN.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::NINE_EIGHT.to_vec())
                    )
                )
            )
        )
    };
    (93+) => {
        Twos::from($crate::analysis::gto::NINE_TREY.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_FOUR.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_FIVE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_SIX.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::NINE_SEVEN.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::NINE_EIGHT.to_vec())
                        )
                    )
                )
            )
        )
    };
    (9x) => {
        Twos::from($crate::analysis::gto::NINE_DEUCE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_TREY.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_FOUR.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_FIVE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::NINE_SIX.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::NINE_SEVEN.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::NINE_EIGHT.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };

    (98s+) => {
        Twos::from($crate::analysis::gto::ACE_KING_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_NINE_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::NINE_EIGHT_SUITED.to_vec())
                        )
                    )
                )
            )
        )
    };
    (97s+) => {
        Twos::from($crate::analysis::gto::NINE_SEVEN_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_EIGHT_SUITED.to_vec())
        )
    };
    (96s+) => {
        Twos::from($crate::analysis::gto::NINE_SIX_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_SEVEN_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_EIGHT_SUITED.to_vec())
            )
        )
    };
    (95s+) => {
        Twos::from($crate::analysis::gto::NINE_FIVE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_SIX_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_SEVEN_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_EIGHT_SUITED.to_vec())
                )
            )
        )
    };
    (94s+) => {
        Twos::from($crate::analysis::gto::NINE_FOUR_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_FIVE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_SIX_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_SEVEN_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::NINE_EIGHT_SUITED.to_vec())
                    )
                )
            )
        )
    };
    (93s+) => {
        Twos::from($crate::analysis::gto::NINE_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_FOUR_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_FIVE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_SIX_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::NINE_SEVEN_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::NINE_EIGHT_SUITED.to_vec())
                        )
                    )
                )
            )
        )
    };
    (92s+) => {
        Twos::from($crate::analysis::gto::NINE_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_FOUR_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_FIVE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::NINE_SIX_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::NINE_SEVEN_SUITED.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::NINE_EIGHT_SUITED.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };

    (98o+) => {
        Twos::from($crate::analysis::gto::ACE_KING_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::KING_QUEEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::QUEEN_JACK_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::JACK_TEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::TEN_NINE_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::NINE_EIGHT_OFFSUIT.to_vec())
                        )
                    )
                )
            )
        )
    };
    (97o+) => {
        Twos::from($crate::analysis::gto::NINE_SEVEN_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_EIGHT_OFFSUIT.to_vec())
        )
    };
    (96o+) => {
        Twos::from($crate::analysis::gto::NINE_SIX_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_SEVEN_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_EIGHT_OFFSUIT.to_vec())
            )
        )
    };
    (95o+) => {
        Twos::from($crate::analysis::gto::NINE_FIVE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_SIX_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_SEVEN_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_EIGHT_OFFSUIT.to_vec())
                )
            )
        )
    };
    (94o+) => {
        Twos::from($crate::analysis::gto::NINE_FOUR_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_FIVE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_SIX_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_SEVEN_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::NINE_EIGHT_OFFSUIT.to_vec())
                    )
                )
            )
        )
    };
    (93o+) => {
        Twos::from($crate::analysis::gto::NINE_TREY_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_FOUR_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_FIVE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_SIX_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::NINE_SEVEN_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::NINE_EIGHT_OFFSUIT.to_vec())
                        )
                    )
                )
            )
        )
    };
    (92o+) => {
        Twos::from($crate::analysis::gto::NINE_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::NINE_TREY_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::NINE_FOUR_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::NINE_FIVE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::NINE_SIX_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::NINE_SEVEN_OFFSUIT.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::NINE_EIGHT_OFFSUIT.to_vec())
                            )
                        )
                    )
                )
            )
        )
    };

    (86+) => {
        Twos::from($crate::analysis::gto::EIGHT_SIX.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_SEVEN.to_vec())
        )
    };
    (85+) => {
        Twos::from($crate::analysis::gto::EIGHT_FIVE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_SIX.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_SEVEN.to_vec())
            )
        )
    };
    (84+) => {
        Twos::from($crate::analysis::gto::EIGHT_FOUR.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_FIVE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_SIX.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::EIGHT_SEVEN.to_vec())
                )
            )
        )
    };
    (83+) => {
        Twos::from($crate::analysis::gto::EIGHT_TREY.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_FOUR.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_FIVE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::EIGHT_SIX.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::EIGHT_SEVEN.to_vec())
                    )
                )
            )
        )
    };
    (8x) => {
        Twos::from($crate::analysis::gto::EIGHT_DEUCE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_TREY.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_FOUR.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::EIGHT_FIVE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::EIGHT_SIX.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::EIGHT_SEVEN.to_vec())
                        )
                    )
                )
            )
        )
    };

    (86s+) => {
        Twos::from($crate::analysis::gto::EIGHT_SIX_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_SEVEN_SUITED.to_vec())
        )
    };
    (85s+) => {
        Twos::from($crate::analysis::gto::EIGHT_FIVE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_SIX_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_SEVEN_SUITED.to_vec())
            )
        )
    };
    (84s+) => {
        Twos::from($crate::analysis::gto::EIGHT_FOUR_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_FIVE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_SIX_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::EIGHT_SEVEN_SUITED.to_vec())
                )
            )
        )
    };
    (83s+) => {
        Twos::from($crate::analysis::gto::EIGHT_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_FOUR_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_FIVE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::EIGHT_SIX_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::EIGHT_SEVEN_SUITED.to_vec())
                    )
                )
            )
        )
    };
    (82s+) => {
        Twos::from($crate::analysis::gto::EIGHT_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_FOUR_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::EIGHT_FIVE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::EIGHT_SIX_SUITED.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::EIGHT_SEVEN_SUITED.to_vec())
                        )
                    )
                )
            )
        )
    };

    (86o+) => {
        Twos::from($crate::analysis::gto::EIGHT_SIX_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_SEVEN_OFFSUIT.to_vec())
        )
    };
    (85o+) => {
        Twos::from($crate::analysis::gto::EIGHT_FIVE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_SIX_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_SEVEN_OFFSUIT.to_vec())
            )
        )
    };
    (84o+) => {
        Twos::from($crate::analysis::gto::EIGHT_FOUR_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_FIVE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_SIX_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::EIGHT_SEVEN_OFFSUIT.to_vec())
                )
            )
        )
    };
    (83o+) => {
        Twos::from($crate::analysis::gto::EIGHT_TREY_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_FOUR_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_FIVE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::EIGHT_SIX_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::EIGHT_SEVEN_OFFSUIT.to_vec())
                    )
                )
            )
        )
    };
    (82o+) => {
        Twos::from($crate::analysis::gto::EIGHT_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::EIGHT_TREY_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::EIGHT_FOUR_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::EIGHT_FIVE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::EIGHT_SIX_OFFSUIT.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::EIGHT_SEVEN_OFFSUIT.to_vec())
                        )
                    )
                )
            )
        )
    };

    (75+) => {
        Twos::from($crate::analysis::gto::SEVEN_FIVE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_SIX.to_vec())
        )
    };
    (74+) => {
        Twos::from($crate::analysis::gto::SEVEN_FOUR.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_FIVE.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SEVEN_SIX.to_vec())
            )
        )
    };
    (73+) => {
        Twos::from($crate::analysis::gto::SEVEN_TREY.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_FOUR.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SEVEN_FIVE.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SEVEN_SIX.to_vec())
                )
            )
        )
    };
    (7x) => {
        Twos::from($crate::analysis::gto::SEVEN_DEUCE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_TREY.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SEVEN_FOUR.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SEVEN_FIVE.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::SEVEN_SIX.to_vec())
                    )
                )
            )
        )
    };

    (75s+) => {
        Twos::from($crate::analysis::gto::SEVEN_FIVE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_SIX_SUITED.to_vec())
        )
    };
    (74s+) => {
        Twos::from($crate::analysis::gto::SEVEN_FOUR_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_FIVE_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SEVEN_SIX_SUITED.to_vec())
            )
        )
    };
    (73s+) => {
        Twos::from($crate::analysis::gto::SEVEN_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_FOUR_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SEVEN_FIVE_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SEVEN_SIX_SUITED.to_vec())
                )
            )
        )
    };
    (72s+) => {
        Twos::from($crate::analysis::gto::SEVEN_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SEVEN_FOUR_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SEVEN_FIVE_SUITED.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::SEVEN_SIX_SUITED.to_vec())
                    )
                )
            )
        )
    };

    (75o+) => {
        Twos::from($crate::analysis::gto::SEVEN_FIVE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_SIX_OFFSUIT.to_vec())
        )
    };
    (74o+) => {
        Twos::from($crate::analysis::gto::SEVEN_FOUR_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_FIVE_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SEVEN_SIX_OFFSUIT.to_vec())
            )
        )
    };
    (73o+) => {
        Twos::from($crate::analysis::gto::SEVEN_TREY_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_FOUR_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SEVEN_FIVE_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SEVEN_SIX_OFFSUIT.to_vec())
                )
            )
        )
    };
    (72o+) => {
        Twos::from($crate::analysis::gto::SEVEN_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SEVEN_TREY_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SEVEN_FOUR_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SEVEN_FIVE_OFFSUIT.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::SEVEN_SIX_OFFSUIT.to_vec())
                    )
                )
            )
        )
    };

    (64+) => {
        Twos::from($crate::analysis::gto::SIX_FOUR.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SIX_FIVE.to_vec())
        )
    };
    (63+) => {
        Twos::from($crate::analysis::gto::SIX_TREY.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SIX_FOUR.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SIX_FIVE.to_vec())
            )
        )
    };
    (62+) => {
        Twos::from($crate::analysis::gto::SIX_DEUCE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SIX_TREY.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SIX_FOUR.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SIX_FIVE.to_vec())
                )
            )
        )
    };

    (64s+) => {
        Twos::from($crate::analysis::gto::SIX_FOUR_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SIX_FIVE_SUITED.to_vec())
        )
    };
    (63s+) => {
        Twos::from($crate::analysis::gto::SIX_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SIX_FOUR_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SIX_FIVE_SUITED.to_vec())
            )
        )
    };
    (62s+) => {
        Twos::from($crate::analysis::gto::SIX_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SIX_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SIX_FOUR_SUITED.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SIX_FIVE_SUITED.to_vec())
                )
            )
        )
    };

    (64o+) => {
        Twos::from($crate::analysis::gto::SIX_FOUR_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SIX_FIVE_OFFSUIT.to_vec())
        )
    };
    (63o+) => {
        Twos::from($crate::analysis::gto::SIX_TREY_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SIX_FOUR_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SIX_FIVE_OFFSUIT.to_vec())
            )
        )
    };
    (62o+) => {
        Twos::from($crate::analysis::gto::SIX_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::SIX_TREY_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::SIX_FOUR_OFFSUIT.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SIX_FIVE_OFFSUIT.to_vec())
                )
            )
        )
    };

    (53+) => {
        Twos::from($crate::analysis::gto::FIVE_TREY.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FIVE_FOUR.to_vec())
        )
    };
    (52+) => {
        Twos::from($crate::analysis::gto::FIVE_DEUCE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FIVE_TREY.to_vec()).extend(
                &Twos::from($crate::analysis::gto::FIVE_FOUR.to_vec())
            )
        )
    };

    (53s+) => {
        Twos::from($crate::analysis::gto::FIVE_TREY_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FIVE_FOUR_SUITED.to_vec())
        )
    };
    (52s+) => {
        Twos::from($crate::analysis::gto::FIVE_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FIVE_TREY_SUITED.to_vec()).extend(
                &Twos::from($crate::analysis::gto::FIVE_FOUR_SUITED.to_vec())
            )
        )
    };

    (53o+) => {
        Twos::from($crate::analysis::gto::FIVE_TREY_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FIVE_FOUR_OFFSUIT.to_vec())
        )
    };
    (52o+) => {
        Twos::from($crate::analysis::gto::FIVE_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FIVE_TREY_OFFSUIT.to_vec()).extend(
                &Twos::from($crate::analysis::gto::FIVE_FOUR_OFFSUIT.to_vec())
            )
        )
    };

    (42+) => {
        Twos::from($crate::analysis::gto::FOUR_DEUCE.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FOUR_TREY.to_vec())
        )
    };

    (42s+) => {
        Twos::from($crate::analysis::gto::FOUR_DEUCE_SUITED.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FOUR_TREY_SUITED.to_vec())
        )
    };

    (42o+) => {
        Twos::from($crate::analysis::gto::FOUR_DEUCE_OFFSUIT.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FOUR_TREY_OFFSUIT.to_vec())
        )
    };
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__gto__range_tests {
    use crate::analysis::gto::twos::{DISTINCT_POCKET_PAIRS, Twos};
    use crate::rank::Rank;

    fn assert_on_pair(range: Twos, rank: Rank) {
        assert_eq!(
            range.hashset(),
            DISTINCT_POCKET_PAIRS.filter_on_rank(rank).filter_is_paired().hashset()
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
        let twos = DISTINCT_POCKET_PAIRS
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
        let twos = DISTINCT_POCKET_PAIRS
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
        let twos = DISTINCT_POCKET_PAIRS
            .filter_on_rank(top)
            .filter_on_rank(bottom)
            .hashset();
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
    }

    #[test]
    fn qq_plus() {
        let expected = range!(AA).extend(&range!(KK)).extend(&range!(QQ));

        let actual = range!(QQ+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn jj_plus() {
        let expected = range!(AA).extend(&range!(KK)).extend(&range!(QQ)).extend(&range!(JJ));

        let actual = range!(JJ+);

        assert_eq!(expected, actual);
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
    }

    #[test]
    fn aq_plus() {
        let expected = range!(AQ).extend(&range!(AK));

        let actual = range!(AQ+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn aj_plus() {
        let expected = range!(AJ).extend(&range!(AQ)).extend(&range!(AK));

        let actual = range!(AJ+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn at_plus() {
        let expected = range!(AT).extend(&range!(AJ)).extend(&range!(AQ)).extend(&range!(AK));

        let actual = range!(AT+);

        assert_eq!(expected, actual);
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
    }

    #[test]
    fn kj_plus() {
        let expected = range!(KJ).extend(&range!(KQ));

        let actual = range!(KJ+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn kt_plus() {
        let expected = range!(KT).extend(&range!(KJ)).extend(&range!(KQ));

        let actual = range!(KT+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn k9_plus() {
        let expected = range!(K9).extend(&range!(KT)).extend(&range!(KJ)).extend(&range!(KQ));

        let actual = range!(K9+);

        assert_eq!(expected, actual);
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
    }

    #[test]
    fn kjs_plus() {
        let expected = range!(KJs).extend(&range!(KQs));

        let actual = range!(KJs+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn kts_plus() {
        let expected = range!(KTs).extend(&range!(KJs)).extend(&range!(KQs));

        let actual = range!(KTs+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn k9s_plus() {
        let expected = range!(K9s)
            .extend(&range!(KTs))
            .extend(&range!(KJs))
            .extend(&range!(KQs));

        let actual = range!(K9s+);

        assert_eq!(expected, actual);
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
    }

    #[test]
    fn kjo_plus() {
        let expected = range!(KJo).extend(&range!(KQo));

        let actual = range!(KJo+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn kto_plus() {
        let expected = range!(KTo).extend(&range!(KJo)).extend(&range!(KQo));

        let actual = range!(KTo+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn k9o_plus() {
        let expected = range!(K9o)
            .extend(&range!(KTo))
            .extend(&range!(KJo))
            .extend(&range!(KQo));

        let actual = range!(K9o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn k8o_plus() {
        let expected = range!(K8o)
            .extend(&range!(K9o))
            .extend(&range!(KTo))
            .extend(&range!(KJo))
            .extend(&range!(KQo));

        let actual = range!(K8o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn k7o_plus() {
        let expected = range!(K7o)
            .extend(&range!(K8o))
            .extend(&range!(K9o))
            .extend(&range!(KTo))
            .extend(&range!(KJo))
            .extend(&range!(KQo));

        let actual = range!(K7o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn k6o_plus() {
        let expected = range!(K6o)
            .extend(&range!(K7o))
            .extend(&range!(K8o))
            .extend(&range!(K9o))
            .extend(&range!(KTo))
            .extend(&range!(KJo))
            .extend(&range!(KQo));

        let actual = range!(K6o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn k5o_plus() {
        let expected = range!(K5o)
            .extend(&range!(K6o))
            .extend(&range!(K7o))
            .extend(&range!(K8o))
            .extend(&range!(K9o))
            .extend(&range!(KTo))
            .extend(&range!(KJo))
            .extend(&range!(KQo));

        let actual = range!(K5o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn k4o_plus() {
        let expected = range!(K4o)
            .extend(&range!(K5o))
            .extend(&range!(K6o))
            .extend(&range!(K7o))
            .extend(&range!(K8o))
            .extend(&range!(K9o))
            .extend(&range!(KTo))
            .extend(&range!(KJo))
            .extend(&range!(KQo));

        let actual = range!(K4o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn k3o_plus() {
        let expected = range!(K3o)
            .extend(&range!(K4o))
            .extend(&range!(K5o))
            .extend(&range!(K6o))
            .extend(&range!(K7o))
            .extend(&range!(K8o))
            .extend(&range!(K9o))
            .extend(&range!(KTo))
            .extend(&range!(KJo))
            .extend(&range!(KQo));

        let actual = range!(K3o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn k2o_plus() {
        let expected = range!(K2o)
            .extend(&range!(K3o))
            .extend(&range!(K4o))
            .extend(&range!(K5o))
            .extend(&range!(K6o))
            .extend(&range!(K7o))
            .extend(&range!(K8o))
            .extend(&range!(K9o))
            .extend(&range!(KTo))
            .extend(&range!(KJo))
            .extend(&range!(KQo));

        let actual = range!(K2o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn qt_plus() {
        let expected = range!(QT).extend(&range!(QJ));

        let actual = range!(QT+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q9_plus() {
        let expected = range!(Q9).extend(&range!(QT)).extend(&range!(QJ));

        let actual = range!(Q9+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q8_plus() {
        let expected = range!(Q8).extend(&range!(Q9)).extend(&range!(QT)).extend(&range!(QJ));

        let actual = range!(Q8+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q7_plus() {
        let expected = range!(Q7)
            .extend(&range!(Q8))
            .extend(&range!(Q9))
            .extend(&range!(QT))
            .extend(&range!(QJ));

        let actual = range!(Q7+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q6_plus() {
        let expected = range!(Q6)
            .extend(&range!(Q7))
            .extend(&range!(Q8))
            .extend(&range!(Q9))
            .extend(&range!(QT))
            .extend(&range!(QJ));

        let actual = range!(Q6+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q5_plus() {
        let expected = range!(Q5)
            .extend(&range!(Q6))
            .extend(&range!(Q7))
            .extend(&range!(Q8))
            .extend(&range!(Q9))
            .extend(&range!(QT))
            .extend(&range!(QJ));

        let actual = range!(Q5+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q4_plus() {
        let expected = range!(Q4)
            .extend(&range!(Q5))
            .extend(&range!(Q6))
            .extend(&range!(Q7))
            .extend(&range!(Q8))
            .extend(&range!(Q9))
            .extend(&range!(QT))
            .extend(&range!(QJ));

        let actual = range!(Q4+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q3_plus() {
        let expected = range!(Q3)
            .extend(&range!(Q4))
            .extend(&range!(Q5))
            .extend(&range!(Q6))
            .extend(&range!(Q7))
            .extend(&range!(Q8))
            .extend(&range!(Q9))
            .extend(&range!(QT))
            .extend(&range!(QJ));

        let actual = range!(Q3+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn qx() {
        let expected = range!(Q2)
            .extend(&range!(Q3))
            .extend(&range!(Q4))
            .extend(&range!(Q5))
            .extend(&range!(Q6))
            .extend(&range!(Q7))
            .extend(&range!(Q8))
            .extend(&range!(Q9))
            .extend(&range!(QT))
            .extend(&range!(QJ));

        let actual = range!(Qx);

        assert_eq!(expected, actual);
    }

    #[test]
    fn qts_plus() {
        let expected = range!(QTs).extend(&range!(QJs));

        let actual = range!(QTs+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q9s_plus() {
        let expected = range!(Q9s).extend(&range!(QTs)).extend(&range!(QJs));

        let actual = range!(Q9s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q8s_plus() {
        let expected = range!(Q8s)
            .extend(&range!(Q9s))
            .extend(&range!(QTs))
            .extend(&range!(QJs));

        let actual = range!(Q8s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q7s_plus() {
        let expected = range!(Q7s)
            .extend(&range!(Q8s))
            .extend(&range!(Q9s))
            .extend(&range!(QTs))
            .extend(&range!(QJs));

        let actual = range!(Q7s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q6s_plus() {
        let expected = range!(Q6s)
            .extend(&range!(Q7s))
            .extend(&range!(Q8s))
            .extend(&range!(Q9s))
            .extend(&range!(QTs))
            .extend(&range!(QJs));

        let actual = range!(Q6s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q5s_plus() {
        let expected = range!(Q5s)
            .extend(&range!(Q6s))
            .extend(&range!(Q7s))
            .extend(&range!(Q8s))
            .extend(&range!(Q9s))
            .extend(&range!(QTs))
            .extend(&range!(QJs));

        let actual = range!(Q5s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q4s_plus() {
        let expected = range!(Q4s)
            .extend(&range!(Q5s))
            .extend(&range!(Q6s))
            .extend(&range!(Q7s))
            .extend(&range!(Q8s))
            .extend(&range!(Q9s))
            .extend(&range!(QTs))
            .extend(&range!(QJs));

        let actual = range!(Q4s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q3s_plus() {
        let expected = range!(Q3s)
            .extend(&range!(Q4s))
            .extend(&range!(Q5s))
            .extend(&range!(Q6s))
            .extend(&range!(Q7s))
            .extend(&range!(Q8s))
            .extend(&range!(Q9s))
            .extend(&range!(QTs))
            .extend(&range!(QJs));

        let actual = range!(Q3s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q2s_plus() {
        let expected = range!(Q2s)
            .extend(&range!(Q3s))
            .extend(&range!(Q4s))
            .extend(&range!(Q5s))
            .extend(&range!(Q6s))
            .extend(&range!(Q7s))
            .extend(&range!(Q8s))
            .extend(&range!(Q9s))
            .extend(&range!(QTs))
            .extend(&range!(QJs));

        let actual = range!(Q2s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn qto_plus() {
        let expected = range!(QTo).extend(&range!(QJo));

        let actual = range!(QTo+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q9o_plus() {
        let expected = range!(Q9o).extend(&range!(QTo)).extend(&range!(QJo));

        let actual = range!(Q9o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q8o_plus() {
        let expected = range!(Q8o)
            .extend(&range!(Q9o))
            .extend(&range!(QTo))
            .extend(&range!(QJo));

        let actual = range!(Q8o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q7o_plus() {
        let expected = range!(Q7o)
            .extend(&range!(Q8o))
            .extend(&range!(Q9o))
            .extend(&range!(QTo))
            .extend(&range!(QJo));

        let actual = range!(Q7o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q6o_plus() {
        let expected = range!(Q6o)
            .extend(&range!(Q7o))
            .extend(&range!(Q8o))
            .extend(&range!(Q9o))
            .extend(&range!(QTo))
            .extend(&range!(QJo));

        let actual = range!(Q6o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q5o_plus() {
        let expected = range!(Q5o)
            .extend(&range!(Q6o))
            .extend(&range!(Q7o))
            .extend(&range!(Q8o))
            .extend(&range!(Q9o))
            .extend(&range!(QTo))
            .extend(&range!(QJo));

        let actual = range!(Q5o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q4o_plus() {
        let expected = range!(Q4o)
            .extend(&range!(Q5o))
            .extend(&range!(Q6o))
            .extend(&range!(Q7o))
            .extend(&range!(Q8o))
            .extend(&range!(Q9o))
            .extend(&range!(QTo))
            .extend(&range!(QJo));

        let actual = range!(Q4o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q3o_plus() {
        let expected = range!(Q3o)
            .extend(&range!(Q4o))
            .extend(&range!(Q5o))
            .extend(&range!(Q6o))
            .extend(&range!(Q7o))
            .extend(&range!(Q8o))
            .extend(&range!(Q9o))
            .extend(&range!(QTo))
            .extend(&range!(QJo));

        let actual = range!(Q3o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn q2o_plus() {
        let expected = range!(Q2o)
            .extend(&range!(Q3o))
            .extend(&range!(Q4o))
            .extend(&range!(Q5o))
            .extend(&range!(Q6o))
            .extend(&range!(Q7o))
            .extend(&range!(Q8o))
            .extend(&range!(Q9o))
            .extend(&range!(QTo))
            .extend(&range!(QJo));

        let actual = range!(Q2o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j9_plus() {
        let expected = range!(J9).extend(&range!(JT));

        let actual = range!(J9+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j8_plus() {
        let expected = range!(J8).extend(&range!(J9)).extend(&range!(JT));

        let actual = range!(J8+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j7_plus() {
        let expected = range!(J7).extend(&range!(J8)).extend(&range!(J9)).extend(&range!(JT));

        let actual = range!(J7+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j6_plus() {
        let expected = range!(J6)
            .extend(&range!(J7))
            .extend(&range!(J8))
            .extend(&range!(J9))
            .extend(&range!(JT));

        let actual = range!(J6+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j5_plus() {
        let expected = range!(J5)
            .extend(&range!(J6))
            .extend(&range!(J7))
            .extend(&range!(J8))
            .extend(&range!(J9))
            .extend(&range!(JT));

        let actual = range!(J5+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j4_plus() {
        let expected = range!(J4)
            .extend(&range!(J5))
            .extend(&range!(J6))
            .extend(&range!(J7))
            .extend(&range!(J8))
            .extend(&range!(J9))
            .extend(&range!(JT));

        let actual = range!(J4+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j3_plus() {
        let expected = range!(J3)
            .extend(&range!(J4))
            .extend(&range!(J5))
            .extend(&range!(J6))
            .extend(&range!(J7))
            .extend(&range!(J8))
            .extend(&range!(J9))
            .extend(&range!(JT));

        let actual = range!(J3+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn jx() {
        let expected = range!(J2)
            .extend(&range!(J3))
            .extend(&range!(J4))
            .extend(&range!(J5))
            .extend(&range!(J6))
            .extend(&range!(J7))
            .extend(&range!(J8))
            .extend(&range!(J9))
            .extend(&range!(JT));

        let actual = range!(Jx);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j9s_plus() {
        let expected = range!(J9s).extend(&range!(JTs));

        let actual = range!(J9s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j8s_plus() {
        let expected = range!(J8s).extend(&range!(J9s)).extend(&range!(JTs));

        let actual = range!(J8s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j7s_plus() {
        let expected = range!(J7s)
            .extend(&range!(J8s))
            .extend(&range!(J9s))
            .extend(&range!(JTs));

        let actual = range!(J7s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j6s_plus() {
        let expected = range!(J6s)
            .extend(&range!(J7s))
            .extend(&range!(J8s))
            .extend(&range!(J9s))
            .extend(&range!(JTs));

        let actual = range!(J6s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j5s_plus() {
        let expected = range!(J5s)
            .extend(&range!(J6s))
            .extend(&range!(J7s))
            .extend(&range!(J8s))
            .extend(&range!(J9s))
            .extend(&range!(JTs));

        let actual = range!(J5s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j4s_plus() {
        let expected = range!(J4s)
            .extend(&range!(J5s))
            .extend(&range!(J6s))
            .extend(&range!(J7s))
            .extend(&range!(J8s))
            .extend(&range!(J9s))
            .extend(&range!(JTs));

        let actual = range!(J4s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j3s_plus() {
        let expected = range!(J3s)
            .extend(&range!(J4s))
            .extend(&range!(J5s))
            .extend(&range!(J6s))
            .extend(&range!(J7s))
            .extend(&range!(J8s))
            .extend(&range!(J9s))
            .extend(&range!(JTs));

        let actual = range!(J3s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j2s_plus() {
        let expected = range!(J2s)
            .extend(&range!(J3s))
            .extend(&range!(J4s))
            .extend(&range!(J5s))
            .extend(&range!(J6s))
            .extend(&range!(J7s))
            .extend(&range!(J8s))
            .extend(&range!(J9s))
            .extend(&range!(JTs));

        let actual = range!(J2s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j9o_plus() {
        let expected = range!(J9o).extend(&range!(JTo));

        let actual = range!(J9o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j8o_plus() {
        let expected = range!(J8o).extend(&range!(J9o)).extend(&range!(JTo));

        let actual = range!(J8o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j7o_plus() {
        let expected = range!(J7o)
            .extend(&range!(J8o))
            .extend(&range!(J9o))
            .extend(&range!(JTo));

        let actual = range!(J7o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j6o_plus() {
        let expected = range!(J6o)
            .extend(&range!(J7o))
            .extend(&range!(J8o))
            .extend(&range!(J9o))
            .extend(&range!(JTo));

        let actual = range!(J6o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j5o_plus() {
        let expected = range!(J5o)
            .extend(&range!(J6o))
            .extend(&range!(J7o))
            .extend(&range!(J8o))
            .extend(&range!(J9o))
            .extend(&range!(JTo));

        let actual = range!(J5o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j4o_plus() {
        let expected = range!(J4o)
            .extend(&range!(J5o))
            .extend(&range!(J6o))
            .extend(&range!(J7o))
            .extend(&range!(J8o))
            .extend(&range!(J9o))
            .extend(&range!(JTo));

        let actual = range!(J4o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j3o_plus() {
        let expected = range!(J3o)
            .extend(&range!(J4o))
            .extend(&range!(J5o))
            .extend(&range!(J6o))
            .extend(&range!(J7o))
            .extend(&range!(J8o))
            .extend(&range!(J9o))
            .extend(&range!(JTo));

        let actual = range!(J3o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn j2o_plus() {
        let expected = range!(J2o)
            .extend(&range!(J3o))
            .extend(&range!(J4o))
            .extend(&range!(J5o))
            .extend(&range!(J6o))
            .extend(&range!(J7o))
            .extend(&range!(J8o))
            .extend(&range!(J9o))
            .extend(&range!(JTo));

        let actual = range!(J2o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t8_plus() {
        let expected = range!(T8).extend(&range!(T9));

        let actual = range!(T8+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t7_plus() {
        let expected = range!(T7).extend(&range!(T8)).extend(&range!(T9));

        let actual = range!(T7+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t6_plus() {
        let expected = range!(T6).extend(&range!(T7)).extend(&range!(T8)).extend(&range!(T9));

        let actual = range!(T6+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t5_plus() {
        let expected = range!(T5)
            .extend(&range!(T6))
            .extend(&range!(T7))
            .extend(&range!(T8))
            .extend(&range!(T9));

        let actual = range!(T5+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t4_plus() {
        let expected = range!(T4)
            .extend(&range!(T5))
            .extend(&range!(T6))
            .extend(&range!(T7))
            .extend(&range!(T8))
            .extend(&range!(T9));

        let actual = range!(T4+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t3_plus() {
        let expected = range!(T3)
            .extend(&range!(T4))
            .extend(&range!(T5))
            .extend(&range!(T6))
            .extend(&range!(T7))
            .extend(&range!(T8))
            .extend(&range!(T9));

        let actual = range!(T3+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn tx() {
        let expected = range!(T2)
            .extend(&range!(T3))
            .extend(&range!(T4))
            .extend(&range!(T5))
            .extend(&range!(T6))
            .extend(&range!(T7))
            .extend(&range!(T8))
            .extend(&range!(T9));

        let actual = range!(Tx);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t8s_plus() {
        let expected = range!(T8s).extend(&range!(T9s));

        let actual = range!(T8s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t7s_plus() {
        let expected = range!(T7s).extend(&range!(T8s)).extend(&range!(T9s));

        let actual = range!(T7s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t6s_plus() {
        let expected = range!(T6s)
            .extend(&range!(T7s))
            .extend(&range!(T8s))
            .extend(&range!(T9s));

        let actual = range!(T6s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t5s_plus() {
        let expected = range!(T5s)
            .extend(&range!(T6s))
            .extend(&range!(T7s))
            .extend(&range!(T8s))
            .extend(&range!(T9s));

        let actual = range!(T5s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t4s_plus() {
        let expected = range!(T4s)
            .extend(&range!(T5s))
            .extend(&range!(T6s))
            .extend(&range!(T7s))
            .extend(&range!(T8s))
            .extend(&range!(T9s));

        let actual = range!(T4s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t3s_plus() {
        let expected = range!(T3s)
            .extend(&range!(T4s))
            .extend(&range!(T5s))
            .extend(&range!(T6s))
            .extend(&range!(T7s))
            .extend(&range!(T8s))
            .extend(&range!(T9s));

        let actual = range!(T3s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t2s_plus() {
        let expected = range!(T2s)
            .extend(&range!(T3s))
            .extend(&range!(T4s))
            .extend(&range!(T5s))
            .extend(&range!(T6s))
            .extend(&range!(T7s))
            .extend(&range!(T8s))
            .extend(&range!(T9s));

        let actual = range!(T2s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t8o_plus() {
        let expected = range!(T8o).extend(&range!(T9o));

        let actual = range!(T8o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t7o_plus() {
        let expected = range!(T7o).extend(&range!(T8o)).extend(&range!(T9o));

        let actual = range!(T7o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t6o_plus() {
        let expected = range!(T6o)
            .extend(&range!(T7o))
            .extend(&range!(T8o))
            .extend(&range!(T9o));

        let actual = range!(T6o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t5o_plus() {
        let expected = range!(T5o)
            .extend(&range!(T6o))
            .extend(&range!(T7o))
            .extend(&range!(T8o))
            .extend(&range!(T9o));

        let actual = range!(T5o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t4o_plus() {
        let expected = range!(T4o)
            .extend(&range!(T5o))
            .extend(&range!(T6o))
            .extend(&range!(T7o))
            .extend(&range!(T8o))
            .extend(&range!(T9o));

        let actual = range!(T4o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t3o_plus() {
        let expected = range!(T3o)
            .extend(&range!(T4o))
            .extend(&range!(T5o))
            .extend(&range!(T6o))
            .extend(&range!(T7o))
            .extend(&range!(T8o))
            .extend(&range!(T9o));

        let actual = range!(T3o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn t2o_plus() {
        let expected = range!(T2o)
            .extend(&range!(T3o))
            .extend(&range!(T4o))
            .extend(&range!(T5o))
            .extend(&range!(T6o))
            .extend(&range!(T7o))
            .extend(&range!(T8o))
            .extend(&range!(T9o));

        let actual = range!(T2o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine7_plus() {
        let expected = range!(97).extend(&range!(98));

        let actual = range!(97+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine6_plus() {
        let expected = range!(96).extend(&range!(97)).extend(&range!(98));

        let actual = range!(96+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine5_plus() {
        let expected = range!(95).extend(&range!(96)).extend(&range!(97)).extend(&range!(98));

        let actual = range!(95+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine4_plus() {
        let expected = range!(94)
            .extend(&range!(95))
            .extend(&range!(96))
            .extend(&range!(97))
            .extend(&range!(98));

        let actual = range!(94+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine3_plus() {
        let expected = range!(93)
            .extend(&range!(94))
            .extend(&range!(95))
            .extend(&range!(96))
            .extend(&range!(97))
            .extend(&range!(98));

        let actual = range!(93+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn ninex_plus() {
        let expected = range!(92)
            .extend(&range!(93))
            .extend(&range!(94))
            .extend(&range!(95))
            .extend(&range!(96))
            .extend(&range!(97))
            .extend(&range!(98));

        let actual = range!(9x);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine7s_plus() {
        let expected = range!(97s).extend(&range!(98s));

        let actual = range!(97s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine6s_plus() {
        let expected = range!(96s).extend(&range!(97s)).extend(&range!(98s));

        let actual = range!(96s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine5s_plus() {
        let expected = range!(95s)
            .extend(&range!(96s))
            .extend(&range!(97s))
            .extend(&range!(98s));

        let actual = range!(95s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine4s_plus() {
        let expected = range!(94s)
            .extend(&range!(95s))
            .extend(&range!(96s))
            .extend(&range!(97s))
            .extend(&range!(98s));

        let actual = range!(94s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine3s_plus() {
        let expected = range!(93s)
            .extend(&range!(94s))
            .extend(&range!(95s))
            .extend(&range!(96s))
            .extend(&range!(97s))
            .extend(&range!(98s));

        let actual = range!(93s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine2s_plus() {
        let expected = range!(92s)
            .extend(&range!(93s))
            .extend(&range!(94s))
            .extend(&range!(95s))
            .extend(&range!(96s))
            .extend(&range!(97s))
            .extend(&range!(98s));

        let actual = range!(92s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine7o_plus() {
        let expected = range!(97o).extend(&range!(98o));

        let actual = range!(97o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine6o_plus() {
        let expected = range!(96o).extend(&range!(97o)).extend(&range!(98o));

        let actual = range!(96o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine5o_plus() {
        let expected = range!(95o)
            .extend(&range!(96o))
            .extend(&range!(97o))
            .extend(&range!(98o));

        let actual = range!(95o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine4o_plus() {
        let expected = range!(94o)
            .extend(&range!(95o))
            .extend(&range!(96o))
            .extend(&range!(97o))
            .extend(&range!(98o));

        let actual = range!(94o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine3o_plus() {
        let expected = range!(93o)
            .extend(&range!(94o))
            .extend(&range!(95o))
            .extend(&range!(96o))
            .extend(&range!(97o))
            .extend(&range!(98o));

        let actual = range!(93o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn nine2o_plus() {
        let expected = range!(92o)
            .extend(&range!(93o))
            .extend(&range!(94o))
            .extend(&range!(95o))
            .extend(&range!(96o))
            .extend(&range!(97o))
            .extend(&range!(98o));

        let actual = range!(92o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight6_plus() {
        let expected = range!(86).extend(&range!(87));

        let actual = range!(86+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight5_plus() {
        let expected = range!(85).extend(&range!(86)).extend(&range!(87));

        let actual = range!(85+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight4_plus() {
        let expected = range!(84).extend(&range!(85)).extend(&range!(86)).extend(&range!(87));

        let actual = range!(84+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight3_plus() {
        let expected = range!(83)
            .extend(&range!(84))
            .extend(&range!(85))
            .extend(&range!(86))
            .extend(&range!(87));

        let actual = range!(83+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eightx_plus() {
        let expected = range!(82)
            .extend(&range!(83))
            .extend(&range!(84))
            .extend(&range!(85))
            .extend(&range!(86))
            .extend(&range!(87));

        let actual = range!(8x);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight6s_plus() {
        let expected = range!(86s).extend(&range!(87s));

        let actual = range!(86s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight5s_plus() {
        let expected = range!(85s).extend(&range!(86s)).extend(&range!(87s));

        let actual = range!(85s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight4s_plus() {
        let expected = range!(84s)
            .extend(&range!(85s))
            .extend(&range!(86s))
            .extend(&range!(87s));

        let actual = range!(84s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight3s_plus() {
        let expected = range!(83s)
            .extend(&range!(84s))
            .extend(&range!(85s))
            .extend(&range!(86s))
            .extend(&range!(87s));

        let actual = range!(83s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight2s_plus() {
        let expected = range!(82s)
            .extend(&range!(83s))
            .extend(&range!(84s))
            .extend(&range!(85s))
            .extend(&range!(86s))
            .extend(&range!(87s));

        let actual = range!(82s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight6o_plus() {
        let expected = range!(86o).extend(&range!(87o));

        let actual = range!(86o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight5o_plus() {
        let expected = range!(85o).extend(&range!(86o)).extend(&range!(87o));

        let actual = range!(85o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight4o_plus() {
        let expected = range!(84o)
            .extend(&range!(85o))
            .extend(&range!(86o))
            .extend(&range!(87o));

        let actual = range!(84o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight3o_plus() {
        let expected = range!(83o)
            .extend(&range!(84o))
            .extend(&range!(85o))
            .extend(&range!(86o))
            .extend(&range!(87o));

        let actual = range!(83o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn eight2o_plus() {
        let expected = range!(82o)
            .extend(&range!(83o))
            .extend(&range!(84o))
            .extend(&range!(85o))
            .extend(&range!(86o))
            .extend(&range!(87o));

        let actual = range!(82o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn seven5_plus() {
        let expected = range!(75).extend(&range!(76));

        let actual = range!(75+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn seven4_plus() {
        let expected = range!(74).extend(&range!(75)).extend(&range!(76));

        let actual = range!(74+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn seven3_plus() {
        let expected = range!(73).extend(&range!(74)).extend(&range!(75)).extend(&range!(76));

        let actual = range!(73+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn sevenx_plus() {
        let expected = range!(72)
            .extend(&range!(73))
            .extend(&range!(74))
            .extend(&range!(75))
            .extend(&range!(76));

        let actual = range!(7x);

        assert_eq!(expected, actual);
    }

    #[test]
    fn seven5s_plus() {
        let expected = range!(75s).extend(&range!(76s));

        let actual = range!(75s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn seven4s_plus() {
        let expected = range!(74s).extend(&range!(75s)).extend(&range!(76s));

        let actual = range!(74s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn seven3s_plus() {
        let expected = range!(73s)
            .extend(&range!(74s))
            .extend(&range!(75s))
            .extend(&range!(76s));

        let actual = range!(73s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn seven2s_plus() {
        let expected = range!(72s)
            .extend(&range!(73s))
            .extend(&range!(74s))
            .extend(&range!(75s))
            .extend(&range!(76s));

        let actual = range!(72s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn seven5o_plus() {
        let expected = range!(75o).extend(&range!(76o));

        let actual = range!(75o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn seven4o_plus() {
        let expected = range!(74o).extend(&range!(75o)).extend(&range!(76o));

        let actual = range!(74o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn seven3o_plus() {
        let expected = range!(73o)
            .extend(&range!(74o))
            .extend(&range!(75o))
            .extend(&range!(76o));

        let actual = range!(73o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn seven2o_plus() {
        let expected = range!(72o)
            .extend(&range!(73o))
            .extend(&range!(74o))
            .extend(&range!(75o))
            .extend(&range!(76o));

        let actual = range!(72o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn six4_plus() {
        let expected = range!(64).extend(&range!(65));

        let actual = range!(64+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn six3_plus() {
        let expected = range!(63).extend(&range!(64)).extend(&range!(65));

        let actual = range!(63+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn sixx_plus() {
        let expected = range!(62).extend(&range!(63)).extend(&range!(64)).extend(&range!(65));

        let actual = range!(62+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn six4s_plus() {
        let expected = range!(64s).extend(&range!(65s));

        let actual = range!(64s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn six3s_plus() {
        let expected = range!(63s).extend(&range!(64s)).extend(&range!(65s));

        let actual = range!(63s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn six2s_plus() {
        let expected = range!(62s)
            .extend(&range!(63s))
            .extend(&range!(64s))
            .extend(&range!(65s));

        let actual = range!(62s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn six4o_plus() {
        let expected = range!(64o).extend(&range!(65o));

        let actual = range!(64o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn six3o_plus() {
        let expected = range!(63o).extend(&range!(64o)).extend(&range!(65o));

        let actual = range!(63o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn six2o_plus() {
        let expected = range!(62o)
            .extend(&range!(63o))
            .extend(&range!(64o))
            .extend(&range!(65o));

        let actual = range!(62o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn five3_plus() {
        let expected = range!(53).extend(&range!(54));

        let actual = range!(53+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn fivex_plus() {
        let expected = range!(52).extend(&range!(53)).extend(&range!(54));

        let actual = range!(52+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn five3s_plus() {
        let expected = range!(53s).extend(&range!(54s));

        let actual = range!(53s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn five2s_plus() {
        let expected = range!(52s).extend(&range!(53s)).extend(&range!(54s));

        let actual = range!(52s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn five3o_plus() {
        let expected = range!(53o).extend(&range!(54o));

        let actual = range!(53o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn five2o_plus() {
        let expected = range!(52o).extend(&range!(53o)).extend(&range!(54o));

        let actual = range!(52o+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn fourx_plus() {
        let expected = range!(42).extend(&range!(43));

        let actual = range!(42+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn fourxs_plus() {
        let expected = range!(42s).extend(&range!(43s));

        let actual = range!(42s+);

        assert_eq!(expected, actual);
    }

    #[test]
    fn fourxo_plus() {
        let expected = range!(42o).extend(&range!(43o));

        let actual = range!(42o+);

        assert_eq!(expected, actual);
    }
}
