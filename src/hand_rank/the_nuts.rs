use crate::arrays::five::Five;
use crate::hand_rank::eval::Eval;

/// The immediate need for this class is so that we can have an easy way to hold and sort the
/// hands possible at a particular point in a game, usually the flop. I'm thinking that we can
/// return this object as a part of our Pile trait, so that if we want to get all the possible
/// hands at the flop or turn, we can just call that method.
///
/// See `CaseEval` for the etymology being the phrase the nuts.
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct TheNuts(Vec<Eval>);

impl TheNuts {
    #[must_use]
    pub fn get(&self, i: usize) -> Option<&Eval> {
        self.0.get(i)
    }
    #[must_use]
    pub fn sort(&self) -> TheNuts {
        let mut v = self.to_vec();
        v.sort();
        v.reverse();
        TheNuts(v)
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<Eval> {
        self.0.clone()
    }
}

impl From<Vec<Eval>> for TheNuts {
    fn from(v: Vec<Eval>) -> Self {
        TheNuts(v)
    }
}

impl From<Vec<Five>> for TheNuts {
    fn from(v: Vec<Five>) -> Self {
        TheNuts(v.iter().map(Eval::from).collect())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_rank__the_nuts_tests {
    use super::*;
    use crate::arrays::three::Three;
    use crate::arrays::two::Two;
    use crate::hand_rank::class::Class;
    use crate::util::data::TestData;
    use crate::Card;

    #[test]
    fn sort() {
        let the_nuts = TheNuts::from(TestData::fives_the_fold());

        let sorted = the_nuts.sort();

        assert_eq!(Class::ThreeNines, sorted.0.get(0).unwrap().hand_rank.class);
        assert_eq!(Class::ThreeFives, sorted.0.get(1).unwrap().hand_rank.class);
        assert_eq!(Class::PairOfTens, sorted.0.get(2).unwrap().hand_rank.class);
    }

    #[test]
    fn to_vec() {
        let daniel = TestData::daniel_eval_at_flop();
        let gus = TestData::gus_eval_at_flop();
        let v = vec![daniel, gus];
        let the_nuts = TheNuts::from(v.clone());

        assert_eq!(v, the_nuts.to_vec());
    }

    #[test]
    fn from__eval() {
        let daniel = TestData::daniel_eval_at_flop();
        let gus = TestData::gus_eval_at_flop();
        let v = vec![daniel, gus];

        let the_nuts = TheNuts::from(v.clone());

        assert_eq!(v, the_nuts.0.to_vec());
    }

    #[test]
    fn from__five() {
        let the_flop = Three::from([Card::FIVE_CLUBS, Card::NINE_DIAMONDS, Card::TEN_HEARTS]);
        let antonius = Eval::from(Five::from_2and3(Two::HAND_5S_5D, the_flop));
        let phil = Eval::from(Five::from_2and3(Two::HAND_KC_TD, the_flop));
        let daniel = Eval::from(Five::from_2and3(Two::HAND_9S_9H, the_flop));

        let the_nuts = TheNuts::from(TestData::fives_the_fold());

        assert_eq!(antonius, *the_nuts.to_vec().get(0).unwrap());
        assert_eq!(phil, *the_nuts.to_vec().get(1).unwrap());
        assert_eq!(daniel, *the_nuts.to_vec().get(2).unwrap());
    }
}
