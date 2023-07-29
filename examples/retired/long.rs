use indexmap::IndexSet;
use pkcore::arrays::three::Three;
use pkcore::arrays::two::Two;
use pkcore::card::Card;
use pkcore::PKError;
use std::str::FromStr;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CardVec(Vec<Card>);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CardISet(IndexSet<Card>);

fn main() -> Result<(), PKError> {
    let _now = std::time::Instant::now();
    env_logger::init();

    let daniel = Two::HAND_6S_6H;
    let gus = Two::HAND_5D_5C;
    let _hands = vec![daniel, gus];

    let _the_flop = Three::from_str("9♣ 6♦ 5♥")?;

    Ok(())
}
