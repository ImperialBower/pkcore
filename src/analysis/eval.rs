use crate::PKError;
use crate::analysis::hand_rank::HandRank;
use crate::analysis::store::bcm::binary_card_map::FiveBCM;
use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

/// # Analysis Saga: Step 1
///
/// `Eval` is a term I coined for a specific instance of analysis when iterating through
/// all possible combinations of hands for a specific game of poker. For instance: Given
/// `THE HAND` between Daniel Negreanu and Gus Hansen, where Daniel held `6♠ 6♥` and Gus held
/// `5♦ 5♣`, with the flop of `9♣ 6♦ 5♥`, one possible `Eval` would be `6♣` on the turn,
/// giving Daniel quads, and then `5♠` on the river giving Gus quads as well. Quads over quads.
/// Another case was what actually happened: `5♠` and then `8♠` giving Daniel a full house,
/// and Gus quads.
///
/// `Eval` is an example of a utilitarian data struct. It's a simple immutable collection of state,
/// that doesn't need to worry it's pretty little bites about anything but keeping my code clean.
/// I really don't want to pollute my code with tons of functions that return tuples of information
/// willy nilly.
///
/// Now, there is a downside to this way of coding. It locks me in to structures that as I build
/// my library make the code harder and harder to untangle. If done wrong, I could tie my code into
/// knots. Let's see how it goes.
///
/// ## Sorting
///
/// We want to validate that `Eval` is primarily sorting on `HandRank` and then
/// subsequently sorting on the cards itself. This test validates by
/// creating two royal flushes, one of spades, and one of hearts. In the vector,
/// the hearts is first, but in the sort we want to make sure that it can tell the
/// difference. We've also added a simple ace high straight to throw into the mix.
///
/// We know that a `Card` will sort based purely on the value of the u32 Cactus Kev value (CKC) inside
/// single field the tuple struct. Since CKC numbers have their highest bits set to Rank and their
/// next set to Suit, a sorted Card vector would look like this:
///
/// ```
/// use pkcore::card::Card;
///
/// let mut v = vec![Card::ACE_HEARTS, Card::ACE_SPADES, Card::KING_SPADES];
///
/// v.sort();
/// v.reverse(); // Reverse it so we see the highest card first.
///
/// // A♠ A♥ K♠
/// assert_eq!(v, vec![Card::ACE_SPADES, Card::ACE_HEARTS, Card::KING_SPADES]);
/// ```
///
/// Now, we might expect cards to sort Suit before Rank, as in `A♠ K♠ A♥`, but that entirely
/// depends on the perspective of how we are viewing the `Cards`. For now, we're going to let the
/// Card sort do its thing based on pure CKC numbers, and let `Five` handle those special cases where
/// we would expect the ace and the end when the hand is a wheel.
///
/// ASIDE: In the game of bridge, one traditionally sorts `Suit` first, and to handle that I
/// created my [cardpack.rs crate](https://crates.io/crates/cardpack) which provides additional
/// flexibility by how you can sort cards. Since it is doing a lot of things that we don't need,
/// and besides, I wanted to do everything from scratch for this work, we're doing everything
/// internally, and adding additional functionality directly to the library as our requirements
/// demand.
///
/// Let's try sorting it:
/// ```
/// use std::str::FromStr;
/// use pkcore::analysis::eval::Eval;
/// use pkcore::arrays::five::Five;
///
/// let straight = Eval::from(Five::from_str("Q♠ A♥ T♠ K♠ J♠").unwrap());
/// let royal_flush_spades = Eval::from(Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap());
/// let royal_flush_hearts = Eval::from(Five::from_str("Q♥ J♥ A♥ T♥ K♥").unwrap());
/// let mut v = vec![straight, royal_flush_hearts, royal_flush_spades];
///
/// v.sort();
///
/// assert_eq!(v, vec![straight, royal_flush_hearts, royal_flush_spades]);
/// ```
///
/// Now at first, this would seem backwards. Doesn't a spades royal flush come before a simple
/// straight? In terms of one hand beating another, this is true; but remember, sorts work from
/// lowest to highest:
///
/// ```
/// let mut v = vec![1, 3, 2];
///
/// v.sort();
///
/// assert_eq!(v, vec![1, 2, 3]);
/// ```
/// [Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=f5952c6d45ba4bcc43c44699856fb7c6)
///
/// Since the primary struct inside Eval is `HandRank`, and it classifies a royal flush `HandRankValue`
/// of 1 as the highest possible number, basically reversing the order of integers for the field, \
/// by a default sort, a royal flush would come after a straight even though the primary field
/// for the `HandRank` is a lower integer.
///
/// OK, now, I realize that this might sound like
/// [Peanuts parents talking](https://www.youtube.com/watch?v=ss2hULhXf04), and I will confess
/// that this is a [fair cop](https://dictionary.cambridge.org/us/dictionary/english/it-s-a-fair-cop),
/// one will find as a software developer, that it's these sort of annoying little details that will
/// occupy a significant portion of your time.
///
/// So, that means to get the highest Eval in front, we need to reverse the sort:
///
/// ```
/// use std::str::FromStr;
/// use pkcore::analysis::eval::Eval;
/// use pkcore::arrays::five::Five;
///
/// let straight = Eval::from(Five::from_str("Q♠ A♥ T♠ K♠ J♠").unwrap());
/// let royal_flush_spades = Eval::from(Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap());
/// let royal_flush_hearts = Eval::from(Five::from_str("Q♥ J♥ A♥ T♥ K♥").unwrap());
/// let mut v = vec![royal_flush_spades, royal_flush_hearts, straight];
///
/// v.sort();
/// v.reverse();
///
/// assert_eq!(v, vec![royal_flush_spades, royal_flush_hearts, straight]);
/// ```
///
/// Note, that this is in contrast to if we sorted a vector or pure `Five` hands. without the
/// `HandRank` field proceeding it in the `Eval` struct:
///
/// ```
/// use pkcore::arrays::five::Five;
/// use std::str::FromStr;
/// use pkcore::arrays::HandRanker;
///
/// let straight = Five::from_str("Q♠ A♥ T♠ K♠ J♠").unwrap().sort();
/// let royal_flush_spades = Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap().sort();
/// let royal_flush_hearts = Five::from_str("Q♥ J♥ A♥ T♥ K♥").unwrap().sort();
/// let mut v = vec![straight, royal_flush_spades, royal_flush_hearts];
/// let expected = vec![royal_flush_spades, straight, royal_flush_hearts];
///
/// v.sort();
/// v.reverse();
///
/// assert_eq!(expected, v);
/// ```
///
/// OK, I think I've driven this point sufficiently into the ditch at this point, so lets move on to some
/// actual value. I just wanted to give you a feel for how much much of the artistry of coding is
/// an obsessive focus on annoying little details that will make you scream if you're not used to
/// it.
///
/// My favorite book that delves in this subject is Robert Bringhurst's wonderful
/// [The Elements of Typographic Style](https://en.wikipedia.org/wiki/The_Elements_of_Typographic_Style).
/// The big idea that he talks about that had a profound impact on me, is that
/// _“Typography exists to honor content.”_ It doesn't exist to show off the typography... it exists
/// to as clearly as possible to show off the substance expressed in the words.
///
/// This way of thinking is essential in programming. We don't code to show off how brilliant
/// our code is. No one besides other programmers really cares how totally complicated and brilliant
/// your programming is. The priorities for a programmer should be the following:
///
/// * How clearly does it express domain information to the user?
/// * How easy is it for future developers to maintain?
/// * How flexible is it for use in building more complex systems?
///
/// Unfortunately, it's been my experience that most software code reviews spend most of the
/// time arguing over issues that have nothing to do with this. Teams of developers will burn
/// through millions of dollars happily debating for loops over iterators as their company drives
/// off a financial cliff.
///
/// This is why it's so important to remove these factors as much as possible from the table. As I
/// like to say, _"it's always better to remove a problem than to solve it."_ Why spend
/// endless hours wasting someone else's money debating holy wars such as
/// [tabs vs. spaces](https://www.youtube.com/watch?v=SsoOG6ZeyUI) when I can have a linter such
/// as [Clippy](https://github.com/rust-lang/rust-clippy) solve it automatically for me? If you want
/// to argue over it, take your coworkers down to a bar and have at it. Nobody paying for this shit
/// gives a flying fuck.
///
/// PRO TIP: Set up linters and have them enforced in your CI server before you do any real work.
/// I have spent weeks doing this after the fact because people didn't start out right. Trust me,
/// if they're arguing with you that they don't have time, they are short sighted dweebs. One of
/// the clearest signs on Robert Burton's classic adage _Penny wise and pound foolish_ are people
/// who say things like, _"we don't have time to test our software..."_ and _"we will deal with
/// making our code cleaner after we launch..."_ or _"why should I write tests... I know what my
/// code does..."_ trust me... **TRUST ME**... this always... **ALWAYS** bites you
/// in the ass. If the culture where you are working is infested with these sorts of asshats,
/// and they refuse to do anything about it, my advice for you is to
/// [run](https://www.youtube.com/watch?v=YzXvAxp-X5Q), don't walk your ass out of there.
///
/// Luckily for us, the rust community is made up of the polar opposites of asshats. _What's
/// the opposite of an asshat? A headhat?_ Rust provides linting and formatting right out of the
/// box so we don't have to debate, or really even think that much about it. You see, as the
/// founder of the `Dumb Coding` movement, I want to spend as little time as possible thinking
/// about shit. As the great Larry Wall said, _"We will encourage you to develop the three
/// great virtues of a programmer: [laziness, impatience, and hubris](https://wiki.c2.com/?LazinessImpatienceHubris)."_
///
/// At its core, craft is about knowing how to do things right. Yes, it's a pain having to always
/// wear safety goggles when using a power saw to cut wood, but anyone with half a brain who enjoys
/// seeing knows that you'd be an idiot not to. Know your craft, and anytime you are going against
/// the traditional rules you have learned, have a damn good reason.
#[derive(Clone, Copy, Debug, Default)]
pub struct Eval {
    pub hand_rank: HandRank,
    pub hand: Five,
}

