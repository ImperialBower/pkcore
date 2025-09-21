pub mod omaha;
pub mod razz;
pub mod stud;

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub enum GameType {
    NoLimitHoldem,
    PLO,
    Razz,
}
