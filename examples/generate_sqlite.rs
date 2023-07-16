use pkcore::analysis::store::bcm::binary_card_map::BinaryCardMap;
use pkcore::util::data::TestData;
use rusqlite::{named_params, Connection, Result};
use pkcore::bard::Bard;

fn main() -> Result<()> {
    let conn = Connection::open("generated/bcm.db")?;

    let bcm = TestData::spades_royal_flush_bcm();

    create_table(&conn)?;
    // insert_bcm(&conn, &bcm)?;
    select_bcm(&conn, &bcm.bc);

    Ok(())
}

fn create_table(conn: &Connection) -> Result<usize> {
    conn.execute(
        "create table if not exists bcm (
            bc integer primary key,
            best integer not null,
            rank integer not null
         )",
        [],
    )
}

fn insert_bcm(conn: &Connection, bcm: &BinaryCardMap) -> Result<usize> {
    let mut stmt = conn.prepare("INSERT INTO bcm (bc, best, rank) VALUES (:bc, :best, :rank)")?;
    stmt.execute(named_params! {
    ":bc": bcm.bc.as_u64(),
    ":best": bcm.best.as_u64(),
    ":rank": u64::from(bcm.rank)
    })
}

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
/// of massive amounts of stupidity and waste. That's just how we humans roll. (Can you tell I
fn select_bcm(conn: &Connection, bc: &Bard) -> Option<()> {
    let mut stmt = conn.prepare("SELECT bc, best, rank FROM bcm WHERE bc=:bc?").ok()?;

    let mut rows = stmt.query(named_params! {":bc": bc.as_u64()}).ok()?;

    while let Some(row) = rows.next().ok()? {
        row.get(0).ok()?;
        println!("{:?}", row);
    }
    None
}
