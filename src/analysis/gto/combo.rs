use crate::PKError;
use crate::arrays::two::Two;
use crate::rank::Rank;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// The `ranges` module is an attempt to create a progromatic representation of poker ranges.
///
/// - [Poker Ranges & Range Reading](https://www.splitsuit.com/poker-ranges-reading)
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Qualifier {
    OFFSUIT,
    SUITED,
    #[default]
    ALL,
}

impl Display for Qualifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Qualifier::ALL => write!(f, ""),
            Qualifier::SUITED => write!(f, "s"),
            Qualifier::OFFSUIT => write!(f, "o"),
        }
    }
}

/// DIARY: I'm trying to decide if my need to create a completely structured struct :-P that
/// represents a poker combo. This seems to be a pattern of mine. An attempt to take absolute
/// control of something in my life in an otherwise chaotic mess.
///
/// The truth is that I hate the idea of a computational state being governed by a raw string.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Combo {
    pub first: Rank,
    pub second: Rank,
    pub plus: bool,
    pub qualifier: Qualifier,
}

impl Combo {
    // region pocket pairs
    pub const COMBO_AA: Combo = Combo {
        first: Rank::ACE,
        second: Rank::ACE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_KK: Combo = Combo {
        first: Rank::KING,
        second: Rank::KING,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_KK_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::KING,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_QQ: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::QUEEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_QQ_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::QUEEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_JJ: Combo = Combo {
        first: Rank::JACK,
        second: Rank::JACK,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_JJ_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::JACK,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_TT: Combo = Combo {
        first: Rank::TEN,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_TT_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_99: Combo = Combo {
        first: Rank::NINE,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_99_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_88: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_88_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_77: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_77_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_66: Combo = Combo {
        first: Rank::SIX,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_66_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_55: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_55_PLUS: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_44: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_44_PLUS: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_33: Combo = Combo {
        first: Rank::TREY,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_33_PLUS: Combo = Combo {
        first: Rank::TREY,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_22: Combo = Combo {
        first: Rank::DEUCE,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_22_PLUS: Combo = Combo {
        first: Rank::DEUCE,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    // endregion

    // region connectors
    pub const COMBO_AKs: Combo = Combo {
        first: Rank::ACE,
        second: Rank::KING,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_AKo: Combo = Combo {
        first: Rank::ACE,
        second: Rank::KING,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_AK: Combo = Combo {
        first: Rank::ACE,
        second: Rank::KING,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_AQs: Combo = Combo {
        first: Rank::ACE,
        second: Rank::QUEEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_AQo: Combo = Combo {
        first: Rank::ACE,
        second: Rank::QUEEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_AQ: Combo = Combo {
        first: Rank::ACE,
        second: Rank::QUEEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_AQs_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::QUEEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_AQo_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::QUEEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_AQ_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::QUEEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_AJs: Combo = Combo {
        first: Rank::ACE,
        second: Rank::JACK,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_AJo: Combo = Combo {
        first: Rank::ACE,
        second: Rank::JACK,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_AJ: Combo = Combo {
        first: Rank::ACE,
        second: Rank::JACK,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_AJs_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::JACK,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_AJo_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::JACK,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_AJ_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::JACK,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_ATs: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_ATo: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_AT: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_ATs_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_ATo_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_AT_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A9s: Combo = Combo {
        first: Rank::ACE,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A9o: Combo = Combo {
        first: Rank::ACE,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A9: Combo = Combo {
        first: Rank::ACE,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A9s_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A9o_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A9_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A8s: Combo = Combo {
        first: Rank::ACE,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A8o: Combo = Combo {
        first: Rank::ACE,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A8: Combo = Combo {
        first: Rank::ACE,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A8s_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A8o_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A8_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A7s: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A7o: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A7: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A7s_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A7o_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A7_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A6s: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A6o: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A6: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A6s_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A6o_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A6_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A5s: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A5o: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A5: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A5s_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A5o_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A5_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A4s: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A4o: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A4: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A4s_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A4o_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A4_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A3s: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A3o: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A3: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A3s_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A3o_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A3_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A2s: Combo = Combo {
        first: Rank::ACE,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A2o: Combo = Combo {
        first: Rank::ACE,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A2: Combo = Combo {
        first: Rank::ACE,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_A2s_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_A2o_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_A2_PLUS: Combo = Combo {
        first: Rank::ACE,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_KQs: Combo = Combo {
        first: Rank::KING,
        second: Rank::QUEEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_KQo: Combo = Combo {
        first: Rank::KING,
        second: Rank::QUEEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_KQ: Combo = Combo {
        first: Rank::KING,
        second: Rank::QUEEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_KQs_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::QUEEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_KQo_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::QUEEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_KQ_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::QUEEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_KJs: Combo = Combo {
        first: Rank::KING,
        second: Rank::JACK,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_KJo: Combo = Combo {
        first: Rank::KING,
        second: Rank::JACK,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_KJ: Combo = Combo {
        first: Rank::KING,
        second: Rank::JACK,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_KJs_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::JACK,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_KJo_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::JACK,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_KJ_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::JACK,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_KTs: Combo = Combo {
        first: Rank::KING,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_KTo: Combo = Combo {
        first: Rank::KING,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_KT: Combo = Combo {
        first: Rank::KING,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_KTs_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_KTo_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_KT_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K9s: Combo = Combo {
        first: Rank::KING,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K9o: Combo = Combo {
        first: Rank::KING,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K9: Combo = Combo {
        first: Rank::KING,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K9s_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K9o_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K9_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K8s: Combo = Combo {
        first: Rank::KING,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K8o: Combo = Combo {
        first: Rank::KING,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K8: Combo = Combo {
        first: Rank::KING,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K8s_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K8o_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K8_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K7s: Combo = Combo {
        first: Rank::KING,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K7o: Combo = Combo {
        first: Rank::KING,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K7: Combo = Combo {
        first: Rank::KING,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K7s_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K7o_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K7_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K6s: Combo = Combo {
        first: Rank::KING,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K6o: Combo = Combo {
        first: Rank::KING,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K6: Combo = Combo {
        first: Rank::KING,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K6s_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K6o_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K6_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K5s: Combo = Combo {
        first: Rank::KING,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K5o: Combo = Combo {
        first: Rank::KING,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K5: Combo = Combo {
        first: Rank::KING,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K5s_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K5o_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K5_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K4s: Combo = Combo {
        first: Rank::KING,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K4o: Combo = Combo {
        first: Rank::KING,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K4: Combo = Combo {
        first: Rank::KING,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K4s_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K4o_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K4_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K3s: Combo = Combo {
        first: Rank::KING,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K3o: Combo = Combo {
        first: Rank::KING,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K3: Combo = Combo {
        first: Rank::KING,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K3s_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K3o_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K3_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K2s: Combo = Combo {
        first: Rank::KING,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K2o: Combo = Combo {
        first: Rank::KING,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K2: Combo = Combo {
        first: Rank::KING,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_K2s_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_K2o_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_K2_PLUS: Combo = Combo {
        first: Rank::KING,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_QJs: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::JACK,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_QJo: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::JACK,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_QJ: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::JACK,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_QJs_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::JACK,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_QJo_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::JACK,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_QJ_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::JACK,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_QTs: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_QTo: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_QT: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_QTs_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_QTo_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_QT_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q9s: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q9o: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q9: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q9s_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q9o_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q9_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q8s: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q8o: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q8: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q8s_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q8o_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q8_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::ALL,
    };

    pub const COMBO_Q7s: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q7o: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q7: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q7s_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q7o_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q7_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q6s: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q6o: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q6: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q6s_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q6o_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q6_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q5s: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q5o: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q5: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q5s_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q5o_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q5_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q4s: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q4o: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q4: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q4s_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q4o_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q4_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q3s: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q3o: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q3: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q3s_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q3o_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q3_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q2s: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q2o: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q2: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_Q2s_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_Q2o_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_Q2_PLUS: Combo = Combo {
        first: Rank::QUEEN,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_JTs: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_JTo: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_JT: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_JTs_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_JTo_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_JT_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J9s: Combo = Combo {
        first: Rank::JACK,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J9o: Combo = Combo {
        first: Rank::JACK,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J9: Combo = Combo {
        first: Rank::JACK,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J9s_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J9o_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J9_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J8s: Combo = Combo {
        first: Rank::JACK,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J8o: Combo = Combo {
        first: Rank::JACK,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J8: Combo = Combo {
        first: Rank::JACK,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J8s_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J8o_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J8_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J7s: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J7o: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J7: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J7s_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J7o_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J7_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J6s: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J6o: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J6: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J6s_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J6o_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J6_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J5s: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J5o: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J5: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J5s_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J5o_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J5_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J4s: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J4o: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J4: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J4s_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J4o_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J4_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J3s: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J3o: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J3: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J3s_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J3o_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J3_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J2s: Combo = Combo {
        first: Rank::JACK,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J2o: Combo = Combo {
        first: Rank::JACK,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J2: Combo = Combo {
        first: Rank::JACK,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_J2s_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_J2o_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_J2_PLUS: Combo = Combo {
        first: Rank::JACK,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T9s: Combo = Combo {
        first: Rank::TEN,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T9o: Combo = Combo {
        first: Rank::TEN,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T9: Combo = Combo {
        first: Rank::TEN,
        second: Rank::NINE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T9s_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T9o_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T9_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::NINE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T8s: Combo = Combo {
        first: Rank::TEN,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T8o: Combo = Combo {
        first: Rank::TEN,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T8: Combo = Combo {
        first: Rank::TEN,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T8s_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T8o_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T8_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T7s: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T7o: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T7: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T7s_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T7o_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T7_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T6s: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T6o: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T6: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T6s_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T6o_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T6_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T5s: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T5o: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T5: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T5s_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T5o_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T5_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T4s: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T4o: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T4: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T4s_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T4o_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T4_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T3s: Combo = Combo {
        first: Rank::TEN,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T3o: Combo = Combo {
        first: Rank::TEN,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T3: Combo = Combo {
        first: Rank::TEN,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T3s_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T3o_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T3_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T2s: Combo = Combo {
        first: Rank::TEN,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T2o: Combo = Combo {
        first: Rank::TEN,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T2: Combo = Combo {
        first: Rank::TEN,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_T2s_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_T2o_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_T2_PLUS: Combo = Combo {
        first: Rank::TEN,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_98s: Combo = Combo {
        first: Rank::NINE,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_98o: Combo = Combo {
        first: Rank::NINE,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_98: Combo = Combo {
        first: Rank::NINE,
        second: Rank::EIGHT,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_98s_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_98o_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_98_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::EIGHT,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_97s: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_97o: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_97: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_97s_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_97o_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_97_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_96s: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_96o: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_96: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_96s_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_96o_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_96_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_95s: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_95o: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_95: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_95s_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_95o_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_95_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_94s: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_94o: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_94: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_94s_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_94o_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_94_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_93s: Combo = Combo {
        first: Rank::NINE,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_93o: Combo = Combo {
        first: Rank::NINE,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_93: Combo = Combo {
        first: Rank::NINE,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_93s_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_93o_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_93_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_92s: Combo = Combo {
        first: Rank::NINE,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_92o: Combo = Combo {
        first: Rank::NINE,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_92: Combo = Combo {
        first: Rank::NINE,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_92s_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_92o_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_92_PLUS: Combo = Combo {
        first: Rank::NINE,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_87s: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_87o: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_87: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SEVEN,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_87s_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_87o_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_87_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SEVEN,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_86s: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_86o: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_86: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_86s_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_86o_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_86_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_85s: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_85o: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_85: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_85s_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_85o_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_85_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_84s: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_84o: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_84: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_84s_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_84o_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_84_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_83s: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_83o: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_83: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_83s_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_83o_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_83_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_82s: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_82o: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_82: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_82s_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_82o_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_82_PLUS: Combo = Combo {
        first: Rank::EIGHT,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_76s: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_76o: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_76: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::SIX,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_76s_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_76o_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_76_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::SIX,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_75s: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_75o: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_75: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_75s_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_75o_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_75_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_74s: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_74o: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_74: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_74s_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_74o_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_74_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_73s: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_73o: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_73: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_73s_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_73o_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_73_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_72s: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_72o: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_72: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_72s_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_72o_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_72_PLUS: Combo = Combo {
        first: Rank::SEVEN,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_65s: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_65o: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_65: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FIVE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_65s_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_65o_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_65_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FIVE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_64s: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_64o: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_64: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_64s_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_64o_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_64_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_63s: Combo = Combo {
        first: Rank::SIX,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_63o: Combo = Combo {
        first: Rank::SIX,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_63: Combo = Combo {
        first: Rank::SIX,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_63s_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_63o_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_63_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_62s: Combo = Combo {
        first: Rank::SIX,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_62o: Combo = Combo {
        first: Rank::SIX,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_62: Combo = Combo {
        first: Rank::SIX,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_62s_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_62o_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_62_PLUS: Combo = Combo {
        first: Rank::SIX,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_54s: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_54o: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_54: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::FOUR,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_54s_PLUS: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_54o_PLUS: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_54_PLUS: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::FOUR,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_53s: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_53o: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_53: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_53s_PLUS: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_53o_PLUS: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_53_PLUS: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_52s: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_52o: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_52: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_52s_PLUS: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_52o_PLUS: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_52_PLUS: Combo = Combo {
        first: Rank::FIVE,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_43s: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_43o: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_43: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::TREY,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_43s_PLUS: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_43o_PLUS: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_43_PLUS: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::TREY,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_42s: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_42o: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_42: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_42s_PLUS: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_42o_PLUS: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_42_PLUS: Combo = Combo {
        first: Rank::FOUR,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_32s: Combo = Combo {
        first: Rank::TREY,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_32o: Combo = Combo {
        first: Rank::TREY,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_32: Combo = Combo {
        first: Rank::TREY,
        second: Rank::DEUCE,
        plus: false,
        qualifier: Qualifier::ALL,
    };
    pub const COMBO_32s_PLUS: Combo = Combo {
        first: Rank::TREY,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::SUITED,
    };
    pub const COMBO_32o_PLUS: Combo = Combo {
        first: Rank::TREY,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::OFFSUIT,
    };
    pub const COMBO_32_PLUS: Combo = Combo {
        first: Rank::TREY,
        second: Rank::DEUCE,
        plus: true,
        qualifier: Qualifier::ALL,
    };
    // endregion

    #[must_use]
    pub fn distance_first(&self, other: &Self) -> i8 {
        let first_diff = self.first as i8 - other.first as i8;
        first_diff.abs()
    }

    #[must_use]
    pub fn distance_second(&self, other: &Self) -> i8 {
        let second_diff = self.second as i8 - other.second as i8;
        second_diff.abs()
    }

    #[must_use]
    pub fn is_aligned_with(&self, other: &Self) -> bool {
        if self.plus || other.plus {
            return false; // Combo plus can't be aligned since they rep multiple gto
        }
        if self == other {
            return false; // Same combo
        }
        if self.is_pair() {
            return other.is_pair();
        }

        if other.is_ace_x() {
            if self.is_ace_x() {
                return true; // Ace-x vs Ace-x
            }
            if self.is_connector() {
                return true; // Ace-x vs connector
            }

            if self.is_ace_x_suited() || self.is_ace_x_offsuit() {
                return true; // Ace-x suited/offsuit vs Ace-x
            }
            return false; // Ace-x vs non-connector, non-ace-x
        }

        // :-P This may be some of the most convoluted shit I have ever written.
        if self.is_connector() {
            if self.is_suited_connector() {
                if other.is_suited_connector() {
                    return true;
                }
                return other.is_ace_x_suited();
            }
            if self.is_offsuit_connector() {
                if other.is_offsuit_connector() {
                    return true;
                }
                return other.is_ace_x_offsuit();
            }
            if other.is_connector() && !other.is_suited_connector() && !other.is_offsuit_connector() {
                return true;
            }
            if other.is_ace_x() && !other.is_ace_x_suited() && !other.is_ace_x_offsuit() {
                return true;
            }
            return false;
        }

        if self.is_ace_x_suited() {
            return other.is_ace_x_suited();
        }
        if self.is_ace_x_offsuit() {
            return other.is_ace_x_offsuit();
        }
        if self.is_ace_x() {
            return other.is_ace_x();
        }
        false
    }

    fn is_ace_x_internal(self, qualifier: Qualifier) -> bool {
        if self.is_pair() {
            return false;
        }
        if (self.first == Rank::ACE) && self.qualifier == qualifier {
            return self.second != Rank::ACE;
        }
        false
    }

    /// NOTE: The `AceX` gto are the biggest pain in the ass.
    #[must_use]
    pub fn is_ace_x(&self) -> bool {
        self.is_ace_x_internal(Qualifier::ALL)
    }

    #[must_use]
    pub fn is_ace_x_suited(&self) -> bool {
        self.is_ace_x_internal(Qualifier::SUITED)
    }

    #[must_use]
    pub fn is_ace_x_offsuit(&self) -> bool {
        self.is_ace_x_internal(Qualifier::OFFSUIT)
    }

    #[must_use]
    pub fn is_connector(&self) -> bool {
        if self.is_pair() {
            return false;
        }
        let rank_diff = self.first as i8 - self.second as i8;
        rank_diff.abs() == 1
    }

    #[must_use]
    pub fn is_pair(&self) -> bool {
        self.first == self.second
    }

    #[must_use]
    pub fn is_same_type(&self, other: &Self) -> bool {
        if self.is_pair() {
            return other.is_pair();
        }
        (self.plus == other.plus) && (self.qualifier == other.qualifier)
    }

    #[must_use]
    pub fn is_suited(&self) -> bool {
        self.qualifier == Qualifier::SUITED
    }

    #[must_use]
    pub fn is_offsuit(&self) -> bool {
        self.qualifier == Qualifier::OFFSUIT
    }

    #[must_use]
    pub fn is_suited_connector(&self) -> bool {
        self.is_connector() && self.is_suited()
    }

    #[must_use]
    pub fn is_offsuit_connector(&self) -> bool {
        self.is_connector() && self.is_offsuit()
    }

    #[must_use]
    pub fn total_pairs(&self) -> usize {
        if self.is_pair() {
            6
        } else {
            match self.qualifier {
                Qualifier::SUITED => 4,
                Qualifier::OFFSUIT => 12,
                Qualifier::ALL => 16,
            }
        }
    }
}

impl Display for Combo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let first = self.first.to_string();
        let second = self.second.to_string();
        let qualifier = self.qualifier.to_string();

        if self.plus {
            write!(f, "{first}{second}{qualifier}+")
        } else {
            write!(f, "{first}{second}{qualifier}")
        }
    }
}

impl From<Two> for Combo {
    fn from(two: Two) -> Self {
        if two.is_blank() {
            return Combo::default();
        }
        Combo {
            first: two.first().get_rank(),
            second: two.second().get_rank(),
            plus: false,
            qualifier: {
                if two.is_pair() {
                    Qualifier::ALL
                } else if two.is_suited() {
                    Qualifier::SUITED
                } else {
                    Qualifier::OFFSUIT
                }
            },
        }
    }
}

impl FromStr for Combo {
    type Err = PKError;

    /// CHALLENGE: Write a script that will generate this code automatically
    /// instead of the stupid manual way I did it.
    #[allow(clippy::too_many_lines)]
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let s = raw.trim().to_lowercase();

        match s.as_str() {
            "aa" => Ok(Combo::COMBO_AA),
            "kk" => Ok(Combo::COMBO_KK),
            "kk+" => Ok(Combo::COMBO_KK_PLUS),
            "qq" => Ok(Combo::COMBO_QQ),
            "qq+" => Ok(Combo::COMBO_QQ_PLUS),
            "jj" => Ok(Combo::COMBO_JJ),
            "jj+" => Ok(Combo::COMBO_JJ_PLUS),
            "tt" => Ok(Combo::COMBO_TT),
            "tt+" => Ok(Combo::COMBO_TT_PLUS),
            "99" => Ok(Combo::COMBO_99),
            "99+" => Ok(Combo::COMBO_99_PLUS),
            "88" => Ok(Combo::COMBO_88),
            "88+" => Ok(Combo::COMBO_88_PLUS),
            "77" => Ok(Combo::COMBO_77),
            "77+" => Ok(Combo::COMBO_77_PLUS),
            "66" => Ok(Combo::COMBO_66),
            "66+" => Ok(Combo::COMBO_66_PLUS),
            "55" => Ok(Combo::COMBO_55),
            "55+" => Ok(Combo::COMBO_55_PLUS),
            "44" => Ok(Combo::COMBO_44),
            "44+" => Ok(Combo::COMBO_44_PLUS),
            "33" => Ok(Combo::COMBO_33),
            "33+" => Ok(Combo::COMBO_33_PLUS),
            "22" => Ok(Combo::COMBO_22),
            "22+" => Ok(Combo::COMBO_22_PLUS),
            "aks" => Ok(Combo::COMBO_AKs),
            "ako" => Ok(Combo::COMBO_AKo),
            "ak" => Ok(Combo::COMBO_AK),
            "aqs" => Ok(Combo::COMBO_AQs),
            "aqo" => Ok(Combo::COMBO_AQo),
            "aq" => Ok(Combo::COMBO_AQ),
            "aqs+" => Ok(Combo::COMBO_AQs_PLUS),
            "aqo+" => Ok(Combo::COMBO_AQo_PLUS),
            "aq+" => Ok(Combo::COMBO_AQ_PLUS),
            "ajs" => Ok(Combo::COMBO_AJs),
            "ajo" => Ok(Combo::COMBO_AJo),
            "aj" => Ok(Combo::COMBO_AJ),
            "ajs+" => Ok(Combo::COMBO_AJs_PLUS),
            "ajo+" => Ok(Combo::COMBO_AJo_PLUS),
            "aj+" => Ok(Combo::COMBO_AJ_PLUS),
            "ats" => Ok(Combo::COMBO_ATs),
            "ato" => Ok(Combo::COMBO_ATo),
            "at" => Ok(Combo::COMBO_AT),
            "ats+" => Ok(Combo::COMBO_ATs_PLUS),
            "ato+" => Ok(Combo::COMBO_ATo_PLUS),
            "at+" => Ok(Combo::COMBO_AT_PLUS),
            "a9s" => Ok(Combo::COMBO_A9s),
            "a9o" => Ok(Combo::COMBO_A9o),
            "a9" => Ok(Combo::COMBO_A9),
            "a9s+" => Ok(Combo::COMBO_A9s_PLUS),
            "a9o+" => Ok(Combo::COMBO_A9o_PLUS),
            "a9+" => Ok(Combo::COMBO_A9_PLUS),
            "a8s" => Ok(Combo::COMBO_A8s),
            "a8o" => Ok(Combo::COMBO_A8o),
            "a8" => Ok(Combo::COMBO_A8),
            "a8s+" => Ok(Combo::COMBO_A8s_PLUS),
            "a8o+" => Ok(Combo::COMBO_A8o_PLUS),
            "a8+" => Ok(Combo::COMBO_A8_PLUS),
            "a7s" => Ok(Combo::COMBO_A7s),
            "a7o" => Ok(Combo::COMBO_A7o),
            "a7" => Ok(Combo::COMBO_A7),
            "a7s+" => Ok(Combo::COMBO_A7s_PLUS),
            "a7o+" => Ok(Combo::COMBO_A7o_PLUS),
            "a7+" => Ok(Combo::COMBO_A7_PLUS),
            "a6s" => Ok(Combo::COMBO_A6s),
            "a6o" => Ok(Combo::COMBO_A6o),
            "a6" => Ok(Combo::COMBO_A6),
            "a6s+" => Ok(Combo::COMBO_A6s_PLUS),
            "a6o+" => Ok(Combo::COMBO_A6o_PLUS),
            "a6+" => Ok(Combo::COMBO_A6_PLUS),
            "a5s" => Ok(Combo::COMBO_A5s),
            "a5o" => Ok(Combo::COMBO_A5o),
            "a5" => Ok(Combo::COMBO_A5),
            "a5s+" => Ok(Combo::COMBO_A5s_PLUS),
            "a5o+" => Ok(Combo::COMBO_A5o_PLUS),
            "a5+" => Ok(Combo::COMBO_A5_PLUS),
            "a4s" => Ok(Combo::COMBO_A4s),
            "a4o" => Ok(Combo::COMBO_A4o),
            "a4" => Ok(Combo::COMBO_A4),
            "a4s+" => Ok(Combo::COMBO_A4s_PLUS),
            "a4o+" => Ok(Combo::COMBO_A4o_PLUS),
            "a4+" => Ok(Combo::COMBO_A4_PLUS),
            "a3s" => Ok(Combo::COMBO_A3s),
            "a3o" => Ok(Combo::COMBO_A3o),
            "a3" => Ok(Combo::COMBO_A3),
            "a3s+" => Ok(Combo::COMBO_A3s_PLUS),
            "a3o+" => Ok(Combo::COMBO_A3o_PLUS),
            "a3+" => Ok(Combo::COMBO_A3_PLUS),
            "a2s" => Ok(Combo::COMBO_A2s),
            "a2o" => Ok(Combo::COMBO_A2o),
            "a2" => Ok(Combo::COMBO_A2),
            "a2s+" => Ok(Combo::COMBO_A2s_PLUS),
            "a2o+" => Ok(Combo::COMBO_A2o_PLUS),
            "a2+" => Ok(Combo::COMBO_A2_PLUS),
            "kqs" => Ok(Combo::COMBO_KQs),
            "kqo" => Ok(Combo::COMBO_KQo),
            "kq" => Ok(Combo::COMBO_KQ),
            "kqs+" => Ok(Combo::COMBO_KQs_PLUS),
            "kqo+" => Ok(Combo::COMBO_KQo_PLUS),
            "kq+" => Ok(Combo::COMBO_KQ_PLUS),
            "kjs" => Ok(Combo::COMBO_KJs),
            "kjo" => Ok(Combo::COMBO_KJo),
            "kj" => Ok(Combo::COMBO_KJ),
            "kjs+" => Ok(Combo::COMBO_KJs_PLUS),
            "kjo+" => Ok(Combo::COMBO_KJo_PLUS),
            "kj+" => Ok(Combo::COMBO_KJ_PLUS),
            "kts" => Ok(Combo::COMBO_KTs),
            "kto" => Ok(Combo::COMBO_KTo),
            "kt" => Ok(Combo::COMBO_KT),
            "kts+" => Ok(Combo::COMBO_KTs_PLUS),
            "kto+" => Ok(Combo::COMBO_KTo_PLUS),
            "kt+" => Ok(Combo::COMBO_KT_PLUS),
            "k9s" => Ok(Combo::COMBO_K9s),
            "k9o" => Ok(Combo::COMBO_K9o),
            "k9" => Ok(Combo::COMBO_K9),
            "k9s+" => Ok(Combo::COMBO_K9s_PLUS),
            "k9o+" => Ok(Combo::COMBO_K9o_PLUS),
            "k9+" => Ok(Combo::COMBO_K9_PLUS),
            "k8s" => Ok(Combo::COMBO_K8s),
            "k8o" => Ok(Combo::COMBO_K8o),
            "k8" => Ok(Combo::COMBO_K8),
            "k8s+" => Ok(Combo::COMBO_K8s_PLUS),
            "k8o+" => Ok(Combo::COMBO_K8o_PLUS),
            "k8+" => Ok(Combo::COMBO_K8_PLUS),
            "k7s" => Ok(Combo::COMBO_K7s),
            "k7o" => Ok(Combo::COMBO_K7o),
            "k7" => Ok(Combo::COMBO_K7),
            "k7s+" => Ok(Combo::COMBO_K7s_PLUS),
            "k7o+" => Ok(Combo::COMBO_K7o_PLUS),
            "k7+" => Ok(Combo::COMBO_K7_PLUS),
            "k6s" => Ok(Combo::COMBO_K6s),
            "k6o" => Ok(Combo::COMBO_K6o),
            "k6" => Ok(Combo::COMBO_K6),
            "k6s+" => Ok(Combo::COMBO_K6s_PLUS),
            "k6o+" => Ok(Combo::COMBO_K6o_PLUS),
            "k6+" => Ok(Combo::COMBO_K6_PLUS),
            "k5s" => Ok(Combo::COMBO_K5s),
            "k5o" => Ok(Combo::COMBO_K5o),
            "k5" => Ok(Combo::COMBO_K5),
            "k5s+" => Ok(Combo::COMBO_K5s_PLUS),
            "k5o+" => Ok(Combo::COMBO_K5o_PLUS),
            "k5+" => Ok(Combo::COMBO_K5_PLUS),
            "k4s" => Ok(Combo::COMBO_K4s),
            "k4o" => Ok(Combo::COMBO_K4o),
            "k4" => Ok(Combo::COMBO_K4),
            "k4s+" => Ok(Combo::COMBO_K4s_PLUS),
            "k4o+" => Ok(Combo::COMBO_K4o_PLUS),
            "k4+" => Ok(Combo::COMBO_K4_PLUS),
            "k3s" => Ok(Combo::COMBO_K3s),
            "k3o" => Ok(Combo::COMBO_K3o),
            "k3" => Ok(Combo::COMBO_K3),
            "k3s+" => Ok(Combo::COMBO_K3s_PLUS),
            "k3o+" => Ok(Combo::COMBO_K3o_PLUS),
            "k3+" => Ok(Combo::COMBO_K3_PLUS),
            "k2s" => Ok(Combo::COMBO_K2s),
            "k2o" => Ok(Combo::COMBO_K2o),
            "k2" => Ok(Combo::COMBO_K2),
            "k2s+" => Ok(Combo::COMBO_K2s_PLUS),
            "k2o+" => Ok(Combo::COMBO_K2o_PLUS),
            "k2+" => Ok(Combo::COMBO_K2_PLUS),
            "qjs" => Ok(Combo::COMBO_QJs),
            "qjo" => Ok(Combo::COMBO_QJo),
            "qj" => Ok(Combo::COMBO_QJ),
            "qjs+" => Ok(Combo::COMBO_QJs_PLUS),
            "qjo+" => Ok(Combo::COMBO_QJo_PLUS),
            "qj+" => Ok(Combo::COMBO_QJ_PLUS),
            "qts" => Ok(Combo::COMBO_QTs),
            "qto" => Ok(Combo::COMBO_QTo),
            "qt" => Ok(Combo::COMBO_QT),
            "qts+" => Ok(Combo::COMBO_QTs_PLUS),
            "qto+" => Ok(Combo::COMBO_QTo_PLUS),
            "qt+" => Ok(Combo::COMBO_QT_PLUS),
            "q9s" => Ok(Combo::COMBO_Q9s),
            "q9o" => Ok(Combo::COMBO_Q9o),
            "q9" => Ok(Combo::COMBO_Q9),
            "q9s+" => Ok(Combo::COMBO_Q9s_PLUS),
            "q9o+" => Ok(Combo::COMBO_Q9o_PLUS),
            "q9+" => Ok(Combo::COMBO_Q9_PLUS),
            "q8s" => Ok(Combo::COMBO_Q8s),
            "q8o" => Ok(Combo::COMBO_Q8o),
            "q8" => Ok(Combo::COMBO_Q8),
            "q8s+" => Ok(Combo::COMBO_Q8s_PLUS),
            "q8o+" => Ok(Combo::COMBO_Q8o_PLUS),
            "q8+" => Ok(Combo::COMBO_Q8_PLUS),
            "q7s" => Ok(Combo::COMBO_Q7s),
            "q7o" => Ok(Combo::COMBO_Q7o),
            "q7" => Ok(Combo::COMBO_Q7),
            "q7s+" => Ok(Combo::COMBO_Q7s_PLUS),
            "q7o+" => Ok(Combo::COMBO_Q7o_PLUS),
            "q7+" => Ok(Combo::COMBO_Q7_PLUS),
            "q6s" => Ok(Combo::COMBO_Q6s),
            "q6o" => Ok(Combo::COMBO_Q6o),
            "q6" => Ok(Combo::COMBO_Q6),
            "q6s+" => Ok(Combo::COMBO_Q6s_PLUS),
            "q6o+" => Ok(Combo::COMBO_Q6o_PLUS),
            "q6+" => Ok(Combo::COMBO_Q6_PLUS),
            "q5s" => Ok(Combo::COMBO_Q5s),
            "q5o" => Ok(Combo::COMBO_Q5o),
            "q5" => Ok(Combo::COMBO_Q5),
            "q5s+" => Ok(Combo::COMBO_Q5s_PLUS),
            "q5o+" => Ok(Combo::COMBO_Q5o_PLUS),
            "q5+" => Ok(Combo::COMBO_Q5_PLUS),
            "q4s" => Ok(Combo::COMBO_Q4s),
            "q4o" => Ok(Combo::COMBO_Q4o),
            "q4" => Ok(Combo::COMBO_Q4),
            "q4s+" => Ok(Combo::COMBO_Q4s_PLUS),
            "q4o+" => Ok(Combo::COMBO_Q4o_PLUS),
            "q4+" => Ok(Combo::COMBO_Q4_PLUS),
            "q3s" => Ok(Combo::COMBO_Q3s),
            "q3o" => Ok(Combo::COMBO_Q3o),
            "q3" => Ok(Combo::COMBO_Q3),
            "q3s+" => Ok(Combo::COMBO_Q3s_PLUS),
            "q3o+" => Ok(Combo::COMBO_Q3o_PLUS),
            "q3+" => Ok(Combo::COMBO_Q3_PLUS),
            "q2s" => Ok(Combo::COMBO_Q2s),
            "q2o" => Ok(Combo::COMBO_Q2o),
            "q2" => Ok(Combo::COMBO_Q2),
            "q2s+" => Ok(Combo::COMBO_Q2s_PLUS),
            "q2o+" => Ok(Combo::COMBO_Q2o_PLUS),
            "q2+" => Ok(Combo::COMBO_Q2_PLUS),
            "jts" => Ok(Combo::COMBO_JTs),
            "jto" => Ok(Combo::COMBO_JTo),
            "jt" => Ok(Combo::COMBO_JT),
            "jts+" => Ok(Combo::COMBO_JTs_PLUS),
            "jto+" => Ok(Combo::COMBO_JTo_PLUS),
            "jt+" => Ok(Combo::COMBO_JT_PLUS),
            "j9s" => Ok(Combo::COMBO_J9s),
            "j9o" => Ok(Combo::COMBO_J9o),
            "j9" => Ok(Combo::COMBO_J9),
            "j9s+" => Ok(Combo::COMBO_J9s_PLUS),
            "j9o+" => Ok(Combo::COMBO_J9o_PLUS),
            "j9+" => Ok(Combo::COMBO_J9_PLUS),
            "j8s" => Ok(Combo::COMBO_J8s),
            "j8o" => Ok(Combo::COMBO_J8o),
            "j8" => Ok(Combo::COMBO_J8),
            "j8s+" => Ok(Combo::COMBO_J8s_PLUS),
            "j8o+" => Ok(Combo::COMBO_J8o_PLUS),
            "j8+" => Ok(Combo::COMBO_J8_PLUS),
            "j7s" => Ok(Combo::COMBO_J7s),
            "j7o" => Ok(Combo::COMBO_J7o),
            "j7" => Ok(Combo::COMBO_J7),
            "j7s+" => Ok(Combo::COMBO_J7s_PLUS),
            "j7o+" => Ok(Combo::COMBO_J7o_PLUS),
            "j7+" => Ok(Combo::COMBO_J7_PLUS),
            "j6s" => Ok(Combo::COMBO_J6s),
            "j6o" => Ok(Combo::COMBO_J6o),
            "j6" => Ok(Combo::COMBO_J6),
            "j6s+" => Ok(Combo::COMBO_J6s_PLUS),
            "j6o+" => Ok(Combo::COMBO_J6o_PLUS),
            "j6+" => Ok(Combo::COMBO_J6_PLUS),
            "j5s" => Ok(Combo::COMBO_J5s),
            "j5o" => Ok(Combo::COMBO_J5o),
            "j5" => Ok(Combo::COMBO_J5),
            "j5s+" => Ok(Combo::COMBO_J5s_PLUS),
            "j5o+" => Ok(Combo::COMBO_J5o_PLUS),
            "j5+" => Ok(Combo::COMBO_J5_PLUS),
            "j4s" => Ok(Combo::COMBO_J4s),
            "j4o" => Ok(Combo::COMBO_J4o),
            "j4" => Ok(Combo::COMBO_J4),
            "j4s+" => Ok(Combo::COMBO_J4s_PLUS),
            "j4o+" => Ok(Combo::COMBO_J4o_PLUS),
            "j4+" => Ok(Combo::COMBO_J4_PLUS),
            "j3s" => Ok(Combo::COMBO_J3s),
            "j3o" => Ok(Combo::COMBO_J3o),
            "j3" => Ok(Combo::COMBO_J3),
            "j3s+" => Ok(Combo::COMBO_J3s_PLUS),
            "j3o+" => Ok(Combo::COMBO_J3o_PLUS),
            "j3+" => Ok(Combo::COMBO_J3_PLUS),
            "j2s" => Ok(Combo::COMBO_J2s),
            "j2o" => Ok(Combo::COMBO_J2o),
            "j2" => Ok(Combo::COMBO_J2),
            "j2s+" => Ok(Combo::COMBO_J2s_PLUS),
            "j2o+" => Ok(Combo::COMBO_J2o_PLUS),
            "j2+" => Ok(Combo::COMBO_J2_PLUS),
            "t9s" => Ok(Combo::COMBO_T9s),
            "t9o" => Ok(Combo::COMBO_T9o),
            "t9" => Ok(Combo::COMBO_T9),
            "t9s+" => Ok(Combo::COMBO_T9s_PLUS),
            "t9o+" => Ok(Combo::COMBO_T9o_PLUS),
            "t9+" => Ok(Combo::COMBO_T9_PLUS),
            "t8s" => Ok(Combo::COMBO_T8s),
            "t8o" => Ok(Combo::COMBO_T8o),
            "t8" => Ok(Combo::COMBO_T8),
            "t8s+" => Ok(Combo::COMBO_T8s_PLUS),
            "t8o+" => Ok(Combo::COMBO_T8o_PLUS),
            "t8+" => Ok(Combo::COMBO_T8_PLUS),
            "t7s" => Ok(Combo::COMBO_T7s),
            "t7o" => Ok(Combo::COMBO_T7o),
            "t7" => Ok(Combo::COMBO_T7),
            "t7s+" => Ok(Combo::COMBO_T7s_PLUS),
            "t7o+" => Ok(Combo::COMBO_T7o_PLUS),
            "t7+" => Ok(Combo::COMBO_T7_PLUS),
            "t6s" => Ok(Combo::COMBO_T6s),
            "t6o" => Ok(Combo::COMBO_T6o),
            "t6" => Ok(Combo::COMBO_T6),
            "t6s+" => Ok(Combo::COMBO_T6s_PLUS),
            "t6o+" => Ok(Combo::COMBO_T6o_PLUS),
            "t6+" => Ok(Combo::COMBO_T6_PLUS),
            "t5s" => Ok(Combo::COMBO_T5s),
            "t5o" => Ok(Combo::COMBO_T5o),
            "t5" => Ok(Combo::COMBO_T5),
            "t5s+" => Ok(Combo::COMBO_T5s_PLUS),
            "t5o+" => Ok(Combo::COMBO_T5o_PLUS),
            "t5+" => Ok(Combo::COMBO_T5_PLUS),
            "t4s" => Ok(Combo::COMBO_T4s),
            "t4o" => Ok(Combo::COMBO_T4o),
            "t4" => Ok(Combo::COMBO_T4),
            "t4s+" => Ok(Combo::COMBO_T4s_PLUS),
            "t4o+" => Ok(Combo::COMBO_T4o_PLUS),
            "t4+" => Ok(Combo::COMBO_T4_PLUS),
            "t3s" => Ok(Combo::COMBO_T3s),
            "t3o" => Ok(Combo::COMBO_T3o),
            "t3" => Ok(Combo::COMBO_T3),
            "t3s+" => Ok(Combo::COMBO_T3s_PLUS),
            "t3o+" => Ok(Combo::COMBO_T3o_PLUS),
            "t3+" => Ok(Combo::COMBO_T3_PLUS),
            "t2s" => Ok(Combo::COMBO_T2s),
            "t2o" => Ok(Combo::COMBO_T2o),
            "t2" => Ok(Combo::COMBO_T2),
            "t2s+" => Ok(Combo::COMBO_T2s_PLUS),
            "t2o+" => Ok(Combo::COMBO_T2o_PLUS),
            "t2+" => Ok(Combo::COMBO_T2_PLUS),
            "98s" => Ok(Combo::COMBO_98s),
            "98o" => Ok(Combo::COMBO_98o),
            "98" => Ok(Combo::COMBO_98),
            "98s+" => Ok(Combo::COMBO_98s_PLUS),
            "98o+" => Ok(Combo::COMBO_98o_PLUS),
            "98+" => Ok(Combo::COMBO_98_PLUS),
            "97s" => Ok(Combo::COMBO_97s),
            "97o" => Ok(Combo::COMBO_97o),
            "97" => Ok(Combo::COMBO_97),
            "97s+" => Ok(Combo::COMBO_97s_PLUS),
            "97o+" => Ok(Combo::COMBO_97o_PLUS),
            "97+" => Ok(Combo::COMBO_97_PLUS),
            "96s" => Ok(Combo::COMBO_96s),
            "96o" => Ok(Combo::COMBO_96o),
            "96" => Ok(Combo::COMBO_96),
            "96s+" => Ok(Combo::COMBO_96s_PLUS),
            "96o+" => Ok(Combo::COMBO_96o_PLUS),
            "96+" => Ok(Combo::COMBO_96_PLUS),
            "95s" => Ok(Combo::COMBO_95s),
            "95o" => Ok(Combo::COMBO_95o),
            "95" => Ok(Combo::COMBO_95),
            "95s+" => Ok(Combo::COMBO_95s_PLUS),
            "95o+" => Ok(Combo::COMBO_95o_PLUS),
            "95+" => Ok(Combo::COMBO_95_PLUS),
            "94s" => Ok(Combo::COMBO_94s),
            "94o" => Ok(Combo::COMBO_94o),
            "94" => Ok(Combo::COMBO_94),
            "94s+" => Ok(Combo::COMBO_94s_PLUS),
            "94o+" => Ok(Combo::COMBO_94o_PLUS),
            "94+" => Ok(Combo::COMBO_94_PLUS),
            "93s" => Ok(Combo::COMBO_93s),
            "93o" => Ok(Combo::COMBO_93o),
            "93" => Ok(Combo::COMBO_93),
            "93s+" => Ok(Combo::COMBO_93s_PLUS),
            "93o+" => Ok(Combo::COMBO_93o_PLUS),
            "93+" => Ok(Combo::COMBO_93_PLUS),
            "92s" => Ok(Combo::COMBO_92s),
            "92o" => Ok(Combo::COMBO_92o),
            "92" => Ok(Combo::COMBO_92),
            "92s+" => Ok(Combo::COMBO_92s_PLUS),
            "92o+" => Ok(Combo::COMBO_92o_PLUS),
            "92+" => Ok(Combo::COMBO_92_PLUS),
            "87s" => Ok(Combo::COMBO_87s),
            "87o" => Ok(Combo::COMBO_87o),
            "87" => Ok(Combo::COMBO_87),
            "87s+" => Ok(Combo::COMBO_87s_PLUS),
            "87o+" => Ok(Combo::COMBO_87o_PLUS),
            "87+" => Ok(Combo::COMBO_87_PLUS),
            "86s" => Ok(Combo::COMBO_86s),
            "86o" => Ok(Combo::COMBO_86o),
            "86" => Ok(Combo::COMBO_86),
            "86s+" => Ok(Combo::COMBO_86s_PLUS),
            "86o+" => Ok(Combo::COMBO_86o_PLUS),
            "86+" => Ok(Combo::COMBO_86_PLUS),
            "85s" => Ok(Combo::COMBO_85s),
            "85o" => Ok(Combo::COMBO_85o),
            "85" => Ok(Combo::COMBO_85),
            "85s+" => Ok(Combo::COMBO_85s_PLUS),
            "85o+" => Ok(Combo::COMBO_85o_PLUS),
            "85+" => Ok(Combo::COMBO_85_PLUS),
            "84s" => Ok(Combo::COMBO_84s),
            "84o" => Ok(Combo::COMBO_84o),
            "84" => Ok(Combo::COMBO_84),
            "84s+" => Ok(Combo::COMBO_84s_PLUS),
            "84o+" => Ok(Combo::COMBO_84o_PLUS),
            "84+" => Ok(Combo::COMBO_84_PLUS),
            "83s" => Ok(Combo::COMBO_83s),
            "83o" => Ok(Combo::COMBO_83o),
            "83" => Ok(Combo::COMBO_83),
            "83s+" => Ok(Combo::COMBO_83s_PLUS),
            "83o+" => Ok(Combo::COMBO_83o_PLUS),
            "83+" => Ok(Combo::COMBO_83_PLUS),
            "82s" => Ok(Combo::COMBO_82s),
            "82o" => Ok(Combo::COMBO_82o),
            "82" => Ok(Combo::COMBO_82),
            "82s+" => Ok(Combo::COMBO_82s_PLUS),
            "82o+" => Ok(Combo::COMBO_82o_PLUS),
            "82+" => Ok(Combo::COMBO_82_PLUS),
            "76s" => Ok(Combo::COMBO_76s),
            "76o" => Ok(Combo::COMBO_76o),
            "76" => Ok(Combo::COMBO_76),
            "76s+" => Ok(Combo::COMBO_76s_PLUS),
            "76o+" => Ok(Combo::COMBO_76o_PLUS),
            "76+" => Ok(Combo::COMBO_76_PLUS),
            "75s" => Ok(Combo::COMBO_75s),
            "75o" => Ok(Combo::COMBO_75o),
            "75" => Ok(Combo::COMBO_75),
            "75s+" => Ok(Combo::COMBO_75s_PLUS),
            "75o+" => Ok(Combo::COMBO_75o_PLUS),
            "75+" => Ok(Combo::COMBO_75_PLUS),
            "74s" => Ok(Combo::COMBO_74s),
            "74o" => Ok(Combo::COMBO_74o),
            "74" => Ok(Combo::COMBO_74),
            "74s+" => Ok(Combo::COMBO_74s_PLUS),
            "74o+" => Ok(Combo::COMBO_74o_PLUS),
            "74+" => Ok(Combo::COMBO_74_PLUS),
            "73s" => Ok(Combo::COMBO_73s),
            "73o" => Ok(Combo::COMBO_73o),
            "73" => Ok(Combo::COMBO_73),
            "73s+" => Ok(Combo::COMBO_73s_PLUS),
            "73o+" => Ok(Combo::COMBO_73o_PLUS),
            "73+" => Ok(Combo::COMBO_73_PLUS),
            "72s" => Ok(Combo::COMBO_72s),
            "72o" => Ok(Combo::COMBO_72o),
            "72" => Ok(Combo::COMBO_72),
            "72s+" => Ok(Combo::COMBO_72s_PLUS),
            "72o+" => Ok(Combo::COMBO_72o_PLUS),
            "72+" => Ok(Combo::COMBO_72_PLUS),
            "65s" => Ok(Combo::COMBO_65s),
            "65o" => Ok(Combo::COMBO_65o),
            "65" => Ok(Combo::COMBO_65),
            "65s+" => Ok(Combo::COMBO_65s_PLUS),
            "65o+" => Ok(Combo::COMBO_65o_PLUS),
            "65+" => Ok(Combo::COMBO_65_PLUS),
            "64s" => Ok(Combo::COMBO_64s),
            "64o" => Ok(Combo::COMBO_64o),
            "64" => Ok(Combo::COMBO_64),
            "64s+" => Ok(Combo::COMBO_64s_PLUS),
            "64o+" => Ok(Combo::COMBO_64o_PLUS),
            "64+" => Ok(Combo::COMBO_64_PLUS),
            "63s" => Ok(Combo::COMBO_63s),
            "63o" => Ok(Combo::COMBO_63o),
            "63" => Ok(Combo::COMBO_63),
            "63s+" => Ok(Combo::COMBO_63s_PLUS),
            "63o+" => Ok(Combo::COMBO_63o_PLUS),
            "63+" => Ok(Combo::COMBO_63_PLUS),
            "62s" => Ok(Combo::COMBO_62s),
            "62o" => Ok(Combo::COMBO_62o),
            "62" => Ok(Combo::COMBO_62),
            "62s+" => Ok(Combo::COMBO_62s_PLUS),
            "62o+" => Ok(Combo::COMBO_62o_PLUS),
            "62+" => Ok(Combo::COMBO_62_PLUS),
            "54s" => Ok(Combo::COMBO_54s),
            "54o" => Ok(Combo::COMBO_54o),
            "54" => Ok(Combo::COMBO_54),
            "54s+" => Ok(Combo::COMBO_54s_PLUS),
            "54o+" => Ok(Combo::COMBO_54o_PLUS),
            "54+" => Ok(Combo::COMBO_54_PLUS),
            "53s" => Ok(Combo::COMBO_53s),
            "53o" => Ok(Combo::COMBO_53o),
            "53" => Ok(Combo::COMBO_53),
            "53s+" => Ok(Combo::COMBO_53s_PLUS),
            "53o+" => Ok(Combo::COMBO_53o_PLUS),
            "53+" => Ok(Combo::COMBO_53_PLUS),
            "52s" => Ok(Combo::COMBO_52s),
            "52o" => Ok(Combo::COMBO_52o),
            "52" => Ok(Combo::COMBO_52),
            "52s+" => Ok(Combo::COMBO_52s_PLUS),
            "52o+" => Ok(Combo::COMBO_52o_PLUS),
            "52+" => Ok(Combo::COMBO_52_PLUS),
            "43s" => Ok(Combo::COMBO_43s),
            "43o" => Ok(Combo::COMBO_43o),
            "43" => Ok(Combo::COMBO_43),
            "43s+" => Ok(Combo::COMBO_43s_PLUS),
            "43o+" => Ok(Combo::COMBO_43o_PLUS),
            "43+" => Ok(Combo::COMBO_43_PLUS),
            "42s" => Ok(Combo::COMBO_42s),
            "42o" => Ok(Combo::COMBO_42o),
            "42" => Ok(Combo::COMBO_42),
            "42s+" => Ok(Combo::COMBO_42s_PLUS),
            "42o+" => Ok(Combo::COMBO_42o_PLUS),
            "42+" => Ok(Combo::COMBO_42_PLUS),
            "32s" => Ok(Combo::COMBO_32s),
            "32o" => Ok(Combo::COMBO_32o),
            "32" => Ok(Combo::COMBO_32),
            "32s+" => Ok(Combo::COMBO_32s_PLUS),
            "32o+" => Ok(Combo::COMBO_32o_PLUS),
            "32+" => Ok(Combo::COMBO_32_PLUS),
            _ => {
                // TODO: Add logging
                println!("Unable to process {s}");
                Err(PKError::InvalidComboIndex)
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__ranges__combo_tests {
    use super::*;
    use rand::rng;
    use rand::seq::SliceRandom;
    use rstest::rstest;

    #[test]
    fn distance_first() {
        assert_eq!(Combo::COMBO_AA.distance_first(&Combo::COMBO_AK), 0);
        assert_eq!(Combo::COMBO_AK.distance_first(&Combo::COMBO_AA), 0);
        assert_eq!(Combo::COMBO_AKs.distance_first(&Combo::COMBO_AQo), 0);
        assert_eq!(Combo::COMBO_AQo.distance_first(&Combo::COMBO_AKs), 0);
        assert_eq!(Combo::COMBO_76s.distance_first(&Combo::COMBO_65s), 1);
        assert_eq!(Combo::COMBO_65s.distance_first(&Combo::COMBO_76s), 1);
        assert_eq!(Combo::COMBO_76s.distance_first(&Combo::COMBO_54s), 2);
        assert_eq!(Combo::COMBO_54s.distance_first(&Combo::COMBO_76s), 2);
        assert_eq!(Combo::COMBO_AK.distance_first(&Combo::COMBO_22), 12);
        assert_eq!(Combo::COMBO_22.distance_first(&Combo::COMBO_AK), 12);
    }

    #[test]
    fn distance_second() {
        assert_eq!(Combo::COMBO_AA.distance_second(&Combo::COMBO_AK), 1);
        assert_eq!(Combo::COMBO_AK.distance_second(&Combo::COMBO_AA), 1);
        assert_eq!(Combo::COMBO_AKs.distance_second(&Combo::COMBO_AQo), 1);
        assert_eq!(Combo::COMBO_AQo.distance_second(&Combo::COMBO_AKs), 1);
        assert_eq!(Combo::COMBO_76s.distance_second(&Combo::COMBO_65s), 1);
        assert_eq!(Combo::COMBO_65s.distance_second(&Combo::COMBO_76s), 1);
        assert_eq!(Combo::COMBO_76s.distance_second(&Combo::COMBO_54s), 2);
        assert_eq!(Combo::COMBO_54s.distance_second(&Combo::COMBO_76s), 2);
        assert_eq!(Combo::COMBO_AK.distance_second(&Combo::COMBO_22), 11);
        assert_eq!(Combo::COMBO_22.distance_second(&Combo::COMBO_AK), 11);
    }

    #[test]
    fn equals() {
        assert_eq!(Combo::COMBO_AA, Combo::COMBO_AA);
        assert_eq!(Combo::COMBO_AK, Combo::COMBO_AK);
        assert_eq!(Combo::COMBO_AKs, Combo::COMBO_AKs);
        assert_eq!(Combo::COMBO_AKo, Combo::COMBO_AKo);
        assert_ne!(Combo::COMBO_AK, Combo::COMBO_QQ_PLUS);
        assert_ne!(Combo::COMBO_AKs, Combo::COMBO_76s);
        assert_ne!(Combo::COMBO_AKo, Combo::COMBO_22);
    }

    #[test]
    fn is_aligned_with() {
        assert!(Combo::COMBO_AA.is_aligned_with(&Combo::COMBO_KK));
        assert!(Combo::COMBO_AKs.is_aligned_with(&Combo::COMBO_KQs));
        assert!(Combo::COMBO_AKo.is_aligned_with(&Combo::COMBO_KQo));
        assert!(Combo::COMBO_AK.is_aligned_with(&Combo::COMBO_KQ));
        assert!(Combo::COMBO_AQ.is_aligned_with(&Combo::COMBO_A2));
        assert!(Combo::COMBO_AQo.is_aligned_with(&Combo::COMBO_A2o));
        assert!(Combo::COMBO_AQs.is_aligned_with(&Combo::COMBO_A2s));

        // Negative tests
        // Either plus
        assert!(!Combo::COMBO_QQ_PLUS.is_aligned_with(&Combo::COMBO_99_PLUS));
        assert!(!Combo::COMBO_AQo_PLUS.is_aligned_with(&Combo::COMBO_AQo_PLUS));
        assert!(!Combo::COMBO_AQo_PLUS.is_aligned_with(&Combo::COMBO_AQo));
        // Same
        assert!(!Combo::COMBO_AK.is_aligned_with(&Combo::COMBO_AK));
        assert!(!Combo::COMBO_AKs.is_aligned_with(&Combo::COMBO_AKs));
        assert!(!Combo::COMBO_AKo.is_aligned_with(&Combo::COMBO_AKo));
        assert!(!Combo::COMBO_AA.is_aligned_with(&Combo::COMBO_AA));

        assert!(!Combo::COMBO_AQo.is_aligned_with(&Combo::COMBO_AQo_PLUS));
    }

    #[test]
    fn is_ace_x() {
        assert!(Combo::COMBO_AK.is_ace_x());
        assert!(Combo::COMBO_AJ.is_ace_x());
        assert!(!Combo::COMBO_AKo.is_ace_x());
        assert!(!Combo::COMBO_AJs.is_ace_x());
        assert!(!Combo::COMBO_AQo_PLUS.is_ace_x());
        assert!(!Combo::COMBO_AA.is_ace_x());
        assert!(!Combo::COMBO_QQ_PLUS.is_ace_x());
        assert!(!Combo::COMBO_76s.is_ace_x());
    }

    #[test]
    fn is_ace_x_suited() {
        assert!(Combo::COMBO_AKs.is_ace_x_suited());
        assert!(Combo::COMBO_AJs.is_ace_x_suited());
        assert!(!Combo::COMBO_AQo_PLUS.is_ace_x_suited());
        assert!(!Combo::COMBO_AA.is_ace_x_suited());
        assert!(!Combo::COMBO_QQ_PLUS.is_ace_x_suited());
        assert!(!Combo::COMBO_76s.is_ace_x_suited());
    }

    #[test]
    fn is_ace_x_offsuit() {
        assert!(Combo::COMBO_AKo.is_ace_x_offsuit());
        assert!(Combo::COMBO_AQo_PLUS.is_ace_x_offsuit());
        assert!(!Combo::COMBO_AJs.is_ace_x_offsuit());
        assert!(!Combo::COMBO_AA.is_ace_x_offsuit());
        assert!(!Combo::COMBO_QQ_PLUS.is_ace_x_offsuit());
        assert!(!Combo::COMBO_76s.is_ace_x_offsuit());
    }

    #[test]
    fn is_connector() {
        assert!(Combo::COMBO_76s.is_connector());
        assert!(Combo::COMBO_65s.is_connector());
        assert!(Combo::COMBO_AKo.is_connector());
        assert!(!Combo::COMBO_AA.is_connector());
        assert!(!Combo::COMBO_AQo.is_connector());
    }

    #[test]
    fn is_pocket_pair() {
        assert!(Combo::COMBO_AA.is_pair());
        assert!(Combo::COMBO_22.is_pair());
        assert!(!Combo::COMBO_AKs.is_pair());
        assert!(!Combo::COMBO_AQo_PLUS.is_pair());
    }

    #[test]
    fn is_same_type() {
        assert!(Combo::COMBO_AA.is_same_type(&Combo::COMBO_KK));
        assert!(Combo::COMBO_QQ_PLUS.is_same_type(&Combo::COMBO_99_PLUS));
        assert!(Combo::COMBO_AJo_PLUS.is_same_type(&Combo::COMBO_AQo_PLUS));
        assert!(!Combo::COMBO_AKs.is_same_type(&Combo::COMBO_AQo_PLUS));
    }

    #[test]
    fn is_suited() {
        assert!(Combo::COMBO_AKs.is_suited());
        assert!(Combo::COMBO_QJs.is_suited());
        assert!(Combo::COMBO_QTs.is_suited());
        assert!(Combo::COMBO_Q9s_PLUS.is_suited());
        assert!(!Combo::COMBO_AKo.is_suited());
        assert!(!Combo::COMBO_AQo_PLUS.is_suited());
        assert!(!Combo::COMBO_AA.is_suited());
    }

    #[test]
    fn is_suited_connector() {
        assert!(Combo::COMBO_AKs.is_suited_connector());
        assert!(Combo::COMBO_KQs.is_suited_connector());
        assert!(Combo::COMBO_QJs.is_suited_connector());
        assert!(Combo::COMBO_76s.is_suited_connector());
        assert!(!Combo::COMBO_AKo.is_suited_connector());
        assert!(!Combo::COMBO_AQo_PLUS.is_suited_connector());
        assert!(!Combo::COMBO_AA.is_suited_connector());
    }

    #[test]
    fn is_offsuit_connector() {
        assert!(Combo::COMBO_AKo.is_offsuit_connector());
        assert!(Combo::COMBO_QJo.is_offsuit_connector());
        assert!(Combo::COMBO_76o.is_offsuit_connector());
        assert!(!Combo::COMBO_AKs.is_offsuit_connector());
        assert!(!Combo::COMBO_AQo_PLUS.is_offsuit_connector());
        assert!(!Combo::COMBO_AA.is_offsuit_connector());
        assert!(!Combo::COMBO_AK.is_offsuit_connector());
    }

    #[test]
    fn display() {
        assert_eq!(Combo::COMBO_AA.to_string(), "AA");
        assert_eq!(Combo::COMBO_AKs.to_string(), "AKs");
        assert_eq!(Combo::COMBO_AKo.to_string(), "AKo");
        assert_eq!(Combo::COMBO_AQo_PLUS.to_string(), "AQo+");
        assert_eq!(Combo::COMBO_QQ_PLUS.to_string(), "QQ+");
        assert_eq!(Combo::COMBO_99_PLUS.to_string(), "99+");
    }

    #[rstest]
    #[case(Two::HAND_AS_AC, Combo::COMBO_AA)]
    #[case(Two::HAND_AC_KC, Combo::COMBO_AKs)]
    #[case(Two::HAND_AH_KS, Combo::COMBO_AKo)]
    #[case(Two::HAND_9S_2S, Combo::COMBO_92s)]
    #[case(Two::HAND_6S_4D, Combo::COMBO_64o)]
    fn from__two(#[case] two: Two, #[case] combo: Combo) {
        assert_eq!(Combo::from(two), combo);
    }

    // region Combo::from_str
    #[rstest]
    #[case("AA ", Combo::COMBO_AA)]
    #[case(" Aa", Combo::COMBO_AA)]
    #[case(" aa ", Combo::COMBO_AA)]
    #[case("KK", Combo::COMBO_KK)]
    #[case("kk+", Combo::COMBO_KK_PLUS)]
    #[case("QQ", Combo::COMBO_QQ)]
    #[case("QQ+", Combo::COMBO_QQ_PLUS)]
    #[case("JJ", Combo::COMBO_JJ)]
    #[case("JJ+", Combo::COMBO_JJ_PLUS)]
    #[case("TT", Combo::COMBO_TT)]
    #[case("TT+", Combo::COMBO_TT_PLUS)]
    #[case("99", Combo::COMBO_99)]
    #[case("99+", Combo::COMBO_99_PLUS)]
    #[case("88", Combo::COMBO_88)]
    #[case("88+", Combo::COMBO_88_PLUS)]
    #[case("77", Combo::COMBO_77)]
    #[case("77+", Combo::COMBO_77_PLUS)]
    #[case("66", Combo::COMBO_66)]
    #[case("66+", Combo::COMBO_66_PLUS)]
    #[case("55", Combo::COMBO_55)]
    #[case("55+", Combo::COMBO_55_PLUS)]
    #[case("44", Combo::COMBO_44)]
    #[case("44+", Combo::COMBO_44_PLUS)]
    #[case("33", Combo::COMBO_33)]
    #[case("33+", Combo::COMBO_33_PLUS)]
    #[case("22", Combo::COMBO_22)]
    #[case("22+", Combo::COMBO_22_PLUS)]
    #[case("AKs", Combo::COMBO_AKs)]
    #[case("AKo", Combo::COMBO_AKo)]
    #[case("AK", Combo::COMBO_AK)]
    #[case("AQs", Combo::COMBO_AQs)]
    #[case("AQo", Combo::COMBO_AQo)]
    #[case("AQ", Combo::COMBO_AQ)]
    #[case("AQs+", Combo::COMBO_AQs_PLUS)]
    #[case("AQo+", Combo::COMBO_AQo_PLUS)]
    #[case("AQ+", Combo::COMBO_AQ_PLUS)]
    #[case("AJs", Combo::COMBO_AJs)]
    #[case("AJo", Combo::COMBO_AJo)]
    #[case("AJ", Combo::COMBO_AJ)]
    #[case("AJs+", Combo::COMBO_AJs_PLUS)]
    #[case("AJo+", Combo::COMBO_AJo_PLUS)]
    #[case("AJ+", Combo::COMBO_AJ_PLUS)]
    #[case("ATs", Combo::COMBO_ATs)]
    #[case("ATo", Combo::COMBO_ATo)]
    #[case("AT", Combo::COMBO_AT)]
    #[case("ATs+", Combo::COMBO_ATs_PLUS)]
    #[case("ATo+", Combo::COMBO_ATo_PLUS)]
    #[case("AT+", Combo::COMBO_AT_PLUS)]
    #[case("A9s", Combo::COMBO_A9s)]
    #[case("A9o", Combo::COMBO_A9o)]
    #[case("A9", Combo::COMBO_A9)]
    #[case("A9s+", Combo::COMBO_A9s_PLUS)]
    #[case("A9o+", Combo::COMBO_A9o_PLUS)]
    #[case("A9+", Combo::COMBO_A9_PLUS)]
    #[case("A8s", Combo::COMBO_A8s)]
    #[case("A8o", Combo::COMBO_A8o)]
    #[case("A8", Combo::COMBO_A8)]
    #[case("A8s+", Combo::COMBO_A8s_PLUS)]
    #[case("A8o+", Combo::COMBO_A8o_PLUS)]
    #[case("A8+", Combo::COMBO_A8_PLUS)]
    #[case("A7s", Combo::COMBO_A7s)]
    #[case("A7o", Combo::COMBO_A7o)]
    #[case("A7", Combo::COMBO_A7)]
    #[case("A7s+", Combo::COMBO_A7s_PLUS)]
    #[case("A7o+", Combo::COMBO_A7o_PLUS)]
    #[case("A7+", Combo::COMBO_A7_PLUS)]
    #[case("A6s", Combo::COMBO_A6s)]
    #[case("A6o", Combo::COMBO_A6o)]
    #[case("A6", Combo::COMBO_A6)]
    #[case("A6s+", Combo::COMBO_A6s_PLUS)]
    #[case("A6o+", Combo::COMBO_A6o_PLUS)]
    #[case("A6+", Combo::COMBO_A6_PLUS)]
    #[case("A5s", Combo::COMBO_A5s)]
    #[case("A5o", Combo::COMBO_A5o)]
    #[case("A5", Combo::COMBO_A5)]
    #[case("A5s+", Combo::COMBO_A5s_PLUS)]
    #[case("A5o+", Combo::COMBO_A5o_PLUS)]
    #[case("A5+", Combo::COMBO_A5_PLUS)]
    #[case("A4s", Combo::COMBO_A4s)]
    #[case("A4o", Combo::COMBO_A4o)]
    #[case("A4", Combo::COMBO_A4)]
    #[case("A4s+", Combo::COMBO_A4s_PLUS)]
    #[case("A4o+", Combo::COMBO_A4o_PLUS)]
    #[case("A4+", Combo::COMBO_A4_PLUS)]
    #[case("A3s", Combo::COMBO_A3s)]
    #[case("A3o", Combo::COMBO_A3o)]
    #[case("A3", Combo::COMBO_A3)]
    #[case("A3s+", Combo::COMBO_A3s_PLUS)]
    #[case("A3o+", Combo::COMBO_A3o_PLUS)]
    #[case("A3+", Combo::COMBO_A3_PLUS)]
    #[case("A2s", Combo::COMBO_A2s)]
    #[case("A2o", Combo::COMBO_A2o)]
    #[case("A2", Combo::COMBO_A2)]
    #[case("A2s+", Combo::COMBO_A2s_PLUS)]
    #[case("A2o+", Combo::COMBO_A2o_PLUS)]
    #[case("A2+", Combo::COMBO_A2_PLUS)]
    #[case("KQs", Combo::COMBO_KQs)]
    #[case("KQo", Combo::COMBO_KQo)]
    #[case("KQ", Combo::COMBO_KQ)]
    #[case("KQs+", Combo::COMBO_KQs_PLUS)]
    #[case("KQo+", Combo::COMBO_KQo_PLUS)]
    #[case("KQ+", Combo::COMBO_KQ_PLUS)]
    #[case("KJs", Combo::COMBO_KJs)]
    #[case("KJo", Combo::COMBO_KJo)]
    #[case("KJ", Combo::COMBO_KJ)]
    #[case("KJs+", Combo::COMBO_KJs_PLUS)]
    #[case("KJo+", Combo::COMBO_KJo_PLUS)]
    #[case("KJ+", Combo::COMBO_KJ_PLUS)]
    #[case("KTs", Combo::COMBO_KTs)]
    #[case("KTo", Combo::COMBO_KTo)]
    #[case("KT", Combo::COMBO_KT)]
    #[case("KTs+", Combo::COMBO_KTs_PLUS)]
    #[case("KTo+", Combo::COMBO_KTo_PLUS)]
    #[case("KT+", Combo::COMBO_KT_PLUS)]
    #[case("K9s", Combo::COMBO_K9s)]
    #[case("K9o", Combo::COMBO_K9o)]
    #[case("K9", Combo::COMBO_K9)]
    #[case("K9s+", Combo::COMBO_K9s_PLUS)]
    #[case("K9o+", Combo::COMBO_K9o_PLUS)]
    #[case("K9+", Combo::COMBO_K9_PLUS)]
    #[case("K8s", Combo::COMBO_K8s)]
    #[case("K8o", Combo::COMBO_K8o)]
    #[case("K8", Combo::COMBO_K8)]
    #[case("K8s+", Combo::COMBO_K8s_PLUS)]
    #[case("K8o+", Combo::COMBO_K8o_PLUS)]
    #[case("K8+", Combo::COMBO_K8_PLUS)]
    #[case("K7s", Combo::COMBO_K7s)]
    #[case("K7o", Combo::COMBO_K7o)]
    #[case("K7", Combo::COMBO_K7)]
    #[case("K7s+", Combo::COMBO_K7s_PLUS)]
    #[case("K7o+", Combo::COMBO_K7o_PLUS)]
    #[case("K7+", Combo::COMBO_K7_PLUS)]
    #[case("K6s", Combo::COMBO_K6s)]
    #[case("K6o", Combo::COMBO_K6o)]
    #[case("K6", Combo::COMBO_K6)]
    #[case("K6s+", Combo::COMBO_K6s_PLUS)]
    #[case("K6o+", Combo::COMBO_K6o_PLUS)]
    #[case("K6+", Combo::COMBO_K6_PLUS)]
    #[case("K5s", Combo::COMBO_K5s)]
    #[case("K5o", Combo::COMBO_K5o)]
    #[case("K5", Combo::COMBO_K5)]
    #[case("K5s+", Combo::COMBO_K5s_PLUS)]
    #[case("K5o+", Combo::COMBO_K5o_PLUS)]
    #[case("K5+", Combo::COMBO_K5_PLUS)]
    #[case("K4s", Combo::COMBO_K4s)]
    #[case("K4o", Combo::COMBO_K4o)]
    #[case("K4", Combo::COMBO_K4)]
    #[case("K4s+", Combo::COMBO_K4s_PLUS)]
    #[case("K4o+", Combo::COMBO_K4o_PLUS)]
    #[case("K4+", Combo::COMBO_K4_PLUS)]
    #[case("K3s", Combo::COMBO_K3s)]
    #[case("K3o", Combo::COMBO_K3o)]
    #[case("K3", Combo::COMBO_K3)]
    #[case("K3s+", Combo::COMBO_K3s_PLUS)]
    #[case("K3o+", Combo::COMBO_K3o_PLUS)]
    #[case("K3+", Combo::COMBO_K3_PLUS)]
    #[case("K2s", Combo::COMBO_K2s)]
    #[case("K2o", Combo::COMBO_K2o)]
    #[case("K2", Combo::COMBO_K2)]
    #[case("K2s+", Combo::COMBO_K2s_PLUS)]
    #[case("K2o+", Combo::COMBO_K2o_PLUS)]
    #[case("K2+", Combo::COMBO_K2_PLUS)]
    #[case("QJs", Combo::COMBO_QJs)]
    #[case("QJo", Combo::COMBO_QJo)]
    #[case("QJ", Combo::COMBO_QJ)]
    #[case("QJs+", Combo::COMBO_QJs_PLUS)]
    #[case("QJo+", Combo::COMBO_QJo_PLUS)]
    #[case("QJ+", Combo::COMBO_QJ_PLUS)]
    #[case("QTs", Combo::COMBO_QTs)]
    #[case("QTo", Combo::COMBO_QTo)]
    #[case("QT", Combo::COMBO_QT)]
    #[case("QTs+", Combo::COMBO_QTs_PLUS)]
    #[case("QTo+", Combo::COMBO_QTo_PLUS)]
    #[case("QT+", Combo::COMBO_QT_PLUS)]
    #[case("Q9s", Combo::COMBO_Q9s)]
    #[case("Q9o", Combo::COMBO_Q9o)]
    #[case("Q9", Combo::COMBO_Q9)]
    #[case("Q9s+", Combo::COMBO_Q9s_PLUS)]
    #[case("Q9o+", Combo::COMBO_Q9o_PLUS)]
    #[case("Q9+", Combo::COMBO_Q9_PLUS)]
    #[case("Q8s", Combo::COMBO_Q8s)]
    #[case("Q8o", Combo::COMBO_Q8o)]
    #[case("Q8", Combo::COMBO_Q8)]
    #[case("Q8s+", Combo::COMBO_Q8s_PLUS)]
    #[case("Q8o+", Combo::COMBO_Q8o_PLUS)]
    #[case("Q8+", Combo::COMBO_Q8_PLUS)]
    #[case("Q7s", Combo::COMBO_Q7s)]
    #[case("Q7o", Combo::COMBO_Q7o)]
    #[case("Q7", Combo::COMBO_Q7)]
    #[case("Q7s+", Combo::COMBO_Q7s_PLUS)]
    #[case("Q7o+", Combo::COMBO_Q7o_PLUS)]
    #[case("Q7+", Combo::COMBO_Q7_PLUS)]
    #[case("Q6s", Combo::COMBO_Q6s)]
    #[case("Q6o", Combo::COMBO_Q6o)]
    #[case("Q6", Combo::COMBO_Q6)]
    #[case("Q6s+", Combo::COMBO_Q6s_PLUS)]
    #[case("Q6o+", Combo::COMBO_Q6o_PLUS)]
    #[case("Q6+", Combo::COMBO_Q6_PLUS)]
    #[case("Q5s", Combo::COMBO_Q5s)]
    #[case("Q5o", Combo::COMBO_Q5o)]
    #[case("Q5", Combo::COMBO_Q5)]
    #[case("Q5s+", Combo::COMBO_Q5s_PLUS)]
    #[case("Q5o+", Combo::COMBO_Q5o_PLUS)]
    #[case("Q5+", Combo::COMBO_Q5_PLUS)]
    #[case("Q4s", Combo::COMBO_Q4s)]
    #[case("Q4o", Combo::COMBO_Q4o)]
    #[case("Q4", Combo::COMBO_Q4)]
    #[case("Q4s+", Combo::COMBO_Q4s_PLUS)]
    #[case("Q4o+", Combo::COMBO_Q4o_PLUS)]
    #[case("Q4+", Combo::COMBO_Q4_PLUS)]
    #[case("Q3s", Combo::COMBO_Q3s)]
    #[case("Q3o", Combo::COMBO_Q3o)]
    #[case("Q3", Combo::COMBO_Q3)]
    #[case("Q3s+", Combo::COMBO_Q3s_PLUS)]
    #[case("Q3o+", Combo::COMBO_Q3o_PLUS)]
    #[case("Q3+", Combo::COMBO_Q3_PLUS)]
    #[case("Q2s", Combo::COMBO_Q2s)]
    #[case("Q2o", Combo::COMBO_Q2o)]
    #[case("Q2", Combo::COMBO_Q2)]
    #[case("Q2s+", Combo::COMBO_Q2s_PLUS)]
    #[case("Q2o+", Combo::COMBO_Q2o_PLUS)]
    #[case("Q2+", Combo::COMBO_Q2_PLUS)]
    #[case("JTs", Combo::COMBO_JTs)]
    #[case("JTo", Combo::COMBO_JTo)]
    #[case("JT", Combo::COMBO_JT)]
    #[case("JTs+", Combo::COMBO_JTs_PLUS)]
    #[case("JTo+", Combo::COMBO_JTo_PLUS)]
    #[case("JT+", Combo::COMBO_JT_PLUS)]
    #[case("J9s", Combo::COMBO_J9s)]
    #[case("J9o", Combo::COMBO_J9o)]
    #[case("J9", Combo::COMBO_J9)]
    #[case("J9s+", Combo::COMBO_J9s_PLUS)]
    #[case("J9o+", Combo::COMBO_J9o_PLUS)]
    #[case("J9+", Combo::COMBO_J9_PLUS)]
    #[case("J8s", Combo::COMBO_J8s)]
    #[case("J8o", Combo::COMBO_J8o)]
    #[case("J8", Combo::COMBO_J8)]
    #[case("J8s+", Combo::COMBO_J8s_PLUS)]
    #[case("J8o+", Combo::COMBO_J8o_PLUS)]
    #[case("J8+", Combo::COMBO_J8_PLUS)]
    #[case("J7s", Combo::COMBO_J7s)]
    #[case("J7o", Combo::COMBO_J7o)]
    #[case("J7", Combo::COMBO_J7)]
    #[case("J7s+", Combo::COMBO_J7s_PLUS)]
    #[case("J7o+", Combo::COMBO_J7o_PLUS)]
    #[case("J7+", Combo::COMBO_J7_PLUS)]
    #[case("J6s", Combo::COMBO_J6s)]
    #[case("J6o", Combo::COMBO_J6o)]
    #[case("J6", Combo::COMBO_J6)]
    #[case("J6s+", Combo::COMBO_J6s_PLUS)]
    #[case("J6o+", Combo::COMBO_J6o_PLUS)]
    #[case("J6+", Combo::COMBO_J6_PLUS)]
    #[case("J5s", Combo::COMBO_J5s)]
    #[case("J5o", Combo::COMBO_J5o)]
    #[case("J5", Combo::COMBO_J5)]
    #[case("J5s+", Combo::COMBO_J5s_PLUS)]
    #[case("J5o+", Combo::COMBO_J5o_PLUS)]
    #[case("J5+", Combo::COMBO_J5_PLUS)]
    #[case("J4s", Combo::COMBO_J4s)]
    #[case("J4o", Combo::COMBO_J4o)]
    #[case("J4", Combo::COMBO_J4)]
    #[case("J4s+", Combo::COMBO_J4s_PLUS)]
    #[case("J4o+", Combo::COMBO_J4o_PLUS)]
    #[case("J4+", Combo::COMBO_J4_PLUS)]
    #[case("J3s", Combo::COMBO_J3s)]
    #[case("J3o", Combo::COMBO_J3o)]
    #[case("J3", Combo::COMBO_J3)]
    #[case("J3s+", Combo::COMBO_J3s_PLUS)]
    #[case("J3o+", Combo::COMBO_J3o_PLUS)]
    #[case("J3+", Combo::COMBO_J3_PLUS)]
    #[case("J2s", Combo::COMBO_J2s)]
    #[case("J2o", Combo::COMBO_J2o)]
    #[case("J2", Combo::COMBO_J2)]
    #[case("J2s+", Combo::COMBO_J2s_PLUS)]
    #[case("J2o+", Combo::COMBO_J2o_PLUS)]
    #[case("J2+", Combo::COMBO_J2_PLUS)]
    #[case("T9s", Combo::COMBO_T9s)]
    #[case("T9o", Combo::COMBO_T9o)]
    #[case("T9", Combo::COMBO_T9)]
    #[case("T9s+", Combo::COMBO_T9s_PLUS)]
    #[case("T9o+", Combo::COMBO_T9o_PLUS)]
    #[case("T9+", Combo::COMBO_T9_PLUS)]
    #[case("T8s", Combo::COMBO_T8s)]
    #[case("T8o", Combo::COMBO_T8o)]
    #[case("T8", Combo::COMBO_T8)]
    #[case("T8s+", Combo::COMBO_T8s_PLUS)]
    #[case("T8o+", Combo::COMBO_T8o_PLUS)]
    #[case("T8+", Combo::COMBO_T8_PLUS)]
    #[case("T7s", Combo::COMBO_T7s)]
    #[case("T7o", Combo::COMBO_T7o)]
    #[case("T7", Combo::COMBO_T7)]
    #[case("T7s+", Combo::COMBO_T7s_PLUS)]
    #[case("T7o+", Combo::COMBO_T7o_PLUS)]
    #[case("T7+", Combo::COMBO_T7_PLUS)]
    #[case("T6s", Combo::COMBO_T6s)]
    #[case("T6o", Combo::COMBO_T6o)]
    #[case("T6", Combo::COMBO_T6)]
    #[case("T6s+", Combo::COMBO_T6s_PLUS)]
    #[case("T6o+", Combo::COMBO_T6o_PLUS)]
    fn from_str(#[case] s: &str, #[case] combo: Combo) {
        assert_eq!(Combo::from_str(s), Ok(combo));
    }
    // endregion

    #[test]
    fn sort() {
        let v: Vec<Combo> = vec![
            Combo::COMBO_AA,
            Combo::COMBO_AKs,
            Combo::COMBO_AKo,
            Combo::COMBO_AQo_PLUS,
            Combo::COMBO_QQ_PLUS,
            Combo::COMBO_99_PLUS,
        ];
        let mut shuffled = v.clone();
        shuffled.shuffle(&mut rng());
        shuffled.sort();
        shuffled.reverse();

        assert_eq!(shuffled, v);
    }
}
