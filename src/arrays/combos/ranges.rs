///
#[macro_export]
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


}

#[cfg(test)]
mod tests {
    use crate::arrays::combos::twos::Twos;
    use crate::rank::Rank;

    #[test]
    fn poker_pairs() {
        assert_eq!(
            range!(AA).hashset(),
            Twos::unique().filter_on_rank(Rank::ACE).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(KK).hashset(),
            Twos::unique().filter_on_rank(Rank::KING).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(QQ).hashset(),
            Twos::unique().filter_on_rank(Rank::QUEEN).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(JJ).hashset(),
            Twos::unique().filter_on_rank(Rank::JACK).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(TT).hashset(),
            Twos::unique().filter_on_rank(Rank::TEN).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(99).hashset(),
            Twos::unique().filter_on_rank(Rank::NINE).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(88).hashset(),
            Twos::unique().filter_on_rank(Rank::EIGHT).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(77).hashset(),
            Twos::unique().filter_on_rank(Rank::SEVEN).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(66).hashset(),
            Twos::unique().filter_on_rank(Rank::SIX).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(55).hashset(),
            Twos::unique().filter_on_rank(Rank::FIVE).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(44).hashset(),
            Twos::unique().filter_on_rank(Rank::FOUR).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(33).hashset(),
            Twos::unique().filter_on_rank(Rank::TREY).filter_is_paired().hashset()
        );
        assert_eq!(
            range!(22).hashset(),
            Twos::unique().filter_on_rank(Rank::DEUCE).filter_is_paired().hashset()
        );
    }
}
