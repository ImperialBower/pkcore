use crate::arrays::five::Five;
use crate::hand_rank::eval::Eval;
use crate::hand_rank::HandRank;
use std::collections::HashMap;

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
/// ```txt
/// 9♣ 8♠ 7♠ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♠ 7♥ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♠ 7♦ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♠ 7♣ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♥ 7♠ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♥ 7♥ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♥ 7♦ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♥ 7♣ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♦ 7♠ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♦ 7♥ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♦ 7♦ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♦ 7♣ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♣ 7♠ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♣ 7♥ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♣ 7♦ 6♦ 5♥ - 1605: `NineHighStraight`
/// 9♣ 8♣ 7♣ 6♦ 5♥ - 1605: `NineHighStraight`
/// ```
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
/// you can go to meetups, learn stuff, and make friends along the way. Those friends will be the
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
/// One of my biggest regrets is that, as an introvert, I tend to forget to let the people who
/// really made a difference in my life know. They're in my heart, but that's no enough. Take the
/// time.
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
///
/// ## OK, back to the hellfactoring...
///
/// How do we want to do this? I can see two ways:
///
/// 1. Throw it all away and start over.
/// 2. Create a temporary struct with a different name, AB the functionality over from what we've done so far, and then swap them out when we're done.
///
/// A lot of the programmers I really respect would do plan 1. Me, I tend to do plan #2. I do love
/// my training wheels. Feel free to try out Plan #1 for yourself. Me... it's Sunday. I'm tired of being
/// in the red for over two days. #2 it is.
///
/// Here's the plan: We're going to create a temporary struct with our target structure and walk
/// through the functionality from our soon to be mothballed struct.
///
/// ```
/// use std::collections::HashMap;
/// use pkcore::hand_rank::eval::Eval;
/// use pkcore::hand_rank::HandRank;
///
/// #[derive(Clone, Debug, Default, Eq, PartialEq)]
/// pub struct Nutty(HashMap<HandRank, Vec<Eval>>);
/// ```
///
/// One big problem is that the dynamics of a `HashMap` are radically different than a `Vec`. Can
/// you think of what the biggest difference is?
///
/// A `Vec` is ordered. A `HashMap` isn't. This is going to be a little bit of a hassle for us.
/// Luckily, the vast majority of the work is done with the calculation of the structure. Once it's
/// set up, we can just grab what we need and be done with it. What are the possible use cases?
/// Here's what I can think of:
///
/// 1. Give a list of representative vector of `Evals`; one representing each possible `HandRank`.
///
/// Here's how this could look for `The Hand`:
///
/// ```txt
/// 9♣ 8♠ 7♠ 6♦ 5♥ HandRank { value: 1605, name: Straight, class: NineHighStraight }
/// 9♠ 9♥ 9♣ 6♦ 5♥ HandRank { value: 1996, name: ThreeOfAKind, class: ThreeNines }
/// 6♠ 6♥ 6♦ 9♣ 5♥ HandRank { value: 2185, name: ThreeOfAKind, class: ThreeSixes }
/// 5♠ 5♥ 5♦ 9♣ 6♦ HandRank { value: 2251, name: ThreeOfAKind, class: ThreeFives }
/// 9♠ 9♣ 6♠ 6♦ 5♥ HandRank { value: 3047, name: TwoPair, class: NinesAndSixes }
/// ...
/// ```
///
/// 2. Return a probability distribution for every type of possible `HandRank`s.
///
/// 3. Finally, return an integer indicating where a specific player's hand is in relationship to the
/// nuts. So, for Daniel's hand of `9♠ 9♥ 9♣ 6♦ 5♥`, it would return three, since he has the third
/// nuts, as they say; over a nine high straight and three nines.
///
///
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TheNuts(HashMap<HandRank, Vec<Five>>);

impl TheNuts {
    /// This is going to be a lot more complicated than with our original stab at this problem.
    /// We need a way to return a vector made up of one `Eval` for each `HandRank`, sorted by order
    /// of strength. If only we had a data structure to easily do that.
    ///
    /// Turns out we already do, with the code we are replacing with our refactoring. And just like
    /// that, `TheNuts` becomes `Evals`. `TheNuts` is dead. Long live `TheNuts`.
    #[must_use]
    pub fn get(&self, _i: usize) -> Option<&Eval> {
        todo!()
    }
}
