use pkcore::arrays::two::Two;
use pkcore::cards::Cards;
use pkcore::{Pile, PKError};

fn main() -> Result<(), PKError> {
    go()
}

/// **STEP 1**: Generate an iterator with every possible hole cards.
///
/// ```
/// let deck = Cards::deck();
///
/// for v in deck.combinations(2) {
///       println!("{:?}", v);
/// }
/// ```
///
/// **STEP 2**: Convert every two `Card` vector into a `Two` struct.
///
/// ```
/// let deck = Cards::deck();
///
/// for v in deck.combinations(2) {
///     let hero = Two::from(v);
///     println!("{hero}");
/// }
/// ```
///
/// While this works, I really hate that Two implements `From<Vec<Card>>` instead of
/// `TryFrom<Vec<Card>>`. This is me trying to exercise my old demons.
///
/// **STEP 2a**: DETOUR... can I implement both `Try` and `TryFrom`?
///
/// DUHH, I've been here before. What I need is a simple
/// [vector slice](https://doc.rust-lang.org/core/slice/trait.SlicePattern.html#tymethod.as_slice).
/// This will then call `impl TryFrom<&[Card]> for Two` and return an error if the `Cards` aren't
/// correct.
///
/// ```
/// let deck = Cards::deck();
///
/// for v in deck.combinations(2) {
///     let hero = Two::try_from(v.as_slice())?;
///     println!("{hero}");
/// }
///```
///
/// This allows me to use the [? operator](https://doc.rust-lang.org/rust-by-example/std/result/question_mark.html),
/// which I totally love.
///
/// **STEP 3**: Give me a count for every iteration
///
/// This one is simple. Use vector's [Enumerate Trait](https://doc.rust-lang.org/std/iter/struct.Enumerate.html):
/// ```
/// let deck = Cards::deck();
///
/// for (i, v) in deck.combinations(2).enumerate() {
///     let hero = Two::try_from(v.as_slice())?;
///     println!("{} - {hero}", i + 1);
/// }
/// ```
///
/// This shows us that we have 1,326 different hands.
///
/// **STEP 4**: Every other possible hand against that hand.
///
/// Now things are going to get fun.
///
/// ```
/// let deck = Cards::deck();
///
/// for (i, v) in deck.combinations(2).enumerate() {
///     let hero = Two::try_from(v.as_slice())?;
///
///     println!("{} - {hero}", i + 1);
///     let remaining = Cards::deck_minus(&hero.cards());
///     for r in remaining.combinations(2) {
///         let villain = Two::try_from(r.as_slice())?;
///         println!("... {hero} v. {villain}");
///     }
/// }
/// ```
///
/// Hey, I just remembered something... Two implements the Pile trait which has it's
/// own remaining method...
///
/// ```
/// let deck = Cards::deck();
///
/// for (i, v) in deck.combinations(2).enumerate() {
///     let hero = Two::try_from(v.as_slice())?;
///
///     println!("{} - {hero}", i + 1);
///     let remaining = Cards::deck_minus(&hero.cards());
///     for r in remaining.combinations(2) {
///         let villain = Two::try_from(r.as_slice())?;
///         println!("... {hero} v. {villain}");
///     }
/// }
/// ```
fn go() -> Result<(), PKError> {
    let deck = Cards::deck();

    let mut count: u32 = 1;
    for (i, v) in deck.combinations(2).enumerate() {
        let hero = Two::try_from(v.as_slice())?;

        println!("{} - {hero}", i + 1);
        for r in hero.remaining().combinations(2) {
            let villain = Two::try_from(r.as_slice())?;
            println!("{count} {i}  {hero} v. {villain}");
            count = count + 1;
        }
    }

    Ok(())
}