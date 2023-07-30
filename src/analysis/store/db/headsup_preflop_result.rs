use crate::analysis::store::bcm::binary_card_map::BC_RANK_HASHMAP;
use crate::analysis::store::db::sqlite::Sqlable;
use crate::arrays::five::Five;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::arrays::seven::Seven;
use crate::bard::Bard;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::wins::Wins;
use crate::{PKError, Pile, Shifty, SuitShift};
use csv::{Reader, WriterBuilder};
use rusqlite::{named_params, Connection};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs::File;

/// TODO TD: Why u64 not usize?
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct HUPResult {
    pub higher: Bard,
    pub lower: Bard,
    pub higher_wins: u64,
    pub lower_wins: u64,
    pub ties: u64,
}

impl HUPResult {
    pub fn db_count(conn: &Connection) -> (usize, usize) {
        let all = HUPResult::select_all(conn);
        let len = all.len();
        let mut hs = HashSet::new();
        for hup in all {
            hs.insert(hup);
        }
        (len, hs.len())
    }

    pub fn db_is_valid(conn: &Connection) -> bool {
        let (v, hs) = HUPResult::db_count(conn);
        v == hs
    }

    /// `assert_eq!(first_ties, second_ties);`
    /// This is something I want to get much more into the habit of writing. An assertion that's
    /// simply a sanity check. There is no way that these two values shouldn't be equal, so,
    /// just to be safe, let's add an a check here.
    ///
    /// I haven't used `.into()` before. It's really cute, but does have a
    /// [gotcha](https://users.rust-lang.org/t/cant-convert-usize-to-u64/6243/4). I'm not
    /// worried about it, but let's see a few years from now if my future self is cursing me
    /// over this.
    ///
    /// BOO!!! Doesn't work, and I was all excited it. This is a no go:
    ///
    /// ```txt
    /// HUPResult {
    ///   higher: Default::default(),
    ///   lower: Default::default(),
    ///   higher_wins: first_wins.into(),
    ///   lower_wins: second_wins.into(),
    ///   ties: first_ties.into(),
    /// }
    /// error[E0277]: the trait bound `u64: From<usize>` is not satisfied
    ///   --> src/analysis/store/db/headsup_preflop_result.rs:39:37
    ///    |
    /// 39 |             higher_wins: first_wins.into(),
    ///    |                                     ^^^^ the trait `From<usize>` is not implemented for `u64`
    ///    |
    ///    = help: the following other types implement trait `From<T>`:
    ///              <u64 as From<bool>>
    ///              <u64 as From<char>>
    ///              <u64 as From<u8>>
    ///              <u64 as From<u16>>
    ///              <u64 as From<u32>>
    ///              <u64 as From<gimli::read::cfi::Pointer>>
    ///              <u64 as From<NonZeroU64>>
    ///    = note: required for `usize` to implement `Into<u64>`
    /// ```
    ///
    /// How about we write a doctest to make sure things are working OK?
    ///
    /// ```
    /// use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
    /// use pkcore::util::data::TestData;
    ///
    /// assert_eq!(
    ///     TestData::the_hand_as_hup_result(),
    ///     HUPResult::from_sorted_heads_up(
    ///         &TestData::the_hand_sorted_headsup(),
    ///         &TestData::the_hand_as_wins()
    ///     )
    /// );
    /// ```
    ///
    /// # Panics
    ///
    /// Casting from usize to u64. I'd be impressed if we got hit with this one.
    #[must_use]
    pub fn from_sorted_heads_up(shu: &SortedHeadsUp, wins: &Wins) -> Self {
        let (first_wins, first_ties) = wins.wins_for(Win::FIRST);
        let (second_wins, second_ties) = wins.wins_for(Win::SECOND);

        assert_eq!(first_ties, second_ties);

        HUPResult {
            higher: shu.higher_as_bard(),
            lower: shu.lower_as_bard(),
            higher_wins: u64::try_from(first_wins - first_ties).unwrap(),
            lower_wins: u64::try_from(second_wins - second_ties).unwrap(),
            ties: u64::try_from(first_ties).unwrap(),
        }
    }

