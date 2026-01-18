use crate::cards_cell::CardsCell;

pub mod omaha;
pub mod razz;
pub mod stud;

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub enum GameType {
    #[default]
    NoLimitHoldem,
    PLO,
    Razz,
}

impl GameType {
    #[must_use]
    pub fn cards_per_player(&self) -> u8 {
        match self {
            GameType::NoLimitHoldem => 2,
            GameType::PLO => 4,
            GameType::Razz => 7,
        }
    }

    #[must_use]
    pub fn cards_on_board(&self) -> u8 {
        match self {
            GameType::NoLimitHoldem => 5,
            _ => 0,
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
}

impl std::fmt::Display for GameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameType::NoLimitHoldem => write!(f, "No Limit Hold'em"),
            GameType::PLO => write!(f, "Pot Limit Omaha"),
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
            GamePhase::BettingRiver => GamePhase::Showdown,
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
        assert_eq!(4, GameType::PLO.cards_per_player());
        assert_eq!(7, GameType::Razz.cards_per_player());
    }

    #[test]
    fn cards_on_board() {
        assert_eq!(5, GameType::NoLimitHoldem.cards_on_board());
        assert_eq!(0, GameType::PLO.cards_on_board());
        assert_eq!(0, GameType::Razz.cards_on_board());
    }

    #[test]
    fn get_deck() {
        assert_eq!(CardsCell::deck(), GameType::NoLimitHoldem.get_deck());
        assert_eq!(CardsCell::deck(), GameType::PLO.get_deck());
        assert_eq!(CardsCell::deck(), GameType::Razz.get_deck());
    }
}
