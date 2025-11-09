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
    ShuffleNewDeck,
    ForcedBets,
    BettingPreFlop,
    BurnCardBeforeFlop,
    DealHoleCards,
    ConsolidatePreFlopBets,
    DealFlop,
    BettingFlop,
    ConsolidateFlopBets,
    BurnCardBeforeTurn,
    DealTurn,
    BettingTurn,
    ConsolidateTurnBets,
    BurnCardBeforeRiver,
    DealRiver,
    BettingRiver,
    AwardWinners,
}

impl GamePhase {
    #[must_use]
    pub fn next(&self) -> GamePhase {
        match self {
            GamePhase::NewHand => GamePhase::ShuffleNewDeck,
            GamePhase::ShuffleNewDeck => GamePhase::ForcedBets,
            GamePhase::ForcedBets => GamePhase::DealHoleCards,
            GamePhase::DealHoleCards => GamePhase::BettingPreFlop,
            GamePhase::BettingPreFlop => GamePhase::ConsolidatePreFlopBets,
            GamePhase::ConsolidatePreFlopBets => GamePhase::BurnCardBeforeFlop,
            GamePhase::BurnCardBeforeFlop => GamePhase::DealFlop,
            GamePhase::DealFlop => GamePhase::BettingFlop,
            GamePhase::BettingFlop => GamePhase::ConsolidateFlopBets,
            GamePhase::ConsolidateFlopBets => GamePhase::BurnCardBeforeTurn,
            GamePhase::BurnCardBeforeTurn => GamePhase::DealTurn,
            GamePhase::DealTurn => GamePhase::BettingTurn,
            GamePhase::BettingTurn => GamePhase::ConsolidateTurnBets,
            GamePhase::ConsolidateTurnBets => GamePhase::BurnCardBeforeRiver,
            GamePhase::BurnCardBeforeRiver => GamePhase::DealRiver,
            GamePhase::DealRiver => GamePhase::BettingRiver,
            GamePhase::BettingRiver => GamePhase::AwardWinners,
            GamePhase::Break | GamePhase::AwardWinners => GamePhase::NewHand,
        }
    }
}

impl std::fmt::Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GamePhase::Break => write!(f, "Break"),
            GamePhase::NewHand => write!(f, "New Hand"),
            GamePhase::ShuffleNewDeck => write!(f, "Shuffle New Deck"),
            GamePhase::ForcedBets => write!(f, "Forced Bets"),
            GamePhase::BettingPreFlop => write!(f, "Pre-Flop Betting"),
            GamePhase::BurnCardBeforeFlop => write!(f, "Burn Card Before Flop"),
            GamePhase::DealHoleCards => write!(f, "Deal Hole Cards"),
            GamePhase::ConsolidatePreFlopBets => write!(f, "Consolidate Pre-Flop Bets"),
            GamePhase::DealFlop => write!(f, "Deal Flop"),
            GamePhase::BettingFlop => write!(f, "Flop Betting"),
            GamePhase::ConsolidateFlopBets => write!(f, "Consolidate Flop Bets"),
            GamePhase::BurnCardBeforeTurn => write!(f, "Burn Card Before Turn"),
            GamePhase::DealTurn => write!(f, "Deal Turn"),
            GamePhase::BettingTurn => write!(f, "Turn Betting"),
            GamePhase::ConsolidateTurnBets => write!(f, "Consolidate Turn Bets"),
            GamePhase::BurnCardBeforeRiver => write!(f, "Burn Card Before River"),
            GamePhase::DealRiver => write!(f, "Deal River"),
            GamePhase::BettingRiver => write!(f, "River Betting"),
            GamePhase::AwardWinners => write!(f, "Award Winners"),
        }
    }
}