    /// # Errors
    ///
    /// Unable to create csv file.
    pub fn generate_csv_from_hash_set(
        path: &str,
        hups: HashSet<HUPResult>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        HUPResult::generate_csv_from_vector(path, &Vec::from_iter(hups))
    }

    /// # Errors
    ///
    /// Unable to create csv file.
    pub fn generate_csv_from_vector(
        path: &str,
        hups: &[HUPResult],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = WriterBuilder::new().has_headers(true).from_path(path)?;
        for hup in hups {
            wtr.serialize(hup)?;
        }
        wtr.flush()?;
        Ok(())
    }

    #[must_use]
    pub fn get_sorted_heads_up(&self) -> Option<SortedHeadsUp> {
        match SortedHeadsUp::try_from(self) {
            Ok(shu) => Some(shu),
            Err(_) => None,
        }
    }

    /// # Errors
    ///
    /// Unable to open connection
    ///
    /// # Panics
    ///
    /// Unable to close connection
    pub fn read_db(path: &str) -> rusqlite::Result<Vec<HUPResult>> {
        let conn = Connection::open(path)?;
        let hups = HUPResult::select_all(&conn);
        conn.close().unwrap();
        Ok(hups)
    }

    /// # Errors
    ///
    /// * Throws `PKError::InvalidBinaryFormat` if the csv file is corrupted.
    /// * Throws `PKError::Fubar` if unable to open at all.
    pub fn read_csv(path: &str) -> Result<Vec<HUPResult>, PKError> {
        match File::open(path) {
            Ok(file) => {
                let mut rdr = Reader::from_reader(file);
                let mut v = Vec::new();
                for hup in rdr.deserialize::<HUPResult>() {
                    match hup {
                        Ok(r) => v.push(r),
                        Err(_) => {
                            return Err(PKError::InvalidBinaryFormat);
                        }
                    }
                }
                Ok(v)
            }
            Err(_) => Err(PKError::Fubar),
        }
    }
}

impl Display for HUPResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let sho = match SortedHeadsUp::try_from(self) {
            Ok(s) => s,
            Err(_) => SortedHeadsUp::default(),
        };

        // let higher_two = match Two::try_from(self.higher) {
        //     Ok(t) => t,
        //     Err(_) => Two::default(),
        // };
        // let lower_two = match Two::try_from(self.lower) {
        //     Ok(t) => t,
        //     Err(_) => Two::default(),
        // };
        write!(
            f,
            "{} ({}) {} ({}) ties: ({})",
            sho.higher, self.higher_wins, sho.lower, self.lower_wins, self.ties
        )
    }
}

impl From<&SortedHeadsUp> for HUPResult {
    /// Clippy doesn't like our higher lower section. Normally, this is a
    /// lint I turn off, but let's do it.
    ///
    /// Here's the original:
    ///
    /// ```txt
    /// if high_rank.rank < low_rank.rank {
    ///   wins.add(Win::FIRST);
    /// } else if low_rank.rank < high_rank.rank {
    ///   wins.add(Win::SECOND);
    /// } else {
    ///   wins.add(Win::FIRST | Win::SECOND);
    /// }
    /// ```
    ///
    /// And, of course, I invert the match, which loses me another 10 minutes. Once we close this
    /// epic, we're going to need to setup an odds service to isolate this into something we can
    /// just keep running in the background.
    fn from(shu: &SortedHeadsUp) -> Self {
        let higher_bard = shu.higher.bard();
        let lower_bard = shu.lower.bard();

        let mut wins = Wins::default();

        // I honestly love how easy our code makes us do stuff like that. When it flows like
        // water, you know you're on the right track.
        for combo in shu.remaining().combinations(5) {
            let five = Five::try_from(combo).unwrap();
            let high7 = Seven::from_case_at_deal(shu.higher, five)
                .unwrap()
                .to_bard();
            let low7 = Seven::from_case_at_deal(shu.lower, five).unwrap().to_bard();

            let high_rank = BC_RANK_HASHMAP.get(&high7).unwrap();
            let low_rank = BC_RANK_HASHMAP.get(&low7).unwrap();

            match high_rank.rank.cmp(&low_rank.rank) {
                Ordering::Less => wins.add(Win::FIRST),
                Ordering::Greater => wins.add(Win::SECOND),
                Ordering::Equal => wins.add(Win::FIRST | Win::SECOND),
            };
        }

        let (higher_wins, higher_ties) = wins.wins_for(Win::FIRST);
        let (lower_wins, lower_ties) = wins.wins_for(Win::SECOND);
        assert_eq!(higher_ties, lower_ties);

        let ties = u64::try_from(lower_ties).unwrap();

        HUPResult {
            higher: higher_bard,
            lower: lower_bard,
            higher_wins: u64::try_from(higher_wins).unwrap() - ties,
            lower_wins: u64::try_from(lower_wins).unwrap() - ties,
            ties: u64::try_from(lower_ties).unwrap(),
        }
    }
}

