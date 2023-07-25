use crate::analysis::the_nuts::TheNuts;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use crate::util::wincounter::wins::Wins;
use crate::{PKError, Pile, Shifty, SuitShift};
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct SortedHeadsUp {
    pub higher: Two,
    pub lower: Two,
}

impl SortedHeadsUp {
    #[must_use]
    pub fn new(first: Two, second: Two) -> SortedHeadsUp {
        if first > second {
            SortedHeadsUp {
                higher: first,
                lower: second,
            }
        } else {
            SortedHeadsUp {
                higher: second,
                lower: first,
            }
        }
    }

    #[must_use]
    pub fn contains(&self, two: &Two) -> bool {
        self.is_higher(two) || self.is_lower(two)
    }

    #[must_use]
    pub fn is_higher(&self, two: &Two) -> bool {
        &self.higher == two
    }

    #[must_use]
    pub fn is_lower(&self, two: &Two) -> bool {
        &self.lower == two
    }

    #[must_use]
    pub fn higher(&self) -> Two {
        self.higher
    }

    #[must_use]
    pub fn higher_as_bard(&self) -> Bard {
        self.higher.bard()
    }

    #[must_use]
    pub fn lower(&self) -> Two {
        self.lower
    }

    #[must_use]
    pub fn lower_as_bard(&self) -> Bard {
        self.lower.bard()
    }

    /// This is going to be a heavy calculation.
    ///
    /// Oops!
    /// ```txt
    /// error[E0599]: the method `insert` exists for struct `HashSet<SortedHeadsUp>`, but its trait bounds were not satisfied
    ///   --> src/arrays/matchups/mod.rs:63:20
    ///    |
    /// 13 | pub struct SortedHeadsUp {
    ///    | ------------------------ doesn't satisfy `SortedHeadsUp: Hash`
    /// ...
    /// 63 |                 hs.insert(SortedHeadsUp::new(hero, villain));
    ///    |                    ^^^^^^
    ///    |
    ///    = note: the following trait bounds were not satisfied:
    ///            `SortedHeadsUp: Hash`
    /// help: consider annotating `SortedHeadsUp` with `#[derive(Hash)]`
    ///    |
    /// 13 + #[derive(Hash)]
    /// 14 | pub struct SortedHeadsUp {
    ///    |
    /// ```
    ///
    /// Man, I love the [Rust compiler](https://doc.rust-lang.org/std/collections/struct.HashSet.html#examples).
    /// Much love to everyone who worked on this thing.
    ///
    /// # Errors
    ///
    /// Shrugs.
    pub fn unique() -> Result<HashSet<SortedHeadsUp>, PKError> {
        let mut hs: HashSet<SortedHeadsUp> = HashSet::new();
        for v in Cards::deck().combinations(2) {
            let hero = Two::try_from(v.as_slice())?;
            for r in hero.remaining().combinations(2) {
                let villain = Two::try_from(r.as_slice())?;
                hs.insert(SortedHeadsUp::new(hero, villain));
            }
        }
        Ok(hs)
    }

    /// Renaming `all_possible()` to `unique()`.#
    ///
    /// # Errors
    ///
    /// If a deck isn't divisible by 2, which shouldn't happen. Maybe if we add jokers some day.
    pub fn distinct() -> Result<HashSet<SortedHeadsUp>, PKError> {
        let mut hs: HashSet<SortedHeadsUp> = HashSet::new();
        for v in Cards::deck().combinations(2) {
            let hero = Two::try_from(v.as_slice())?;
            for r in hero.remaining().combinations(2) {
                let villain = Two::try_from(r.as_slice())?;
                hs.insert(SortedHeadsUp::new(hero, villain));
            }
        }
        Ok(hs)
    }

