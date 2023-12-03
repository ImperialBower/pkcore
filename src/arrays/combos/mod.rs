use crate::arrays::two::Two;

pub mod ranges;
pub mod twos;

pub const AA: [Two; 6] = [
    Two::HAND_AS_AH,
    Two::HAND_AS_AD,
    Two::HAND_AS_AC,
    Two::HAND_AH_AD,
    Two::HAND_AH_AC,
    Two::HAND_AD_AC,
];
pub const KK: [Two; 6] = [
    Two::HAND_KS_KH,
    Two::HAND_KS_KD,
    Two::HAND_KS_KC,
    Two::HAND_KH_KD,
    Two::HAND_KH_KC,
    Two::HAND_KD_KC,
];
pub const QQ: [Two; 6] = [
    Two::HAND_QS_QH,
    Two::HAND_QS_QD,
    Two::HAND_QS_QC,
    Two::HAND_QH_QD,
    Two::HAND_QH_QC,
    Two::HAND_QD_QC,
];
pub const JJ: [Two; 6] = [
    Two::HAND_JS_JH,
    Two::HAND_JS_JD,
    Two::HAND_JS_JC,
    Two::HAND_JH_JD,
    Two::HAND_JH_JC,
    Two::HAND_JD_JC,
];
pub const TENS: [Two; 6] = [
    Two::HAND_TS_TH,
    Two::HAND_TS_TD,
    Two::HAND_TS_TC,
    Two::HAND_TH_TD,
    Two::HAND_TH_TC,
    Two::HAND_TD_TC,
];
pub const NINES: [Two; 6] = [
    Two::HAND_9S_9H,
    Two::HAND_9S_9D,
    Two::HAND_9S_9C,
    Two::HAND_9H_9D,
    Two::HAND_9H_9C,
    Two::HAND_9D_9C,
];
pub const EIGHTS: [Two; 6] = [
    Two::HAND_8S_8H,
    Two::HAND_8S_8D,
    Two::HAND_8S_8C,
    Two::HAND_8H_8D,
    Two::HAND_8H_8C,
    Two::HAND_8D_8C,
];
pub const SEVENS: [Two; 6] = [
    Two::HAND_7S_7H,
    Two::HAND_7S_7D,
    Two::HAND_7S_7C,
    Two::HAND_7H_7D,
    Two::HAND_7H_7C,
    Two::HAND_7D_7C,
];
pub const SIXES: [Two; 6] = [
    Two::HAND_6S_6H,
    Two::HAND_6S_6D,
    Two::HAND_6S_6C,
    Two::HAND_6H_6D,
    Two::HAND_6H_6C,
    Two::HAND_6D_6C,
];
pub const FIVES: [Two; 6] = [
    Two::HAND_5S_5H,
    Two::HAND_5S_5D,
    Two::HAND_5S_5C,
    Two::HAND_5H_5D,
    Two::HAND_5H_5C,
    Two::HAND_5D_5C,
];
pub const FOURS: [Two; 6] = [
    Two::HAND_4S_4H,
    Two::HAND_4S_4D,
    Two::HAND_4S_4C,
    Two::HAND_4H_4D,
    Two::HAND_4H_4C,
    Two::HAND_4D_4C,
];
pub const TREYS: [Two; 6] = [
    Two::HAND_3S_3H,
    Two::HAND_3S_3D,
    Two::HAND_3S_3C,
    Two::HAND_3H_3D,
    Two::HAND_3H_3C,
    Two::HAND_3D_3C,
];
pub const DEUCES: [Two; 6] = [
    Two::HAND_2S_2H,
    Two::HAND_2S_2D,
    Two::HAND_2S_2C,
    Two::HAND_2H_2D,
    Two::HAND_2H_2C,
    Two::HAND_2D_2C,
];
