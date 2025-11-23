use crate::analysis::store::bcm::binary_card_map::BC_RANK_HASHMAP;
use crate::analysis::store::db::hup::HUPResult;
use crate::analysis::the_nuts::TheNuts;
use crate::arrays::five::Five;
use crate::arrays::matchups::masked::{MASKED_UNIQUE, Masked};
use crate::arrays::seven::Seven;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::wins::Wins;
use crate::{PKError, Pile, Shifty, SuitShift};
use csv::{Reader, WriterBuilder};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::str::FromStr;

pub static SORTED_HEADS_UP_UNIQUE: std::sync::LazyLock<HashSet<SortedHeadsUp>> = std::sync::LazyLock::new(|| {
    let mut hs: HashSet<SortedHeadsUp> = HashSet::new();
    for v in Cards::deck().combinations(2) {
        let hero = Two::try_from(v.as_slice()).unwrap_or_default();
        for r in hero.remaining().combinations(2) {
            let villain = Two::try_from(r.as_slice()).unwrap_or_default();
            hs.insert(SortedHeadsUp::new(hero, villain));
        }
    }
    hs
});
pub static SORTED_HEADS_UP_UNIQUE_TYPE_ONE: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_one));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_TWO_A: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_two_a));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_TWO_B: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_two_b));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_TWO_C: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_two_c));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_TWO_D: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_two_d));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_TWO_E: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_two_e));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_THREE: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_three));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_FOUR: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_four));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_A: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_five_a));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_B: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_five_b));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_C: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_five_c));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_D: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_five_d));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_SIX_A: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_six_a));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_SIX_B: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_six_b));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_SEVEN: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_seven));
pub static SORTED_HEADS_UP_UNIQUE_TYPE_EIGHT: std::sync::LazyLock<HashSet<SortedHeadsUp>> =
    std::sync::LazyLock::new(|| Masked::filter_into_shu(&MASKED_UNIQUE, Masked::is_type_eight));

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

    /// # Errors
    ///
    /// Throws `PKError::SqlError` if unable to select from db.
    pub fn hup_result(&self, conn: &Connection) -> Result<HUPResult, PKError> {
        HUPResult::from_db(conn, &self.higher, &self.lower)
    }

    /// ## Aside
    ///
    /// Not sure if mixing the connection to the data struct is Kosher, but I can refactor later.
    /// Yeah, right.
    ///
    /// TODO: My eyes!!! My eyes!!! They burn!!! Refactor out this pollution of the struct space, you fiend.
    ///
    /// # Errors
    ///
    /// If the connection to the database fails, or if the query fails.
    pub fn hup_result_from_shift(&self, _conn: &Connection) -> Result<HUPResult, PKError> {
        let _shifts = self.shifts();
        todo!()
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

    /// Renaming `all_possible()` to `unique()`.
    ///
    /// OK, we are down the rabbit hole here. I've added `Shifty.other_shifts()` so that I can
    /// easily create a distinct `HashSet` of possible `SortedHeadsUp`s. I've been obsessed with this
    /// idea of shifting optimization for a while now, and so I might as well see how it plays out.
    ///
    /// This is one of the dangers of programming solo. High risk of driving into a ditch, but also
    /// you can come up with some really cool scheit.
    ///
    /// ## The Big Test
    ///
    /// The big test of this function will be if when I distill unique down to distinct and back
    /// up again, there should be the same collection of unique matchups.
    ///
    /// Here's our first stab at a `distinct()` function.
    ///
    /// It's a little wheels withing wheels for my taste, but if it works, it will be cool AF. Plus,
    /// come on, shiftshu is a great name for a variable.
    ///
    /// Let's create a variant of `examples/generate_all_possible_shu.rs` for distinct values. For this
    /// I am going to need to update our `generate_csv` method to be able to pass in collections of
    /// shus.
    ///
    /// Taking bets on what it actually does.
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
    /// use pkcore::{PKError, Shifty};
    /// pub fn distinct() -> Result<HashSet<SortedHeadsUp>, PKError> {
    ///   let mut hs = SortedHeadsUp::unique()?;
    ///
    ///   let v = Vec::from_iter(hs.clone());
    ///   for shu in &v {
    ///     if hs.contains(shu) {
    ///       let shifts = shu.shifts();
    ///       for shiftshu in Vec::from_iter(shifts) {
    ///         if hs.contains(shu) {
    ///           hs.remove(&shiftshu);
    ///         }
    ///       }
    ///     }
    ///   }
    ///
    ///   Ok(hs)
    /// }
    /// ```
    ///
    /// And if you bet that it returns a completely empty collection, you would be correct. __Sad
    /// trombone sound.__
    ///
    /// Let us tweak it a little bit... shall we?
    ///
    /// For this version I've added `println!("{}", hs.len());` to both versions of the generate
    /// examples to get a count. Again, place your bets on what the count will be for our distinct
    /// friend here...
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
    /// use pkcore::{PKError, Shifty};
    ///
    /// pub fn distinct() -> Result<HashSet<SortedHeadsUp>, PKError> {
    ///   let mut unique = SortedHeadsUp::unique()?;
    ///   let mut distinct: HashSet<SortedHeadsUp> = HashSet::new();
    ///
    ///   let v = Vec::from_iter(unique.clone());
    ///   for shu in &v {
    ///     if unique.contains(shu) {
    ///       distinct.insert(*shu);
    ///       let shifts = shu.shifts();
    ///       for shiftshu in Vec::from_iter(shifts) {
    ///         if unique.contains(shu) {
    ///           unique.remove(&shiftshu);
    ///         }
    ///       }
    ///     }
    ///   }
    ///
    ///   Ok(distinct)
    /// }
    /// ```
    ///
    /// If you bet 451,524 entries, you would be a winner. The unique version generated 812,175
    /// matchups, which I find interesting.
    ///
    /// Now, for this I am going to do a spot check on our little distinct results here, violating
    /// my primary rule of doing a manual test over automation. Now, technically it's more of a
    /// value than a rule, so that gives me a get out of developer jail free card.
    ///
    /// Here are the four records I am going to spot check on. If things are working correctly,
    /// there should only be one of them in the file.
    ///
    /// ```csv
    /// 7♦ 7♣,6♠ 6♥
    /// 7♠ 7♣,6♥ 6♦
    /// 7♠ 7♥,6♦ 6♣
    /// 7♥ 7♦,6♠ 6♣
    /// ```
    ///
    /// Alas, the first two are present. It's closer to what we wanted, but not exactly what we
    /// wanted. This probably explains why there were a lot more records there then we thought there
    /// would be.
    ///
    /// Let's distill down the problem, and write some unit tests, like we should have right from
    /// the beginning.
    ///
    /// Let's add `remove_shifts()`, and test drive it.
    ///
    /// __...one hour later...__
    ///
    /// ```txt
    /// pub fn distinct() -> Result<HashSet<SortedHeadsUp>, PKError> {
    ///    let mut unique = SortedHeadsUp::unique()?;
    ///
    ///    let v = Vec::from_iter(unique.clone());
    ///    for shu in &v {
    ///      if unique.contains(shu) {
    ///        shu.remove_shifts(&mut unique)
    ///      }
    ///    }
    ///
    ///    Ok(unique)
    ///  }
    /// ```
    ///
    /// Wow...this brought our distinct results length down to 202,800. This is more of what I
    /// expected.
    ///
    /// Now... how do we prove it?
    ///
    /// Our manual spot check passes now. Our of our four shifts, only one is in the result:
    /// `7♠ 7♥,6♦ 6♣`. What about our two gapped matchups: `7♠ 7♦,6♥ 6♣` & `7♥ 7♣,6♠ 6♦`? There
    /// should be only one of them.
    ///
    /// Sheit... neither is there.
    ///
    /// OK, so after our revision of `other_shifts()` we now have 203,294 distinct matchups. Our
    /// spot check now passes.
    ///
    /// Searching for "[203,294 distinct holdem heads up matchups]" returns
    /// [this response](https://poker.stackexchange.com/questions/5682/distinct-head-to-head-match-ups-in-holdem)
    /// on `StackExchange`:
    ///
    /// ===
    /// Note 1 in the article on Hold'em Odds elaborates on this a bit further:
    ///
    /// | [Note 1] By removing reflection and applying aggressive search tree pruning, it is possible to reduce the number of unique head-to-head hand combinations from 207,025 to 47,008. Reflection eliminates redundant calculations by observing that given hands `h_1` and `h_2`, if `w_1` is the probability of `h_1` beating `h_2` in a showdown and s is the probability of `h_1` splitting the pot with `h_2`, then the probability `w_2` of `h_2` beating `h_1` is `w_2 = 1 - (s + w_1)`, thus eliminating the need to evaluate `h_2` against `h_1`. Pruning is possible, for example, by observing that Q♥J♥ has the same chance of winning against both 8♦7♣ and 8♦7♠ (but not the same probability as against 8♥7♣ because sharing the heart affects the flush possibilities for each hand).
    ///
    /// Your thinking was correct that 169x1225 doesn't make sense. The actual number is less than 169x1225, though not quite as small as 169x169. 169x278 ≈ 47,008.
    ///
    /// With two cards there are only two unique 'suits' possible. Hands are either suited or off suited.
    ///
    /// Adding two more cards gives more combinations of suits, now we can have the following suit possibilities:
    ///
    /// 1. 1111 - suited, suited, same suit
    /// 2. 1112 - suited, off suit, sharing suit
    /// 3. 1122 - suited, suited, different suits
    /// 4. 1123 - suited, off suit, different suits
    /// 5. 1223 - off suit, off suit, sharing one suit
    /// 6. 1212 - off suit, off suit, sharing both suits
    /// 7. 1234 - off suit, off suit, sharing no suits
    ///
    /// Due to symmetry 1123 is the same as 2311 is the same as 3211; we ignore all symmetrical possibilities.
    ///
    /// This doesn't fully explain the actual number, though it gets you most of the way there and gives a
    /// mental model of the possible combinations of suits with two hands.
    /// ===
    ///
    /// At least that confirms our number, but it also shows us that this is still too many. I am
    /// happy with our results given that we have applied no actual math to the problem, but have
    /// instead focused entirely on simple brute force of the domain.
    ///
    /// This does uncover how a major gap in my skills makes the work harder. While it is rare that
    /// math saves your ass, when it does, it really really saves your ass.
    ///
    /// One of my favorite interactions as a developer was listening to an engineer of a major
    /// automotive manufacturer explain the math behind determining the temperature inside of a
    /// vehicle. It's a surprisingly difficult problem, since the temperature is based on the ouyside
    /// of the car, and the sensors are inside the car.
    ///
    /// Another instance was when I was testing an early prototype of a vehicle and the gas indicator
    /// kept jumping wildly up and down. This was because the liquid was sloshing all around as we
    /// drove and they hadn't finished working out the sensors for it.
    ///
    /// Programming on actual things instead of just doing stuff for web pages is a really life
    /// changing experience. It says a lot about our industry, and our society in general, that while
    /// the work is much more challenging and generally much more significant, the jobs themselves
    /// usually pay much less than the lastest silicone valley hype fest for some new way to share
    /// pictures of your cat on the blockchain.
    ///
    /// We're going to need to break down the math. Can you test drive math? Let's find out.
    ///
    /// # Errors
    ///
    /// If a deck isn't divisible by 2, which shouldn't happen. Maybe if we add jokers some day.
    pub fn distinct() -> Result<HashSet<SortedHeadsUp>, PKError> {
        let mut unique = SORTED_HEADS_UP_UNIQUE.clone();

        let v = Vec::from_iter(unique.clone());
        for shu in &v {
            if unique.contains(shu) {
                shu.remove_shifts(&mut unique);
            }
        }

        Ok(unique)
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
    /// and `unwrap()` like the lazy f I am. Really need to figure out a good way to deal with this.
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
    /// How it works:
    ///
    /// ```txt
    /// let distinct = Masked::distinct();
    /// SortedHeadsUp::generate_csv(
    ///     "generated/distinct_masked_shus.csv",
    ///     Masked::into_shus(&distinct),
    /// )
    /// .expect("TODO: panic message");
    /// ```
    ///
    /// # Errors
    ///
    /// When it can't write to path.
    ///
    /// # Panics
    ///
    /// When can't write to file system
    pub fn generate_csv(path: &str, shus: HashSet<SortedHeadsUp>) -> Result<(), Box<dyn std::error::Error>> {
        let mut v = Vec::from_iter(shus);
        v.sort();
        v.reverse();
        let mut wtr = WriterBuilder::new().has_headers(true).from_path(path)?;
        for shu in &v {
            wtr.serialize(shu)?;
        }
        wtr.flush()?;
        Ok(())
    }

    /// # Errors
    ///
    /// * Throws `PKError::InvalidBinaryFormat` if the csv file is corrupted.
    /// * Throws `PKError::Fubar` if unable to open at all.
    pub fn read_csv(path: &str) -> Result<Vec<SortedHeadsUp>, PKError> {
        match File::open(path) {
            Ok(file) => {
                let mut rdr = Reader::from_reader(file);
                let mut v = Vec::new();
                for hup in rdr.deserialize::<SortedHeadsUp>() {
                    match hup {
                        Ok(r) => v.push(r),
                        Err(_) => {
                            return Err(PKError::InvalidBinaryFormat);
                        }
                    }
                }
                Ok(v)
            }
            Err(_) => Err(PKError::Fubar),
        }
    }

    #[must_use]
    pub fn invert_suits(&self) -> Self {
        SortedHeadsUp::new(self.higher.invert_suits(), self.lower.invert_suits())
    }

    /// This should be interesting. Certainly testable.
    ///
    /// First we start with the bare signature for the function:
    ///
    /// ```txt
    /// pub fn remove_shifts(&self, from: &mut HashSet<SortedHeadsUp>) {
    ///   todo!()
    /// }
    /// ```
    ///
    /// A couple things of note. This is a method with side effects. That's something to watch
    /// out for. When it's the best tool for the job, go right ahead, but just be careful. Also,
    /// not that we have a &self in the sig, which may, or may not, bite us in the ass. __We shall
    /// see... 😈__
    ///
    /// Here's our sure to fail test after letting the compiler guide us:
    ///
    /// ```txt
    /// #[test]
    /// fn remove_shifts() {
    ///     let mut hs = SortedHeadsUp::unique().unwrap();
    ///     let shu = HANDS_7D_7C_V_6S_6H;
    ///
    ///     shu.remove_shifts(&mut hs);
    /// }
    /// ```
    ///
    /// I'll confess that the line `shu.remove_shifts(&mut hs)` surprised me. I was a tad surprised
    /// about needing the `&mut` prefix. It's been a long time since I passed in a mutable
    /// reference.
    ///
    /// Let's add a little more heft to the test.
    ///
    /// ```txt
    /// #[test]
    /// fn remove_shifts() {
    ///     let mut hs = SortedHeadsUp::unique().unwrap();
    ///     let shu = HANDS_7D_7C_V_6S_6H;
    ///
    ///     shu.remove_shifts(&mut hs);
    ///
    ///     assert!(!hs.contains(&SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D)));
    /// }
    /// ```
    ///
    /// This makes the test pass:
    ///
    /// ```txt
    ///     pub fn remove_shifts(&self, from: &mut HashSet<SortedHeadsUp>) {
    ///         for shift in self.other_shifts() {
    ///             if from.contains(&shift) {
    ///                 from.remove(&shift);
    ///             }
    ///         }
    ///     }
    /// ```
    ///
    /// Let's add the other two shifts to the test.
    ///
    /// ```
    /// use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
    /// use pkcore::arrays::matchups::masks::suit_texture::SuitTexture::Type1234;
    /// use pkcore::arrays::two::Two;
    ///
    /// let mut hs = SortedHeadsUp::unique().unwrap();
    /// let shu = SortedHeadsUp {
    ///   higher: Two::HAND_7D_7C,
    ///   lower: Two::HAND_6S_6H,
    /// };
    ///
    /// shu.remove_shifts(&mut hs);
    ///
    /// assert!(!hs.contains(&SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D)));
    /// assert!(!hs.contains(&SortedHeadsUp::new(Two::HAND_7S_7H, Two::HAND_6D_6C)));
    /// assert!(!hs.contains(&SortedHeadsUp::new(Two::HAND_7H_7D, Two::HAND_6S_6C)));
    /// ```
    ///
    /// It works. Huzzah!
    ///
    /// Now, can we leverage this in refactoring our distinct function? How about we use the exact
    /// same test against it?
    pub fn remove_shifts(&self, from: &mut HashSet<SortedHeadsUp>) {
        for shift in self.other_shifts() {
            if from.contains(&shift) {
                from.remove(&shift);
            }
        }
    }

    #[must_use]
    pub fn suit_binaries(&self) -> (u32, u32) {
        (self.higher.suit_binary(), self.lower().suit_binary())
    }

    #[must_use]
    pub fn get_letter_index(&self) -> String {
        let higher = self.higher().get_letter_index();
        let lower = self.lower().get_letter_index();
        format!("{higher} - {lower}")
    }

    /// For now, returning default is all part of the blueprint. Still, let's write a test
    /// that fails that we can ignore later on when we get everything wired in.
    ///
    /// Now, let's try refactoring `HUPResult::from(SortedHeadsUp)` to use this method
    /// to calculate wins. Running a validation test will be slow, but it will be worth it.
    ///
    /// TODO: Is there any reason to dive into [`std::sync::atomic::Ordering`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)?
    ///
    /// Test moved to `tests/heavy_tests.rs`.
    ///
    /// # Errors
    ///
    /// Throws `PKError` when unable to cast cards correctly.
    pub fn wins(&self) -> Result<Wins, PKError> {
        let mut wins = Wins::default();

        for combo in self.remaining().combinations(5) {
            let (high7, low7) = self.sevens(Five::try_from(combo)?)?;

            let high_rank = BC_RANK_HASHMAP.get(&high7.bard()).ok_or(PKError::InvalidHand)?;
            let low_rank = BC_RANK_HASHMAP.get(&low7.bard()).ok_or(PKError::InvalidHand)?;

            match high_rank.rank.cmp(&low_rank.rank) {
                Ordering::Less => wins.add(Win::FIRST),
                Ordering::Greater => wins.add(Win::SECOND),
                Ordering::Equal => wins.add(Win::FIRST | Win::SECOND),
            }
        }

        Ok(wins)
    }

    /// # Errors
    ///
    /// Throws `PKError` when an invalid cast occurs due to bad `Card` arrays passed in.
    pub fn sevens(&self, five: Five) -> Result<(Seven, Seven), PKError> {
        let high7 = Seven::from_case_at_deal(self.higher, five)?;
        let low7 = Seven::from_case_at_deal(self.lower, five)?;
        Ok((high7, low7))
    }
}

impl Display for SortedHeadsUp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.higher, self.lower)
    }
}

