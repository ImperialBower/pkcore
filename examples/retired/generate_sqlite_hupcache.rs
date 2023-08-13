/// Let's walk through the plan.
///
/// STEP 1: Generate every possible preflop hup combination.
/// STEP 2: Check to see if the result is already in the DB.
/// STEP 3: If not, do the calc.(The first time this will load the bcm cache.
/// STEP 4: Add it to the DB.
/// STEP 5: Make sure that it's in there.
///
/// # Step 1: Generate every possible preflop hup combination.
///
/// Here's the function to loop through the things:
///
/// ```
/// use pkcore::arrays::two::Two;
/// use pkcore::cards::Cards;
/// use pkcore::{PKError, Pile};
///
/// fn main() -> Result<(), PKError> {
///     go()
/// }
/// fn go() -> Result<(), PKError> {
///     let deck = Cards::deck();
///
///     let mut count: u32 = 1;
///     for (i, v) in deck.combinations(2).enumerate() {
///         let hero = Two::try_from(v.as_slice())?;
///
///         println!("{} - {hero}", i + 1);
///         for r in hero.remaining().combinations(2) {
///             let villain = Two::try_from(r.as_slice())?;
///             println!("{count} {i}  {hero} v. {villain}");
///             count = count + 1;
///         }
///     }
///
///     Ok(())
/// }
/// ```
fn main() {}
