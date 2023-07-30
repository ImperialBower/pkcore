use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::{PKError, Shifty};
use rusqlite::Connection;
use std::collections::HashSet;

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
///
/// Now I'm trying to figure out how I can codify this logic. Maybe a method in `SortedHeadsUp` that
/// returns all possible versions.
///
/// Interestingly, when you filter the heads up combinations by sorting them and then putting them
/// in a HashSet you get a smaller number. This isn't surprising, but to me at least, interesting.
///
/// ```txt
/// Raw Combination Count: 1624351
/// SortedHeadsUp::allpossible() count: 812175
/// ```
///
/// This is all brought to us thanks to that remarkable of techs, the
/// [hash table](https://en.wikipedia.org/wiki/Hash_table).
///
/// OK, now let's destroy what was to create what will be. First, we codify what was for our
/// narrative...
///
/// ```
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
///
/// Much better:
///
/// ```
/// fn go() -> Result<(), PKError> {
///     let now = std::time::Instant::now();
///     let deck = Cards::deck();
///
///     let all_possible = SortedHeadsUp::all_possible()?;
///
///     for hup in all_possible.iter() {
///         println!("{hup}");
///     }
///
///     Ok(())
/// }
/// ```
///
/// Now the scary part. We're going to hit a time wall to drive through our code. Every calculation
/// is going to take a ton of time, and we can't really verify that it's working until we've loaded
/// our `BC_RANK_HASHMAP`, which takes a long, 4 geebees worth of time until we've got it in memory.
/// We're going to have to do it live `docs/files/bill-o-reilly-fuck-it-2746501037.gif`.
///
/// In a way this is exciting AND this is why Rust rules. So much of the stop and go pain of this
/// type of work is removed because the compiler catches things long before we press play.
///
/// The way I'm going to do this is by creating the scaffolding before I do the grunt work. This is
/// the area where system's developers using fuck up. They build things in a linear fashion, based
/// on "business value" not taking structural considerations into account. It's like putting in
/// all your kitchen appliances and tiles before you've built your foundation and plumbing.
/// Software developers are so far behind construction when it comes to building things.
///
/// Here's the blueprint:
///
/// * SortedHeadsUp::wins() where it just returns defaults.
/// * Convert SortedHeadsUp and Wins into a blank HUPResult.
/// * Store them in our DB.
///   * FORGOT TO ADD TRANSPOSITION.
/// * Once we verified that our DB plays nice, we can wire in our BC_RANK_HASHMAP megacache and do the real work.
///
/// So much of this is a refactoring of the `HeadsUp` struct work we did before when we were trying
/// to just get something into CSV. I've been here before, but that's OK, because I'm liking how
/// the code is flowing out much more now.
///
/// **STEP 5a** Transposition
///
/// Our next phase is going to slow us down in the short term but speed things up immensely in the
/// long term.
///
/// Well... I think we've taken this about as far as it will go without actually doing the thing...
/// aka calculating the odds. We could probably afford to clean up this code first.
///
/// There... that's a little better.
///
/// Let's try to tweak this a little bit.
fn go() -> Result<(), PKError> {
    let now = std::time::Instant::now();

    let conn = Connection::open(":memory:").unwrap();
    HUPResult::create_table(&conn).expect("TODO: panic message");
    let all_possible = SortedHeadsUp::unique()?;

    insert(&conn, &all_possible);
    validate(&conn, &all_possible);

    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}

fn insert(conn: &Connection, all_possible: &HashSet<SortedHeadsUp>) {
    let mut count = 0;

    for shu in all_possible.iter() {
        count = count + 1;
        println!("Inserting #{count} {shu}");
        let wins = shu.wins().unwrap();
        let possible_sorts = shu.shifts();

        for sorted in possible_sorts.iter() {
            if HUPResult::select(&conn, &sorted).is_none() {
                let hupr = HUPResult::from_sorted_heads_up(&sorted, &wins);
                HUPResult::insert(&conn, &hupr).expect("TODO: panic message");
            } else {
                println!(">>>>> NO INSERT");
            }
        }
    }
}

fn validate(conn: &Connection, all_possible: &HashSet<SortedHeadsUp>) {
    let mut count = 0;
    // Now let's make sure they're there.
    for hup in all_possible.iter() {
        count = count + 1;
        print!("Validating #{count} {hup}");
        let r = HUPResult::select(&conn, &hup);
        assert!(r.is_some());
        println!(" || {}", r.unwrap().to_string());
    }
}

/// cargo run --example hup
fn main() -> Result<(), PKError> {
    go()
}
