# pkcore

## Outline

* Got rust?
    * Cargo clippy BEAST MODE
    * Cargo fmt
* [Setup wasm](https://rustwasm.github.io/docs/book/game-of-life/setup.html).
* Create pkcore lib
    * Set #![warn(clippy::pedantic)]
* Rank
    * lib:PKError
    * create enum
    * ::from(char)
      * Testing: TELL THE HEROES STORY
        * Tests using [rstest](https://crates.io/crates/rstest) BRUTE FORCE
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
        * TRAIT: SOK
        * arrays::HandRanker
          * remove Five::rank() replace with HandRanker
    

## LATER

* improved hand hash https://github.com/HenryRLee/PokerHandEvaluator/blob/master/Documentation/Algorithm.md
  * split flushes out to only focus on rank brilliant
