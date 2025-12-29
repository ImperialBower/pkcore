use crate::analysis::case_eval::CaseEval;
use crate::analysis::case_evals::CaseEvals;
use crate::util::wincounter::PlayerFlag;
use crate::util::wincounter::win::Win;
use crate::{Card, Cards};
use indexmap::IndexMap;

/// This is old `Fudd` code.
#[derive(Clone, Debug)]
pub struct Outs(IndexMap<usize, Cards>);

impl Outs {
    /// I'll confess that the `get_mut()` function threw me off.
    /// `let ref mut set = self.0.get_mut(&player).unwrap();` generates this error message:
    ///
    /// ```txt
    /// warning: `ref` on an entire `let` pattern is discouraged, take a reference with `&` instead
    ///   --> src/analysis/outs.rs:24:13
    ///    |
    /// 24 |         let ref mut set = self.0.get_mut(&player).unwrap();
    ///    |         ----^^^^^^^^^^^------------------------------------ help: try: `let set = &mut self.0.get_mut(&player).unwrap();`
    ///    |
    ///    = note: `#[warn(clippy::toplevel_ref_arg)]` on by default
    ///    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#toplevel_ref_arg
    /// ```
    ///
    /// But when I use the code suggested (`let set = &mut self.0.get_mut(&player).unwrap();`) I get
    /// this clippy warning:
    ///
    /// ```txt
    /// warning: this expression mutably borrows a mutable reference. Consider reborrowing
    ///   --> src/analysis/outs.rs:39:19
    ///    |
    /// 39 |         let set = &mut self.0.get_mut(&player).unwrap();
    ///    |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    ///    |
    /// ```
    ///
    /// Then, on a lark I tried removing the `&mut`s all together, and what do you know, it worked.
    /// This is why we write unit tests. The rust compiler, no matter how good it is, can only show
    /// us so much. This gives us this:
    ///
    /// ```txt
    /// pub fn add(&mut self, player: usize, card: Card) {
    ///     self.touch(player);
    ///     let set = self.0.get_mut(&player).unwrap();
    ///     set.insert(card);
    /// }
    /// ```
    ///
    /// Let's try one last little change. Do we really need to set the set variable
    /// before we call insert? Turns out the answer is no.
    ///
    /// # Panics
    ///
    /// Shouldn't be possible 🤞
    pub fn add(&mut self, player: usize, card: Card) {
        self.touch(player);
        if let Some(cards) = self.0.get_mut(&player) {
            cards.insert(card);
        }
    }

    pub fn add_from_case_eval(&mut self, case_eval: &CaseEval) {
        self.add_from_player_flag(case_eval.win_count(), case_eval.card());
    }

    /// Our goal of this method is to add the `Card` for every player bit flag that is set.
    /// We are going to test drive through this method.
    ///
    /// The big idea is that we have a `PlayerFlag` that we need to check if the first bit is
    /// set and then do a shift right on the value and see if the flag is set for the player.
    /// Something like:
    ///
    /// ```
    /// use pkcore::util::wincounter::PlayerFlag;
    /// use pkcore::util::wincounter::win::Win;
    ///
    /// let mut i: PlayerFlag = 0b0000_0000_0000_0101;
    ///
    /// for _ in 1..8 {
    ///     let is_set = i & Win::FIRST == Win::FIRST;
    ///     i = i >> 1;
    /// }
    /// ```
    pub fn add_from_player_flag(&mut self, flag: PlayerFlag, card: Card) {
        let mut bite = flag;
        for player in 1..11 {
            let is_set = bite & Win::FIRST == Win::FIRST;
            if is_set {
                self.add(player, card);
            }
            // CLIPPY CORRECTION OF: bite = bite >> 1;
            bite >>= 1;
        }
    }

