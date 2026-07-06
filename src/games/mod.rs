use crate::cards_cell::CardsCell;
use crate::games::betting_structure::BettingStructure;
use crate::games::street::{HOLDEM_STREETS, OMAHA_STREETS, RAZZ_STREETS, STUD_HI_STREETS, StreetDescriptor};

pub mod betting_structure;
pub mod kuhn;
pub mod omaha;
pub mod razz;
pub mod street;
pub mod stud;

/// Game family — the structural shape of a poker variant, orthogonal to
/// betting structure ([`crate::games::betting_structure::BettingStructure`]).
///
/// `Holdem` and `Omaha` use a community board; `StudHi` and `Razz` use
/// per-seat upcards instead. EPIC-30 adds Fixed-Limit Hold'em as
/// `(Holdem, FixedLimit)`; EPIC-31 ties `Omaha` to `PotLimit`; EPIC-32 /
/// EPIC-33 add the stud-family variants.
///
/// # Examples
///
/// ```
/// use pkcore::games::GameFamily;
///
/// assert!(GameFamily::Holdem.uses_community_board());
/// assert!(!GameFamily::StudHi.uses_community_board());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GameFamily {
    #[default]
    Holdem,
    Omaha,
    StudHi,
    Razz,
}

impl GameFamily {
    /// True for community-board variants (Hold'em and Omaha).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::GameFamily;
    ///
    /// assert!(GameFamily::Holdem.uses_community_board());
    /// assert!(GameFamily::Omaha.uses_community_board());
    /// assert!(!GameFamily::StudHi.uses_community_board());
    /// assert!(!GameFamily::Razz.uses_community_board());
    /// ```
    #[must_use]
    pub fn uses_community_board(&self) -> bool {
        matches!(self, GameFamily::Holdem | GameFamily::Omaha)
    }

    /// True for stud-family variants (`StudHi` and `Razz`). Stud-family
    /// games use ante + bring-in instead of blinds and deal per-seat
    /// upcards.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::GameFamily;
    ///
    /// assert!(GameFamily::StudHi.is_stud_family());
    /// assert!(GameFamily::Razz.is_stud_family());
    /// assert!(!GameFamily::Holdem.is_stud_family());
    /// ```
    #[must_use]
    pub fn is_stud_family(&self) -> bool {
        matches!(self, GameFamily::StudHi | GameFamily::Razz)
    }

    /// True for variants where the ace ranks **low** when comparing upcards and
    /// visible hands — Razz (ace-to-five low). Every other family ranks the ace
    /// high.
    ///
    /// This lives here, beside [`Self::is_stud_family`] /
    /// [`Self::uses_community_board`], so the razz-specific ace-low mapping is
    /// not hard-coded into shared table code as a side effect of the bring-in's
    /// scan-direction flag. Decoupling the two keeps a future deuce-to-seven
    /// variant — highest upcard brings in, but the ace is *high* — expressible
    /// (audit P9j.5).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::GameFamily;
    ///
    /// assert!(GameFamily::Razz.ranks_ace_low());
    /// assert!(!GameFamily::StudHi.ranks_ace_low());
    /// assert!(!GameFamily::Holdem.ranks_ace_low());
    /// assert!(!GameFamily::Omaha.ranks_ace_low());
    /// ```
    #[must_use]
    pub fn ranks_ace_low(&self) -> bool {
        matches!(self, GameFamily::Razz)
    }
}

impl std::fmt::Display for GameFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameFamily::Holdem => write!(f, "Hold'em"),
            GameFamily::Omaha => write!(f, "Omaha"),
            GameFamily::StudHi => write!(f, "Seven-Card Stud Hi"),
            GameFamily::Razz => write!(f, "Razz"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, Hash, PartialEq)]
#[non_exhaustive] // 0.2.0: new poker variants can be added without breaking downstream matches.
pub enum GameType {
    #[default]
    NoLimitHoldem,
    LimitHoldem,
    PLO,
    StudHi,
    Razz,
}

impl GameType {
    /// Cards dealt per player across the hand. Hold'em variants get 2;
    /// Omaha 4; Stud-family 7 (built up across streets).
    #[must_use]
    pub fn cards_per_player(&self) -> u8 {
        match self {
            GameType::NoLimitHoldem | GameType::LimitHoldem => 2,
            GameType::PLO => 4,
            GameType::StudHi | GameType::Razz => 7,
        }
    }