impl Sqlable<HUPResult, SortedHeadsUp> for HUPResult {
    fn create_table(conn: &Connection) -> rusqlite::Result<usize> {
        log::debug!("HUPResult::create_table({:?})", conn);
        conn.execute(
            "create table if not exists nlh_headsup_result
            (
                id          integer not null
                    constraint nlh_headsup_result_pk
                        primary key,
                higher      integer not null,
                lower       integer not null,
                higher_wins integer not null,
                lower_wins  integer not null,
                ties        integer not null
            );

            create index if not exists nlh_headsup_result_higher_index
                on nlh_headsup_result (higher);

            create index if not exists nlh_headsup_result_lower_index
                on nlh_headsup_result (lower);",
            [],
        )
    }

    /// This was written to Paul van Dyk's
    /// [VONYC Sessions #873](https://www.youtube.com/watch?v=9NdjCGH83UI&t=5073s).
    ///
    /// TODO: Write about music and mood and pairing.
    ///
    /// Oops. Little miss on the sig. Fixed now.
    fn exists(conn: &Connection, shu: &SortedHeadsUp) -> bool {
        HUPResult::select(conn, shu).is_some()
    }

    /// Refactoring this to only insert if the record isn't already there.
    ///
    /// Returns true if the record isn't already there. False if it is.
    fn insert(conn: &Connection, hup: &HUPResult) -> rusqlite::Result<bool> {
        log::debug!("HUPResult::insert({})", hup);

        let shu = hup
            .get_sorted_heads_up()
            .ok_or(rusqlite::Error::ExecuteReturnedResults)?;

        if HUPResult::exists(conn, &shu) {
            log::debug!("Record {shu} already exists.");
            Ok(false)
        } else {
            let mut stmt = conn.prepare(
                "INSERT INTO nlh_headsup_result \
            (higher, lower, higher_wins, lower_wins, ties) VALUES \
            (:higher, :lower, :higher_wins, :lower_wins, :ties)",
            )?;
            stmt.execute(named_params! {
            ":higher": hup.higher.as_u64(),
            ":lower": hup.lower.as_u64(),
            ":higher_wins": hup.higher_wins,
            ":lower_wins": hup.lower_wins,
            ":ties": hup.ties})?;
            Ok(true)
        }
    }

    fn insert_many(_conn: &Connection, _records: Vec<&HUPResult>) -> rusqlite::Result<usize> {
        todo!()
    }

    fn select(conn: &Connection, key: &SortedHeadsUp) -> Option<HUPResult> {
        log::debug!("HUPResult::select({:?})", conn);
        let mut stmt = conn
            .prepare(
                "SELECT higher_wins, lower_wins, ties \
            FROM nlh_headsup_result WHERE higher=:higher and lower=:lower",
            )
            .ok()?;

        let hb = key.higher().bard();
        let lb = key.lower().bard();

        let hup = stmt
            .query_row(
                named_params! {
                    ":higher": hb.as_u64(),
                    ":lower": lb.as_u64(),
                },
                |row| {
                    let hw = row.get(0)?;
                    let lw = row.get(1)?;
                    let ties = row.get(2)?;

                    let r = HUPResult {
                        higher: hb,
                        lower: lb,
                        higher_wins: hw,
                        lower_wins: lw,
                        ties,
                    };
                    Ok(r)
                },
            )
            .ok()?;

        Some(hup)
    }

