#[derive(Clone, Debug, Default, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub enum TableAction {
    #[default]
    Pause,
    ShuffleNewDeck,
    ForcedBets,
    ForcedBetSmallBlind(u8, usize),
    ForcedBetBigBlind(u8, usize),
    BetAnteForced(u8, usize),
    DealCard(u8, String),
}

#[derive(Clone, Debug, Default, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub struct TableLog(Vec<TableAction>);