    /// Community cards dealt on the board. Hold'em and Omaha share a
    /// 5-card community board (flop/turn/river); stud-family variants
    /// have no community board.
    #[must_use]
    pub fn cards_on_board(&self) -> u8 {
        match self {
            GameType::NoLimitHoldem | GameType::LimitHoldem | GameType::PLO => 5,
            GameType::StudHi | GameType::Razz => 0,
        }
    }

    #[must_use]
    pub fn get_deck(&self) -> CardsCell {
        CardsCell::deck()
    }

    #[must_use]
    pub fn get_deck_size(&self) -> usize {
        52
    }

    /// The game family this `GameType` belongs to. EPIC-29 introduces
    /// [`GameFamily`] as an orthogonal axis from [`BettingStructure`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::{GameFamily, GameType};
    ///
    /// assert_eq!(GameFamily::Holdem, GameType::NoLimitHoldem.family());
    /// assert_eq!(GameFamily::Holdem, GameType::LimitHoldem.family());
    /// assert_eq!(GameFamily::Omaha, GameType::PLO.family());
    /// assert_eq!(GameFamily::StudHi, GameType::StudHi.family());
    /// assert_eq!(GameFamily::Razz, GameType::Razz.family());
    /// ```
    #[must_use]
    pub fn family(&self) -> GameFamily {
        match self {
            GameType::NoLimitHoldem | GameType::LimitHoldem => GameFamily::Holdem,
            GameType::PLO => GameFamily::Omaha,
            GameType::StudHi => GameFamily::StudHi,
            GameType::Razz => GameFamily::Razz,
        }
    }

    /// Default betting structure for this `GameType`. Per-variant
    /// constructors in EPIC-30 / EPIC-32 override the placeholder
    /// fixed-limit sizes with table-supplied values; this accessor only
    /// reports the **shape** of the structure (no-limit / pot-limit /
    /// fixed-limit), not concrete bet amounts.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::GameType;
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// assert_eq!(BettingStructure::NoLimit, GameType::NoLimitHoldem.betting());
    /// assert_eq!(BettingStructure::PotLimit, GameType::PLO.betting());
    /// ```
    #[must_use]
    pub fn betting(&self) -> BettingStructure {
        match self {
            GameType::NoLimitHoldem => BettingStructure::NoLimit,
            GameType::PLO => BettingStructure::PotLimit,
            GameType::LimitHoldem | GameType::StudHi | GameType::Razz => {
                // Placeholder Fixed-Limit shape; per-variant constructors
                // (EPIC-30 / EPIC-32 / EPIC-33) override with actual table
                // bet sizes.
                BettingStructure::FixedLimit {
                    small_bet: 0,
                    big_bet: 0,
                    raise_cap: 3,
                }
            }
        }
    }

    /// The static street-descriptor table for this `GameType`. Hold'em
    /// variants share `HOLDEM_STREETS` (4 streets); Omaha uses
    /// `OMAHA_STREETS` (4 streets, 4 hole cards preflop); Stud and Razz
    /// share the 5-street stud layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::GameType;
    ///
    /// assert_eq!(4, GameType::NoLimitHoldem.streets().len());
    /// assert_eq!(4, GameType::PLO.streets().len());
    /// assert_eq!(5, GameType::StudHi.streets().len());
    /// assert_eq!(5, GameType::Razz.streets().len());
    /// ```
    #[must_use]
    pub fn streets(&self) -> &'static [StreetDescriptor] {
        match self {
            GameType::NoLimitHoldem | GameType::LimitHoldem => HOLDEM_STREETS,
            GameType::PLO => OMAHA_STREETS,
            GameType::StudHi => STUD_HI_STREETS,
            GameType::Razz => RAZZ_STREETS,
        }
    }
}