    /// OK, so these results are completely foobared.
    ///
    /// ```txt
    /// /home/gaoler/.cargo/bin/cargo run --color=always --package pkcore --example hups
    ///     Finished dev [unoptimized + debuginfo] target(s) in 0.05s
    ///      Running `target/debug/examples/hups`
    /// K♠ K♦ (137438955520) __ __ (37210) ties: (37210)
    /// K♥ 6♦ (268435584) __ __ (1090190) ties: (610489)
    /// K♥ 6♦ (268435584) 3♣ 2♣ (1090190) ties: (610489)
    /// A♠ 5♦ (412316860416) __ __ (406764) ties: (1228716)
    /// Q♦ J♦ (70369012613120) 4♣ 2♣ (1198761) ties: (498275)
    /// Q♦ J♦ (70369012613120) 4♣ 3♣ (1198761) ties: (498275)
    /// 9♠ 6♣ (67239936) 4♣ 3♣ (1136466) ties: (393246)
    /// 8♣ 5♣ (3221225472) __ __ (906176) ties: (729584)
    /// ...
    /// ```
    ///
    /// Oopsie... forgot that there's an index column.
    ///
    /// Much better:
    ///
    /// ```txt
    /// /home/gaoler/.cargo/bin/cargo run --color=always --package pkcore --example hups
    ///      Finished dev [unoptimized + debuginfo] target(s) in 0.05s
    ///       Running `target/debug/examples/hups`
    ///  K♠ K♦ (37210) K♥ K♣ (37210) ties: (1637884)
    ///  K♥ 6♦ (1090190) 9♣ 4♥ (610489) ties: (11625)
    ///  K♥ 6♦ (1090190) 9♣ 4♥ (610489) ties: (11625)
    ///  A♠ 5♦ (406764) A♥ K♥ (1228716) ties: (76824)
    ///  Q♦ J♦ (1198761) 9♠ 4♥ (498275) ties: (15268)
    /// ...
    /// ```
    fn select_all(conn: &Connection) -> Vec<HUPResult> {
        log::debug!("HUPResult::select_all({:?})", conn);

        let mut stmt = conn
            .prepare("SELECT * FROM nlh_headsup_result")
            .ok()
            .unwrap();

        let mut r: Vec<HUPResult> = Vec::new();
        let mut hups = stmt.query(()).unwrap();
        while let Some(row) = hups.next().unwrap() {
            let higher: u64 = row.get(1).unwrap();
            let lower: u64 = row.get(2).unwrap();
            let higher_wins: u64 = row.get(3).unwrap();
            let lower_wins: u64 = row.get(4).unwrap();
            let ties: u64 = row.get(5).unwrap();
            let hup = HUPResult {
                higher: Bard::from(higher),
                lower: Bard::from(lower),
                higher_wins,
                lower_wins,
                ties,
            };
            r.push(hup);
        }
        r
    }
}

impl SuitShift for HUPResult {
    fn shift_suit_down(&self) -> Self {
        let shu = match SortedHeadsUp::try_from(self) {
            Ok(s) => s.shift_suit_down(),
            Err(_) => SortedHeadsUp::default(),
        };
        HUPResult {
            higher: shu.higher_as_bard(),
            lower: shu.lower_as_bard(),
            higher_wins: self.higher_wins,
            lower_wins: self.lower_wins,
            ties: self.ties,
        }
    }

    /// I AM AN IDIOT!
    ///
    /// The original version of this function does the `SuitShift` twice. That's why it isn't
    /// working correctly.
    ///
    /// ```txt
    /// fn shift_suit_up(&self) -> Self {
    ///   let mut shu = match SortedHeadsUp::try_from(self) {
    ///     Ok(s) => s.shift_suit_up(),
    ///     Err(_) => SortedHeadsUp::default(),
    ///   };
    ///   shu = shu.shift_suit_up(); //AHHH!!!!!
    ///   HUPResult {
    ///     higher: shu.higher_as_bard(),
    ///     lower: shu.lower_as_bard(),
    ///     higher_wins: self.higher_wins,
    ///     lower_wins: self.lower_wins,
    ///     ties: self.ties,
    ///   }
    /// }
    /// ```
    fn shift_suit_up(&self) -> Self {
        let shu = match SortedHeadsUp::try_from(self) {
            Ok(s) => s.shift_suit_up(),
            Err(_) => SortedHeadsUp::default(),
        };
        HUPResult {
            higher: shu.higher_as_bard(),
            lower: shu.lower_as_bard(),
            higher_wins: self.higher_wins,
            lower_wins: self.lower_wins,
            ties: self.ties,
        }
    }

