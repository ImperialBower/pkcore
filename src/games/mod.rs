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
    NewHand,
    ShuffleNewDeck,
    ForcedBets,
    BurnCardBeforeFlop,
    DealHoleCards,
    PreFlopBetting,
    ConsolidatePreFlopBets,
    DealFlop,
    FlopBetting,
    ConsolidateFlopBets,
    BurnCardBeforeTurn,
    DealTurn,
    TurnBetting,
    ConsolidateTurnBets,
    BurnCardBeforeRiver,
    DealRiver,
    RiverBetting,
    AwardWinners,
}

impl GamePhase {
    #[must_use]
    pub fn next(&self) -> GamePhase {
        match self {
            GamePhase::NewHand => GamePhase::ShuffleNewDeck,
            GamePhase::ShuffleNewDeck => GamePhase::ForcedBets,
            GamePhase::ForcedBets => GamePhase::BurnCardBeforeFlop,
            GamePhase::BurnCardBeforeFlop => GamePhase::DealHoleCards,
            GamePhase::DealHoleCards => GamePhase::PreFlopBetting,
            GamePhase::PreFlopBetting => GamePhase::ConsolidatePreFlopBets,
            GamePhase::ConsolidatePreFlopBets => GamePhase::DealFlop,
            GamePhase::DealFlop => GamePhase::FlopBetting,
            GamePhase::FlopBetting => GamePhase::ConsolidateFlopBets,
            GamePhase::ConsolidateFlopBets => GamePhase::BurnCardBeforeTurn,
            GamePhase::BurnCardBeforeTurn => GamePhase::DealTurn,
            GamePhase::DealTurn => GamePhase::TurnBetting,
            GamePhase::TurnBetting => GamePhase::ConsolidateTurnBets,
            GamePhase::ConsolidateTurnBets => GamePhase::BurnCardBeforeRiver,
            GamePhase::BurnCardBeforeRiver => GamePhase::DealRiver,
            GamePhase::DealRiver => GamePhase::RiverBetting,
            GamePhase::RiverBetting => GamePhase::AwardWinners,
            GamePhase::AwardWinners => GamePhase::NewHand,
        }
    }
}

impl std::fmt::Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GamePhase::NewHand => write!(f, "New Hand"),
            GamePhase::ShuffleNewDeck => write!(f, "Shuffle New Deck"),
            GamePhase::ForcedBets => write!(f, "Forced Bets"),
            GamePhase::BurnCardBeforeFlop => write!(f, "Burn Card Before Flop"),
            GamePhase::DealHoleCards => write!(f, "Deal Hole Cards"),
            GamePhase::PreFlopBetting => write!(f, "Pre-Flop Betting"),
            GamePhase::ConsolidatePreFlopBets => write!(f, "Consolidate Pre-Flop Bets"),
            GamePhase::DealFlop => write!(f, "Deal Flop"),
            GamePhase::FlopBetting => write!(f, "Flop Betting"),
            GamePhase::ConsolidateFlopBets => write!(f, "Consolidate Flop Bets"),
            GamePhase::BurnCardBeforeTurn => write!(f, "Burn Card Before Turn"),
            GamePhase::DealTurn => write!(f, "Deal Turn"),
            GamePhase::TurnBetting => write!(f, "Turn Betting"),
            GamePhase::ConsolidateTurnBets => write!(f, "Consolidate Turn Bets"),
            GamePhase::BurnCardBeforeRiver => write!(f, "Burn Card Before River"),
            GamePhase::DealRiver => write!(f, "Deal River"),
            GamePhase::RiverBetting => write!(f, "River Betting"),
            GamePhase::AwardWinners => write!(f, "Award Winners"),
        }
    }
}
