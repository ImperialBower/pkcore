# pkcore

## Outline

* Got rust?
    * Cargo clippy
    * Cargo fmt
* [Setup wasm](https://rustwasm.github.io/docs/book/game-of-life/setup.html).
* Create pkcore lib
    * Set #![warn(clippy::pedantic)]
* Rank
    * lib:PKError
    * create enum
    * ::from(char)
        * Tests using [rstest](https://crates.io/crates/rstest)
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