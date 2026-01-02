use crate::analysis::eval::Eval;
use crate::arrays::five::Five;

/// # The Nuts
///
/// Originally, the code to display values like the nuts was in the `Game` struct. The problem was
/// that in reality, it's just a specific type of `Evals` collection.
///
/// One of the things that I have discovered working through this logic the second time
/// is that there are two perspectives on "the nuts":
///
/// The *at the time* flop perspective, which only deals with the three community cards on the
/// board plus any two hole cards that a player might hand. I'm going to call this the
/// *now* perspective, as in _the nuts, as of now._
///
/// The *what might be* river perspective, where you can into account not just any two
/// cars that a player might have, as well as the cards that might come down at the turn
/// and river. This perspective has a lot more possibilities. I'm going to call this the
/// *future* perspective.
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Evals(Vec<Eval>);

impl Evals {
    #[must_use]
    pub fn get(&self, i: usize) -> Option<&Eval> {
        self.0.get(i)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn sort(&self) -> Evals {
        let mut v = self.to_vec();
        v.sort();
        v.reverse();
        Evals(v)
    }

    pub fn sort_in_place(&mut self) {
        self.0.sort();
        self.0.reverse();
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<Eval> {
        self.0.clone()
    }
}

impl std::fmt::Display for Evals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let evals = self.sort();
        let mut v = Vec::new();

        for (i, eval) in evals.0.iter().enumerate() {
            v.push(format!("  #{}: {eval}", i + 1));
        }

        write!(f, "{}", v.join("\n"))
    }
}

impl From<Vec<Eval>> for Evals {
    fn from(v: Vec<Eval>) -> Self {
        Evals(v)
    }
}

impl From<Vec<Five>> for Evals {
    fn from(v: Vec<Five>) -> Self {
        Evals(v.iter().map(Eval::from).collect())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_rank__evals_tests {
    use super::*;
    use crate::Card;
    use crate::analysis::class::HandRankClass;
    use crate::arrays::three::Three;
    use crate::arrays::two::Two;
    use crate::util::data::TestData;

    #[test]
    fn sort() {
        let the_nuts = Evals::from(TestData::fives_the_fold());

        let sorted = the_nuts.sort();

        assert_eq!(HandRankClass::ThreeNines, sorted.0.get(0).unwrap().hand_rank.class);
        assert_eq!(HandRankClass::ThreeFives, sorted.0.get(1).unwrap().hand_rank.class);
        assert_eq!(HandRankClass::PairOfTens, sorted.0.get(2).unwrap().hand_rank.class);
    }

    #[test]
    fn to_vec() {
        let daniel = TestData::daniel_eval_at_flop();
        let gus = TestData::gus_eval_at_flop();
        let v = vec![daniel, gus];
        let the_nuts = Evals::from(v.clone());

        assert_eq!(v, the_nuts.to_vec());
    }

    #[test]
    fn from__eval() {
        let daniel = TestData::daniel_eval_at_flop();
        let gus = TestData::gus_eval_at_flop();
        let v = vec![daniel, gus];

        let the_nuts = Evals::from(v.clone());

        assert_eq!(v, the_nuts.0.to_vec());
    }

    #[test]
    fn from__five() {
        let the_flop = Three::from([Card::FIVE_CLUBS, Card::NINE_DIAMONDS, Card::TEN_HEARTS]);
        let antonius = Eval::from(Five::from_2and3(Two::HAND_5S_5D, the_flop));
        let phil = Eval::from(Five::from_2and3(Two::HAND_KC_TD, the_flop));
        let daniel = Eval::from(Five::from_2and3(Two::HAND_9S_9H, the_flop));

        let the_nuts = Evals::from(TestData::fives_the_fold());

        assert_eq!(antonius, *the_nuts.to_vec().get(0).unwrap());
        assert_eq!(phil, *the_nuts.to_vec().get(1).unwrap());
        assert_eq!(daniel, *the_nuts.to_vec().get(2).unwrap());
    }
}
