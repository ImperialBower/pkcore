use std::collections::HashMap;
use crate::arrays::five::Five;
use crate::hand_rank::eval::Eval;
use crate::hand_rank::HandRank;

pub struct Nutty(HashMap<HandRank, Vec<Eval>>);

/// The immediate need for this class is so that we can have an easy way to hold and sort the
/// hands possible at a particular point in a game, usually the flop. I'm thinking that we can
/// return this object as a part of our Pile trait, so that if we want to get all the possible
/// hands at the flop or turn, we can just call that method.
///
/// See `CaseEval` for the etymology being the phrase the nuts.
///
/// # REFACTOR
///
/// OK, we've hit a snag. There's not one Eval for the nuts with any given flop. For instance, there
/// are 16 variations:
///
/// * 9♣ 8♠ 7♠ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♠ 7♥ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♠ 7♦ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♠ 7♣ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♥ 7♠ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♥ 7♥ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♥ 7♦ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♥ 7♣ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♦ 7♠ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♦ 7♥ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♦ 7♦ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♦ 7♣ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♣ 7♠ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♣ 7♥ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♣ 7♦ 6♦ 5♥ - 1605: `NineHighStraight`
/// * 9♣ 8♣ 7♣ 6♦ 5♥ - 1605: `NineHighStraight`
///
/// We're either going to have to find a better data structure, or distill our vector down to only
/// one entry for each `HandRank`.
///
/// Sigh... this is one of the harder things about programming. You've gotten all your nice little
/// programmatic 🦆🦆🦆🦆🦆 in a row only to discover that it just doesn't work. Hours and hours
/// of testing all needing to be redone. Time to light a match, and watch it burn.
///
/// So, I'm going to need to refactor `TheNuts`. Here's what I'm thinking:
///
/// ```
/// use std::collections::HashMap;
/// use pkcore::hand_rank::eval::Eval;
/// use pkcore::hand_rank::HandRank;
///
/// pub struct TheNuts(HashMap<HandRank, Vec<Eval>>);
/// ```
///
/// A collection containing all the possible `Evals` for a specific `HandRank`. The problem is,
/// a vector can have dupes. What about something like this:
///
/// ```
///
/// use std::collections::{HashMap, HashSet};
/// use pkcore::hand_rank::eval::Eval;
/// use pkcore::hand_rank::HandRank;
///
/// pub struct TheNuts(HashMap<HandRank, HashSet<Eval>>);
/// ```
///
/// One potential problem with that though is that an Eval with the exact same hand, but with the
/// cards in different order, could be seen as a different eval. This problem stems from the hand
/// element in the `Eval` struct. Two different orders of the same hand are not seen as equal:
///
/// ```
/// use pkcore::card::Card;
///
/// let royal_flush_1 = [
///     Card::ACE_DIAMONDS,
///     Card::KING_DIAMONDS,
///     Card::QUEEN_DIAMONDS,
///     Card::JACK_DIAMONDS,
///     Card::TEN_DIAMONDS,
/// ];
///
/// let royal_flush_2 = [
///     Card::KING_DIAMONDS,
///     Card::ACE_DIAMONDS,
///     Card::QUEEN_DIAMONDS,
///     Card::JACK_DIAMONDS,
///     Card::TEN_DIAMONDS,
/// ];
///
/// assert_ne!(royal_flush_1, royal_flush_2)
/// ```
///
/// Evan though these are exactly the same hands, from a pure data representation, the cards are in
/// a different order, so they are different. What we need, is a way to override equal for `Five`
/// and `Eval`.
///
/// Let's try test-driving this through `Five` and then see if there's a way for it to cascade down
/// to `Pile` so that it can apply to any collection of cards.
///
/// So, we've figured out a way to implement an equality test for `Five` that ignores card order:
///
/// ```
/// use pkcore::arrays::five::Five;
/// use pkcore::card::Card;
/// fn eq(a: Five, b: Five) -> bool {
///     let mut a = a.to_arr();
///     a.sort();
///
///     let mut b = b.to_arr();
///     b.sort();
///
///     a == b
/// }
///
/// let royal_flush_1 = Five::from([
///     Card::ACE_DIAMONDS,
///     Card::KING_DIAMONDS,
///     Card::QUEEN_DIAMONDS,
///     Card::JACK_DIAMONDS,
///     Card::TEN_DIAMONDS,
/// ]);
///
/// let royal_flush_2 = Five::from([
///     Card::KING_DIAMONDS,
///     Card::ACE_DIAMONDS,
///     Card::QUEEN_DIAMONDS,
///     Card::JACK_DIAMONDS,
///     Card::TEN_DIAMONDS,
/// ]);
///
/// assert!(eq(royal_flush_1, royal_flush_2));
/// ```
///
/// The problem with using this functionality for a manual implementation of the `PartialEq` trait
/// is that clippy complains "you are deriving `Hash` but have implemented `PartialEq` explicitly".
///
/// This feels like we're falling down a rabbit's hole. I really don't want to be overriding the
/// default implementations of `PartialEq` and `Hash` if I don't really have to, especially for a
/// fundamental data type like `Five`. It's designed to be simple and fast.
///
/// I can think of three ways of dealing with this edge case:
///
/// 1. Ignoring it until it because a real issue.
/// 2. Forcing a sort everytime you instantiate a `HandRank` struct.
/// 3. `Bard`!!!
///
/// What's a `Bard` you ask? Let's go over to that file and find out.
///
/// OK, now that you're back, I've come to the conclusion that I am once again overthinking the
/// problem. One of the really great things about pair programming, is that you always have someone
/// there calling you on your bullshit. _Do we really need that?_ _What exactly is the point?_ _Does
/// this have anything to do with the story we're working on?_
///
/// When I am flying solo, like right now, I will often take some wild detours exploring strange
/// corners, and enjoy what I might find a long the way. When I started on `Fudd` this was one of
/// the fun things about working on it. Just playing with the code. Seeing what I could do with it.
///
/// The two things that are really holding me back are I want to create a tight library that others,
/// including me, can use to crete cool shit, and that voice in the back of my head warning me that
/// you, gentle reader, will be suffering through my ramblings. For this role, I always have that
/// wise sage [Gold Five](https://www.youtube.com/watch?v=2kObBphkNiU) counselling me,
/// _"Stay on target! STAY ON TARGET!!!"_
///
/// I will confess, that I love bitwise operations. I still remember the first time I saw them
/// in code when I was working for my first professional programming gig, at the now defunct
/// [XOOM.com](https://en.wikipedia.org/wiki/Xoom_(web_hosting)) startup. I was looking through
/// some of their php code and they had a bunch of constants that were made up of a bunch of zeros and ones .
/// I had no idea
/// how the code worked. Luckily, [Jeff Glover](https://jeffglover.com/) took the time to show me.
/// I hope that you are as lucky as I was in having a mentor like Jeff. He's an amazing developer,
/// and designer, and most importantly... he is always having a good time doing it.
///
/// Back in the 90s I just
/// happened to recognize him from his website while walking from a temp agency interview. Now,
/// you can go to MeetUps, learn stuff, and make friends along the way. Those friends will be the
/// most important connections in your career.
///
/// I can't underestimate how important it is to bring joy to your work. When I got a job working
/// for a very very large financial institution, I noticed how downtrodden everyone seemed to be who
/// worked there. I resolved that, no matter what, I was going to have a good time. I am not going
/// to let anyone's negative energy bring me down. This was the best resolution I ever made
/// in my career. So many managers confuse abuse with leadership. They're idiots. All they
/// accomplish is to incentivize concealment and sloth. Most organizational dysfunction can be traced
/// back to this dynamic. Look for it... avoid it if you can... don't let it become you if you
/// can't. You can't help it if others need to be miserable. Protect yourself. Take the time. Enjoy
/// life.
///
/// Positive energy... being present... embracing challenges... these skills are way more important
/// than any individual tech foo. When I interview people, this is what I look for. I can
/// teach someone how to use almost any technology. I can't teach them how to be present. This is
/// one of the main reasons so many tech initiatives fail. Nietzschean will to power only gets you
/// so far. Eventually, you're going to need a team, working together, all striving for the same
/// goal.
///
/// ![BELIEVE](https://awfulannouncing.com/wp-content/uploads/sites/94/2021/08/lasso_ep6.jpeg)
///
/// I remember talking to my Uncle Leon over a decade after the
/// [Challenger disaster](https://en.wikipedia.org/wiki/Space_Shuttle_Challenger_disaster). He
/// followed in my grandfather Joe's footsteps, who was one of the
/// engineers for the first moon landing.
///
/// We were sitting around my grandmother's death bed, taking turns reading to her from the Bible,
/// and watching DVDs from his favorite show, Stargate SG-1. I asked him how things were at NASA.
/// His reply chilled me to the bone:
///
/// "NASA is dead. They've replaced the engineering managers with bean counters. They only care
/// about hitting their budget targets."
///
/// A lot of my imposter syndrome stems from being Joe's grandson. He put a man on the moon with a
/// fucking slide rule, and that after being taught all twelve grades of his primary education from
/// a one room school room. _This is when my grandmother Hazel would send me to my room for cursing._
///
/// This is why I will never call myself an engineer. My grandfather was an engineer. My dad was an
/// engineer. My uncle Leon was an engineer. I'm a programmer. I'm a software developer. It's like
/// the difference between a person who plays the bassoon and someone who makes them. We're all on
/// the same team, but we are not doing the same work. I've helped made the works of Beethoven and
/// Musgrave come to life. I've also helped turn engineer's designs into functioning cars. How cool is
/// that?
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

    pub fn sort_in_place(&mut self) {
        self.0.sort();
        self.0.reverse();
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