impl Eval {
    #[must_use]
    pub fn new(hand_rank: HandRank, hand: Five) -> Self {
        Eval { hand_rank, hand }
    }
}

impl Display for Eval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.hand, self.hand_rank)
    }
}

impl From<Five> for Eval {
    fn from(five: Five) -> Self {
        let (hand_rank, hand) = five.hand_rank_and_hand();

        Eval { hand_rank, hand }
    }
}

impl TryFrom<FiveBCM> for Eval {
    type Error = PKError;

    fn try_from(bcm: FiveBCM) -> Result<Self, Self::Error> {
        Ok(Eval {
            hand_rank: HandRank::from(bcm.rank),
            hand: Five::try_from(bcm.bc)?,
        })
    }
}

impl From<&Five> for Eval {
    fn from(five: &Five) -> Self {
        Eval::from(*five)
    }
}

/// FROM PLOF 1.1: Eval Display and starting on observability
/// commit 2c73e2722ebcdf4dfc3afad5857f8fb87458b985
///
/// I don't like this as the entry point for a specific case. It destroys
/// the structure for the case, specifically what's the hole cards, what's the flop
/// and what's the instance.
impl From<Seven> for Eval {
    fn from(seven: Seven) -> Self {
        let (hand_rank, hand) = seven.hand_rank_and_hand();

        Eval { hand_rank, hand }
    }
}

