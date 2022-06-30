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
    pub fn to_vec(&self) -> Vec<Eval> {
        self.0.clone()
    }
}

impl From<Vec<Eval>> for TheNuts {
    fn from(v: Vec<Eval>) -> Self {
        TheNuts(v)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_rank__the_nuts_tests {
    use super::*;
    use crate::util::data::TestData;

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
}