impl From<HUPResult> for SortedHeadsUp {
    fn from(hup: HUPResult) -> Self {
        SortedHeadsUp::new(
            Two::try_from(hup.higher).unwrap_or_else(|_| Two::default()),
            Two::try_from(hup.lower).unwrap_or_else(|_| Two::default()),
        )
    }
}

impl From<Masked> for SortedHeadsUp {
    fn from(masked: Masked) -> Self {
        SortedHeadsUp::new(masked.shu.higher, masked.shu.lower)
    }
}

impl FromStr for SortedHeadsUp {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SortedHeadsUp::try_from(Cards::from_str(s)?)
    }
}

impl Pile for SortedHeadsUp {
    fn card_at(self, _index: usize) -> Option<Card> {
        todo!()
    }

    /// Shoot. Forgot about my frequency mask idea. Still has potential, but later.
    fn clean(&self) -> Self {
        todo!()
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
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

impl Shifty for SortedHeadsUp {
    fn shifts(&self) -> HashSet<Self>
    where
        Self: Sized,
    {
        Masked::from(*self)
            .shifts()
            .into_iter()
            .map(SortedHeadsUp::from)
            .collect()

        // let mut shifts: HashSet<Self> = HashSet::new();
        // for mask in masks {
        //     shifts.insert(mask.shu);
        // }
        // shifts
    }
}

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

impl TryFrom<&HUPResult> for SortedHeadsUp {
    type Error = PKError;

    fn try_from(hup: &HUPResult) -> Result<Self, Self::Error> {
        let higher_two = Two::try_from(hup.higher).unwrap_or_else(|_| Two::default());
        let lower_two = Two::try_from(hup.lower).unwrap_or_else(|_| Two::default());
        Ok(SortedHeadsUp::new(higher_two, lower_two))
    }
}

impl TryFrom<Vec<Two>> for SortedHeadsUp {
    type Error = PKError;

    fn try_from(v: Vec<Two>) -> Result<Self, Self::Error> {
        match v.len() {
            0..=1 => Err(PKError::NotEnoughHands),
            2 => Ok(SortedHeadsUp::new(
                *v.first().ok_or(PKError::InvalidHand)?,
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
                **v.first().ok_or(PKError::InvalidHand)?,
                **v.get(1).ok_or(PKError::InvalidHand)?,
            )),
            _ => Err(PKError::TooManyHands),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__sorted_heads_up_tests {
    use super::*;
    use crate::util::data::TestData;

    const HANDS_7D_7C_V_6S_6H: SortedHeadsUp = SortedHeadsUp {
        higher: Two::HAND_7D_7C,
        lower: Two::HAND_6S_6H,
    };

    #[test]
    fn unique() {
        assert_eq!(812175, SORTED_HEADS_UP_UNIQUE.len());
    }

    #[test]
    fn unique_types() {
        assert_eq!(8580, SORTED_HEADS_UP_UNIQUE_TYPE_ONE.len());
        assert_eq!(10296, SORTED_HEADS_UP_UNIQUE_TYPE_TWO_A.len());
        assert_eq!(32604, SORTED_HEADS_UP_UNIQUE_TYPE_TWO_B.len());
        assert_eq!(29172, SORTED_HEADS_UP_UNIQUE_TYPE_TWO_C.len());
        assert_eq!(29172, SORTED_HEADS_UP_UNIQUE_TYPE_TWO_C.len());
        assert_eq!(32604, SORTED_HEADS_UP_UNIQUE_TYPE_TWO_D.len());
        assert_eq!(29172, SORTED_HEADS_UP_UNIQUE_TYPE_TWO_E.len());
        assert_eq!(36504, SORTED_HEADS_UP_UNIQUE_TYPE_THREE.len());
        assert_eq!(81120, SORTED_HEADS_UP_UNIQUE_TYPE_FOUR.len());
        assert_eq!(88608, SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_A.len());
        assert_eq!(73008, SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_B.len());
        assert_eq!(89544, SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_C.len());
        assert_eq!(65208, SORTED_HEADS_UP_UNIQUE_TYPE_FIVE_D.len());
        assert_eq!(39936, SORTED_HEADS_UP_UNIQUE_TYPE_SIX_A.len());
        assert_eq!(33072, SORTED_HEADS_UP_UNIQUE_TYPE_SIX_B.len());
        assert_eq!(85683, SORTED_HEADS_UP_UNIQUE_TYPE_SEVEN.len());
        assert_eq!(77064, SORTED_HEADS_UP_UNIQUE_TYPE_EIGHT.len());
    }

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

    /// This test makes no sense...
    ///
    /// Refactored. Much uglier but more accurate. You don't know which of the shifts will be pulled
    /// out first, so anyone of them could be the last shu standing.
    ///
    /// UPDATE: Suddenly this test is causing a core dump (sic) running 1.73.0-nightly. Running it create:
    ///
    /// ```txt
    /// 4♦\n8♣ 6♦ - 6♣ 4♥\n8♦ 6♥ - 6♦ 4♠\nA♥ K♥
    /// ```
    ///
    /// I'm going to ignore this test because it causes a wacky dump in `CLion`.
    ///
    /// This has now stopped dumping on `Build #CL-232.8660.186, built on July 25, 2023`. Removing
    /// the ignore.
    #[test]
    #[ignore]
    fn distinct() {
        let distinct = SortedHeadsUp::distinct().unwrap();
        let mut holding = HashSet::new();

        if distinct.contains(&HANDS_7D_7C_V_6S_6H) {
            holding.insert(HANDS_7D_7C_V_6S_6H);
        }
        if distinct.contains(&SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D)) {
            holding.insert(SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D));
        }
        if distinct.contains(&SortedHeadsUp::new(Two::HAND_7S_7H, Two::HAND_6D_6C)) {
            holding.insert(SortedHeadsUp::new(Two::HAND_7S_7H, Two::HAND_6D_6C));
        }
        if distinct.contains(&SortedHeadsUp::new(Two::HAND_7H_7D, Two::HAND_6S_6C)) {
            holding.insert(SortedHeadsUp::new(Two::HAND_7H_7D, Two::HAND_6S_6C));
        }

        assert_eq!(holding.len(), 1);
    }

    #[test]
    fn invert_suits() {
        assert_eq!(
            SortedHeadsUp::new(Two::HAND_8S_7H, Two::HAND_8H_7S),
            SortedHeadsUp::new(Two::HAND_8S_7H, Two::HAND_8H_7S).invert_suits()
        );
        assert_eq!(
            SortedHeadsUp::new(Two::HAND_AS_KD, Two::HAND_8H_7S),
            SortedHeadsUp::new(Two::HAND_AD_KS, Two::HAND_8S_7H).invert_suits()
        );
    }

    #[test]
    fn remove_shifts() {
        let mut hs = SortedHeadsUp::unique().unwrap();
        let shu = HANDS_7D_7C_V_6S_6H;

        shu.remove_shifts(&mut hs);

        assert!(!hs.contains(&SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D)));
        assert!(!hs.contains(&SortedHeadsUp::new(Two::HAND_7S_7H, Two::HAND_6D_6C)));
        assert!(!hs.contains(&SortedHeadsUp::new(Two::HAND_7H_7D, Two::HAND_6S_6C)));
    }

    #[test]
    fn get_letter_index() {
        assert_eq!("7D 7C - 6S 6H", HANDS_7D_7C_V_6S_6H.get_letter_index());
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
        assert_eq!("6♠ 6♥ - 5♦ 5♣", TestData::the_hand_sorted_headsup().to_string());
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
        assert_eq!(
            HANDS_7D_7C_V_6S_6H.remaining().sort().to_string(),
            "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 6♣ 5♣ 4♣ 3♣ 2♣"
        );
    }

    #[test]
    fn suit_shift() {
        let expected = SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D);
        assert_eq!(HANDS_7D_7C_V_6S_6H.shift_suit_down(), expected);
    }

    #[test]
    fn suit_shift__opposite() {
        let shu = SortedHeadsUp::new(Two::HAND_AS_KD, Two::HAND_KH_TC);
        let expected = SortedHeadsUp::new(Two::HAND_AD_KS, Two::HAND_KC_TH);

        let actual = shu.opposite();

        assert_eq!(expected, actual);

        // TODO: Why doesn't this work. I am too befuddled to dive into it now. Need to get preflop done.
        // let mut alt = shu.shift_suit_up();
        // alt = shu.shift_suit_up();
        // println!("{alt}");
        // println!("{actual}");
        // assert_eq!(expected, alt);
    }

    #[test]
    fn shifty__other_shifts() {
        let mut expected = HashSet::new();
        expected.insert(SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D));
        expected.insert(SortedHeadsUp::new(Two::HAND_7S_7H, Two::HAND_6D_6C));
        expected.insert(SortedHeadsUp::new(Two::HAND_7H_7D, Two::HAND_6S_6C));
        expected.insert(SortedHeadsUp::new(Two::HAND_7S_7D, Two::HAND_6H_6C));
        expected.insert(SortedHeadsUp::new(Two::HAND_7H_7C, Two::HAND_6S_6D));

        let actual = HANDS_7D_7C_V_6S_6H.other_shifts();

        assert_eq!(5, actual.len());
        assert_eq!(expected, actual);
    }