    /// *FRACK*
    ///
    /// Writing tests for this method has uncovered a defect with `Cards.sort()`.
    /// __NOT ANY MORE__
    ///
    /// ```
    /// use pkcore::analysis::outs::Outs;
    /// use pkcore::card::Card;
    ///
    /// let mut outs = Outs::default();
    /// outs.add(1, Card::SIX_CLUBS);
    /// outs.add(1, Card::SEVEN_SPADES);
    /// outs.add(1, Card::SEVEN_DIAMONDS);
    /// outs.add(1, Card::EIGHT_DIAMONDS);
    ///
    /// assert_eq!("7♠ 8♦ 7♦ 6♣", outs.get(1).unwrap().sort().to_string());
    /// ```
    ///
    /// This sort result is `Rank` weighted. Ideally, we'd like this to be `Suit`
    /// weighted, followed by `Rank`. This would create a result from the test of `7♠ 8♦ 7♦ 6♣`.
    /// For now we're going to mark this as a `todo` in `Cards` and add a test that we ignore for
    /// now. This is one of those nice to haves right now.
    ///
    /// This is why, when you're writing your tests, that you don't want to always do the same thing over
    /// and over again. Right now, we're a bit guilty of that with our `TestData` struct. Later on,
    /// I would like to level it up a bit by having it be able to load serialized data from a file
    /// so that we can add more scenarios.
    ///
    /// Many years ago I wrote a library in Java that I called `Faker` in Java for a company I was
    /// working for. It could instantiate state from CSV files and build relationships between
    /// dynamic entities. It allowed me to quickly create complex relationships, that I could place
    /// in front of Spring MVC controllers via a @Fake annotation, allowing me to stand up web
    /// services for front end devs before the back end devs had finished writing them. I was really
    /// proud of this work.
    ///
    /// Alas, this library died when I left the company, because I am sure that the people I worked
    /// under didn't take a minute to understand what I was doing, and only really cared about
    /// protecting their IP. If the company I worked for had had a rational open source policy,
    /// instead of one driven by ol' timey lawyers, the code would have been able to be shared,
    /// which would have made it much more visible to the company's devs, allowed for other's to
    /// contribute to the code globally, and been free advertisement to the kind of cool work
    /// that the company does. This is the power of the commons. As the Belters say in The Expanse:
    ///
    /// > The more you share the more your bowl will be plentiful.
    ///
    /// I wish more companies took the time to have a rational open source policy, allowing
    /// themselves to leverage its power. Unfortunately, most people are binary thinkers. They see
    /// everything in terms of black and white. Yes, you're here to make money, but sometimes the
    /// best way to earn a profit is to give something away.
    ///
    /// Anywho, the plan is to do this for our library. Part of the reason for this is that I want
    /// to create a fundamental way for this library to serialize state into text files, so that I
    /// can quickly generate hand calculations. For now, this is one of those features in the
    /// backlog.
    ///
    /// I think of this as making my data plastic. I use the original term for the word, as opposed
    /// to the modern, [life killing family of chemicals](https://education.nationalgeographic.org/resource/great-pacific-garbage-patch)
    /// that we generally think of today when we hear the word.
    /// [Plastic](https://www.etymonline.com/word/plastic) comes
    /// from the Latin word _plasticus_, which in turn comes from the Greek word _plastikos_,
    /// meaning _capable of being molded into various forms_.
    ///
    /// I want my data to be plastic. When I write libraries, I want the domain entities is wrangles
    /// to be able to be twisted and turned into many shapes. I want to be able to easily save them
    /// to files, and write them to databases, and when I look at that raw data, I want to be able
    /// to easily understand what it is. Making my code plastic, is for me the highest state of
    /// quality. While it may constantly change its state, it never loses its core identity. I don't
    /// know how people in the future will use this code, so I want to make it as flexible as
    /// possible, and learn from how people stress it beyond its limits so that I can make it
    /// stronger later on.
    ///
    /// Most coders and product people see this as _gold plating_. Just give me what my requirements
    /// are and don't worry about making it flexible. If I had listen to this advice, many of the
    /// projects I had worked on would never had made it to launch. The truth is, that people don't
    /// really know what they want until they see it. If Steve Jobs had given consumers what they
    /// wanted, Apple wouldn't exist as a company. You will find as a developer that product owners
    /// change their minds on a dime with no consideration for your time or efforts. They always
    /// think they know better and see your work as trivial efforts manifesting their majestic
    /// will-to-power genius. Fuck your kids... fuck your life... I need you working 80 weeks
    /// because you didn't psychically read my mind when I asked you for X when I really wanted Y.
    /// This is why they pay me the big bucks. Because I am an inflated ego masquerading as an
    /// innovative genius, with my world shattering powerpoint skills causing VCs the world over
    /// to line up to give me all of their money. The bottom of startup ocean is littered with the
    /// wrecks of these fools.
    ///
    /// Define what your core domain is, make their libraries plastic, and you can mold them into
    /// whatever shapes you need. I wish for you experiences where you work with people strong
    /// enough to trust you to do this. I test my code. I harden my domain. I write things that
    /// empower creators. This is how I code when I am coding as my best self. It has always been
    /// the source of my greatest wins, and has rarely been understood by anyone I worked with. Part
    /// of the reason for this book is to get these ideas down.
    ///
    /// ## A little story...
    ///
    /// For many, many years when you wrote apps for the iPhone, you would do so in the
    /// [Objective-C](https://en.wikipedia.org/wiki/Objective-C)
    /// programming language. In your code, you would constantly see calls to libraries with the
    /// prefix NS. This always seemed strange to me. Turns out, it's from the work done at Jobs'
    /// company [`NeXTSTEP`](https://en.wikipedia.org/wiki/NeXTSTEP), which was acquired by Apple in
    /// 1996. It always seemed strange to be that he insisted on Apple acquiring back `NeXTSTEP` when
    /// he returned to Apple.
    ///
    /// Turns out that one of the core reasons for the Apple's success was what I call the most
    /// beautiful hack in programming. While Sun Microsystems was creating an object-oriented
    /// version of the C programming language that was bloated and slow, Tom Love and Brad Cox
    /// were creating a small, tiny extension to C that achieved the same goal, while being tiny
    /// and fast.
    ///
    /// Objective-C is what allowed Apple developers to innovate so fast. It was the jump start that
    /// allowed Apple to quickly innovate. Now, when I code in Swift, it makes me sad how much
    /// slower `XCode` is from my days coding in Objective-C. I understand why they created Swift,
    /// but they gave up a lot when they moved to it.
    ///
    /// ## UPDATE
    ///
    /// OK, so I've fixed the sorting bug through a very hacky hack. Not trying to optimize it
    /// now, just get to a sound place.
    ///
    /// # Panics
    ///
    /// Shouldn't be possible 🤞
    pub fn append(&mut self, other: &Outs) {
        for (player, cards) in other.iter() {
            self.touch(*player);
            if let Some(dest) = self.0.get_mut(player) {
                dest.insert_all(cards);
            }
        }
    }

