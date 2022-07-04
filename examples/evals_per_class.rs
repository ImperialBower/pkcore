use std::collections::HashSet;
use pkcore::arrays::five::Five;
use pkcore::arrays::HandRanker;
use pkcore::arrays::three::Three;
use pkcore::arrays::two::Two;
use pkcore::card::Card;
use pkcore::hand_rank::class::Class;
use pkcore::hand_rank::eval::Eval;
use pkcore::Pile;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvalsPerClass(Vec<Eval>, HashSet<Class>);

impl EvalsPerClass {
    pub fn push(&mut self, evaluated_hand: Eval) {
        if self.1.insert(evaluated_hand.hand_rank.class) {
            self.0.push(evaluated_hand);
        }
    }

    #[must_use]
    pub fn sort(&self) -> EvalsPerClass {
        let mut cards = self.clone();
        cards.sort_in_place();
        cards
    }

    pub fn sort_in_place(&mut self) {
        self.0.sort_unstable();
        self.0.reverse();
    }

    #[must_use]
    pub fn to_vec(&self) -> &Vec<Eval> {
        &self.0
    }
}

fn main() {
    let three = Three::from([Card::NINE_CLUBS, Card::SIX_DIAMONDS, Card::FIVE_HEARTS]);
    let mut evals = EvalsPerClass::default();

    for v in three.remaining().combinations(2) {
        let hand = Five::from_2and3(Two::from(v), three);
        evals.push(hand.eval());
    }
    evals.sort_in_place();

    for eval in evals.to_vec().iter() {
        println!("{}", eval);
    }
}