    fn opposite(&self) -> Self {
        let shu = match SortedHeadsUp::try_from(self) {
            Ok(s) => s.opposite(),
            Err(_) => SortedHeadsUp::default(),
        };
        HUPResult {
            higher: shu.higher_as_bard(),
            lower: shu.lower_as_bard(),
            higher_wins: self.higher_wins,
            lower_wins: self.lower_wins,
            ties: self.ties,
        }
    }
}

impl Shifty for HUPResult {}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__store__db__hupresult_tests {
    use super::*;
    use crate::analysis::store::db::sqlite::Connect;
    use crate::arrays::two::Two;
    use crate::util::data::TestData;
    use std::collections::HashSet;

    const SAMPLE_DB_PATH: &str = "data/sample_hups.db";

    #[test]
    fn db_count() {
        let conn = Connection::open(SAMPLE_DB_PATH).unwrap();
        let (v, hs) = HUPResult::db_count(&conn);
        assert_eq!(v, hs);
        conn.close().unwrap();
    }

    #[test]
    fn db_is_valid() {
        let conn = Connection::open(SAMPLE_DB_PATH).unwrap();
        assert!(HUPResult::db_is_valid(&conn));
        conn.close().unwrap();
    }

    #[test]
    fn get_sorted_heads_up() {
        assert_eq!(
            TestData::the_hand_sorted_headsup(),
            TestData::the_hand_as_hup_result()
                .get_sorted_heads_up()
                .unwrap()
        );
    }

    /// I'm test driving this one backwards. I do that some time.
    #[test]
    fn display() {
        assert_eq!(
            "6♠ 6♥ (1365284) 5♦ 5♣ (314904) ties: (32116)",
            TestData::the_hand_as_hup_result().to_string()
        );
    }

    /// This is going to be a very very heavy test, since we will need to load our
    /// 4GB binary bard map cache into memory before we can even do the calculation.
    /// Once we get it to pass, we can ignore it, and punch it into an example to run.
    ///
    /// Fudge! The test fails.
    ///
    /// ```txt
    /// Left:  HUPResult { higher: Bard(8797166764032), lower: Bard(65544), higher_wins: 1397400, lower_wins: 347020, ties: 32116 }
    /// Right: HUPResult { higher: Bard(8797166764032), lower: Bard(65544), higher_wins: 1365284, lower_wins: 314904, ties: 32116 }
    /// ```
    ///
    /// So, let's see what the difference is.
    ///
    /// ```txt
    /// 1397400 - 1365284 = 32116
    /// 347020 - 314904 = 32116
    /// ```
    ///
    /// **Smacks forehead.** Our old bcrepl subtracts the ties from the wins entries. That explains
    /// that. I could try to consolidate the code, but right now I just want to start getting results
    /// into sqlite.
    ///
    /// This time for sure!
    ///
    /// Subtracting times from each wins makes the test pass. Now, we're going to lock it in the
    /// vault with an ignore.
    #[test]
    #[ignore]
    fn from__sorted_heads_up() {
        let actual = HUPResult::from(&TestData::the_hand_sorted_headsup());

        assert_eq!(actual, TestData::the_hand_as_hup_result());
    }

    #[test]
    fn sqlable__create_table() {
        let conn = Connect::in_memory_connection().unwrap().connection;
        assert!(HUPResult::create_table(&conn).is_ok());
        conn.close().unwrap();
    }

    #[test]
    fn sqlable__exists() {
        // Preamble
        let conn = Connect::in_memory_connection().unwrap().connection;
        HUPResult::create_table(&conn).unwrap();
        let the_hand = TestData::the_hand_as_hup_result();

        // the work
        let inserted = HUPResult::insert(&conn, &the_hand).unwrap();

        // the proof
        assert!(HUPResult::exists(
            &conn,
            &TestData::the_hand_sorted_headsup()
        ));
        assert!(inserted);
        conn.close().unwrap()
    }

