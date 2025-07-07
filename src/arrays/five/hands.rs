use crate::arrays::five::Five;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Hands(Vec<Five>);

impl Hands {
    pub fn get(&self, index: usize) -> Option<&Five> {
        self.0.get(index)
    }
}

impl From<Vec<Five>> for Hands {
    fn from(hands: Vec<Five>) -> Self {
        Hands(hands)
    }
}

impl IntoIterator for Hands {
    type Item = Five;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__five__hands_tests {
    use crate::card::Card;
    use super::*;

    const ROYAL_FLUSH: [Card; 5] = [
        Card::ACE_DIAMONDS,
        Card::KING_DIAMONDS,
        Card::QUEEN_DIAMONDS,
        Card::JACK_DIAMONDS,
        Card::TEN_DIAMONDS,
    ];

    const WHEEL: [Card; 5] = [
        Card::ACE_CLUBS,
        Card::DEUCE_DIAMONDS,
        Card::TREY_DIAMONDS,
        Card::FOUR_HEARTS,
        Card::FIVE_SPADES,
    ];

    #[test]
    fn get() {
        let hands: Vec<Five> = vec![Five::from(ROYAL_FLUSH), Five::from(WHEEL)];

        assert_eq!(&Five::from(WHEEL), hands.get(1).unwrap());
    }
}