    /// I want to be able to generate these values into a CSV file, so that I can use them to
    /// load into our odds db.
    ///
    /// OK, this is insanely easy now that we've mastered the magic spell. Thank you serde!
    ///
    /// ```txt
    /// pub fn generate_csv(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///         let mut wtr = WriterBuilder::new().has_headers(true).from_path(path)?;
    ///         for shu in SortedHeadsUp::all_possible().iter() {
    ///             wtr.serialize(shu)?;
    ///         }
    ///         wtr.flush()?;
    ///         Ok(())
    ///     }
    /// ```
    ///
    /// Except, it doesn't work. :-( All we see is:
    ///
    /// ```txt
    /// Higher,Lower,Higher,Lower,Higher,Lower,Higher,Lower,Higher,Lower,Higher,Lower,Higher,Lower,Higher,Lower,Higher,Lower,Higher,
    /// ```
    ///
    /// I totally deserve this wonderfully condescending reply from
    /// [Shepmaster](https://stackoverflow.com/users/155423/shepmaster) on stackoverflow
    /// to the question of
    /// [How do I convert a HashSet of Strings into a Vector?](https://stackoverflow.com/questions/60893051/how-do-i-convert-a-hashset-of-strings-into-a-vector).
    ///
    /// ```txt
    /// I encourage you to re-read The Rust Programming Language, specifically the chapter on iterators. Next, become familiar with the methods of Iterator.
    ///
    /// The normal way I'd expect to see this implemented is to convert the HashSet to an iterator and then collect the iterator to a Vec:
    ///
    /// let mut v: Vec<_> = hs.into_iter().collect();
    /// In this case, I'd prefer to use FromIterator directly (the same trait that powers collect):
    ///
    /// let mut v = Vec::from_iter(hs);
    /// Focusing on your larger problem, use a BTreeSet instead, coupled with What's an idiomatic way to print an iterator separated by spaces in Rust?
    ///
    /// use itertools::Itertools; // 0.10.1
    /// use std::collections::BTreeSet;
    ///
    /// fn main() {
    ///     // Create the set somehow
    ///     let hs: BTreeSet<_> = ["fee", "fie", "foo", "fum"]
    ///         .into_iter()
    ///         .map(String::from)
    ///         .collect();
    ///
    ///     println!("{}", hs.iter().format(", "));
    /// }
    /// ```
    ///
    /// But, that's not how I learn. I read the manual, try to let it soak into my subconscious, and
    /// then let failure lock in the learning.
    ///
    /// Here's our revision:
    ///
    /// ```txt
    /// pub fn generate_csv(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///         let mut wtr = WriterBuilder::new().has_headers(true).from_path(path)?;
    ///         let hs = SortedHeadsUp::all_possible()?;
    ///         let mut v = Vec::from_iter(hs);
    ///         for shu in v.iter() {
    ///             wtr.serialize(shu)?;
    ///         }
    ///         wtr.flush()?;
    ///         Ok(())
    ///     }
    /// ```
    ///
    /// Grrrr....
    ///
    /// ```txt
    /// error[E0277]: the trait bound `PKError: StdError` is not satisfied
    ///    --> src/arrays/matchups/sorted_heads_up.rs:124:47
    ///     |
    /// 124 |         let hs = SortedHeadsUp::all_possible()?;
    ///     |                                               ^ the trait `StdError` is not implemented for `PKError`
    ///     |
    ///     = help: the following other types implement trait `FromResidual<R>`:
    ///               <std::result::Result<T, F> as FromResidual<Yeet<E>>>
    ///               <std::result::Result<T, F> as FromResidual<std::result::Result<Infallible, E>>>
    ///     = note: required for `Box<dyn StdError>` to implement `From<PKError>`
    ///     = note: required for `std::result::Result<(), Box<dyn StdError>>` to implement `FromResidual<std::result::Result<Infallible, PKError>>`
    /// ```
    ///
    /// I'm working with two kinds of errors: `std::error::Error` and my `PKError`. So, I will punt
    /// and unwrap() like the lazy f I am. Really need to figure out a good way to deal with this.
    /// This would be so easy in Java. :-P
    ///
    /// Still, I think we've got it now...
    ///
    /// ```txt
    /// pub fn generate_csv(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///         let mut wtr = WriterBuilder::new().has_headers(true).from_path(path)?;
    ///         let hs = SortedHeadsUp::all_possible().unwrap();
    ///         let v = Vec::from_iter(hs);
    ///         for shu in v.iter() {
    ///             wtr.serialize(shu)?;
    ///         }
    ///         wtr.flush()?;
    ///         Ok(())
    ///     }
    /// ```
    ///
    /// FUCK!!!!
    ///
    /// ```txt
    /// thread 'main' panicked at 'TODO: panic message: Error(UnequalLengths { pos: None, expected_len: 2, len: 4 })', examples/generate_all_possible_shu.rs:7:67
    /// ```
    ///
    /// This is the file it generates:
    ///
    /// ```txt
    /// Higher,Lower
    /// Q♠,9♥,T♠,8♦
    /// ```
    ///
    /// I know the problem. We've been relying on `Card's` custom serializer. We're going to need one
    /// for `Two`.
    ///
    /// Scheiße!!! I was really hoping that this would be easy... but remember the rule:
    /// **nothing is ever easy** I should bail, but now I'm kinda obsessed.
    ///
    /// OK, we've updated Two. Let us see if this cracks the case.
    ///
    /// Yessir! That did it. Interestingly enough, running the code while at the same time running
    /// `examples/hup.rs` AND `examples/bcrepl.rs` from both here and `Fudd` caused my `CLion` to
    /// crash my system, and when I rebooted I needed to reinstall the `JetBrains ToolBox`. Maybe
    /// don't run these things all at the same time. This did give me an idea...
    ///
    /// Thanks to `Shifty`, I don't need to store all of the possible shifts, only the original ones.
    /// I can remove all the shifts and just run through the ones remaining, doing the shift when
    /// I perform the preflop calculation. This will leverage the power of
    /// [Distinct vs Unique](http://suffe.cool/poker/evaluator.html). I only want the Distinct
    /// entries.
    ///
    /// Yes yes yes, I am getting sidetracked, but that's the fun of coding for fun. It's my party,
    /// and whilst you are invited, I am providing the entertainment.
    ///
    /// Let's try it and see.
    ///
    /// # Errors
    ///
    /// When it can't write to path.
    ///
    /// # Panics
    ///
    /// When can't write to file system
    pub fn generate_csv(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = WriterBuilder::new().has_headers(true).from_path(path)?;
        let hs = SortedHeadsUp::unique().unwrap();
        let v = Vec::from_iter(hs);
        for shu in &v {
            wtr.serialize(shu)?;
        }
        wtr.flush()?;
        Ok(())
    }

