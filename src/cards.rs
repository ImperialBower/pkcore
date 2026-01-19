use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::card::Card;
use crate::card_number::CardNumber;
use crate::cards_cell::CardsCell;
use crate::prelude::{BoxedCards, Boxes};
use crate::rank::Rank;
use crate::suit::Suit;
use crate::util::terminal::Terminal;
use crate::{Forgiving, PKError, Pile, SuitShift, TheNuts};
use indexmap::IndexSet;
use indexmap::set::{IntoIter, Iter};
use itertools::{Combinations, Itertools};
use rand::prelude::SliceRandom;
use rand::rng;
use rayon::iter::{IterBridge, ParallelBridge};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
use std::str::FromStr;
use strum::IntoEnumIterator;

pub static FIVE_CARD_COMBOS: std::sync::LazyLock<Combinations<IntoIter<Card>>> =
    std::sync::LazyLock::new(|| Cards::deck().combinations(5));

/// What are the contracts for Cards?
///
/// 1. Cards should be saved in order.
/// 2. Cards should be unique.
/// 3. Cards should be legitimate cards. (No blanks)
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Cards(pub IndexSet<Card>);

impl Cards {
    /// This is an example of my stupidity. I want to full the collection of `Cards` with blanks
    /// so that I can tell what needs to be dealt, but it's a damned `IndexSet`. There will always
    /// only be one.
    /// ```
    /// use pkcore::cards::Cards;
    ///
    /// let blanks = Cards::blanks(3);
    ///
    /// assert_eq!(blanks.len(), 1);
    /// assert_eq!(blanks.to_string(), "__");
    /// ```
    #[must_use]
    pub fn blanks(len: usize) -> Self {
        let mut i: IndexSet<Card> = IndexSet::new();
        for _ in 0..len {
            i.insert(Card::BLANK);
        }
        Cards(i)
    }

    /// ```
    /// use pkcore::cards::Cards;
    /// use pkcore::deck;
    ///
    /// let deck = Cards::deck();
    ///
    /// assert_eq!(deck!(), deck);
    /// assert_eq!(deck.len(), 52);
    /// assert_eq!(
    ///     deck.to_string(),
    ///     "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣"
    /// );
    /// ```
    #[must_use]
    pub fn deck() -> Cards {
        let mut cards = Cards::default();
        for card_number in CardNumber::iter() {
            cards.insert(Card::from(card_number as u32));
        }
        cards
    }

    /// TODO RF: :-P
    ///
    /// UPDATE: Originally had this doing sway remove with totally fucks with the order, grabbing the
    /// last item in the list and using it as a replacement. Yes. it's faster, but for me, useless.
    #[must_use]
    pub fn deck_minus(cards: &Cards) -> Cards {
        Cards::deck().into_iter().filter(|c| !cards.contains(c)).collect()
    }

    /// A Deck primed, is one where the initial cards are the ones passed in. This is to facilitate
    /// testing specific scenarios.
    ///
    /// This initial version of this code was particularly clunky:
    ///
    /// ```
    /// use pkcore::cards::Cards;
    ///
    /// pub fn deck_primed(cards: &Cards) -> Cards {
    ///     let deck_minus = Cards::deck_minus(cards);
    ///     let mut cloned = cards.clone();
    ///     cloned.append(&deck_minus);
    ///     cloned
    /// }
    /// ```
    ///
    /// TODO: Add the ability to pass in burn cards
    #[must_use]
    pub fn deck_primed(cards: &Cards) -> Cards {
        cards.clone().into_iter().chain(Cards::deck_minus(cards)).collect()
    }

    #[must_use]
    pub fn as_chunks(&self, chunk_size: usize) -> Vec<Vec<Card>> {
        self.to_vec().chunks(chunk_size).map(<[Card]>::to_vec).collect()
    }

    pub fn append(&mut self, appended: &Cards) {
        let mut to_append = appended.0.clone();
        self.0.append(&mut to_append);
    }

    /// DEFECT bad twos STEP 3
    ///
    /// This is how we got it to pass"
    ///
    /// ```
    /// use std::str::FromStr;
    /// use pkcore::arrays::two::Two;
    /// use pkcore::cards::Cards;
    /// use pkcore::PKError;
    ///
    /// fn as_twos(cards: Cards) -> Result<Vec<Two>, PKError> {
    ///     if !cards.divisible_by(2) {
    ///         return Err(PKError::InvalidCardCount);
    ///     }
    ///     let mut v: Vec<Two> = Vec::new();
    ///     let mut cards = cards.clone();
    ///     loop {
    ///         let c1 = match cards.draw_one() {
    ///             Ok(card) => card,
    ///             Err(_) => break,
    ///         };
    ///         let c2 = match cards.draw_one() {
    ///             Ok(card) => card,
    ///             Err(_) => break,
    ///         };
    ///         let two = Two::new(c1, c2)?;
    ///         v.push(two);
    ///     }
    ///     Ok(v)
    /// }
    ///
    /// ```
    /// While this works and makes the test pass, it generates the following
    /// clippy error:
    ///
    /// ```txt
    /// warning: this loop could be written as a `while let` loop
    ///    --> src/cards.rs:106:9
    ///     |
    /// 106 | /         loop {
    /// 107 | |             let c1 = match cards.draw_one() {
    /// 108 | |                 Ok(card) => card,
    /// 109 | |                 Err(_) => break,
    /// ...   |
    /// 116 | |             v.push(two);
    /// 117 | |         }
    ///     | |_________^ help: try: `while let Ok(card) = cards.draw_one() { .. }`
    ///     |
    ///     = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#while_let_loop
    ///     = note: `#[warn(clippy::while_let_loop)]` on by default
    /// ```
    ///
    /// STEP 4
    ///
    /// While I don't think that the clippy suggestion will really work since
    /// we need to get two `Cards`, not just one. Let's try a refactoring...
    ///
    /// BOOM! Ship it.
    ///
    /// Turns out that a slight change to the clippy suggestion worked perfectly.
    /// Clippy is your friend.
    ///
    /// # Errors
    ///
    /// Will return `PKError::InvalidCardCount` for an invalid index.
    pub fn as_twos(&self) -> Result<Vec<Two>, PKError> {
        if !self.divisible_by(2) {
            return Err(PKError::InvalidCardCount);
        }
        let mut v: Vec<Two> = Vec::new();
        let mut cards = self.clone();

        while let Ok(two_cards) = cards.draw(2) {
            let two = Two::try_from(two_cards)?;
            v.push(two);
        }
        Ok(v)
    }

