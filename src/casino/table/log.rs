#[derive(Clone, Debug, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub enum TableAction {
    ForcedBetSmallBlind(u8, usize),
    ForcedBetBigBlind(u8, usize),
    BetAnteForced(u8, usize),
    ShuffleNewDeck,
    DealCard(u8, String),
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub struct TableLog(Vec<TableAction>);