    /// 7♦ 7♣,6♠ 6♥
    /// 7♠ 7♣,6♥ 6♦
    /// 7♠ 7♥,6♦ 6♣
    /// 7♥ 7♦,6♠ 6♣
    #[test]
    fn shifty__shifts() {
        let mut expected = HashSet::new();
        expected.insert(SortedHeadsUp::new(Two::HAND_7D_7C, Two::HAND_6S_6H));
        expected.insert(SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D));
        expected.insert(SortedHeadsUp::new(Two::HAND_7S_7H, Two::HAND_6D_6C));
        expected.insert(SortedHeadsUp::new(Two::HAND_7H_7D, Two::HAND_6S_6C));
        expected.insert(SortedHeadsUp::new(Two::HAND_7S_7D, Two::HAND_6H_6C));
        expected.insert(SortedHeadsUp::new(Two::HAND_7H_7C, Two::HAND_6S_6D));

        let actual = HANDS_7D_7C_V_6S_6H.shifts();

        assert_eq!(6, actual.len());
        assert!(actual.contains(&SortedHeadsUp::new(Two::HAND_7D_7C, Two::HAND_6S_6H)));
        assert!(actual.contains(&SortedHeadsUp::new(Two::HAND_7S_7C, Two::HAND_6H_6D)));
        assert!(actual.contains(&SortedHeadsUp::new(Two::HAND_7S_7H, Two::HAND_6D_6C)));
        assert!(actual.contains(&SortedHeadsUp::new(Two::HAND_7H_7D, Two::HAND_6S_6C)));
        assert_eq!(expected, actual);
    }

    // No longer needed.
    // #[test]
    // fn shifty__shifts_gapped() {
    //     let first = SortedHeadsUp::new(Two::HAND_7S_7D, Two::HAND_6H_6C);
    //     let second = SortedHeadsUp::new(Two::HAND_7H_7C, Two::HAND_6S_6D);
    //     let mut expected = HashSet::new();
    //     expected.insert(first);
    //     expected.insert(second);
    //
    //     let actual = first.shifts();
    //
    //     assert_eq!(actual, expected);
    // }

    /// This test failed on the original version of this test.
    /// This test is no longer needed.
    // #[test]
    // fn shifty__other_shifts_gapped() {
    //     let first = SortedHeadsUp::new(Two::HAND_7S_7D, Two::HAND_6H_6C);
    //     let second = SortedHeadsUp::new(Two::HAND_7H_7C, Two::HAND_6S_6D);
    //     let mut expected = HashSet::new();
    //     expected.insert(second);
    //
    //     let actual = first.other_shifts();
    //
    //     assert_eq!(actual, expected);
    // }

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
            SortedHeadsUp::try_from(vec![Two::HAND_6S_6H, Two::HAND_7S_7C, Two::HAND_2D_2C]).unwrap_err()
        );
    }

    #[test]
    fn try_from__hup_result() {}
}
