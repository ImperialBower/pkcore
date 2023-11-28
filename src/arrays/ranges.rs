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
        Two::AKs.to_vec()
    };
    (AKo) => {
        Two::AKo.to_vec()
    };
    (AQs) => {
        Two::AQs.to_vec()
    };
    (AQo) => {
        Two::AQo.to_vec()
    };
    (AJs) => {
        Two::AJs.to_vec()
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