    #[must_use]
    pub fn get(&self, player: usize) -> Option<&Cards> {
        self.0.get(&player)
    }

    /// OK, this is why you don't code after doing a shot of Jäger. This code
    /// is virtually pointless, but I want it. I just know I need it. I can hear
    /// R.J. telling me to move on.
    #[must_use]
    pub fn is_longest(&self, player: usize) -> bool {
        self.longest_player() == player
    }

    #[must_use]
    pub fn iter(&self) -> indexmap::map::Iter<'_, usize, Cards> {
        self.0.iter()
    }

    #[must_use]
    pub fn len_for_player(&self, player: usize) -> usize {
        match self.0.get(&player) {
            None => 0,
            Some(cards) => cards.len(),
        }
    }

    #[must_use]
    pub fn len_longest(&self) -> usize {
        let mut longest = 0_usize;
        for key in self.0.keys() {
            let len = self.len_for_player(*key);
            if len > longest {
                longest = len;
            }
        }
        longest
    }

    /// Returns the player id that has the most outs.
    ///
    /// While I am worried that this code is getting heavy, since it is only
    /// going to be used after the flop, the amount of calculations shouldn't
    /// be too much. I know I will have much bigger fish to fry, and I want
    /// to close on this card. It's turning out to be harder than I thought
    /// it would be.
    ///
    /// The truth is, that coding is always going to have tension between the
    /// desire to do things perfectly, and the need to get things done. A big
    /// part of the craft is having a sense for when to refactor and optimize
    /// and when to just get shit done. Sometimes you leave behind technical
    /// debt... sometimes nobody gives a fuck but you. Pick your battles; learn
    /// from your mistakes; forgive yourself. You have this brain for a reason.
    /// Coders who only write perfect code are coders who never launch.
    ///
    /// ## DEFECT: Outs displayed wrong
    ///
    /// This code is trash, and needed tests.
    ///
    /// ### Defect Resolved
    ///
    /// There was some really illogical logic in the original version of this code, which has been
    /// resolved. But...
    ///
    /// There is still a big problem with this code. 10 points if you can spot it...
    /// .
    /// .
    /// .
    /// .
    /// .
    /// .
    /// .
    /// .
    /// .
    /// .
    /// *ANSWER:* There can be more than on player who has the most outs. This code doesn't deal
    /// with ties. This data dimension has been one of the biggest challenges in the early stages
    /// of this library.
    ///
    /// You will find in your coding adventures that outliers like this will eat up a
    /// disproportionate amount of your time. Treat these problems as a gift. Yes, it's a
    /// pain in the ass, but the people using your shit don't give a flying fuck about your
    /// pain, and TBH, these are some of the most interesting coding problems you will find.
    /// Your systems need to be smooth as silk, and your domain will work overtime and on weekends
    /// to thwart that goal. Don't let it win. Most shitty user experiences can be traced back
    /// to a groups unwillingness to take the time to make things smooth. Take the time. Be better
    /// that these schlubs.
    #[must_use]
    pub fn longest_player(&self) -> usize {
        let mut player = 0_usize;
        let mut top_len = 0_usize;
        for key in self.0.keys() {
            let len = self.len_for_player(*key);
            if len > top_len {
                player = *key;
                top_len = len;
            }
        }
        player
    }

    pub fn touch(&mut self, player: usize) -> bool {
        if self.0.get(&player).is_none() {
            self.0.insert(player, Cards::default());
            true
        } else {
            false
        }
    }
}