    /// ```
    /// use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
    /// use pkcore::bard::Bard;
    /// HUPResult {
    ///     higher: Bard::SIX_SPADES | Bard::SIX_HEARTS,
    ///     lower: Bard::FIVE_DIAMONDS | Bard::FIVE_CLUBS,
    ///     higher_wins: 1_365_284,
    ///     lower_wins: 314_904,
    ///     ties: 32_116,
    /// };
    /// ```
    #[test]
    fn sqlable__insert() {
        let conn = Connect::in_memory_connection().unwrap().connection;
        HUPResult::create_table(&conn).unwrap();

        let first_time = HUPResult::insert(&conn, &TestData::the_hand_as_hup_result());
        let second_time = HUPResult::insert(&conn, &TestData::the_hand_as_hup_result());

        assert!(first_time.is_ok());
        assert!(first_time.unwrap());
        assert!(second_time.is_ok());
        assert!(!second_time.unwrap());
        conn.close().unwrap();
    }

    #[test]
    fn sqlable__select() {
        let conn = Connect::in_memory_connection().unwrap().connection;
        HUPResult::create_table(&conn).unwrap();
        HUPResult::insert(&conn, &TestData::the_hand_as_hup_result()).unwrap();

        let actual = HUPResult::select(&conn, &TestData::the_hand_sorted_headsup());
        let nope = HUPResult::select(&conn, &SortedHeadsUp::new(Two::HAND_6S_6H, Two::HAND_5S_5D));

        assert!(actual.is_some());
        assert_eq!(TestData::the_hand_as_hup_result(), actual.unwrap());
        assert!(nope.is_none());
        conn.close().unwrap()
    }

    #[test]
    fn sqlable__select_all() {
        let conn = Connect::in_memory_connection().unwrap().connection;
        HUPResult::create_table(&conn).unwrap();
        HUPResult::insert(&conn, &TestData::the_hand_as_hup_result()).unwrap();

        let actual = HUPResult::select_all(&conn);

        assert_eq!(actual.len(), 1);
        assert_eq!(&TestData::the_hand_as_hup_result(), actual.get(0).unwrap());
        conn.close().unwrap()
    }

    #[test]
    fn suit_shift__shift_suit_down() {
        assert_eq!(hup1().shift_suit_down(), hup2());
    }

    #[test]
    fn suit_shift__shift_suit_up() {
        assert_eq!(hup1().shift_suit_up(), hup4());
    }

    /// These tests are a pain in the ass to setup. Not sure what an easier way to do it is. Slow
    /// and stupid wins the race I guess.
    #[test]
    fn shifty__shifts() {
        let actual = hup1().shifts();

        assert!(actual.contains(&hup1()));
        assert!(actual.contains(&hup3()));

        assert!(actual.contains(&hup2()));
        assert!(actual.contains(&hup4()));
        assert_eq!(actual.len(), 4);
        assert_eq!(hs(), actual);
    }

    /// Test data
    fn hup1() -> HUPResult {
        HUPResult {
            higher: Two::HAND_7D_7C.bard(),
            lower: Two::HAND_6S_6H.bard(),
            higher_wins: 1375342,
            lower_wins: 315362,
            ties: 21600,
        }
    }

    fn hup2() -> HUPResult {
        HUPResult {
            higher: Two::HAND_7S_7C.bard(),
            lower: Two::HAND_6H_6D.bard(),
            higher_wins: 1375342,
            lower_wins: 315362,
            ties: 21600,
        }
    }

    fn hup3() -> HUPResult {
        HUPResult {
            higher: Two::HAND_7S_7H.bard(),
            lower: Two::HAND_6D_6C.bard(),
            higher_wins: 1375342,
            lower_wins: 315362,
            ties: 21600,
        }
    }

    fn hup4() -> HUPResult {
        HUPResult {
            higher: Two::HAND_7H_7D.bard(),
            lower: Two::HAND_6S_6C.bard(),
            higher_wins: 1375342,
            lower_wins: 315362,
            ties: 21600,
        }
    }

    fn v() -> Vec<HUPResult> {
        let v: Vec<HUPResult> = vec![hup1(), hup2(), hup3(), hup4()];
        v
    }

    fn hs() -> HashSet<HUPResult> {
        let mut hs = HashSet::new();
        for hup in v() {
            hs.insert(hup);
        }
        hs
    }
}
