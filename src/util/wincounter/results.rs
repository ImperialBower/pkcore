use crate::util::wincounter::wins::Wins;

/// # PHASE 2.2/Step 4: Results
///
/// Results is a utility state class designed to make it as easy as possible to get and display
/// winning and tie percentages for any game.
pub struct Results(Vec<(usize, usize)>);

impl Results {}

impl From<Wins> for Results {
    fn from(_: Wins) -> Self {
        todo!()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__wincounter__results__tests {
    use super::*;

    #[test]
    fn from__wins() {}
}