    /// Returns a `HashSet` of the possible suit shifts. I'm thinking that I want to add this to the
    /// `SuitShift` trait. This would require that the trait would need Copy as a
    /// [supertrait](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-supertraits-to-require-one-traits-functionality-within-another-trait).
    ///
    /// I've never used a supertrait before. This should be fun.
    ///
    /// Firs, let's implement it on `SuitShift` without changing anything, and then we'll see if
    /// we can make this method apply to any struct that implements the trait.
    ///
    /// Adding the supertrait was easy:
    ///
    /// ```txt
    /// pub trait SuitShift: Copy {
    ///     #[must_use]
    ///     fn shift_suit_down(&self) -> Self;
    ///
    ///     #[must_use]
    ///     fn shift_suit_up(&self) -> Self;
    ///
    ///     #[must_use]
    ///     fn opposite(&self) -> Self;
    /// }
    /// ```
    ///
    /// But that won't work:
    ///
    /// ```txt
    /// error[E0277]: the trait bound `Cards: std::marker::Copy` is not satisfied
    ///    --> src/cards.rs:640:20
    ///     |
    /// 640 | impl SuitShift for Cards {
    ///     |                    ^^^^^ the trait `std::marker::Copy` is not implemented for `Cards`
    ///     |
    /// note: required by a bound in `SuitShift`
    ///    --> src/lib.rs:183:22
    ///     |
    /// 183 | pub trait SuitShift: Copy {
    ///     |                      ^^^^ required by this bound in `SuitShift`
    /// ```
    ///
    /// `Cards` doesn't implement `Copy`, and since it's an `IndexSet`, it isn't going to. Back to
    /// the drawing board.
    ///
    /// How about we create a trait called `Shifty`, and make `SuitShift` its supertrait? Something like:

    /// ```txt
    /// use std::collections::HashSet;
    /// use pkcore::SuitShift;
    /// pub trait Shifty: SuitShift {
    ///     #[must_use]
    ///     fn shifts(&self) -> HashSet<Self>;
    /// }
    /// ```
    ///
    /// Nope. Strike two!
    ///
    /// ```txt
    /// error[E0277]: the size for values of type `Self` cannot be known at compilation time
    ///    --> src/arrays/matchups/sorted_heads_up.rs:151:25
    ///     |
    /// 9   |     fn shifts(&self) -> HashSet<Self>;
    ///     |                         ^^^^^^^^^^^^^ doesn't have a size known at compile-time
    ///     |
    /// note: required by a bound in `HashSet`
    ///    --> ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/set.rs:106:20
    ///     |
    /// 106 | pub struct HashSet<T, S = RandomState> {
    ///     |                    ^ required by this bound in `HashSet`
    /// help: consider further restricting `Self`
    ///     |
    /// 9   |     fn shifts(&self) -> HashSet<Self> where Self: Sized;
    ///     |                                       +++++++++++++++++
    ///
    /// error: aborting due to previous error
    /// ```
    ///
    ///  Wonder if its recommendations will work?
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use pkcore::SuitShift;
    /// pub trait Shifty: SuitShift {
    ///     #[must_use]
    ///     fn shifts(&self) -> HashSet<Self> where Self: Sized;
    /// }
    /// ```
    ///
    /// 💥! That compiles! But... will it actually work?
    ///
    /// First, we'll need to rewrite shifts into the trait, and then swap it out inside here.
    ///
    /// ```txt
    /// use std::collections::HashSet;
    /// use pkcore::SuitShift;
    /// pub trait Shifty: SuitShift {
    ///     #[must_use]
    ///     fn shifts(&self) -> HashSet<Self> where Self: Sized {
    ///         let mut hs = HashSet::new();
    ///         let mut shifty = *self;
    ///         hs.insert(shifty);
    ///         for _ in 1..=3 {
    ///             shifty = shifty.shift_suit_up();
    ///             hs.insert(shifty);
    ///         }
    ///
    ///         hs
    ///     }
    /// }
    /// ```
    ///
    /// Nope... but we're getting closer...
    ///
    /// ```txt
    /// error[E0277]: the trait bound `Self: std::cmp::Eq` is not satisfied
    ///    --> src/lib.rs:200:12
    ///     |
    /// 200 |         hs.insert(shifty);
    ///     |            ^^^^^^ the trait `std::cmp::Eq` is not implemented for `Self`
    ///     |
    /// note: required by a bound in `HashSet::<T, S>::insert`
    ///    --> ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/set.rs:428:8
    ///     |
    /// 428 |     T: Eq + Hash,
    ///     |        ^^ required by this bound in `HashSet::<T, S>::insert`
    /// ...
    /// 887 |     pub fn insert(&mut self, value: T) -> bool {
    ///     |            ------ required by a bound in this associated function
    /// help: consider further restricting `Self`
    /// ```
    ///
    /// Adding that we get
    ///
    /// ```txt
    /// error[E0277]: the trait bound `Self: Hash` is not satisfied
    ///    --> src/lib.rs:200:12
    ///     |
    /// 200 |         hs.insert(shifty);
    ///     |            ^^^^^^ the trait `Hash` is not implemented for `Self`
    ///     |
    /// note: required by a bound in `HashSet::<T, S>::insert`
    ///    --> ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/set.rs:428:13
    ///     |
    /// 428 |     T: Eq + Hash,
    ///     |             ^^^^ required by this bound in `HashSet::<T, S>::insert`
    /// ...
    /// 887 |     pub fn insert(&mut self, value: T) -> bool {
    ///     |            ------ required by a bound in this associated function
    /// help: consider further restricting `Self`
    ///     |
    /// 197 |     fn shifts(&self) -> HashSet<Self> where Self: Sized, Self: std::cmp::Eq, Self: Hash {
    ///     |                                                                            ++++++++++++
    /// ```
    ///
    /// Still no.
    ///
    /// ```txt
    /// error[E0507]: cannot move out of `*self` which is behind a shared reference
    ///    --> src/lib.rs:200:26
    ///     |
    /// 200 |         let mut shifty = *self;
    ///     |                          ^^^^^ move occurs because `*self` has type `Self`, which does not implement the `Copy` trait
    ///     |
    /// help: consider removing the dereference here
    ///     |
    /// 200 -         let mut shifty = *self;
    /// 200 +         let mut shifty = self;
    /// ```
    ///
    /// So let's add the Copy trait as a supertrait of `Shifty`.
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use std::hash::Hash;
    /// use pkcore::SuitShift;
    /// pub trait Shifty: SuitShift + Copy {
    ///     #[must_use]
    ///     fn shifts(&self) -> HashSet<Self> where Self: Sized, Self: std::cmp::Eq, Self: Hash {
    ///         let mut hs = HashSet::new();
    ///         let mut shifty = *self;
    ///         hs.insert(shifty);
    ///         for _ in 1..=3 {
    ///             shifty = shifty.shift_suit_up();
    ///             hs.insert(shifty);
    ///         }
    ///
    ///         hs
    ///     }
    /// }
    /// ```
    ///
    /// 💥💥💥! It compiles! We're back in business. Still, we don't know if it will actually work.
    /// Let's swap out the function for the trait and see what transpires, shall we.
    ///
    /// ```txt
    /// error[E0599]: no method named `shifts` found for reference `&SortedHeadsUp` in the current scope
    ///    --> examples/hup.rs:277:34
    ///     |
    /// 277 |         let possible_sorts = shu.shifts();
    ///     |                                  ^^^^^^
    ///     |
    ///     = help: items from traits can only be used if the trait is in scope
    /// help: the following trait is implemented but not in scope; perhaps add a `use` for it:
    ///     |
    /// 1   + use pkcore::Shifty;
    ///     |
    /// help: there is a method with a similar name
    ///     |
    /// 277 |         let possible_sorts = shu.old_shifts();
    ///     |                                  ~~~~~~~~~~
    /// ```
    ///
    /// Gonna need to import the trait for the code that was using our `shifts()` method.
    ///
    /// Tests pass... let's see if `examples/hup.rs` still does its magic.
    ///
    /// Still works, although to be fair we've never ran it through an entire run. I'm going to
    /// check it out in a different location from this point and let hup run to see what happens
    /// when we get all the way to the end. At the same time I'm going to refactor the code so
    /// that it works on a smaller sample so we can get faster feedback.
    #[must_use]
    pub fn old_shifts(&self) -> HashSet<Self> {
        let mut v = HashSet::new();
        let mut shifty = *self;
        v.insert(shifty);

        for _ in 1..=3 {
            shifty = shifty.shift_suit_up();
            v.insert(shifty);
        }

        v
    }