impl std::fmt::Display for GameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameType::NoLimitHoldem => write!(f, "No Limit Hold'em"),
            GameType::LimitHoldem => write!(f, "Fixed-Limit Hold'em"),
            GameType::PLO => write!(f, "Pot Limit Omaha"),
            GameType::StudHi => write!(f, "Seven-Card Stud Hi"),
            GameType::Razz => write!(f, "Razz"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub enum GamePhase {
    #[default]
    Break,
    NewHand,
    PreFlop,
    ShuffleNewDeck,
    ForcedBets,
    DealHoleCards,
    BettingPreFlop,
    ConsolidatePreFlopBets,
    Flop,
    BurnCardBeforeFlop,
    DealFlop,
    BettingFlop,
    ConsolidateFlopBets,
    Turn,
    BurnCardBeforeTurn,
    DealTurn,
    BettingTurn,
    ConsolidateTurnBets,
    River,
    BurnCardBeforeRiver,
    DealRiver,
    BettingRiver,
    /// EPIC-32: Seven-Card Stud / Razz 3rd street. Each active player has
    /// 2 down + 1 up; bring-in is posted by the lowest (Stud Hi) or
    /// highest (Razz) upcard. Betting at the small tier.
    Stud3rd,
    /// EPIC-32: Stud / Razz 4th street. One more upcard. Small tier.
    Stud4th,
    /// EPIC-32: Stud / Razz 5th street. One more upcard. Betting
    /// transitions to the big tier for the rest of the hand.
    Stud5th,
    /// EPIC-32: Stud / Razz 6th street. One more upcard. Big tier.
    Stud6th,
    /// EPIC-32: Stud / Razz 7th street ("river"). One card dealt face-down.
    /// Big tier.
    Stud7th,
    Showdown,
    PayWinners,
}

impl GamePhase {
    #[must_use]
    pub fn is_preflop(&self) -> bool {
        matches!(
            self,
            GamePhase::NewHand
                | GamePhase::ShuffleNewDeck
                | GamePhase::ForcedBets
                | GamePhase::DealHoleCards
                | GamePhase::BettingPreFlop
                | GamePhase::ConsolidatePreFlopBets
        )
    }

    #[must_use]
    pub fn is_flop(&self) -> bool {
        matches!(
            self,
            GamePhase::BurnCardBeforeFlop
                | GamePhase::DealFlop
                | GamePhase::BettingFlop
                | GamePhase::ConsolidateFlopBets
        )
    }

    #[must_use]
    pub fn is_turn(&self) -> bool {
        matches!(
            self,
            GamePhase::BurnCardBeforeTurn
                | GamePhase::DealTurn
                | GamePhase::BettingTurn
                | GamePhase::ConsolidateTurnBets
        )
    }

    #[must_use]
    pub fn is_river(&self) -> bool {
        matches!(
            self,
            GamePhase::BurnCardBeforeRiver | GamePhase::DealRiver | GamePhase::BettingRiver
        )
    }

    /// EPIC-32: returns the 0-based stud street index (3rd=0 .. 7th=4)
    /// for stud-family phases, `None` otherwise. Used by
    /// `TableNoCell::current_bet_tier` and street-aware dispatch in the
    /// session loop.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::GamePhase;
    ///
    /// assert_eq!(Some(0), GamePhase::Stud3rd.stud_street_index());
    /// assert_eq!(Some(2), GamePhase::Stud5th.stud_street_index());
    /// assert_eq!(Some(4), GamePhase::Stud7th.stud_street_index());
    /// assert_eq!(None, GamePhase::Flop.stud_street_index());
    /// ```
    #[must_use]
    pub fn stud_street_index(&self) -> Option<u8> {
        match self {
            GamePhase::Stud3rd => Some(0),
            GamePhase::Stud4th => Some(1),
            GamePhase::Stud5th => Some(2),
            GamePhase::Stud6th => Some(3),
            GamePhase::Stud7th => Some(4),
            _ => None,
        }
    }

    /// EPIC-32: returns the next Stud street phase given a current one.
    /// Returns `None` when called on `Stud7th` (the hand is complete).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::GamePhase;
    ///
    /// assert_eq!(Some(GamePhase::Stud4th), GamePhase::Stud3rd.next_stud_street());
    /// assert_eq!(Some(GamePhase::Stud7th), GamePhase::Stud6th.next_stud_street());
    /// assert_eq!(None, GamePhase::Stud7th.next_stud_street());
    /// ```
    #[must_use]
    pub fn next_stud_street(&self) -> Option<GamePhase> {
        match self {
            GamePhase::Stud3rd => Some(GamePhase::Stud4th),
            GamePhase::Stud4th => Some(GamePhase::Stud5th),
            GamePhase::Stud5th => Some(GamePhase::Stud6th),
            GamePhase::Stud6th => Some(GamePhase::Stud7th),
            _ => None,
        }
    }

    #[must_use]
    pub fn next(&self) -> GamePhase {
        match self {
            GamePhase::NewHand | GamePhase::PreFlop => GamePhase::ShuffleNewDeck,
            GamePhase::ShuffleNewDeck => GamePhase::ForcedBets,
            GamePhase::ForcedBets => GamePhase::DealHoleCards,
            GamePhase::DealHoleCards => GamePhase::BettingPreFlop,
            GamePhase::BettingPreFlop => GamePhase::ConsolidatePreFlopBets,
            GamePhase::ConsolidatePreFlopBets | GamePhase::Flop => GamePhase::BurnCardBeforeFlop,
            GamePhase::BurnCardBeforeFlop => GamePhase::DealFlop,
            GamePhase::DealFlop => GamePhase::BettingFlop,
            GamePhase::BettingFlop => GamePhase::ConsolidateFlopBets,
            GamePhase::ConsolidateFlopBets | GamePhase::Turn => GamePhase::BurnCardBeforeTurn,
            GamePhase::BurnCardBeforeTurn => GamePhase::DealTurn,
            GamePhase::DealTurn => GamePhase::BettingTurn,
            GamePhase::BettingTurn => GamePhase::ConsolidateTurnBets,
            GamePhase::ConsolidateTurnBets | GamePhase::River => GamePhase::BurnCardBeforeRiver,
            GamePhase::BurnCardBeforeRiver => GamePhase::DealRiver,
            GamePhase::DealRiver => GamePhase::BettingRiver,
            GamePhase::BettingRiver | GamePhase::Stud7th => GamePhase::Showdown,
            GamePhase::Stud3rd => GamePhase::Stud4th,
            GamePhase::Stud4th => GamePhase::Stud5th,
            GamePhase::Stud5th => GamePhase::Stud6th,
            GamePhase::Stud6th => GamePhase::Stud7th,
            GamePhase::Showdown => GamePhase::PayWinners,
            GamePhase::Break | GamePhase::PayWinners => GamePhase::NewHand,
        }
    }
}

impl std::fmt::Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GamePhase::Break => write!(f, "Break"),
            GamePhase::PreFlop => write!(f, "Pre-Flop"),
            GamePhase::NewHand => write!(f, "New Hand"),
            GamePhase::ShuffleNewDeck => write!(f, "Shuffle New Deck"),
            GamePhase::DealHoleCards => write!(f, "Deal Hole Cards"),
            GamePhase::ForcedBets => write!(f, "Forced Bets"),
            GamePhase::BettingPreFlop => write!(f, "Pre-Flop Betting"),
            GamePhase::Flop => write!(f, "Flop"),
            GamePhase::BurnCardBeforeFlop => write!(f, "Burn Card Before Flop"),
            GamePhase::ConsolidatePreFlopBets => write!(f, "Consolidate Pre-Flop Bets"),
            GamePhase::DealFlop => write!(f, "Deal Flop"),
            GamePhase::BettingFlop => write!(f, "Flop Betting"),
            GamePhase::ConsolidateFlopBets => write!(f, "Consolidate Flop Bets"),
            GamePhase::Turn => write!(f, "Turn"),
            GamePhase::BurnCardBeforeTurn => write!(f, "Burn Card Before Turn"),
            GamePhase::DealTurn => write!(f, "Deal Turn"),
            GamePhase::BettingTurn => write!(f, "Turn Betting"),
            GamePhase::ConsolidateTurnBets => write!(f, "Consolidate Turn Bets"),
            GamePhase::River => write!(f, "River"),
            GamePhase::BurnCardBeforeRiver => write!(f, "Burn Card Before River"),
            GamePhase::DealRiver => write!(f, "Deal River"),
            GamePhase::BettingRiver => write!(f, "River Betting"),
            GamePhase::Stud3rd => write!(f, "Stud 3rd Street"),
            GamePhase::Stud4th => write!(f, "Stud 4th Street"),
            GamePhase::Stud5th => write!(f, "Stud 5th Street"),
            GamePhase::Stud6th => write!(f, "Stud 6th Street"),
            GamePhase::Stud7th => write!(f, "Stud 7th Street"),
            GamePhase::Showdown => write!(f, "Award Winners"),
            GamePhase::PayWinners => write!(f, "Pay Winners"),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod games_tests {
    use super::*;

    #[test]
    fn cards_per_player() {
        assert_eq!(2, GameType::NoLimitHoldem.cards_per_player());
        assert_eq!(2, GameType::LimitHoldem.cards_per_player());
        assert_eq!(4, GameType::PLO.cards_per_player());
        assert_eq!(7, GameType::StudHi.cards_per_player());
        assert_eq!(7, GameType::Razz.cards_per_player());
    }

    #[test]
    fn cards_on_board() {
        assert_eq!(5, GameType::NoLimitHoldem.cards_on_board());
        assert_eq!(5, GameType::LimitHoldem.cards_on_board());
        // EPIC-29 Phase 2 bug fix: PLO uses the same 5-card community
        // board as NLHE; previously this method returned 0.
        assert_eq!(5, GameType::PLO.cards_on_board());
        assert_eq!(0, GameType::StudHi.cards_on_board());
        assert_eq!(0, GameType::Razz.cards_on_board());
    }

    #[test]
    fn get_deck() {
        assert_eq!(CardsCell::deck(), GameType::NoLimitHoldem.get_deck());
        assert_eq!(CardsCell::deck(), GameType::LimitHoldem.get_deck());
        assert_eq!(CardsCell::deck(), GameType::PLO.get_deck());
        assert_eq!(CardsCell::deck(), GameType::StudHi.get_deck());
        assert_eq!(CardsCell::deck(), GameType::Razz.get_deck());
    }

    #[test]
    fn game_type_family_accessor() {
        assert_eq!(GameFamily::Holdem, GameType::NoLimitHoldem.family());
        assert_eq!(GameFamily::Holdem, GameType::LimitHoldem.family());
        assert_eq!(GameFamily::Omaha, GameType::PLO.family());
        assert_eq!(GameFamily::StudHi, GameType::StudHi.family());
        assert_eq!(GameFamily::Razz, GameType::Razz.family());
    }

    #[test]
    fn game_type_betting_accessor() {
        assert_eq!(BettingStructure::NoLimit, GameType::NoLimitHoldem.betting());
        assert_eq!(BettingStructure::PotLimit, GameType::PLO.betting());
        // Limit-style variants report a Fixed-Limit shape with placeholder
        // bet sizes; per-variant constructors override at table creation.
        assert!(matches!(
            GameType::LimitHoldem.betting(),
            BettingStructure::FixedLimit { .. }
        ));
        assert!(matches!(
            GameType::StudHi.betting(),
            BettingStructure::FixedLimit { .. }
        ));
        assert!(matches!(GameType::Razz.betting(), BettingStructure::FixedLimit { .. }));
    }

    #[test]
    fn game_type_display_for_new_variants() {
        assert_eq!("Fixed-Limit Hold'em", GameType::LimitHoldem.to_string());
        assert_eq!("Seven-Card Stud Hi", GameType::StudHi.to_string());
    }

    #[test]
    fn game_type_streets_accessor() {
        assert_eq!(4, GameType::NoLimitHoldem.streets().len());
        assert_eq!(4, GameType::LimitHoldem.streets().len());
        assert_eq!(4, GameType::PLO.streets().len());
        assert_eq!(5, GameType::StudHi.streets().len());
        assert_eq!(5, GameType::Razz.streets().len());
    }

    #[test]
    fn game_type_streets_hole_card_totals_match_cards_per_player() {
        for gt in [
            GameType::NoLimitHoldem,
            GameType::LimitHoldem,
            GameType::PLO,
            GameType::StudHi,
            GameType::Razz,
        ] {
            let total: u8 = gt.streets().iter().map(|s| s.hole_dealt).sum();
            assert_eq!(gt.cards_per_player(), total, "{gt} hole card total mismatch");
        }
    }

    #[test]
    fn game_type_streets_community_card_totals_match_cards_on_board() {
        for gt in [
            GameType::NoLimitHoldem,
            GameType::LimitHoldem,
            GameType::PLO,
            GameType::StudHi,
            GameType::Razz,
        ] {
            let total: u8 = gt.streets().iter().map(|s| s.community_dealt).sum();
            assert_eq!(gt.cards_on_board(), total, "{gt} community card total mismatch");
        }
    }

    // ---- GameFamily (EPIC-29 Phase 1) ----

    #[test]
    fn game_family_default_is_holdem() {
        assert_eq!(GameFamily::Holdem, GameFamily::default());
    }

    #[test]
    fn game_family_uses_community_board() {
        assert!(GameFamily::Holdem.uses_community_board());
        assert!(GameFamily::Omaha.uses_community_board());
        assert!(!GameFamily::StudHi.uses_community_board());
        assert!(!GameFamily::Razz.uses_community_board());
    }

    #[test]
    fn game_family_is_stud_family() {
        assert!(!GameFamily::Holdem.is_stud_family());
        assert!(!GameFamily::Omaha.is_stud_family());
        assert!(GameFamily::StudHi.is_stud_family());
        assert!(GameFamily::Razz.is_stud_family());
    }

    #[test]
    fn game_family_display() {
        assert_eq!("Hold'em", GameFamily::Holdem.to_string());
        assert_eq!("Omaha", GameFamily::Omaha.to_string());
        assert_eq!("Seven-Card Stud Hi", GameFamily::StudHi.to_string());
        assert_eq!("Razz", GameFamily::Razz.to_string());
    }
}
