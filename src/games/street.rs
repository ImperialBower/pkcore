//! Per-family street-descriptor tables.
//!
//! Today's engine hard-codes preflop/flop/turn/river streets inside
//! [`crate::games::GamePhase`] and walks them with direct phase assignments
//! in [`crate::casino::table_no_cell::TableNoCell`] (no use of
//! `GamePhase::next`). This module introduces a **data-driven** description
//! of each variant's street layout so future engine work (and the
//! per-variant epics EPIC-30 through EPIC-33) can iterate streets without
//! adding more hardcoded match arms.
//!
//! Phase 3 of EPIC-29 ships only the data definitions; no engine code yet
//! consults [`StreetDescriptor`]. Subsequent phases (and later epics) wire
//! the descriptor table into the dealing loop, betting-tier dispatch, and
//! action-order helpers.
//!
//! [`BetTier`] is defined in [`crate::games::betting_structure`] and
//! re-exported here so the per-family `STREETS` slices can live alongside
//! the rest of the street-descriptor surface.

pub use crate::games::betting_structure::BetTier;

/// Index of a street within a variant's street table, starting at 0.
///
/// - Hold'em / Omaha: 0=preflop, 1=flop, 2=turn, 3=river.
/// - Stud-family: 0=3rd, 1=4th, 2=5th, 3=6th, 4=7th.
///
/// # Examples
///
/// ```
/// use pkcore::games::street::StreetIndex;
///
/// assert_eq!(StreetIndex(0).0, 0);
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StreetIndex(pub u8);

/// A static descriptor for one street of one variant.
///
/// Carries everything the engine needs to deal cards on the street and
/// pick the right betting tier — without per-variant `match` arms in the
/// hand-loop machinery.
///
/// Fields:
/// - `index` — which street this is within the variant.
/// - `name` — short human label ("preflop", "flop", "3rd", "4th", …).
/// - `community_dealt` — community cards dealt on this street (0/3/1/1 for
///   Hold'em; 0 for stud-family).
/// - `hole_dealt` — total hole cards dealt to each player on this street.
/// - `hole_dealt_up` — how many of `hole_dealt` are dealt face-up
///   ([`crate::play::visibility::Visibility::Up`]). NLHE/Omaha always 0;
///   stud-family is mostly 1 per street with exceptions on 3rd and 7th.
/// - `burn_first` — whether the dealer burns a card before dealing the
///   community card(s) (Hold'em/Omaha flop/turn/river).
/// - `bet_tier` — [`BetTier::Small`] for early streets,
///   [`BetTier::Big`] for later streets. Used by Fixed-Limit variants;
///   no-limit / pot-limit games ignore it.
#[derive(Clone, Copy, Debug)]
pub struct StreetDescriptor {
    pub index: StreetIndex,
    pub name: &'static str,
    pub community_dealt: u8,
    pub hole_dealt: u8,
    pub hole_dealt_up: u8,
    pub burn_first: bool,
    pub bet_tier: BetTier,
}

/// 4 streets matching NLHE/FLHE: preflop, flop, turn, river.
pub const HOLDEM_STREETS: &[StreetDescriptor] = &[
    StreetDescriptor {
        index: StreetIndex(0),
        name: "preflop",
        community_dealt: 0,
        hole_dealt: 2,
        hole_dealt_up: 0,
        burn_first: false,
        bet_tier: BetTier::Small,
    },
    StreetDescriptor {
        index: StreetIndex(1),
        name: "flop",
        community_dealt: 3,
        hole_dealt: 0,
        hole_dealt_up: 0,
        burn_first: true,
        bet_tier: BetTier::Small,
    },
    StreetDescriptor {
        index: StreetIndex(2),
        name: "turn",
        community_dealt: 1,
        hole_dealt: 0,
        hole_dealt_up: 0,
        burn_first: true,
        bet_tier: BetTier::Big,
    },
    StreetDescriptor {
        index: StreetIndex(3),
        name: "river",
        community_dealt: 1,
        hole_dealt: 0,
        hole_dealt_up: 0,
        burn_first: true,
        bet_tier: BetTier::Big,
    },
];

/// 4 streets matching Pot-Limit Omaha. Same street structure as Hold'em,
/// but with 4 hole cards dealt on preflop instead of 2.
pub const OMAHA_STREETS: &[StreetDescriptor] = &[
    StreetDescriptor {
        index: StreetIndex(0),
        name: "preflop",
        community_dealt: 0,
        hole_dealt: 4,
        hole_dealt_up: 0,
        burn_first: false,
        bet_tier: BetTier::Small,
    },
    StreetDescriptor {
        index: StreetIndex(1),
        name: "flop",
        community_dealt: 3,
        hole_dealt: 0,
        hole_dealt_up: 0,
        burn_first: true,
        bet_tier: BetTier::Small,
    },
    StreetDescriptor {
        index: StreetIndex(2),
        name: "turn",
        community_dealt: 1,
        hole_dealt: 0,
        hole_dealt_up: 0,
        burn_first: true,
        bet_tier: BetTier::Big,
    },
    StreetDescriptor {
        index: StreetIndex(3),
        name: "river",
        community_dealt: 1,
        hole_dealt: 0,
        hole_dealt_up: 0,
        burn_first: true,
        bet_tier: BetTier::Big,
    },
];