    /// For now, returning default is all part of the blueprint. Still, let's write a test
    /// that fails that we can ignore later on when we get everything wired in.
    #[must_use]
    pub fn wins(&self) -> Wins {
        Wins::default()
    }
}

impl Display for SortedHeadsUp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.higher, self.lower)
    }
}

impl Pile for SortedHeadsUp {
    /// Shoot. Forgot about my frequency mask idea. Still has potential, but later.
    fn clean(&self) -> Self {
        todo!()
    }

    /// Implementing this would be interesting. What's the best possible hand from either of these
    /// two hands?
    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    /// This is the only one we need to implement for what we want. Maybe this interface is doing
    /// too much.
    fn to_vec(&self) -> Vec<Card> {
        let mut v = self.higher.to_vec();
        v.extend(self.lower.to_vec());
        v
    }
}

impl SuitShift for SortedHeadsUp {
    /// I'm not convinced that this is going to work, but I want to try.
    fn shift_suit_down(&self) -> Self {
        SortedHeadsUp::new(self.higher.shift_suit_down(), self.lower.shift_suit_down())
    }

    fn shift_suit_up(&self) -> Self {
        SortedHeadsUp::new(self.higher.shift_suit_up(), self.lower.shift_suit_up())
    }

    /// I especially don't know about opposite.
    fn opposite(&self) -> Self {
        SortedHeadsUp::new(self.higher.opposite(), self.lower.opposite())
    }
}

impl Shifty for SortedHeadsUp {}

