use pkcore::analysis::store::bcm::binary_card_map::SevenFiveBCM;
use pkcore::analysis::store::db::sqlite::{Connect, Sqlable};
use pkcore::util::data::TestData;
use rusqlite::{Connection, Result};

/// [How to get back one row's data in rusqlite?](https://stackoverflow.com/questions/58449840/how-to-get-back-one-rows-data-in-rusqlite#comments-58523070)
///
/// _Old man's voice_: Back in my day we didn't have resources like [stackoverflow](https://stackoverflow.com/).
/// We had [O'Reilly In a Nutshell](https://www.oreilly.com/pub/a/tim/articles/inanut.html) books,
/// IF we were lucky, and we were grateful to have them.
///
/// Strangely enough, the Nutshell book that I got BY FAR the most use out of was David Flanagan's
/// [Java Examples in a Nutshell](https://www.oreilly.com/library/view/java-examples-in/0596006209/)
/// which was made up of code examples sorted by themes. This ended up being the foundation of the
/// Cookbook technical format that has become so popular.
///
/// Up until recently, you would just Bing what you were looking for and hope for the best. It
/// looks like now you will just be ChugGPTing things and letting the recycled intellectual
/// property of coders who actually knew what they were doing do the hard lifting for you.
/// **"[Soylent Green is people!](https://groovyhistory.com/soylent-green-is-people/8)"**
///
/// Fun fact: I went to high school with the daughter of the screen writer for Soylent Green.
///
/// While this is a very snarky take on things, it's what each new generation does. The dynamic is just
/// accelerating exponentially. That means that everytime you use AI to write your code for you,
/// you are helping to make the inevitable destruction of humanity by Skynet happen that much
/// sooner. SHAME! SHAME! (How crazy is it that Ted Lasso's Hannah Waddingham was the [Game of
/// Thrones shame nun](https://www.upi.com/Entertainment_News/TV/2021/09/16/Hannah-Waddingham-nun-Game-Thrones/2811631805048/)?)
///
/// # Meanwhile, back with trying to get the data out of our sqlite DB...
///
/// I must say that figuring out how to do this is difficult in Rust. The wonderful
/// [rusqlite](https://github.com/rusqlite/rusqlite) crate is in a lot of flux. TBH, that seems to
/// be more and more the norm programming. As the tools we use become more sophisticated, and the
/// people developing them get smarter, it's becoming harder and harder for documentation to keep
/// up. In a way, as much as I had the AI hype train does feel inevitable. Just note, that this will
/// only be the case after we've got through another boom/bust cycle ala Web 2.0 crypto and the
/// dot.bomb bubble that saved me from a life of retail management. Civilizations are the children
/// of massive amounts of stupidity and waste. That's just how we humans roll. (Can you tell I am
/// reading [Peter Zeihan](https://www.youtube.com/@ZeihanonGeopolitics) right now.
///
/// I've reached at one of my last resorts, which is running
/// [a query](https://github.com/search?q=named_params%21+rusqlite+select&type=code) against
/// GitHub for clues from other repositories that are doing selects against rusqlite.
///
/// Let's see if this snippet from  [1History](https://github.com/1History/1History) does
/// the trick...
///
/// ```
/// debug!("select from {name}, start:{start}, end:{end}");
///
/// let mut stat = self.conn.prepare(sql_tmpl)?;
/// let rows = stat.query_map(
///     named_params! {
///         ":start": start,
///         ":end": end,
///     },
///     |row| {
///         let detail = VisitDetail {
///             url: row.get(0)?,
///             title: row.get(1).unwrap_or_else(|_| "".to_string()),
///             visit_time: row.get(2)?,
///             visit_type: row.get(3)?,
///         };
///         Ok(detail)
///         },
///     )?;
///
///     let mut res: Vec<VisitDetail> = Vec::new();
///     for r in rows {
///         res.push(r?);
///     }
/// ```
///
/// BTW, Adding a `From<u64>` trait to `Bard` for easy struct realization.
///
/// I was really hoping that this would work for me: `let bc = Bard::from(row.get(0)?);`, but
/// the `Rust` compiler is a cruel mistress.
///
/// ```
/// error[E0277]: the trait bound `Bard: From<()>` is not satisfied
///    --> examples/generate_sqlite.rs:114:33
///     |
/// 114 |             let bc = Bard::from(row.get(0)?);
///     |                      ---------- ^^^^^^^^^^^ the trait `From<()>` is not implemented for `Bard`
///     |                      |
///     |                      required by a bound introduced by this call
///     |
///     = help: the following other types implement trait `From<T>`:
///               <Bard as From<Card>>
///               <Bard as From<Cards>>
///               <Bard as From<Vec<Card>>>
///               <Bard as From<u64>>
/// ```
///
/// Let's see how this does? It feels like we're getting closer.
///
/// ```
/// fn select_bcm(conn: &Connection, bc: &Bard) -> Result<BinaryCardMap, Error> {
///     let mut stmt = conn.prepare("SELECT bc, best, rank FROM bcm WHERE bc=:bc?")?;
///
///     let mut rows = stmt.query_map(
///         named_params! {":bc": bc.as_u64()},
///         |row| {
///             let bc = row.get(0)?;
///             let best = row.get(1)?;
///             let rank = row.get(2)?;
///
///             let bcm = BinaryCardMap {
///                 bc: Bard::from(bc),
///                 best: Bard::from(best),
///                 rank,
///             };
///             Ok(bcm)
///         },
///     )?;
///
///     let result = rows.next().ok_or(Error::InvalidQuery)?;
///     let bcm = result?;
///
///     Ok(bcm)
/// }
/// ```
///
/// Let's run it!
///
/// ```
/// error[E0277]: the trait bound `Bard: From<()>` is not satisfied
///    --> examples/generate_sqlite.rs:195:32
///     |
/// 195 |                 bc: Bard::from(bc),
///     |                     ---------- ^^ the trait `From<()>` is not implemented for `Bard`
///     |                     |
///     |                     required by a bound introduced by this call
///     |
///     = help: the following other types implement trait `From<T>`:
///               <Bard as From<Card>>
///               <Bard as From<Cards>>
///               <Bard as From<Vec<Card>>>
///               <Bard as From<u64>>
/// ```
///
/// What the fuckity fuck!!! The same stupid error. This causes me to remember Christoph's first
/// rule of debugging: The error is probably telling the truth. It's saying there's nothing there.
/// What if this were true? Let's dump some results and see what's what.
///
/// (When I am dumping out variables, this is me at my most masochistic.)
///
/// ```
/// fn select_bcm(conn: &Connection, bc: &Bard) -> Result<BinaryCardMap, Error> {
///     let mut stmt = conn.prepare("SELECT bc, best, rank FROM bcm WHERE bc=:bc?")?;
///
///     let mut rows = stmt.query_map(
///         named_params! {":bc": bc.as_u64()},
///         |row| {
///             println!("{:?}", row);
///             // let bc = row.get(0)?;
///             // let best = row.get(1)?;
///             // let rank = row.get(2)?;
///             //
///             // let bcm = BinaryCardMap {
///             //     bc: Bard::from(bc),
///             //     best: Bard::from(best),
///             //     rank,
///             // };
///             // Ok(bcm)
///             Ok(BinaryCardMap::default())
///         },
///     )?;
///
///     let result = rows.next().ok_or(Error::InvalidQuery)?;
///     let bcm = result?;
///
///     Ok(bcm)
/// }
/// ```
///
/// Well, hello there. This is new.
///
/// `Error: SqlInputError { error: Error { code: Unknown, extended_code: 1 }, msg: "near \"?\": syntax error", sql: "SELECT bc, best, rank FROM bcm WHERE bc=:bc?", offset: 43 }`
///
/// Here's a crazy idea... how about before I try to figure out how to extract the result from
/// sqlite, I make sure that I have a result from sqlite? BRILLIANT!!!
///
///
/// I must say that it's been a very very long time since I've done some serious SQLunking. For most
/// of my career mastering SQL was one of the most important skills in the developer's tool belt. I got
/// my start coding an Access database, than learning about this thing called SQL and betting my
/// career on Oracle. I even have my name misspelled as an editor or the Manning Book Java Persistence
/// in Action, which is a book about Object/Relational technologies like
/// [Hibernate](https://en.wikipedia.org/wiki/Hibernate_(framework)).
///
/// I was shocked when I started working for Pillar and the majority of the developers hated libraries
/// like hibernate, preferring to deal with raw SQL queries. Here I was thinking I was smart by
/// learning some hot tech only to find out that many perceived it as a crutch> Were they right?
/// IDK. I found them handy, but they're not a hill worth dying on. Testing... on the other hand...
///
/// Fired up Datagrip to see how the queries work by themselves [SCREENSHOT]. Turns out that they work just
/// fine through a good ol' fashioned SQL terminal. I always make sure that I have some sort of raw
/// way of trying out what I am doing. Before the tests... before the functions... we play with it.
///
/// Running these bad boys through it and they check out:
///
/// ```sql
/// INSERT INTO bcm (bc, best, rank) VALUES (1, 4468415255281664, 4362862139015168);
/// SELECT bc, best, rank FROM bcm WHERE bc=1;
/// ```
///
/// At first I was thinking that there was a problem with my insert call, but when I run it again
/// I get this error:
///
/// ```
/// Error: SqliteFailure(Error { code: ConstraintViolation, extended_code: 1555 }, Some("UNIQUE constraint failed: bcm.bc"))
/// ```
///
/// So, it's getting in there. Now, how do we get it out?
///
/// This is when I look at my select statement again, and I'm like WHAT THE FUCKING FUCK. What's
/// that question mark doing at the end of the statement???!!!
///
/// ```
/// let mut stmt = conn.prepare("SELECT bc, best, rank FROM bcm WHERE bc=:bc?")?;
/// ```
///
/// Remove it and whadayaknow?
///
/// ```
/// {Ok("bc"): (Integer, 4468415255281664), Ok("best"): (Integer, 4362862139015168), Ok("rank"): (Integer, 1)}
/// ```
///
/// Does this mean that we can actually extract the values from the result?
///
/// ```
/// let mut stmt = conn.prepare("SELECT bc, best, rank FROM bcm WHERE bc=:bc")?;
///
///     let mut rows = stmt.query_map(
///         named_params! {":bc": bc.as_u64()},
///         |row| {
///             let bc = row.get(0)?;
///             let best = row.get(1)?;
///             let rank = row.get(2)?;
///
///             let bcm = BinaryCardMap {
///                 bc: Bard::from(bc),
///                 best: Bard::from(best),
///                 rank,
///             };
///             Ok(bcm)
///         },
///     )?;
///
///     let result = rows.next().ok_or(Error::InvalidQuery)?;
///     let bcm = result?;
///
///     Ok(bcm)
/// ```
///
/// GAHH!!! The same stupid error!
///
/// ```
/// error[E0277]: the trait bound `Bard: From<()>` is not satisfied
///    --> examples/generate_sqlite.rs:301:32
///     |
/// 301 |                 bc: Bard::from(bc),
///     |                     ---------- ^^ the trait `From<()>` is not implemented for `Bard`
///     |                     |
///     |                     required by a bound introduced by this call
///     |
///     = help: the following other types implement trait `From<T>`:
///               <Bard as From<Card>>
///               <Bard as From<Cards>>
///               <Bard as From<Vec<Card>>>
///               <Bard as From<u64>>
/// ```
///
/// Let's try something... what if we assign types to the vars? After all, how does the lib
/// know what it's passing in? It's not like there's a chance in heck that this will work,
/// but it's worth a shot...
///
/// ```
/// let mut stmt = conn.prepare("SELECT bc, best, rank FROM bcm WHERE bc=:bc")?;
///
///     let mut rows = stmt.query_map(
///         named_params! {":bc": bc.as_u64()},
///         |row| {
///             let bc: u64 = row.get(0)?;
///             let best: u64 = row.get(1)?;
///             let rank: u16 = row.get(2)?;
///
///             let bcm = BinaryCardMap {
///                 bc: Bard::from(bc),
///                 best: Bard::from(best),
///                 rank,
///             };
///             Ok(bcm)
///         },
///     )?;
///
///     let result = rows.next().ok_or(Error::InvalidQuery)?;
///     let bcm = result?;
///
///     Ok(bcm)
/// ```
///
/// ```
/// BinaryCardMap { bc: Bard(4468415255281664), best: Bard(4362862139015168), rank: 1 }
/// ```
///
/// WHAT THE??? It worked?!!!
/// [Sometimes you have to roll the hard six.](https://www.youtube.com/watch?v=Dkc0RZ8Ym1Y)
///
/// Ladies and gentlemen, we have ourselves a ballgame.
///
/// The thing is, that while I love having the ability to do this with the `BinaryCardMap`,
/// it's not really the data that I want to store in SQL. The reason is, that this datatype
/// is designed to accelerate determining the `HandRank` for specific combinations of cards,
/// specifically, for preflop calculations. Even if it's significantly faster, we're still
/// doing the calculations millions of times per hand. Where storing values will really come in
/// handy is for caching preflop results. So let's close this chapter of our adventure, and
/// move on to that.
///
/// Oh... actually... there's one thing I want to try out first. I need a way to make sure
/// that a record is even there. Maybe `Option` is the way to go. Right now I'm just passing
/// down what comes from rusqlite, but now that I've got the code flow basically figured out,
/// I should be able to smooth down the edges and design a function contract that communicates
/// only what the caller needs to know.
///
/// This is going to require a major refactoring. Remember, before you refactor, commit. There's
/// nothing worst than getting trapped down a deep rabbit hole only to realize that you didn't
/// commit at a safe place before you fell in.
///
/// ASIDE: One thing you may be wondering is why I'm not test driving this whole adventure. The truth
/// is that I don't know how to code sqlite tests that stand up and tear down tables. This is
/// something I should figure out.
///
/// Right now, I'm using the same generated database over and over, and it's a real file. I don't
/// want to have to commit that to the repo. There has to be a better way. Back in the day I would
/// create this whole harness integration test thingy that would stand up and tear down databases
/// with every run. The tests were heavy, and required a running database connection. That was before
/// I started working with a bunch of
/// [London school mockists](https://medium.com/@adrianbooth/test-driven-development-wars-detroit-vs-london-classicist-vs-mockist-9956c78ae95f),
/// I was all about the integration tests. While I am still a classicist when it comes to testing,
/// (Detroit represent!) I do see the argument that you want your tests as light as possible. It's
/// tototally over the top batshit crazy to require a database be running to prove out your code,
/// even if it is one like sqlite.
///
/// But, I'm betting that sqlite has an in memory option. This is something to add to the TODO
/// column of this little adventure. For now, I've got bigger fish to fry.
///
/// ...
///
/// 'Tis done.
///
/// Here's the code refactored using our in-memory `Connect` struct. Next step is to translate this
/// into unit tests.
///
/// Honestly, I don't see a database as being a good tool for the binary card lookups compared to
/// the simple hashing function we are already using.
fn main() -> Result<()> {
    let conn = Connect::in_memory_connection()?;

    SevenFiveBCM::create_table(&conn.connection)?;

    match SevenFiveBCM::insert(&conn.connection, &TestData::spades_royal_flush_bcm()) {
        Ok(_) => println!("Record inserted"),
        Err(e) => println!("{e}"),
    }

    doit(&conn.connection, &TestData::spades_royal_flush_bcm());
    doit(&conn.connection, &TestData::spades_king_high_flush_bcm());

    Ok(())
}

fn doit(conn: &Connection, bcm: &SevenFiveBCM) {
    match SevenFiveBCM::select(&conn, &bcm.bc) {
        None => {
            println!("No such thing");
        }
        Some(r) => {
            println!("{:?}", r);
        }
    };
}