    /// Confirms if `Cards` can be evenly divided by `x`.
    ///
    /// TODO: Refactor this to return the `Cards` as `BoxedCards`, instead of an examples
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let cards = cards!("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠");
    /// let byx = cards.by_x(2).unwrap();
    /// assert_eq!(cards!("6♠ 5♠"), cards.by_x(2).unwrap());
    /// assert!(cards.by_x(3).is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PKError::Misaligned` if the cards cannot be evenly divided by `x`.
    pub fn by_x(&self, x: usize) -> Result<Self, PKError> {
        if !self.divisible_by(x) {
            return Err(PKError::Misaligned);
        }

        let by = self.len() / x;

        let mut chunk = Vec::new();

        for i in 0..by {
            chunk = self.0.iter().skip(i * x).take(x).copied().collect::<Vec<Card>>();
        }

        Ok(Cards::from(chunk.as_slice()))
    }

    /// Collapse
    /// ```txt
    /// pub fn collapse(&self) -> u32 {
    ///     let mut result: u32 = 0;
    ///     for card in self.iter() {
    ///         result = result | card.as_u32();
    ///     }
    ///     result
    /// }
    /// ```
    pub fn combinations(&self, k: usize) -> Combinations<IntoIter<Card>> {
        self.0.clone().into_iter().combinations(k)
    }

    #[must_use]
    pub fn par_combinations(&self, k: usize) -> IterBridge<Combinations<IntoIter<Card>>> {
        self.0.clone().into_iter().combinations(k).par_bridge()
    }

    #[must_use]
    pub fn divisible_by(&self, x: usize) -> bool {
        (self.len() % x) == 0
    }

    /// # Errors
    ///
    /// Returns `PKError::NotEnoughCards` if not enough cards are available.
    pub fn draw(&mut self, number: usize) -> Result<Self, PKError> {
        if number > self.len() {
            Err(PKError::NotEnoughCards)
        } else {
            Ok(Cards(self.0.drain(0..number).collect()))
        }
    }

    #[must_use]
    pub fn draw_all(&mut self) -> Self {
        let l = self.len();
        Cards(self.0.drain(0..l).collect())
    }

    /// # Errors
    /// Returns `PKError::NotEnoughCards` if there are no more cards left.
    pub fn draw_one(&mut self) -> Result<Card, PKError> {
        match self.0.shift_remove_index(0) {
            Some(card) => Ok(card),
            None => Err(PKError::NotEnoughCards),
        }
    }

    /// # Errors
    ///
    /// Returns `PKError::NotEnoughCards` if not enough cards are available.
    pub fn draw_from_the_bottom(&mut self, number: usize) -> Result<Self, PKError> {
        let l = self.len();
        if number > l {
            Err(PKError::NotEnoughCards)
        } else {
            Ok(Cards(self.0.drain(l - number..l).collect()))
        }
    }

    /// One of the big problems with our Card data type is that it's just a binary number
    /// so it's hard to figure out what's going on with it. To help deal with this I try to
    /// add some methods just to help out with debugging.
    ///
    /// Later on, we might be able to use this for logging as a part of a larger system. Right now
    /// we're using println!, which is in itself a kind of technical debt. Usually, when I reach
    /// a point in a library where I think it's about ready to integrate into the larger crate
    /// community, I will search these out and replace them with actually log statements. For now
    /// though, I don't want to deal with it. Do what you can. Take your time. Perfection is a goal;
    /// never a reality.
    ///
    /// ASIDE: One of the best compliments I ever got from another developer was from the person
    /// I dislike more than any other in my career. _There was this one guy at a startup who tried
    /// to forge commands as if he was me from our servers to try to get me fired because I had
    /// the audacity to call him on his bullshit, but to be honest, he was doing me a favor by
    /// driving me out of that place._
    pub fn dump(&self) {
        for card in self.iter() {
            println!("{} {card}\n", card.bit_string_guided());
        }
    }

    #[must_use]
    pub fn filter_by_suit(&self, suit: Suit) -> Self {
        let filtered: Vec<Card> = self
            .clone()
            .into_iter()
            .filter(|card| card.get_suit() == suit)
            .collect();
        Cards::from(filtered)
    }

    /// Sets the card's paired bit to true for all cards in the collection.
    #[must_use]
    pub fn flag_paired(&self) -> Self {
        Cards::from(self.iter().map(Card::frequency_paired).collect::<Vec<_>>())
    }

    /// Sets the card's tripped bit to true for all cards in the collection.
    #[must_use]
    pub fn flag_tripped(&self) -> Cards {
        Cards::from(self.iter().map(Card::frequency_tripped).collect::<Vec<_>>())
    }

    /// Sets the card's quaded bit to true for all cards in the collection.
    #[must_use]
    pub fn flag_quaded(&self) -> Cards {
        Cards::from(self.iter().map(Card::frequency_quaded).collect::<Vec<_>>())
    }