impl TryFrom<Cards> for SortedHeadsUp {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=3 => Err(PKError::NotEnoughCards),
            4 => {
                let first = Two::new(
                    *cards.get_index(0).ok_or(PKError::InvalidCard)?,
                    *cards.get_index(1).ok_or(PKError::InvalidCard)?,
                )?;
                let second = Two::new(
                    *cards.get_index(2).ok_or(PKError::InvalidCard)?,
                    *cards.get_index(3).ok_or(PKError::InvalidCard)?,
                )?;
                Ok(SortedHeadsUp::new(first, second))
            }

            _ => Err(PKError::TooManyCards),
        }
    }
}

impl TryFrom<Vec<Two>> for SortedHeadsUp {
    type Error = PKError;

    fn try_from(v: Vec<Two>) -> Result<Self, Self::Error> {
        match v.len() {
            0..=1 => Err(PKError::NotEnoughHands),
            2 => Ok(SortedHeadsUp::new(
                *v.get(0).ok_or(PKError::InvalidHand)?,
                *v.get(1).ok_or(PKError::InvalidHand)?,
            )),
            _ => Err(PKError::TooManyHands),
        }
    }
}

/// TODO do I need this? I'm overthinking and underknowing.
impl TryFrom<Vec<&Two>> for SortedHeadsUp {
    type Error = PKError;