/// 5 streets matching Seven-Card Stud Hi: 3rd, 4th, 5th, 6th, 7th.
///
/// - 3rd street: 2 down + 1 up; betting at small tier.
/// - 4th street: 1 up; small tier.
/// - 5th street: 1 up; tier transitions to **Big** here.
/// - 6th street: 1 up; big tier.
/// - 7th street: 1 down (the "river" card, dealt face-down); big tier.
///
/// No community cards on any street. The bring-in mechanic and
/// best-visible-hand action ordering ride elsewhere (EPIC-32).
pub const STUD_HI_STREETS: &[StreetDescriptor] = &[
    StreetDescriptor {
        index: StreetIndex(0),
        name: "3rd",
        community_dealt: 0,
        hole_dealt: 3,
        hole_dealt_up: 1,
        burn_first: false,
        bet_tier: BetTier::Small,
    },
    StreetDescriptor {
        index: StreetIndex(1),
        name: "4th",
        community_dealt: 0,
        hole_dealt: 1,
        hole_dealt_up: 1,
        burn_first: false,
        bet_tier: BetTier::Small,
    },
    StreetDescriptor {
        index: StreetIndex(2),
        name: "5th",
        community_dealt: 0,
        hole_dealt: 1,
        hole_dealt_up: 1,
        burn_first: false,
        bet_tier: BetTier::Big,
    },
    StreetDescriptor {
        index: StreetIndex(3),
        name: "6th",
        community_dealt: 0,
        hole_dealt: 1,
        hole_dealt_up: 1,
        burn_first: false,
        bet_tier: BetTier::Big,
    },
    StreetDescriptor {
        index: StreetIndex(4),
        name: "7th",
        community_dealt: 0,
        hole_dealt: 1,
        hole_dealt_up: 0,
        burn_first: false,
        bet_tier: BetTier::Big,
    },
];

/// 5 streets matching Razz. Structurally identical to `STUD_HI_STREETS`:
/// the differences (highest-upcard brings in; worst visible hand acts
/// first; A-5 lowball evaluator at showdown) ride elsewhere (EPIC-33).
pub const RAZZ_STREETS: &[StreetDescriptor] = STUD_HI_STREETS;

#[cfg(test)]
#[allow(non_snake_case)]
mod games__street__tests {
    use super::*;

    #[test]
    fn holdem_streets_count() {
        assert_eq!(4, HOLDEM_STREETS.len());
    }

    #[test]
    fn omaha_streets_count() {
        assert_eq!(4, OMAHA_STREETS.len());
    }

    #[test]
    fn stud_hi_streets_count() {
        assert_eq!(5, STUD_HI_STREETS.len());
    }

    #[test]
    fn razz_mirrors_stud_hi() {
        assert_eq!(RAZZ_STREETS.len(), STUD_HI_STREETS.len());
        for (a, b) in RAZZ_STREETS.iter().zip(STUD_HI_STREETS.iter()) {
            assert_eq!(a.community_dealt, b.community_dealt);
            assert_eq!(a.hole_dealt, b.hole_dealt);
            assert_eq!(a.hole_dealt_up, b.hole_dealt_up);
        }
    }

    #[test]
    fn holdem_total_community_cards_is_5() {
        let total: u8 = HOLDEM_STREETS.iter().map(|s| s.community_dealt).sum();
        assert_eq!(5, total);
    }

    #[test]
    fn holdem_total_hole_cards_is_2() {
        let total: u8 = HOLDEM_STREETS.iter().map(|s| s.hole_dealt).sum();
        assert_eq!(2, total);
    }

    #[test]
    fn omaha_total_hole_cards_is_4() {
        let total: u8 = OMAHA_STREETS.iter().map(|s| s.hole_dealt).sum();
        assert_eq!(4, total);
    }

    #[test]
    fn stud_hi_total_hole_cards_is_7() {
        let total: u8 = STUD_HI_STREETS.iter().map(|s| s.hole_dealt).sum();
        assert_eq!(7, total);
    }

    #[test]
    fn stud_hi_upcards_are_4() {
        // 3rd street has 1 up; 4th/5th/6th have 1 each; 7th has 0 (down).
        let total_up: u8 = STUD_HI_STREETS.iter().map(|s| s.hole_dealt_up).sum();
        assert_eq!(4, total_up);
    }

    #[test]
    fn holdem_burns_after_preflop() {
        assert!(!HOLDEM_STREETS[0].burn_first);
        assert!(HOLDEM_STREETS[1].burn_first);
        assert!(HOLDEM_STREETS[2].burn_first);
        assert!(HOLDEM_STREETS[3].burn_first);
    }

    #[test]
    fn stud_hi_never_burns() {
        for street in STUD_HI_STREETS {
            assert!(!street.burn_first);
        }
    }

    #[test]
    fn holdem_tier_transitions_at_turn() {
        assert!(matches!(HOLDEM_STREETS[0].bet_tier, BetTier::Small));
        assert!(matches!(HOLDEM_STREETS[1].bet_tier, BetTier::Small));
        assert!(matches!(HOLDEM_STREETS[2].bet_tier, BetTier::Big));
        assert!(matches!(HOLDEM_STREETS[3].bet_tier, BetTier::Big));
    }

    #[test]
    fn stud_hi_tier_transitions_at_fifth() {
        // 3rd and 4th are Small; 5th onward is Big.
        assert!(matches!(STUD_HI_STREETS[0].bet_tier, BetTier::Small));
        assert!(matches!(STUD_HI_STREETS[1].bet_tier, BetTier::Small));
        assert!(matches!(STUD_HI_STREETS[2].bet_tier, BetTier::Big));
        assert!(matches!(STUD_HI_STREETS[3].bet_tier, BetTier::Big));
        assert!(matches!(STUD_HI_STREETS[4].bet_tier, BetTier::Big));
    }

    #[test]
    fn street_index_default_is_zero() {
        assert_eq!(0, StreetIndex::default().0);
    }
}