    /// This function is most likely going to be a shit show. I could just cast everything over
    /// to my [cardpack.rs](https://github.com/ContractBridge/cardpack.rs) library where this is
    /// [already solved](https://github.com/ContractBridge/cardpack.rs/blob/main/src/cards/pile.rs#L448),
    /// but I'm trying to keep this library as dependency clean as possible. Plus, how can I
    /// refactor something if I just pass the work onto a library where that won't work?
    ///
    /// DEFECT: In git history original version fucks up on non weighted cards.
    ///
    /// The only time this is really needed is to display `Five` so that it sorts based on the
    /// `HandRank`.
    ///
    /// ## Many months later...
    ///
    /// Sure enough, I'm trying to figure out WTF with `Five.sort()` and this shit is getting in the
    /// way.
    #[must_use]
    pub fn frequency_weighted(&self) -> Cards {
        let mappy = self.map_by_rank();
        let mut cards = Cards::default();
        for rank in mappy.keys() {
            match mappy.get(rank) {
                None => {}
                Some(c) => match c.len() {
                    0 => {}
                    1 => cards.insert_all(c),
                    2 => cards.insert_all(&c.flag_paired()),
                    3 => cards.insert_all(&c.flag_tripped()),
                    _ => cards.insert_all(&c.flag_quaded()),
                },
            }
        }
        cards.0.sort();
        cards.0.reverse();
        cards
    }

    #[must_use]
    pub fn get(&self, card: &Card) -> Option<&Card> {
        self.0.get(card)
    }

    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<&Card> {
        self.0.get_index(index)
    }

    #[must_use]
    pub fn index_set(&self) -> &IndexSet<Card> {
        &self.0
    }

    /// Allows you to insert a `PlayingCard` provided it isn't blank.
    pub fn insert(&mut self, card: Card) -> bool {
        if card.contains_blank() {
            false
        } else {
            self.0.insert(card)
        }
    }

    pub fn insert_all(&mut self, cards: &Cards) {
        for card in cards.iter() {
            self.insert(*card);
        }
    }