/// [Implementing Hash](https://doc.rust-lang.org/std/hash/trait.Hash.html#implementing-hash)
impl Hash for Eval {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hand_rank.hash(state);
        self.hand.hash(state);
    }
}

impl Ord for Eval {
    /// I originally was using derive for Ord, but adding even a simple test revealed that the
    /// `Five` struct was messing up the concept, since a Royal Flush made up of spades beats one
    /// of hearts, even though they are equal as far as Texas Hold'em is concerned.
    /// The only think that matters is the `HandRank` itself.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hand_rank.cmp(&other.hand_rank)
    }
}

impl PartialOrd<Self> for Eval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Eval {
    fn eq(&self, other: &Self) -> bool {
        self.hand_rank == other.hand_rank
    }
}
impl Eq for Eval {}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_rank__eval_tests {
    use super::*;
    use crate::analysis::class::HandRankClass;
    use crate::analysis::name::HandRankName;
    use std::str::FromStr;

    #[test]
    fn from__five() {
        let hand = Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap();

        let eval = Eval::from(hand);

        assert_eq!(eval.hand, hand.sort());
        assert_eq!(eval.hand_rank, hand.hand_rank());
        assert_eq!(eval.hand_rank.class, HandRankClass::RoyalFlush);
    }

    #[test]
    fn from__seven() {
        let seven = Seven::from_str("6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠").unwrap();
        let expected_hand = Five::from_str("6♠ 6♥ 6♦ 5♠ 5♥").unwrap();

        let eval = Eval::from(seven);

        assert_eq!(eval.hand, expected_hand);
        assert_eq!(eval.hand_rank, seven.hand_rank());
        assert_eq!(eval.hand_rank.value, 271);
        assert_eq!(eval.hand_rank.name, HandRankName::FullHouse);
        assert_eq!(eval.hand_rank.class, HandRankClass::SixesOverFives);
    }

