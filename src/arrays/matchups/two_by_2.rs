use crate::arrays::two::Two;
use crate::PKError;
use crate::Pile;

#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
pub struct TwoBy2 {
    pub first: Two,
    pub second: Two,
}

impl TwoBy2 {
    pub fn new(first: Two, second: Two) -> Result<TwoBy2, PKError> {
        if first.is_dealt() || second.is_dealt() {
            Err(PKError::NotDealt)
        } else {
            Ok(TwoBy2 { first, second })
        }
    }
}