    pub fn insert_at(&mut self, index: usize, card: Card) -> bool {
        if card.contains_blank() {
            return false;
        }

        let mut vec = self.to_vec();
        vec.insert(index, card);
        self.0 = IndexSet::from_iter(vec);
        true
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn iter(&self) -> Iter<'_, Card> {
        self.0.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn deal_from_the_bottom(&mut self) -> Option<Card> {
        self.0.pop()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let mut all_cards = Cards::deck();
    /// all_cards = all_cards.minus(&cards!("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠"));
    /// all_cards = all_cards.minus(&cards!("A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥"));
    ///
    /// assert_eq!("A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣", all_cards.to_string());
    ///
    /// ```
    #[must_use]
    pub fn minus(&self, cards: &Cards) -> Cards {
        self.iter().filter(|c| !cards.contains(c)).copied().collect()
    }

    pub fn remove(&mut self, card: &Card) -> bool {
        self.0.shift_remove(card)
    }

    #[must_use]
    pub fn shuffle(&self) -> Cards {
        let mut shuffled = self.clone();
        shuffled.shuffle_in_place();
        shuffled
    }

    pub fn shuffle_in_place(&mut self) {
        let mut rng = rng();
        let mut vec: Vec<_> = self.0.drain(..).collect();
        vec.shuffle(&mut rng);
        self.0.extend(vec);
    }

    /// We have uncovered a defect with out sort function. Ideally, it should sort with a higher
    /// weight given to `Suit` rather than `Rank` so that when I pass in `6♣ 7♠ 7♦ 8♦` and call
    /// `sort()` it should return `7♠ 8♦ 7♦ 6♣` since spades come before diamonds. Instead we get
    /// `8♦ 7♠ 7♦ 6♣`.
    ///
    /// This is because our modified Cactus Kev `Card` binary format places the `Rank` bit flag
    /// higher than the `Suit` flag. Remember how our Card bits are set:
    ///
    /// ```txt
    /// +--------+--------+--------+--------+
    /// |mmmbbbbb|bbbbbbbb|SHDCrrrr|xxpppppp|
    /// +--------+--------+--------+--------+
    ///
    /// p = prime number of rank (deuce=2,trey=3,four=5,...,ace=41)
    /// r = rank of card (deuce=0,trey=1,four=2,five=3,...,ace=12)
    /// SHDC = suit of card (bit turned on based on suit of card)
    /// b = bit turned on depending on rank of card
    /// m = Flags reserved for multiples of the same rank. Stripped for evals.
    /// ```
    ///
    /// In order for us to get the sort we want, we would have to arrange them thus:
    ///
    /// ```txt
    /// +--------+--------+--------+--------+
    /// |mmmSHDCb|bbbbbbbb|bbbbrrrr|xxpppppp|
    /// +--------+--------+--------+--------+
    /// ```
    ///
    /// This would be a major refactoring; one that we're not prepared to do right now. TBH.
    /// I question if we will ever need to do this. For now, I'm going to mark the issue with
    /// a technical debt TODO, add a test that I annotate as ignore, and call it a day.
    ///
    /// TODO TD: Update `Card` so that sort is `Suit` weighted first.
    #[must_use]
    pub fn sort(&self) -> Self {
        let mut c = self.clone();
        c.sort_in_place();
        c
    }

    pub fn sort_in_place(&mut self) {
        let mut sorted = Cards::default();
        for suit in [Suit::CLUBS, Suit::DIAMONDS, Suit::HEARTS, Suit::SPADES] {
            let mut s = self.filter_by_suit(suit);
            s.0.sort();
            sorted.insert_all(&s);
        }
        sorted.0.reverse();
        self.0 = sorted.0;
    }

    //region private functions

    fn map_by_rank(&self) -> HashMap<Rank, Cards> {
        // Why is this variable called mappy? Now that is a long and winding tale.
        // Many, many years ago, when I was in middle school in SF, me and my friends would
        // Play D&D, eat Georgio's pizza, and play video games at an ice cream show. The two
        // games they had were [Mr. Do!](https://en.wikipedia.org/wiki/Mr._Do!) and
        // [Mappy](https://en.wikipedia.org/wiki/Mappy). In honor of this nostalgia I try to
        // name any private variables of hashmaps after the mouse plagued police cat. _Aside:
        // Everytime [Wil Wheaton posts about his Mr. Do! machine](https://wilwheaton.net/2019/02/)
        // I let out a [Sheldonesque WHEATON!!!!](https://www.youtube.com/watch?v=bUWXjs2jPQI)
        // inside._
        //
        // BTW, if you are ever in the sunset district of SF, checkout Georgio's for dinner and
        // then stop by Toy Boat ice cream for dessert. No, they're not the shop with the
        // video games, which closed a while ago, but they are great.
        let mut mappy: HashMap<Rank, Cards> = HashMap::new();
        for rank in Rank::iter() {
            let pile: Vec<Card> = self.iter().copied().filter(|card| card.get_rank() == rank).collect();
            mappy.insert(rank, Cards::from(pile));
        }
        mappy
    }

    //endregion
}

impl BitAnd for Cards {
    type Output = Self;

    fn bitand(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}

impl BitAndAssign for Cards {
    fn bitand_assign(&mut self, _rhs: Self) {
        todo!()
    }
}

impl BitOr for Cards {
    type Output = Self;

    fn bitor(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}

impl BitOrAssign for Cards {
    fn bitor_assign(&mut self, _rhs: Self) {
        todo!()
    }
}

impl BitXor for Cards {
    type Output = Self;

    fn bitxor(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}

impl BitXorAssign for Cards {
    fn bitxor_assign(&mut self, _rhs: Self) {
        todo!()
    }
}

impl fmt::Display for Cards {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = self.iter().map(Card::to_string).collect::<Vec<String>>().join(" ");

        write!(f, "{s}")
    }
}

impl Forgiving for Cards {}

impl From<Bard> for Cards {
    /// This method is designed to deserialize a binary `Bard` entity into a `Cards` `IndexSet`
    /// type. The `Bard` type is a handy way to store a collection of `Card`s in a single
    /// binary integer, however they have limitations, that `Cards` makes up for.
    ///
    /// # VICTORY!!!
    ///
    /// Here's how I got it to work.
    ///
    /// ## Step 1:
    ///
    /// Implement `impl TryFrom<Bard> for Card`. We're using `try_from` instead of try
    /// so that we are sure that the `Bard`s we're passing in are single `Bard`s. Since
    /// that type can store one or more `Card` entities, we want to make sure that we aren't
    /// losing any information.
    ///
    /// ## Step 2:
    ///
    /// A long time ago I created a `Bard::DECK` that was an array of the 52 cards in a poker
    /// deck (the infamous French Deck). We'll loop through every single card `Bard` in the deck and
    /// see if our passed in Bard contains it. How will we do that.
    ///
    /// ## Step 3:
    ///
    /// Compare the `Bard` in the deck to our passed in `Bard`. The magic spell
    /// is a bitwise AND operation. For example, here we have the binary value for the ace of spades
    /// and after that, the binary value for four aces. We will do an AND operation on it, and the
    /// result should be that they only flag remaining is the one for the ace of spades, since
    /// that is the only bit in common between the two integers. This tells us that the value of
    /// the ace of spaces is in our Bard, and we can pass it on to our Cards instance.
    ///
    /// ```
    /// let ace_of_spades: i64 = 0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
    /// let my_hand: i64       = 0b1000_0000_0000_0100_0000_0000_0010_0000_0000_0001_0000_0000_0000;
    /// ```
    fn from(bard: Bard) -> Self {
        let mut cards = Cards::default();

        for b in Bard::DECK {
            if b & bard == b {
                let c = Card::try_from(b);
                if let Ok(c) = c {
                    let _ = cards.insert(c);
                }
            }
        }

        cards
    }
}

impl From<Box<[Card]>> for Cards {
    fn from(cards: Box<[Card]>) -> Self {
        Cards::from(cards.to_vec())
    }
}

impl From<BoxedCards> for Cards {
    fn from(cards: BoxedCards) -> Self {
        Cards::from(cards.as_slice())
    }
}

impl From<&Boxes> for Cards {
    fn from(boxes: &Boxes) -> Self {
        let mut cards = Cards::default();
        for boxed in &boxes.0 {
            let c = Cards::from(boxed.as_slice());
            cards.insert_all(&c);
        }
        cards
    }
}

impl From<&Card> for Cards {
    /// Turns out we already have a `TryFrom<Card>` implemented, but I want something similar.
    /// This will give us the contract that if it's blank it won't be inserted, which is fine.
    /// I can see wanted to do both versions of the functionality.
    ///
    /// When I am coding in rust, I do feel the constant tension between my desire to make things
    /// just flow as easily as possible in the short term, and wanting to code things the right,
    /// "rusty" way.
    ///
    /// My general rule is to follow Socrates' maxim: _the unexamined life is not worth living._
    /// Know why you are doing anything. Following rules blindly makes you a tool. If you can't
    /// answer questions like: _why did you code it this way?_ and _what's the purpose of this
    /// test?_ you need to take a step back
    fn from(card: &Card) -> Self {
        let mut cards = Cards::default();
        cards.insert(*card);
        cards
    }
}

impl From<&[Card]> for Cards {
    fn from(slice: &[Card]) -> Self {
        Cards::from(slice.to_vec())
    }
}

impl From<[Card; 2]> for Cards {
    fn from(array: [Card; 2]) -> Self {
        Cards::from(array.to_vec())
    }
}

impl From<[Card; 3]> for Cards {
    fn from(array: [Card; 3]) -> Self {
        Cards::from(array.to_vec())
    }
}

impl From<[Card; 4]> for Cards {
    fn from(array: [Card; 4]) -> Self {
        Cards::from(array.to_vec())
    }
}

impl From<[Card; 5]> for Cards {
    fn from(array: [Card; 5]) -> Self {
        Cards::from(array.to_vec())
    }
}

impl From<[Card; 7]> for Cards {
    fn from(array: [Card; 7]) -> Self {
        Cards::from(array.to_vec())
    }
}

impl From<CardsCell> for Cards {
    fn from(cells: CardsCell) -> Self {
        let internal = cells.0.borrow();
        internal.clone()
    }
}

impl From<&CardsCell> for Cards {
    fn from(cells: &CardsCell) -> Self {
        let internal = cells.0.borrow();
        internal.clone()
    }
}

impl From<Vec<Card>> for Cards {
    fn from(v: Vec<Card>) -> Self {
        let filtered = v.iter().filter_map(|c| {
            let pc = *c;
            if pc.contains_blank() { None } else { Some(pc) }
        });
        Cards(filtered.collect())
    }
}

impl From<Vec<&Card>> for Cards {
    fn from(v: Vec<&Card>) -> Self {
        // TODO RF: Hack :-P
        let filtered = v.iter().filter_map(|c| {
            let pc = **c;
            if pc.contains_blank() { None } else { Some(pc) }
        });
        Cards(filtered.collect())
    }
}

impl From<&Vec<Card>> for Cards {
    fn from(v: &Vec<Card>) -> Self {
        let filtered = v.iter().filter_map(|c| {
            let pc = *c;
            if pc.contains_blank() { None } else { Some(pc) }
        });
        Cards(filtered.collect())
    }
}

impl FromIterator<Card> for Cards {
    fn from_iter<T: IntoIterator<Item = Card>>(iter: T) -> Self {
        let mut c = Cards::default();
        for i in iter {
            c.insert(i);
        }
        c
    }
}

impl FromStr for Cards {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cards = Cards::default();
        let binding = Terminal::index_cleaner(s);
        let s = binding.as_str();
        for s in s.split_whitespace() {
            let c = Card::from_str(s)?;
            if c.contains_blank() {
                return Err(PKError::InvalidCardIndex);
            }
            cards.insert(c);
        }
        if cards.is_empty() {
            Err(PKError::InvalidCardIndex)
        } else {
            Ok(cards)
        }
    }
}

impl Hash for Cards {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for card in self.iter() {
            card.hash(state);
        }
    }
}

impl IntoIterator for Cards {
    type Item = Card;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Pile for Cards {
    /// `IndexSet` types can't have dupes.
    fn are_unique(&self) -> bool {
        true
    }

    fn card_at(self, index: usize) -> Option<Card> {
        self.0.get_index(index).copied()
    }

    fn clean(&self) -> Self {
        todo!()
    }

    /// `Cards` always filters out blank cards, and inherently enforces uniqueness, so this should
    /// always be true.
    fn is_dealt(&self) -> bool {
        true
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let mut cards = Cards::forgiving_from_str("A♠ K♠ Q♠ J♠ T♠");
    /// let old_card = cards.swap(2, Card::from_str("9♠").unwrap());
    /// assert_eq!(old_card.unwrap().to_string(), "Q♠");
    /// assert_eq!(cards.to_string(), "A♠ K♠ 9♠ J♠ T♠");
    /// ```
    fn swap(&mut self, index: usize, card: Card) -> Option<Card> {
        self.0.replace_index(index, card).ok()
    }

    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    /// The idea to implement the `Pile` trait came to me when I was looking through the code for
    /// my old [Fudd spike](https://github.com/ContractBridge/fudd/blob/c4172eaac0f3821e9c144562ca912c8c185b7522/src/types/arrays/mod.rs#L39).
    ///
    /// The `Vectorable` trait is the ancestor to `Pile`. I love having the ability to consolidate
    /// functionality into a common trait. If I can turn a structure into a vector in the trait, I
    /// can do all sorts of communal things to the collections of `Cards`.
    ///
    /// This feels like a nice refactoring opportunity. It will allow me to remove the
    /// ridiculous SOK trait.
    ///
    /// I love how easily this flowed for me. You can tell a library is starting to come together
    /// when you begin to use it in more and more complex ways and it just flows naturally, like
    /// water. _I call this the water principal._ Later on, I learned that Bruce Lee talked about
    /// something similar in his craft:
    ///
    /// > Be like water making its way through cracks. Do not be assertive, but adjust to the object, and you shall find a way around or through it. If nothing within you stays rigid, outward things will disclose themselves.
    ///
    /// > Empty your mind, be formless. Shapeless, like water. If you put water into a cup, it becomes the cup. You put water into a bottle and it becomes the bottle. You put it in a teapot, it becomes the teapot. Now, water can flow or it can crash. Be water, my friend.
    ///
    /// One of my primary goal as a software developer is for my code to flow as easily as
    /// possible for users and maintainers. One of the biggest problem I have in software is where
    /// their build instructions are these insane Rube Goldburgesque machinations. Make your code flow.
    /// *BOOP!* There it goes. [Push button, baby.](https://www.youtube.com/watch?v=En3-GWOUCcI)
    /// _Be water, my friend._
    fn to_vec(&self) -> Vec<Card> {
        self.clone().into_iter().collect()
    }
}

impl SuitShift for Cards {
    fn shift_suit_down(&self) -> Self {
        self.clone().into_iter().map(|c| c.shift_suit_down()).collect()
    }

    fn shift_suit_up(&self) -> Self {
        self.clone().into_iter().map(|c| c.shift_suit_up()).collect()
    }

    fn opposite(&self) -> Self {
        self.clone().into_iter().map(|c| c.opposite()).collect()
    }
}

impl TryFrom<Card> for Cards {
    type Error = PKError;

    fn try_from(card: Card) -> Result<Self, Self::Error> {
        if card.is_dealt() {
            let mut cards = Cards::default();
            cards.insert(card);
            Ok(cards)
        } else {
            Err(PKError::BlankCard)
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod cards_tests {
    use super::*;
    use crate::util::data::TestData;
    use rstest::rstest;

    #[test]
    fn deck_macro() {
        assert_eq!(Cards::deck(), deck!());
    }

    #[test]
    fn deck_minus() {
        // let mut cards = Cards::from_str("Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣").unwrap();
        let mut cards = Cards::from_str("A♦ K♦ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥").unwrap();
        cards.shuffle_in_place();

        cards.insert(Card::ACE_CLUBS);
        cards.shuffle_in_place();

        let minus = Cards::deck_minus(&cards!("AS Q♥ 3♦"));

        assert_eq!(
            "K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣",
            minus.to_string()
        );

        // let minus = Cards::deck_minus(&cards);
        //
        // assert_eq!("A♠ K♠".to_string(), minus.to_string());
        //
        // let mut cards = Cards::from_str("A♦ K♦ Q♦ J♦ T♦").unwrap();
        // cards = cards.shuffle();
        // let mut minus = Cards::deck();
        // for card in cards.iter() {
        //     minus.0.swap_remove(card);
        // }
    }

    #[test]
    fn deck_primed() {
        let deck_minus = TestData::the_hand_cards();
        let expected = "T♠ 2♥ 8♣ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♦ 4♣ 7♣ 9♣ 6♦ 5♥ 5♠ 8♠ A♠ Q♠ J♠ 9♠ 7♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ K♦ Q♦ T♦ 9♦ 8♦ 7♦ 3♦ 2♦ A♣ K♣ J♣ T♣ 6♣ 3♣ 2♣";

        let primed = Cards::deck_primed(&deck_minus);

        assert_eq!(52, primed.len());
        assert_eq!(expected, primed.to_string());
    }

    #[test]
    fn collapse() {
        let wheel = Cards::from_str("5♠ 4♠ 3♠ 2♠ A♥").unwrap().shuffle();
        // for card in wheel {
        //     println!("{}", card.bit_string());
        // }
        let expected: u32 = 0b00010000_00001111_11001111_00101111;

        assert_eq!(expected, wheel.collapse());
    }

    #[test]
    fn combinations() {
        assert_eq!(1_326, Cards::deck().combinations(2).count());
        assert_eq!(2_598_960, Cards::deck().combinations(5).count());
    }

    // #[test]
    // fn gto() {
    //     let all = FIVE_CARD_COMBOS.clone();
    //
    //     let flushes = FIVE_CARD_COMBOS
    //         .clone()
    //         .filter(|combo| Five::is_flush(combo))
    //         .collect::<Vec<_>>();
    // }

    #[test]
    fn draw() {
        let mut deck = Cards::deck();

        let drawn = deck.draw(5).unwrap();

        assert_eq!(drawn.len(), 5);
        assert_eq!(deck.len(), 47);
        assert_eq!("A♠ K♠ Q♠ J♠ T♠", drawn.to_string());
    }

    #[test]
    fn draw_all() {
        let mut deck = Cards::deck();

        let drawn = deck.draw_all();

        assert_eq!(deck.len(), 0);
        assert_eq!(drawn.len(), 52);
    }

    #[test]
    fn draw__too_many() {
        let mut deck = Cards::deck();

        let drawn = deck.draw(53);

        assert!(drawn.is_err());
        assert_eq!(PKError::NotEnoughCards, drawn.unwrap_err());
        assert_eq!(deck.len(), 52);
    }

    #[test]
    fn draw_from_the_bottom() {
        let mut deck = Cards::deck();

        let drawn = deck.draw_from_the_bottom(2).unwrap();

        assert_eq!(drawn.len(), 2);
        assert_eq!(deck.len(), 50);
        assert_eq!("3♣ 2♣", drawn.to_string());
    }

    #[test]
    fn draw_from_the_bottom__too_many() {
        let mut deck = Cards::deck();

        let drawn = deck.draw_from_the_bottom(53);

        assert!(drawn.is_err());
        assert_eq!(PKError::NotEnoughCards, drawn.unwrap_err());
        assert_eq!(deck.len(), 52);
    }

    /// DEFECT #BAD_TWOS STEP 2
    ///
    /// This is what you get for not testing your code. It's my own damn fault.
    #[test]
    fn as_twos() {
        let cards = Cards::from_str("A♠ A♥ A♦ A♣").unwrap();
        let twos = cards.as_twos().unwrap();
        assert_eq!(2, twos.len());
    }

    #[test]
    fn draw_one() {
        let mut cards = Cards::default();
        cards.insert(Card::ACE_HEARTS);

        let card = cards.draw_one();

        assert!(cards.is_empty());
        assert!(card.is_ok());
        assert_eq!(card.unwrap(), Card::ACE_HEARTS);
    }

    #[test]
    fn filter_by_suit() {
        let cards = Cards::deck();

        let spades = cards.filter_by_suit(Suit::SPADES);

        assert_eq!("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠", spades.to_string());
    }

    #[test]
    fn flag_paired() {
        let mut cards = Cards::from_str("T♠ T♥").unwrap().flag_paired();

        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_PAIRED_MASK));
        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_PAIRED_MASK));
        assert!(
            !Cards::from_str("T♠")
                .unwrap()
                .draw_one()
                .unwrap()
                .is_flagged(Card::FREQUENCY_PAIRED_MASK)
        );
    }

    #[test]
    fn flag_tripped() {
        let mut cards = Cards::from_str("T♠ T♥ T♦").unwrap().flag_tripped();

        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_TRIPPED_MASK));
        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_TRIPPED_MASK));
        assert!(
            !Cards::from_str("T♠")
                .unwrap()
                .draw_one()
                .unwrap()
                .is_flagged(Card::FREQUENCY_TRIPPED_MASK)
        );
    }

    #[test]
    fn flag_quaded() {
        let mut cards = Cards::from_str("T♠ T♥ T♦ T♣").unwrap().flag_quaded();

        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(
            !Cards::from_str("T♠")
                .unwrap()
                .draw_one()
                .unwrap()
                .is_flagged(Card::FREQUENCY_QUADED_MASK)
        );
    }

    #[test]
    fn frequency_weighted() {
        let cards = Cards::from_str("T♠ T♥ T♦ 9♠ 9♥").unwrap();

        let mut cards = cards.frequency_weighted();

        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_TRIPPED_MASK));
        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_TRIPPED_MASK));
        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_TRIPPED_MASK));
        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_PAIRED_MASK));
        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_PAIRED_MASK));
    }

    #[test]
    fn frequency_weighted_quads() {
        let cards = Cards::from_str("T♠ T♥ T♦ T♣ 9♥").unwrap();

        let mut cards = cards.frequency_weighted();

        assert_eq!(5, cards.len());
        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(!cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_MASK));
    }

    #[test]
    fn get() {
        let cards = wheel();

        assert_eq!(cards.get(&Card::FIVE_CLUBS).unwrap(), &Card::FIVE_CLUBS);
        assert!(cards.get(&Card::FIVE_DIAMONDS).is_none());
    }

    #[test]
    fn get_index() {
        let cards = wheel();

        assert_eq!(cards.get_index(0).unwrap(), &Card::FIVE_CLUBS);
        assert_eq!(cards.get_index(1).unwrap(), &Card::FOUR_CLUBS);
        assert_eq!(cards.get_index(2).unwrap(), &Card::TREY_CLUBS);
        assert_eq!(cards.get_index(3).unwrap(), &Card::DEUCE_CLUBS);
        assert_eq!(cards.get_index(4).unwrap(), &Card::ACE_CLUBS);
        assert!(cards.get_index(5).is_none());
    }

    #[test]
    fn insert() {
        let mut cards = Cards::default();

        cards.insert(Card::ACE_HEARTS);
        cards.insert(Card::KING_HEARTS);

        let mut i = cards.iter();

        assert_eq!(&Card::ACE_HEARTS, i.next().unwrap());
        assert_eq!(&Card::KING_HEARTS, i.next().unwrap());
        assert!(i.next().is_none());
    }

    #[test]
    fn insert_all() {
        let mut pile = Cards::from_str("5♣ 4♣").unwrap();

        pile.insert_all(&Cards::from_str("3♣ 2♣ A♣").unwrap());

        assert_eq!(Cards::from_str("5♣ 4♣ 3♣ 2♣ A♣").unwrap(), pile);
    }

    #[test]
    fn insert_at() {
        let mut pile = cards!("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠");

        pile.insert_at(3, Card::ACE_CLUBS);
        assert_eq!(cards!("A♠ K♠ Q♠ A♣ J♠ T♠ 9♠ 8♠ 7♠ 6♠").to_string(), pile.to_string());

        pile.insert_at(2, Card::DEUCE_CLUBS);
        assert_eq!(cards!("A♠ K♠ 2♣ Q♠ A♣ J♠ T♠ 9♠ 8♠ 7♠ 6♠").to_string(), pile.to_string());

        pile.insert_at(1, Card::TREY_CLUBS);
        pile.insert_at(1, Card::FOUR_CLUBS);
        pile.insert_at(1, Card::FIVE_CLUBS);
        assert_eq!(
            cards!("A♠ 5♣ 4♣ 3♣ K♠ 2♣ Q♠ A♣ J♠ T♠ 9♠ 8♠ 7♠ 6♠").to_string(),
            pile.to_string()
        );
    }

    #[test]
    fn is_empty() {
        assert!(Cards::default().is_empty());
        assert!(!wheel().is_empty());
    }

    #[test]
    fn len() {
        assert_eq!(0, Cards::default().len());
        assert_eq!(5, wheel().len());
    }

    // #[test]
    // fn sort_by_frequency() {
    //     assert_eq!("A♣ 5♣ 4♣ 3♣ 2♣", wheel().sort().to_string());
    // }

    #[test]
    fn sort() {
        assert_eq!("A♣ 5♣ 4♣ 3♣ 2♣", wheel().sort().to_string());
    }

    #[test]
    fn sort__suit_weighted() {
        let cards = Cards::from_str("6♣ 7♠ 7♦ 8♦").unwrap().sort();

        assert_eq!("7♠ 8♦ 7♦ 6♣", cards.to_string());
    }

    #[test]
    fn sort_in_place() {
        let mut wheel = wheel();

        wheel.sort_in_place();

        assert_eq!("A♣ 5♣ 4♣ 3♣ 2♣", wheel.to_string());
    }

    //region private function tests

    #[test]
    fn map_by_rank() {
        let cards = Cards::from_str("A♠ T♠ 9♠ 8♠ T♥").unwrap();

        let mappy = cards.map_by_rank();

        assert_eq!(2, mappy.get(&Rank::TEN).unwrap().len());
        assert_eq!(1, mappy.get(&Rank::ACE).unwrap().len());
        assert_eq!(1, mappy.get(&Rank::NINE).unwrap().len());
        assert_eq!(1, mappy.get(&Rank::EIGHT).unwrap().len());
    }

    //endregion

    //region trait tests

    #[test]
    fn display() {
        assert_eq!("5♣ 4♣ 3♣ 2♣ A♣", wheel().to_string());
    }

    #[test]
    fn from__bard() {
        let my_bard = Bard::TEN_SPADES | Bard::TEN_DIAMONDS | Bard::TEN_CLUBS | Bard::TEN_HEARTS;
        assert_eq!(Cards::from_str("T♣ T♦ T♥ T♠").unwrap(), Cards::from(my_bard));
        assert_eq!(Cards::default(), Cards::from(Bard::BLANK));
    }

    #[test]
    fn from__card() {
        assert_eq!(Cards::from_str("3♣").unwrap(), Cards::from(&Card::TREY_CLUBS));
        assert_eq!(Cards::default(), Cards::from(&Card::BLANK));
    }

    #[test]
    fn from_str() {
        assert_eq!(wheel(), Cards::from_str("5♣ 4♣ 3♣ 2♣ A♣").unwrap());
    }

    #[test]
    fn from_str__invalid() {
        assert!(Cards::from_str("5♣ 4♣ 3A 2♣ A♣").is_err());
    }

    #[test]
    fn into_iterator() {
        let kings = Cards::deck()
            .into_iter()
            .filter(|c| c.get_rank().to_char() == 'K')
            .collect::<Cards>();

        assert_eq!(4, kings.len());
        assert_eq!(Cards::from_str("K♠ K♥ K♦ K♣").unwrap(), kings);
    }

    #[test]
    fn pile__are_unique() {}

    #[test]
    fn pile__common() {
        let cards = Cards::from_str("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠").unwrap();
        let cards2 = Cards::from_str("A♠ K♠ Q♠ JD").unwrap();

        let expected = Cards::from_str("A♠ K♠ Q♠").unwrap();

        let common = cards.common(&cards2);
        let common_inverse = cards2.common(&cards);

        assert_eq!(common, expected);
        assert_eq!(common_inverse, expected);
        assert_eq!(3, cards.how_many(&cards2.clone()));
    }

    #[test]
    fn pile__suits() {
        let aces = Cards::from_str("AS AH AD AC").unwrap();
        let deck = Cards::deck();

        let suits = aces.suits();
        let clubs = deck
            .clone()
            .into_iter()
            .filter(|c| c.get_suit() == Suit::CLUBS)
            .collect::<Cards>();

        assert_eq!(4, suits.len());
        assert_eq!(Suit::all(), suits);
        assert_eq!(Suit::all(), deck.suits());
        assert_eq!(13, clubs.len());
        assert_eq!(1, clubs.suits().len());
    }

    #[test]
    fn pile__to_eight_or_better_bits() {
        let pile = Cards::from_str("A♦ 2♦ 3♦ 4♦ 5♥").unwrap();

        assert_eq!(pile.to_eight_or_better_bits(), 0b11111);
    }

    #[test]
    fn pile__to_vec() {
        let expected: Vec<Card> = vec![
            Card::FIVE_CLUBS,
            Card::FOUR_CLUBS,
            Card::TREY_CLUBS,
            Card::DEUCE_CLUBS,
            Card::ACE_CLUBS,
        ];

        let actual = Cards::from_str("5♣ 4♣ 3♣ 2♣ A♣").unwrap().to_vec();

        assert_eq!(expected, actual);
    }

    #[test]
    #[ignore]
    fn suit_shift() {
        let spades_royal_flush = Cards::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();
        let hearts_royal_flush = Cards::from_str("A♥ K♥ Q♥ J♥ T♥").unwrap();
        let diamonds_royal_flush = Cards::from_str("A♦ K♦ Q♦ J♦ T♦").unwrap();
        let clubs_royal_flush = Cards::from_str("A♣ K♣ Q♣ J♣ T♣").unwrap();

        assert_eq!(hearts_royal_flush, spades_royal_flush.shift_suit_down());
        assert_eq!(clubs_royal_flush, spades_royal_flush.shift_suit_up());
        assert_eq!(diamonds_royal_flush, spades_royal_flush.opposite());
        assert_eq!(Cards::default(), Cards::default().shift_suit_down());
        assert_eq!(Cards::default(), Cards::default().shift_suit_up());
        assert_eq!(Cards::default(), Cards::default().opposite());
    }

    #[test]
    fn try_from__card() {
        assert!(Cards::try_from(Card::FOUR_DIAMONDS).is_ok());
        assert!(Cards::try_from(Card::BLANK).is_err());
    }

    fn wheel() -> Cards {
        let mut cards = Cards::default();

        cards.insert(Card::FIVE_CLUBS);
        cards.insert(Card::FOUR_CLUBS);
        cards.insert(Card::TREY_CLUBS);
        cards.insert(Card::DEUCE_CLUBS);
        cards.insert(Card::ACE_CLUBS);

        cards
    }
    //endregion

    #[rstest]
    #[case(Card::ACE_SPADES, "A♠ K♠ Q♠ J♠ T♠", true)]
    #[case(Card::FIVE_CLUBS, "5♣ 4♣ 3♣ 2♣ A♣", true)]
    #[case(Card::ACE_DIAMONDS, "A♠ K♠ Q♠ J♠ T♠", false)]
    #[case(Card::SIX_CLUBS, "5♣ 4♣ 3♣ 2♣ A♣", false)]
    fn pile__contains(#[case] card: Card, #[case] index: &str, #[case] assert: bool) {
        let cards = Cards::from_str(index).unwrap();

        assert_eq!(cards.contains(&card), assert);
    }

    #[test]
    fn macro__cards() {
        let cards = cards!("AS KH QC JD TC 9H 8D");

        assert_eq!("A♠ K♥ Q♣ J♦ T♣ 9♥ 8♦", cards.to_string());
    }
}