    #[test]
    fn eq() {
        assert_eq!(
            Eval::from(Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap()),
            Eval::from(Five::from_str("Q♥ J♥ A♥ T♥ K♥").unwrap())
        )
    }

    #[test]
    fn hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        Eval::from(Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap()).hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = DefaultHasher::new();
        Eval::from(Five::from_str("A♠ T♠ K♠ J♠ Q♠ ").unwrap()).hash(&mut hasher);
        let hash2 = hasher.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn ord() {
        assert_eq!(
            Eval::from(Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap()),
            Eval::from(Five::from_str("Q♥ J♥ A♥ T♥ K♥").unwrap())
        );
        assert!(
            Eval::from(Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap())
                > Eval::from(Five::from_str("Q♥ J♥ 9♥ T♥ K♥").unwrap())
        );
        assert!(
            Eval::from(Five::from_str("Q♠ 9♠ T♠ 8♠ J♠").unwrap())
                < Eval::from(Five::from_str("Q♥ J♥ 9♥ T♥ K♥").unwrap())
        );
    }

    /// This is to validate that `Eval` is primarily sorting on `HandRank` and then
    /// subsequently sorting on the cards itself. This test validates by
    /// creating two royal flushes, one of spades, and one of hearts. In the vector,
    /// the hearts is first, but in the sort we want to make sure that it can tell the
    /// difference. We've also added a simple ace high straight to throw into the mix.
    #[test]
    fn sort() {
        let straight = Eval::from(Five::from_str("Q♠ A♥ T♠ K♠ J♠").unwrap());
        let royal_flush_spades = Eval::from(Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap());
        let royal_flush_hearts = Eval::from(Five::from_str("Q♥ J♥ A♥ T♥ K♥").unwrap());
        let mut v = vec![straight, royal_flush_hearts, royal_flush_spades];

        v.sort();
        v.reverse();

        assert_eq!(v, vec![royal_flush_spades, royal_flush_hearts, straight]);
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd)]
pub struct SevenEval {
    pub seven: Seven,
    pub five: Five,
    pub hand_rank: HandRank,
}

impl From<Seven> for SevenEval {
    fn from(seven: Seven) -> Self {
        let (hand_rank, five) = seven.hand_rank_and_hand();

        SevenEval { seven, five, hand_rank }
    }
}

/// Equality for Evals is purely factored around the `HandRank`.
impl PartialEq for SevenEval {
    fn eq(&self, other: &Self) -> bool {
        self.hand_rank == other.hand_rank
    }
}
impl Eq for SevenEval {}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_rank__seven_eval_tests {
    use super::*;
    use crate::analysis::class::HandRankClass;
    use crate::analysis::name::HandRankName;
    use std::str::FromStr;

    #[test]
    fn from__seven() {
        let seven = Seven::from_str("6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠").unwrap();
        let expected_five = Five::from_str("6♠ 6♥ 6♦ 5♠ 5♥").unwrap();

        let eval = SevenEval::from(seven);

        assert_eq!(eval.five, expected_five);
        assert_eq!(eval.seven, seven);
        assert_eq!(eval.hand_rank, seven.hand_rank());
        assert_eq!(eval.hand_rank.value, 271);
        assert_eq!(eval.hand_rank.name, HandRankName::FullHouse);
        assert_eq!(eval.hand_rank.class, HandRankClass::SixesOverFives);
    }
}
