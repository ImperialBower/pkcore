# pkcore

## Outline

# PK Book

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
        * card_consts
        * ::new
        * .is_blank()
* Detour on Testing as the Hero's Journey
    * tell the story
    * scannable
