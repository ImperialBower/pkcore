#[macro_export]
macro_rules! range {
    (AA) => {
        Two::AA.to_vec()
    };
    (KK) => {
        Two::KK.to_vec()
    };
    (QQ) => {
        Two::QQ.to_vec()
    };
    (JJ) => {
        Two::JJ.to_vec()
    };
    (TT) => {
        Two::TENS.to_vec()
    };
    (99) => {
        Two::NINES.to_vec()
    };
    (88) => {
        Two::EIGHTS.to_vec()
    };
    (77) => {
        Two::SEVENS.to_vec()
    };
    (66) => {
        Two::SIXES.to_vec()
    };
    (55) => {
        Two::FIVES.to_vec()
    };
    (44) => {
        Two::FOURS.to_vec()
    };
    (33) => {
        Two::TREYS.to_vec()
    };
    (22) => {
        Two::DEUCES.to_vec()
    };

    (AKs) => {
        Two::ACE_KING_SUITED.to_vec()
    };
    (AKo) => {
        Two::ACE_KING_OFFSUIT.to_vec()
    };
    (AK) => {
        Two::ACE_KING.to_vec()
    };
    (AQs) => {
        Two::ACE_QUEEN_SUITED.to_vec()
    };
    (AQo) => {
        Two::ACE_QUEEN_OFFSUIT.to_vec()
    };
    (AQ) => {
        Two::ACE_QUEEN.to_vec()
    };
    (AJs) => {
        Two::ACE_JACK_SUITED.to_vec()
    };
    (AJo) => {
        Two::ACE_JACK_OFFSUIT.to_vec()
    };
    (AJ) => {
        Two::ACE_JACK.to_vec()
    };
    (ATs) => {
        Two::ACE_TEN_SUITED.to_vec()
    };
    (ATo) => {
        Two::ACE_TEN_OFFSUIT.to_vec()
    };
    (AT) => {
        Two::ACE_TEN.to_vec()
    };
    (A4s) => {
        Two::ACE_FOUR_SUITED.to_vec()
    };
    (A4o) => {
        Two::ACE_FOUR_OFFSUIT.to_vec()
    };
    (A4) => {
        Two::ACE_FOUR.to_vec()
    };
    (KQs) => {
        Two::KING_QUEEN_SUITED.to_vec()
    };
    (KQo) => {
        Two::KING_QUEEN_OFFSUIT.to_vec()
    };
    (KQ) => {
        Two::KING_QUEEN.to_vec()
    };
    (QJs) => {
        Two::QUEEN_JACK_SUITED.to_vec()
    };
    (QJo) => {
        Two::QUEEN_JACK_OFFSUIT.to_vec()
    };
    (QJ) => {
        Two::QUEEN_JACK.to_vec()
    };
    (JTs) => {
        Two::JACK_TEN_SUITED.to_vec()
    };
    (JTo) => {
        Two::JACK_TEN_OFFSUIT.to_vec()
    };
    (JT) => {
        Two::JACK_TEN.to_vec()
    };
    (T9s) => {
        Two::TEN_NINE_SUITED.to_vec()
    };
    (T9o) => {
        Two::TEN_NINE_OFFSUIT.to_vec()
    };
    (T9) => {
        Two::TEN_NINE.to_vec()
    };
    (98s) => {
        Two::NINE_EIGHT_SUITED.to_vec()
    };
    (98o) => {
        Two::NINE_EIGHT_OFFSUIT.to_vec()
    };
    (98) => {
        Two::NINE_EIGHT.to_vec()
    };
    (87s) => {
        Two::EIGHT_SEVEN_SUITED.to_vec()
    };
    (87o) => {
        Two::EIGHT_SEVEN_OFFSUIT.to_vec()
    };
    (87) => {
        Two::EIGHT_SEVEN.to_vec()
    };
    (76s) => {
        Two::SEVEN_SIX_SUITED.to_vec()
    };
    (76o) => {
        Two::SEVEN_SIX_OFFSUIT.to_vec()
    };
    (76) => {
        Two::SEVEN_SIX.to_vec()
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
    use crate::arrays::two::Two;

    #[test]
    fn poker_pairs() {
        assert_eq!(range!(AA), Two::AA.to_vec());
        assert_eq!(range!(KK), Two::KK.to_vec());
        assert_eq!(range!(QQ), Two::QQ.to_vec());
        assert_eq!(range!(JJ), Two::JJ.to_vec());
        assert_eq!(range!(TT), Two::TENS.to_vec());
        assert_eq!(range!(99), Two::NINES.to_vec());
        assert_eq!(range!(88), Two::EIGHTS.to_vec());
        assert_eq!(range!(77), Two::SEVENS.to_vec());
        assert_eq!(range!(66), Two::SIXES.to_vec());
        assert_eq!(range!(55), Two::FIVES.to_vec());
        assert_eq!(range!(44), Two::FOURS.to_vec());
        assert_eq!(range!(33), Two::TREYS.to_vec());
        assert_eq!(range!(22), Two::DEUCES.to_vec());
    }
}
