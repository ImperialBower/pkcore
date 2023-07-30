use rusqlite::Connection;

pub struct Connect {
    pub connection: Connection,
}

impl Connect {
    /// Now we're ready to start setting the Sqlite code into the actual library. Right off the path
    /// I am intrigued with what what rusqlite's `Connection::open` takes as a parameter. It looks like
    /// a simple `&str` but it's actually something I didn't even know was a thing, but now that I see
    /// it, of course it is:
    ///
    /// ```text
    /// #[inline]
    /// pub fn open<P: AsRef<Path>>(path: P) -> Result<Connection> {
    ///     let flags = OpenFlags::default();
    ///     Connection::open_with_flags(path, flags)
    /// }
    /// ```
    ///
    /// There's a lot of foo embedded in that little String slice. I am making a mental note to come
    /// back to this. it's cool AF. Now, question is, can my lil' static factory method easily
    /// leverage it without me needing to level up in Rust several classes?
    ///
    /// AND, how cool are [inline functions](https://en.wikipedia.org/wiki/Inline_expansion)? I'm
    /// sure all you college types know all about them, but they only occupy a forgotten part of
    /// my brain. There's also a whole section in the [Rust Performance Book](https://nnethercote.github.io/perf-book/inlining.html).
    ///
    /// Initially, I was going to do something like this:
    ///
    /// ```txt
    /// pub fn new(connection: Connection) -> Connect {
    ///     Connect {
    ///         connection,
    ///     }
    /// }
    /// ```
    ///
    /// This is the old `Java` dev me portion of my brain. So, I decided to write a rusty `From`
    /// trait implementation. Honestly, this may be a bit of my usual overthinking, but I want
    /// to have a convenient to generate different types of connections, as well as save utility
    /// methods for archiving etc. Maybe it's a total waste of ascii, but since I'm in new
    /// territory, I will err on the side of overdoing things.
    ///
    /// Plus, this is going to be the home of all of our caching logic for our odds calculations.
    ///
    /// First thing I need is an easy way to generate an in-memory database for testing. Honestly,
    /// I would have maimed for something like this back in my Oracle days.
    ///
    /// Now, I need some `Bard` backed structs that will hold the results of different preflop
    /// calculations. For now I'm going to just include them in this module.
    ///
    /// # Errors
    ///
    /// Not sure how it would happen, but it's possible. ¯\_(ツ)_/¯
    pub fn in_memory_connection() -> rusqlite::Result<Connect> {
        Ok(Connect {
            connection: Connection::open(":memory:")?,
        })
    }
}

impl From<Connection> for Connect {
    fn from(connection: Connection) -> Self {
        Connect { connection }
    }
}

pub trait Sqlable<T, S> {
    /// OK, this whole trait slaps! It's nothing substantial, but I love the cleanliness of it,
    /// and the fact that I can even code it. Don't forget to take the Ws.
    ///
    /// # Errors
    ///
    /// Throws an error if rusqlite isn't able to create the table.
    fn create_table(conn: &Connection) -> rusqlite::Result<usize>;

    fn exists(conn: &Connection, record: &S) -> bool;

    /// # Errors
    ///
    /// Throws an error if rusqlite isn't able to insert the record into the table. Should not
    /// throw if the record is already there.
    fn insert(conn: &Connection, record: &T) -> rusqlite::Result<bool>;

    /// # Errors
    ///
    /// Throws an error if rusqlite isn't able to insert any record into the table. Should not
    /// throw if the record is already there.
    fn insert_many(conn: &Connection, records: Vec<&T>) -> rusqlite::Result<usize>;

    fn select(conn: &Connection, key: &S) -> Option<T>;

    fn select_all(conn: &Connection) -> Vec<T>;
}
