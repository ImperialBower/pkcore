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

    (AKs) => {
        Twos::from(Two::ACE_KING_SUITED.to_vec())
    };
    (AKo) => {
        Twos::from(Two::ACE_KING_OFFSUIT.to_vec())
    };
    (AK) => {
        Twos::from(Two::ACE_KING.to_vec())
    };
    (AQs) => {
        Twos::from(Two::ACE_QUEEN_SUITED.to_vec())
    };
    (AQo) => {
        Twos::from(Two::ACE_QUEEN_OFFSUIT.to_vec())
    };
    (AQ) => {
        Twos::from(Two::ACE_QUEEN.to_vec())
    };
    (AJs) => {
        Twos::from(Two::ACE_JACK_SUITED.to_vec())
    };
    (AJo) => {
        Twos::from(Two::ACE_JACK_OFFSUIT.to_vec())
    };
    (AJ) => {
        Twos::from(Two::ACE_JACK.to_vec())
    };
    (ATs) => {
        Twos::from(Two::ACE_TEN_SUITED.to_vec())
    };
    (ATo) => {
        Twos::from(Two::ACE_TEN_OFFSUIT.to_vec())
    };
    (AT) => {
        Twos::from(Two::ACE_TEN.to_vec())
    };
    (A9s) => {
        Twos::from(Two::ACE_NINE_SUITED.to_vec())
    };
    (A9o) => {
        Twos::from(Two::ACE_NINE_OFFSUIT.to_vec())
    };
    (A9) => {
        Twos::from(Two::ACE_NINE.to_vec())
    };
    (A8s) => {
        Twos::from(Two::ACE_EIGHT_SUITED.to_vec())
    };
    (A8o) => {
        Twos::from(Two::ACE_EIGHT_OFFSUIT.to_vec())
    };
    (A8) => {
        Twos::from(Two::ACE_EIGHT.to_vec())
    };
    (A7s) => {
        Twos::from(Two::ACE_SEVEN_SUITED.to_vec())
    };
    (A7o) => {
        Twos::from(Two::ACE_SEVEN_OFFSUIT.to_vec())
    };
    (A7) => {
        Twos::from(Two::ACE_SEVEN.to_vec())
    };
    (A6s) => {
        Twos::from(Two::ACE_SIX_SUITED.to_vec())
    };
    (A6o) => {
        Twos::from(Two::ACE_SIX_OFFSUIT.to_vec())
    };
    (A6) => {
        Twos::from(Two::ACE_SIX.to_vec())
    };
    (A5s) => {
        Twos::from(Two::ACE_FIVE_SUITED.to_vec())
    };
    (A5o) => {
        Twos::from(Two::ACE_FIVE_OFFSUIT.to_vec())
    };
    (A5) => {
        Twos::from(Two::ACE_FIVE.to_vec())
    };
    (A4s) => {
        Twos::from(Two::ACE_FOUR_SUITED.to_vec())
    };
    (A4o) => {
        Twos::from(Two::ACE_FOUR_OFFSUIT.to_vec())
    };
    (A4) => {
        Twos::from(Two::ACE_FOUR.to_vec())
    };
    (A3s) => {
        Twos::from(Two::ACE_TREY_SUITED.to_vec())
    };
    (A3o) => {
        Twos::from(Two::ACE_TREY_OFFSUIT.to_vec())
    };
    (A3) => {
        Twos::from(Two::ACE_TREY.to_vec())
    };
    (A2s) => {
        Twos::from(Two::ACE_DEUCE_SUITED.to_vec())
    };
    (A2o) => {
        Twos::from(Two::ACE_DEUCE_OFFSUIT.to_vec())
    };
    (A2) => {
        Twos::from(Two::ACE_DEUCE.to_vec())
    };
    (KQs) => {
        Twos::from(Two::KING_QUEEN_SUITED.to_vec())
    };
    (KQo) => {
        Twos::from(Two::KING_QUEEN_OFFSUIT.to_vec())
    };
    (KQ) => {
        Twos::from(Two::KING_QUEEN.to_vec())
    };
    (QJs) => {
        Twos::from(Two::QUEEN_JACK_SUITED.to_vec())
    };
    (QJo) => {
        Twos::from(Two::QUEEN_JACK_OFFSUIT.to_vec())
    };
    (QJ) => {
        Twos::from(Two::QUEEN_JACK.to_vec())
    };
    (JTs) => {
        Twos::from(Two::JACK_TEN_SUITED.to_vec())
    };
    (JTo) => {
        Twos::from(Two::JACK_TEN_OFFSUIT.to_vec())
    };
    (JT) => {
        Twos::from(Two::JACK_TEN.to_vec())
    };
    (T9s) => {
        Twos::from(Two::TEN_NINE_SUITED.to_vec())
    };
    (T9o) => {
        Twos::from(Two::TEN_NINE_OFFSUIT.to_vec())
    };
    (T9) => {
        Twos::from(Two::TEN_NINE.to_vec())
    };
    (98s) => {
        Twos::from(Two::NINE_EIGHT_SUITED.to_vec())
    };
    (98o) => {
        Twos::from(Two::NINE_EIGHT_OFFSUIT.to_vec())
    };
    (98) => {
        Twos::from(Two::NINE_EIGHT.to_vec())
    };
    (87s) => {
        Twos::from(Two::EIGHT_SEVEN_SUITED.to_vec())
    };
    (87o) => {
        Twos::from(Two::EIGHT_SEVEN_OFFSUIT.to_vec())
    };
    (87) => {
        Twos::from(Two::EIGHT_SEVEN.to_vec())
    };
    (76s) => {
        Twos::from(Two::SEVEN_SIX_SUITED.to_vec())
    };
    (76o) => {
        Twos::from(Two::SEVEN_SIX_OFFSUIT.to_vec())
    };
    (76) => {
        Twos::from(Two::SEVEN_SIX.to_vec())
    };
    (65s) => {
        todo!()
    };
    (65o) => {
        todo!()
    };
    (65) => {
        todo!()
    };
    (54s) => {
        todo!()
    };
    (54o) => {
        todo!()
    };
    (54) => {
        todo!()
    };
    (43s) => {
        todo!()
    };
    (43o) => {
        todo!()
    };
    (43) => {
        todo!()
    };
    (32s) => {
        todo!()
    };
    (32o) => {
        todo!()
    };
    (32) => {
        todo!()
    };
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
