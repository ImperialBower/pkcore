use pkcore::arrays::matchups::SortedHeadsUp;
use pkcore::arrays::two::Two;
use pkcore::cards::Cards;
use pkcore::{PKError, Pile};

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
///     for r in hero.remaining().combinations(2) {
///         let villain = Two::try_from(r.as_slice())?;
///         println!("... {hero} v. {villain}");
///     }
/// }
/// ```
///
/// That simplifies things a little bit.
///
/// **STEP 5**: Storage
///
/// I needed to spike out a solution to storing the results in a database. At first I thought I
/// wanted a simple embedded database like [Sled](https://sled.rs/), but honestly, for what I am
/// doing, it quickly became a pain in the ass. See `examples/generate_sled.rs` for my short
/// foray into it. My head has to be in the right place, and it just felt way to low level for
/// what I wanted.
///
/// The obvious choice was a good ol' fashioned database, like MySQL or my personal fav' PostgreSQL.
///
/// ASIDE:
///
/// My main reason for loving PostgreSQL so much is it's support for
/// [GIS](https://en.wikipedia.org/wiki/Geographic_information_system) with [PostGIS](https://postgis.net/).
/// I worked on a project many years ago that involved geographic data using
/// [Oracle Spatial](https://en.wikipedia.org/wiki/Oracle_Spatial_and_Graph) and
/// [ESRI](https://en.wikipedia.org/wiki/Esri) to track New York City garbage trucks.
/// While these are both great products, it really irritated me how hard it was to setup and play
/// around with so that you could get better at the tech. Then PostGIS came into the picture and
/// the whole space became 100x easier.
///
/// I went through a similar nightmare recently when I started working on a project involving
/// BlackBerry's [QNX](https://blackberry.qnx.com/en/products/foundation-software/qnx-rtos)
/// [RTOS](https://en.wikipedia.org/wiki/Real-time_operating_system). Before they acquired the
/// technology it was very easy to obtain. Now, I double dare you to become an expert in what is
/// a really interesting unix variant. You've got 30 days before your free trial license expires. At least
/// Oracle, Google and Apple have figured out that you need to make it easy for developers to build
/// things with your tech. Hey BlackBerry, you had the coolest phones on the market. I used to dream
/// of the day when I could own one. How's that going for you? Companies want to use your tech.
/// Good like hiring people that know how to develop for it. Dumb fucks.
///
/// There are two fundamental types of systems. Learning and controlling. BlackBerry chose
/// controlling, and it ended up destroying who they were by making them irrelevant.
///
/// Anderson Dawes:
///     We say, The more you share the more your bowl will be plentiful.
///      And those that will not share?
/// CROWD:
///     Welwalla!
/// Anderson Dawes:
///     Welwalla!
///
/// The Expanse - S2E7 [link](https://youtu.be/Db0eTW-1DRk?t=156)
///
/// The problem with using something like MySQL or PostgreSQL is that I would need to set up
/// containers and deal with networking and permissions, and they are a pain in the ass. While I
/// will need to deal with them someday, I don't want to now.
///
/// Then a thought hit me. What about [SQLIte](https://www.sqlite.org/index.html)? It seemed strange
/// to me that I had never used it on a project before. Back in the day I was too stupid and biased
/// to see it as something to do. Then, for this thing, I wanted to stick to something written in
/// Rust no matter how much of a pain in the ass it was.
///
/// But tools are just tools. And any good craftsman knows that you choose the right tool for the
/// job. So I decided to try it. Turns out, that it was a perfect fit. The DB package has the
/// fruitful results of that exploration. For what we're doing, it's perfect.
///
/// I'm going to need to determine the remaining cards for each headsup iteration. The easiest way
/// to do that is with the `Pile` trait, so let's implement it for `SortedHeadsUp`. We're not
/// going to need all of the things, but `remaining()` and all it entails will come in handy for
/// this work.
fn go() -> Result<(), PKError> {
    let deck = Cards::deck();

    let mut count: u32 = 1;
    for (i, v) in deck.combinations(2).enumerate() {
        let hero = Two::try_from(v.as_slice())?;

        println!("{} - {hero}", i + 1);
        for r in hero.remaining().combinations(2) {
            let villain = Two::try_from(r.as_slice())?;

            let hup = SortedHeadsUp::new(hero, villain);

            println!("{count} {i}  {hup}");
            count = count + 1;
        }
    }

    Ok(())
}