    fn try_from(v: Vec<&Two>) -> Result<Self, Self::Error> {
        match v.len() {
            0..=1 => Err(PKError::NotEnoughHands),
            2 => Ok(SortedHeadsUp::new(
                **v.get(0).ok_or(PKError::InvalidHand)?,
                **v.get(1).ok_or(PKError::InvalidHand)?,
            )),
            _ => Err(PKError::TooManyHands),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__sorted_heads_up {
    use super::*;
    use crate::util::data::TestData;
    use crate::util::wincounter::win::Win;

    const HANDS_7D_7C_V_6S_6H: SortedHeadsUp = SortedHeadsUp {
        higher: Two::HAND_7D_7C,
        lower: Two::HAND_6S_6H,
    };

    #[test]
    fn new() {
        assert_eq!(
            HANDS_7D_7C_V_6S_6H,
            SortedHeadsUp::new(Two::HAND_6S_6H, Two::HAND_7D_7C)
        );
        assert_eq!(
            HANDS_7D_7C_V_6S_6H,
            SortedHeadsUp::new(Two::HAND_7D_7C, Two::HAND_6S_6H)
        );
    }

    #[test]
    fn contains() {
        assert!(HANDS_7D_7C_V_6S_6H.contains(&Two::HAND_6S_6H));
        assert!(HANDS_7D_7C_V_6S_6H.contains(&Two::HAND_7D_7C));
        assert!(!HANDS_7D_7C_V_6S_6H.contains(&Two::HAND_7S_7C));
    }

    #[test]
    fn is_higher() {
        assert!(HANDS_7D_7C_V_6S_6H.is_higher(&Two::HAND_7D_7C));
        assert!(!HANDS_7D_7C_V_6S_6H.is_higher(&Two::HAND_6S_6H));
        assert!(!HANDS_7D_7C_V_6S_6H.is_higher(&Two::HAND_7S_7C));
    }

    #[test]
    fn is_lower() {
        assert!(HANDS_7D_7C_V_6S_6H.is_lower(&Two::HAND_6S_6H));
        assert!(!HANDS_7D_7C_V_6S_6H.is_lower(&Two::HAND_7D_7C));
        assert!(!HANDS_7D_7C_V_6S_6H.is_lower(&Two::HAND_7S_7C));
    }

    /// Wow, this test caused a panic:
    ///
    /// ```
    /// use pkcore::util::data::TestData;
    /// assert_eq!(TestData::the_hand_sorted_headsup().wins(), TestData::wins_the_hand());
    /// ```
    ///
    /// Let's try it a different way...
    ///
    /// ```
    /// use pkcore::util::data::TestData;
    /// use pkcore::util::wincounter::win::Win;
    /// assert_eq!(
    ///     TestData::the_hand_sorted_headsup().wins().wins_for(Win::FIRST),
    ///     TestData::wins_the_hand().wins_for(Win::FIRST)
    /// );
    /// ```
    ///
    /// Let's leave this test to fail for now, just so we don't forget it.
    #[test]
    fn wins() {
        assert_eq!(
            TestData::the_hand_sorted_headsup()
                .wins()
                .wins_for(Win::FIRST),
            TestData::wins_the_hand().wins_for(Win::FIRST)
        );
    }

    /// Here's the original test that panics, just for fun. I love it's error message:
    ///
    /// ```txt
    /// t": "thread 'arrays::matchups::sorted_heads_up::arrays__matchups__sorted_heads_up::wins_panic'
    /// panicked at 'assertion failed: `(left == right)`\n  left: `Wins([])`,\n right:
    /// `Wins([1, 1, 1, 1, 1, 1 ...
    /// ```
    ///
    /// for a few million entries. Run it if you want to see it. Let us ignore this test, shall we.
    /// See `docs/data/stacktrace.txt` for the full error.
    ///
    /// In hindsight, maybe deriving Eq, PartialEq on Wins wasn't such a good idea. Let's remove
    /// them, shall we...? Here';s the test for posterity's sake.
    ///
    /// ```txt
    /// #[test]
    /// #[ignore]
    /// fn wins_panic() {
    ///     assert_eq!(TestData::the_hand_sorted_headsup().wins(), TestData::wins_the_hand());
    /// }
    /// ```
    ///
    /// Moving on...
    #[test]
    fn display() {
        assert_eq!(
            "6♠ 6♥ - 5♦ 5♣",
            TestData::the_hand_sorted_headsup().to_string()
        );
    }

    #[test]
    fn pile__to_vec() {
        assert_eq!(
            HANDS_7D_7C_V_6S_6H.to_vec(),
            vec![
                Card::SEVEN_DIAMONDS,
                Card::SEVEN_CLUBS,
                Card::SIX_SPADES,
                Card::SIX_HEARTS
            ]
        );
    }

    /// I don't believe that I need this test. The foundations are already tested. Still, I like
    /// doing double checks. Part of me is just like how cool is it that I can even do this?!
    #[test]
    fn pile__remaining() {
        assert_eq!(HANDS_7D_7C_V_6S_6H.remaining().sort().to_string(), "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 6♣ 5♣ 4♣ 3♣ 2♣");
    }

    #[test]
    fn suit_shift() {
        let expected = SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D);
        assert_eq!(HANDS_7D_7C_V_6S_6H.shift_suit_down(), expected);
    }

    #[test]
    fn shifty__other_shifts() {
        let mut expected = HashSet::new();
        expected.insert(SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D));
        expected.insert(SortedHeadsUp::new(Two::HAND_7S_7H, Two::HAND_6D_6C));
        expected.insert(SortedHeadsUp::new(Two::HAND_7H_7D, Two::HAND_6S_6C));

        let actual = HANDS_7D_7C_V_6S_6H.other_shifts();

        assert_eq!(expected, actual);
    }

    /// 7♦ 7♣ - 6♠ 6♥
    /// 7♠ 7♣ - 6♥ 6♦
    /// 7♠ 7♥ - 6♦ 6♣
    /// 7♥ 7♦ - 6♠ 6♣
    #[test]
    fn shifty__shifts() {
        let mut expected = HashSet::new();
        expected.insert(SortedHeadsUp::new(Two::HAND_7D_7C, Two::HAND_6S_6H));
        expected.insert(SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D));
        expected.insert(SortedHeadsUp::new(Two::HAND_7S_7H, Two::HAND_6D_6C));
        expected.insert(SortedHeadsUp::new(Two::HAND_7H_7D, Two::HAND_6S_6C));

        let actual = HANDS_7D_7C_V_6S_6H.shifts();

        assert_eq!(expected, actual);
    }

    #[test]
    fn try_from() {
        let v = vec![Two::HAND_6S_6H, Two::HAND_7D_7C];

        assert_eq!(HANDS_7D_7C_V_6S_6H, SortedHeadsUp::try_from(v).unwrap());
    }

    #[test]
    fn try_from__error() {
        assert_eq!(
            PKError::NotEnoughHands,
            SortedHeadsUp::try_from(vec![Two::HAND_6S_6H]).unwrap_err()
        );
        assert_eq!(
            PKError::TooManyHands,
            SortedHeadsUp::try_from(vec![Two::HAND_6S_6H, Two::HAND_7S_7C, Two::HAND_2D_2C])
                .unwrap_err()
        );
    }
}
