# pkcore

## Value Stories

* I want a tool that will help me get better at GTO style poker playing. 
* I want a library that can be reused for poker applications.

## Outline

* Got rust?
    * Cargo clippy BEAST MODE
    * Cargo fmt
* [Setup wasm](https://rustwasm.github.io/docs/book/game-of-life/setup.html).
* Why Rust?
  * Inverting the curve
  * Rust TDD loop
    * define
    * create fn sig returning default value
    * create failing test valid on expected value
    * Make test green
    * any more boundary conditions?
    * refactor
    * draw negative boundary refactor to Result for overdraw
* Letting the IDE do a lot of the work (Mad Dog Murdock)
  * Compare CLion to VSCode 
* Create pkcore lib
    * Set #![warn(clippy::pedantic)]
* Rank
    * lib:PKError
    * create enum
    * ::from(char)
      * Testing: TELL THE HEROES STORY
        * Tests using [rstest](https://crates.io/crates/rstest) BRUTE FORCE
        * Readability of tests names (short, matching, clear)
        * Hub and spoke... take your time on the core.
        * Negative boundary conditions
    * ::from_str()
        * test neg scenarios #[allow(non_snake_case)]
* Suit
    * create enum
    * ::from(char)
        * Tests using [rstest](https://crates.io/crates/rstest)
    * ::from_str()
        * test neg scenarios #[allow(non_snake_case)]
    * Card
        * CKC Card Number consts
            * Intro to CKC Numbers
        * ::as_u32()
        * ::from<u32> (filter)
            * #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
            * Talk about brute force testing philosophy.
            * Add Rank.number(), .bits(), .prime(), and .shift8(). 
              * strum::EnumIter && tests
        * card_consts
        * ::new
        * .is_blank()
        * .from_str()
          * Boundary conditions tests.
        * Detour on Testing as the Hero's Journey
            * tell the story
            * scannable
        * Move card numbers and make private
        * .to_string() which means doing it for Rank and Suit.
    * Rank
      * .to_string() thus .to_char()
    * Suit
        * .to_string() thus .to_char() 
    * Card
      * .to_string()
    * Cards
      * [indexmap::IndexSet](https://github.com/bluss/indexmap) 
      * indexmap::set::Iter
      * .len() .is_empty()
      * .insert and .iter()
      * .to_string()
      * .sort() (adding Ord)
    * Card
      * .bit_string() -- Talk about expressive logging.
    * REFACTORING I don't expect you to copy out every part of the test code, although you are more than happy to. Think of this as a cooking show where I, your humble host, does most of the boring parts off camera so that you don't get bored. Voila! Losts of tests refactored!
        * CardNumber enum
      * into()
      * rstests: Consolidating  tests 
        * The problem is that they aren't really testing for value Just number not number to actual card
    * Five INTRODUCING CACTUS KEV
    * Cards
      * implement FromStr so can be used by arrays like Five.
      * need get_index() to do Five.from_str()
    * Five
      * now implement FromStr leveraging Cards.from_str()
      * .to_arr()
      * Get Five eval to return HandRank Number
        * .rank()
          * .or_bits()
          * .or_rank_bits()
          * .and_bits()
          * .is_flush()
          * .rank()
            * flushes
            * unique
            * the rest
              * .multiply_primes()
                * Card.get_prime() (Add to get_rank tests)
              * REFACTORING: Section for pub fn then private fn
              * not_unique()
                * find_in_products()
                  * refactoring from ckc (compare)
    * Cards
      * REFACTORING: Clippy found call to `str::trim` before `str::split_whitespace` cards.rs:72:20
    * Five
      * rank() basic tests
    * INTRODUCING HandRank struct
      * type aliases (HandRankValue, CactusKev)
        * CLIPPY: HandRankValue clippy::module_name_repetitions
      * hand_rank module (talk about separating files as old habit)
      * HandRankName
          * about strum::EnumIter;
          * refactoring HandRank.determine_name() to HandRankName::from (small test range... ranges of testing)
      * HandRankClass
          * refactoring HandRank.determine_class() to HandRankClass::from
            * CLIPPY #[warn(clippy::too_many_lines)]"
          * AUDIBLE (waiting for HandRank MEGA TEST to capture all) (AUDIBLE = change from standard flow)
      * REFACTORING: impl Default to derive
      * HandRank::from(HandRankValue)
        * Favor traits over functions
        * PROCESS NOTE::: Step by step
          * why I do small commits
          * git muscle memory get into the habit of knowing the commands
        * REFACTORING name and class no longer public (remove need for ckc.is_a_valid_hand_rank())
          * HARDENING (over thinking? Maybe)
        * TRAIT: SOK - I find boring names hard to remember
        * arrays::HandRanker
          * remove Five::rank() replace with HandRanker
          * REFACTOR: remove validated methods from Trait
      * Hand HandRanker BIG KAHUNA TEST
        * using //region 
        * REFACTORING HandRankName  HandRankClass to Name and Class
    * DETOUR: Generating lookup tables
      * Is Straight Flush?
        * Is Straight?
          * Let's add tests for all types of straights
            * WHEEL???!!!!
        * Already have .is_flush()
        * .is_straight_flush()
        * INTRODUCING: IterTools
          * Cards.combinations()
          * Cards::from<Vec<Card>>
          * Five::try_from<Vec<Card>>
    * Six
      * Before we get started lets update the repl so that we get more feedback
      * Five.display
        * Cards::from(Card)
        * Cards::from(Five)
      * examples/repl.rs Update to use Cards
      * Cards.draw() reqs Cards.deck()
      * .hand_rank()
        * Let IDE drive with the TODO
          * yay todo!()
          * Understand the dynamics of teams, bend to the norms. Different people react differently to code styles. It's OK Be cool 
        * adding .five_from_permutation to HandRanker
          * Adding to trait because I know I will need it for Seven as well. NOTE: This is not strict TDD.
          * REFACTORING: Originally a separate trait Permutator in ckc-rs. Decided to consolidate make cleaner.
          * REFACTORING: Move to Trait
        * Five.sort() no method named `sort` found for struct `Five` in the current scope
          * Adding to HandRanker trait
          * Later refactoring to handle less then five cards (Another trait?)
        * And we're green!!!
        * More tests
          * REFACTORING: Adding HandRanker.hand_rank_and_hand() to trait
          * Fleshing out
          * Deliberate using different forms of index strings
            * Be careful with your patterns and assumptions. TDD can be a self-fulfilling prophecy divorced from reality
      * .sort()
    * TODO: Note on Five.sort() for wheels Ace on top
      * ASIDE: I am fiercely pro TODOs. Many people hate them. That's OK. However, this is my kingdom and I rule it with an iron fist! When you write your book, you too can be Absolute Ruler of your one person empire (Unless you have an editor 😉).
    * Seven
      * Seven holdem boards etc
      * BONUS CREDIT:
        * OK, if you are seriously hard core and want to go for bonus points (awarded via my GFY Cryptocurrency guaranteed to be worthless!!!) Try coding Seven yourself.
          * What are its reqs? What's different about Six and Seven?
      * .from(array)
      * Drive from implementing HandRanker
        * .five_from_permutation()
          * Get not yet implemented error
          * Fix error
          * make green
        * .hand_rank_value_and_hand()
          * Why AD 6S 4S AS 5D 3C 2S?
          * paste over test from Six (which will fail for Seven)
          * get rid of not yet implemented error
          * make green
          * Clippy
      * TODO: No assessors besides .to_arr() do I really need the others?
    * UPDATE repl
      * Print out HandRank info for best hand if five or more cards.
        * Update to look at Cards length and usher to the right struct
        * NOTE on driving through repl instead of tests
          * move index out to variable. (Why is it called index?)
            * check in repl again
          * let cards = Cards::from_str(index).unwrap();
            * check in repl again
          * Add match for default _
          * Add match for five
            * impl TryFrom<Cards> for Five
              * REFACTOR: move match from from_str to TryFrom
      * NEEDED: to_string() for Five, Six and Seven
        * impl Display for Five
          * 5 => println!("{}", Five::try_from(cards)?), 
            * update main to return error
              * update sig to fn main() -> Result<(), PKError> {
                * must return Ok(())
                * Verify `❯ cargo run --example repl -- -c "AS KS QS JS TS"`
          * 6 => println!("Six: {}", Six::try_from(cards)?),
            * impl Display for Six
          * 7 
            * .to_arr()
            * .display()
      * Display HandRank


## LATER

* improved hand hash https://github.com/HenryRLee/PokerHandEvaluator/blob/master/Documentation/Algorithm.md
  * split flushes out to only focus on rank brilliant

## Resources

* [Are we game yet?](https://arewegameyet.rs/)
* [Are we GUI Yet?](https://www.areweguiyet.com/)