impl Default for Outs {
    fn default() -> Outs {
        Outs(IndexMap::new())
    }
}

impl From<&CaseEvals> for Outs {
    fn from(case_evals: &CaseEvals) -> Self {
        let mut outs = Outs::default();
        for case_eval in case_evals.iter() {
            outs.add_from_case_eval(case_eval);
        }
        outs
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__outs_tests {
    use super::*;
    use crate::arrays::two::Two;
    use crate::play::board::Board;
    use crate::play::game::Game;
    use crate::play::hole_cards::HoleCards;
    use crate::util::data::TestData;
    use std::str::FromStr;

    #[test]
    fn add() {
        let mut outs = Outs::default();

        outs.add(1, Card::SIX_CLUBS);
        outs.add(1, Card::SEVEN_SPADES);

        assert_eq!("6♣ 7♠", outs.get(1).unwrap().to_string());
        assert_eq!("7♠ 6♣", outs.get(1).unwrap().sort().to_string());
    }

    #[test]
    fn add_from_player_flag() {
        let mut outs = Outs::default();

        outs.add_from_player_flag(Win::FIRST, Card::SIX_CLUBS);
        outs.add_from_player_flag(Win::FIRST, Card::SEVEN_SPADES);

        assert_eq!("6♣ 7♠", outs.get(1).unwrap().to_string());
    }

    #[test]
    fn add_from_player_flag__1_and_3() {
        let mut outs = Outs::default();

        outs.add_from_player_flag(Win::THIRD, Card::SIX_CLUBS);
        outs.add_from_player_flag(Win::THIRD, Card::SEVEN_SPADES);
        outs.add_from_player_flag(Win::FIRST, Card::ACE_SPADES);

        assert_eq!("A♠", outs.get(1).unwrap().to_string());
        assert!(outs.get(2).is_none());
        assert!(outs.get(4).is_none());
        assert_eq!("6♣ 7♠", outs.get(3).unwrap().to_string());
    }

    #[test]
    fn append() {
        let mut outs1 = Outs::default();
        let mut outs2 = Outs::default();
        outs1.add(1, Card::SIX_CLUBS);
        outs1.add(1, Card::SEVEN_SPADES);
        outs2.add(1, Card::SEVEN_DIAMONDS);
        outs2.add(1, Card::EIGHT_DIAMONDS);

        outs1.append(&outs2);

        assert_eq!("6♣ 7♠ 7♦ 8♦", outs1.get(1).unwrap().to_string());
        assert_eq!("7♠ 8♦ 7♦ 6♣", outs1.get(1).unwrap().sort().to_string());
    }

    #[test]
    fn is_longest() {
        let mut outs = Outs::default();
        outs.add(1, Card::SIX_CLUBS);
        outs.add(1, Card::SEVEN_SPADES);
        outs.add(2, Card::SEVEN_DIAMONDS);

        assert!(outs.is_longest(1));
        assert!(!outs.is_longest(2));
    }

    #[test]
    fn len_for_player() {
        let mut outs = Outs::default();
        outs.add(1, Card::SIX_CLUBS);
        outs.add(1, Card::SEVEN_SPADES);
        outs.add(2, Card::SEVEN_DIAMONDS);

        assert_eq!(2, outs.len_for_player(1));
        assert_eq!(1, outs.len_for_player(2));
        assert_eq!(0, outs.len_for_player(3));
    }

    #[test]
    fn len_longest() {
        let mut outs = Outs::default();
        outs.add(1, Card::SIX_CLUBS);
        outs.add(1, Card::SEVEN_SPADES);
        outs.add(1, Card::NINE_HEARTS);
        outs.add(1, Card::TEN_HEARTS);
        outs.add(2, Card::SEVEN_DIAMONDS);

        assert_eq!(4, outs.len_longest());
    }

    #[test]
    fn longest_player() {
        let mut outs = Outs::default();
        outs.add(1, Card::TEN_HEARTS);
        outs.add(1, Card::SEVEN_DIAMONDS);
        outs.add(2, Card::SIX_CLUBS);
        outs.add(2, Card::SEVEN_SPADES);
        outs.add(2, Card::NINE_HEARTS);

        assert_eq!(2, outs.longest_player());
    }

    #[test]
    fn touch() {
        let mut outs = Outs::default();

        let touched = outs.touch(1);

        assert!(touched);
        assert_eq!(Cards::default(), *outs.get(1).unwrap());
        assert!(outs.get(2).is_none());
    }

    #[test]
    fn from__case_evals() {
        let case_evals = TestData::the_hand().turn_case_evals();

        let outs = Outs::from(&case_evals);

        assert_eq!("6♣", outs.get(1).unwrap().to_string());
        let outs_cards = outs.get(2).unwrap().sort();
        assert_eq!(
            "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 8♣ 7♣ 4♣ 3♣ 2♣",
            outs_cards.to_string()
        );
    }

    #[test]
    fn from__case_evals__mystal_defect_1() {
        let game = Game::new(
            HoleCards::from(vec![Two::HAND_AS_KH, Two::HAND_8D_6C]),
            Board::from_str("A♣ 8♥ 7♥ 9♠ 5♠").unwrap(),
        );

        let outs = Outs::from(&game.turn_case_evals());

        assert_eq!(1, outs.longest_player());
        assert_eq!(31, outs.get(1).unwrap().len());
        assert_eq!(13, outs.get(2).unwrap().len());
        assert_eq!(
            "T♠ 8♠ 6♠ 5♠ T♥ 6♥ 5♥ T♦ 6♦ 5♦ T♣ 8♣ 5♣",
            outs.get(2).unwrap().sort().to_string()
        );
    }

    #[test]
    fn from__case_evals__mystal_defect_2() {
        let game = Game::new(
            HoleCards::from(vec![Two::HAND_8D_6C, Two::HAND_AS_KH]),
            Board::from_str("A♣ 8♥ 7♥ 9♠ 5♠").unwrap(),
        );

        let outs = Outs::from(&game.turn_case_evals());

        assert_eq!(2, outs.longest_player());
        assert_eq!(13, outs.get(1).unwrap().len());
        assert_eq!(31, outs.get(2).unwrap().len());
        assert_eq!(
            "T♠ 8♠ 6♠ 5♠ T♥ 6♥ 5♥ T♦ 6♦ 5♦ T♣ 8♣ 5♣",
            outs.get(1).unwrap().sort().to_string()
        );
    }
}
