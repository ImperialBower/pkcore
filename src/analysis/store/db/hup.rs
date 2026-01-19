use crate::analysis::gto::odds::WinLoseDraw;
use crate::analysis::store::db::sqlite::Sqlable;
use crate::arrays::matchups::masked::{MASKED_DISTINCT, MASKED_UNIQUE, Masked};
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::wins::Wins;
use crate::{PKError, Pile, Shifty, SuitShift};
use csv::{Reader, WriterBuilder};
use rusqlite::{Connection, Statement, named_params};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs::File;

/// TODO TD: Why u64 not usize?
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct HUPResult {
    pub higher: Bard,
    pub lower: Bard,
    pub odds: WinLoseDraw,
}

impl HUPResult {
    const DEFAULT_DB_PATH: &'static str = "generated/hups.db";
    const ENV_KEY: &'static str = "HUPS_DB_PATH";

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

    #[must_use]
    pub fn db_path() -> String {
        dotenvy::var(Self::ENV_KEY).unwrap_or_else(|_| Self::DEFAULT_DB_PATH.to_string())
    }

    /// # Errors
    ///
    /// Throws `PKError::SqlError` if unable to select from db.
    pub fn from_db(conn: &Connection, from: &Two, to: &Two) -> Result<HUPResult, PKError> {
        let shu = SortedHeadsUp::new(*from, *to);
        HUPResult::select(conn, &shu).ok_or(PKError::SqlError)
    }

    #[must_use]
    pub fn flip_mode(&self) -> Self {
        HUPResult {
            higher: self.lower,
            lower: self.higher,
            odds: WinLoseDraw {
                wins: self.odds.losses,
                losses: self.odds.wins,
                draws: self.odds.draws,
            },
        }
    }

    /// # THIS IS WRONG
    ///
    /// I'm doing it in the inverse order. I need to do this from `SortedHeadsUp` and pass in
    /// the connection to see if it's there.
    #[must_use]
    pub fn from_shift(&self, shu: &SortedHeadsUp) -> Option<Self> {
        let shifts = self.shifts();

        for shift in shifts {
            if shift.get_sorted_heads_up().is_some_and(|s| s == *shu) {
                let hpr = HUPResult {
                    higher: shu.higher_as_bard(),
                    lower: shu.lower_as_bard(),
                    odds: WinLoseDraw {
                        wins: shift.odds.wins,
                        losses: shift.odds.losses,
                        draws: shift.odds.draws,
                    },
                };
                return Some(hpr);
            }
        }
        None
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
    /// use pkcore::analysis::store::db::hup::HUPResult;
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
            odds: WinLoseDraw {
                wins: u64::try_from(first_wins - first_ties).unwrap_or(0),
                losses: u64::try_from(second_wins - second_ties).unwrap_or(0),
                draws: u64::try_from(first_ties).unwrap_or(0),
            },
        }
    }

    /// # Errors
    ///
    /// Unable to create csv file.
    pub fn generate_csv_from_hash_set(path: &str, hups: HashSet<HUPResult>) -> Result<(), Box<dyn std::error::Error>> {
        HUPResult::generate_csv_from_vector(path, &Vec::from_iter(hups))
    }

    /// # Errors
    ///
    /// Unable to create csv file.
    pub fn generate_csv_from_vector(path: &str, hups: &[HUPResult]) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = WriterBuilder::new().has_headers(true).from_path(path)?;
        for hup in hups {
            wtr.serialize(hup)?;
        }
        wtr.flush()?;
        Ok(())
    }

    #[must_use]
    pub fn get_sorted_heads_up(&self) -> Option<SortedHeadsUp> {
        SortedHeadsUp::try_from(self).ok()
    }

    /// # Errors
    ///
    /// Returns error if unable to open connection.
    pub fn open_connection() -> rusqlite::Result<Connection> {
        Connection::open(Self::db_path())
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
        match conn.close() {
            Ok(()) => Ok(hups),
            Err((_conn, err)) => Err(err),
        }
    }

    /// # Errors
    ///
    /// Returns error if db contains duplicate entries..
    pub fn check_db(conn: &Connection) -> Result<usize, PKError> {
        let (v, hs) = HUPResult::db_count(conn);
        if v == hs { Ok(v) } else { Err(PKError::DuplicateCard) }
    }

    pub fn select_from_shifts(conn: &Connection, masked: &Masked) -> Option<HUPResult> {
        for shift in masked.shifts() {
            match HUPResult::select(conn, &shift.shu) {
                None => {}
                Some(hupr) => {
                    return Some(hupr);
                }
            }
        }
        None
    }

    pub fn remaining(conn: &Connection, mut hands: HashSet<Masked>) -> HashSet<Masked> {
        let hups = HUPResult::select_all(conn);
        for hup in hups {
            hands.remove(&Masked::from(hup));
        }
        hands
    }

    pub fn distinct_remaining(conn: &Connection) -> HashSet<Masked> {
        let distinct = MASKED_DISTINCT.clone();
        HUPResult::remaining(conn, distinct)
    }

    pub fn unique_remaining(conn: &Connection) -> HashSet<Masked> {
        let distinct = MASKED_UNIQUE.clone();
        HUPResult::remaining(conn, distinct)
    }

    #[must_use]
    pub fn matches(&self, other: &Self) -> bool {
        (self.odds.wins == other.odds.wins)
            && (self.odds.losses == other.odds.losses)
            && (self.odds.draws == other.odds.draws)
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

    // region private methods

    fn fold(&self, masked: &Masked) -> Self {
        let mymask = Masked::from(self);
        if mymask.rank_mask == masked.rank_mask {
            HUPResult {
                higher: masked.shu.higher_as_bard(),
                lower: masked.shu.lower_as_bard(),
                odds: WinLoseDraw {
                    wins: self.odds.wins,
                    losses: self.odds.losses,
                    draws: self.odds.draws,
                },
            }
        } else if mymask.rank_mask == masked.rank_mask.invert() {
            HUPResult {
                higher: masked.shu.higher_as_bard(),
                lower: masked.shu.lower_as_bard(),
                odds: WinLoseDraw {
                    wins: self.odds.losses,
                    losses: self.odds.wins,
                    draws: self.odds.draws,
                },
            }
        } else {
            HUPResult::default()
        }
    }

    // endregion
}

impl Display for HUPResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let higher = Two::try_from(self.higher).unwrap_or_default();
        let lower = Two::try_from(self.lower).unwrap_or_default();

        write!(
            f,
            "{} {:.2}% ({}) {} {:.2}% ({}) ties: {:.2}% ({})",
            higher,
            self.odds.win_percentage(),
            self.odds.wins,
            lower,
            self.odds.loss_percentage(),
            self.odds.losses,
            self.odds.draw_percentage(),
            self.odds.draws,
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
    ///
    /// ## Refactoring Update
    ///
    /// I've written `SortedHeadsUp.wins()` and tested it, and what do you know, I already have a
    /// test here, that's ignored to validate this calculation. So let's refactor this to leverage
    /// what we've got now.
    fn from(shu: &SortedHeadsUp) -> Self {
        let wins = shu.wins().unwrap_or_default();

        let (higher_wins, higher_ties) = wins.wins_for(Win::FIRST);
        let (lower_wins, lower_ties) = wins.wins_for(Win::SECOND);
        assert_eq!(higher_ties, lower_ties);

        let ties = u64::try_from(lower_ties).unwrap_or_default();

        HUPResult {
            higher: shu.higher.bard(),
            lower: shu.lower.bard(),
            odds: WinLoseDraw {
                wins: u64::try_from(higher_wins).unwrap_or_default() - ties,
                losses: u64::try_from(lower_wins).unwrap_or_default() - ties,
                draws: u64::try_from(lower_ties).unwrap_or_default(),
            },
        }
    }
}

impl Sqlable<HUPResult, SortedHeadsUp> for HUPResult {
    fn create_table(conn: &Connection) -> rusqlite::Result<usize> {
        log::debug!("HUPResult::create_table({conn:?})");
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
        log::debug!("HUPResult::insert({hup})");

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
            ":higher_wins": hup.odds.wins,
            ":lower_wins": hup.odds.losses,
            ":ties": hup.odds.draws})?;
            Ok(true)
        }
    }

    fn insert_many(_conn: &Connection, _records: Vec<&HUPResult>) -> rusqlite::Result<usize> {
        todo!()
    }

    fn select(conn: &Connection, key: &SortedHeadsUp) -> Option<HUPResult> {
        log::debug!("HUPResult::select({conn:?})");
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
                        odds: WinLoseDraw {
                            wins: hw,
                            losses: lw,
                            draws: ties,
                        },
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
    #[allow(clippy::unwrap_used)]
    fn select_all(conn: &Connection) -> Vec<HUPResult> {
        log::debug!("HUPResult::select_all({conn:?})");

        let mut stmt: Statement = match conn.prepare("SELECT * FROM nlh_headsup_result") {
            Ok(statement) => statement,
            Err(e) => {
                log::error!("Error preparing statement: {e}");
                return Vec::new();
            }
        };

        let mut r: Vec<HUPResult> = Vec::new();
        let mut hups = stmt.query(()).unwrap();
        while let Some(row) = hups.next().unwrap() {
            let higher: u64 = row.get(1).unwrap_or_default();
            let lower: u64 = row.get(2).unwrap_or_default();
            let higher_wins: u64 = row.get(3).unwrap_or_default();
            let lower_wins: u64 = row.get(4).unwrap_or_default();
            let ties: u64 = row.get(5).unwrap_or_default();
            let hup = HUPResult {
                higher: Bard::from(higher),
                lower: Bard::from(lower),
                odds: WinLoseDraw {
                    wins: higher_wins,
                    losses: lower_wins,
                    draws: ties,
                },
            };
            r.push(hup);
        }
        r
    }
}

impl SuitShift for HUPResult {
    fn shift_suit_down(&self) -> Self {
        match SortedHeadsUp::try_from(self) {
            Ok(s) => {
                let first = s.higher.shift_suit_down();
                let second = s.lower.shift_suit_down();

                if second > first {
                    HUPResult {
                        higher: second.bard(),
                        lower: first.bard(),
                        odds: WinLoseDraw {
                            wins: self.odds.losses,
                            losses: self.odds.wins,
                            draws: self.odds.draws,
                        },
                    }
                } else {
                    HUPResult {
                        higher: first.bard(),
                        lower: second.bard(),
                        odds: WinLoseDraw {
                            wins: self.odds.wins,
                            losses: self.odds.losses,
                            draws: self.odds.draws,
                        },
                    }
                }
            }
            Err(_) => HUPResult::default(),
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
        match SortedHeadsUp::try_from(self) {
            Ok(s) => {
                let first = s.higher.shift_suit_up();
                let second = s.lower.shift_suit_up();

                if second > first {
                    HUPResult {
                        higher: second.bard(),
                        lower: first.bard(),
                        odds: WinLoseDraw {
                            wins: self.odds.losses,
                            losses: self.odds.wins,
                            draws: self.odds.draws,
                        },
                    }
                } else {
                    HUPResult {
                        higher: first.bard(),
                        lower: second.bard(),
                        odds: WinLoseDraw {
                            wins: self.odds.wins,
                            losses: self.odds.losses,
                            draws: self.odds.draws,
                        },
                    }
                }
            }
            Err(_) => HUPResult::default(),
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
            odds: WinLoseDraw {
                wins: self.odds.wins,
                losses: self.odds.losses,
                draws: self.odds.draws,
            },
        }
    }
}

impl Shifty for HUPResult {
    fn shifts(&self) -> HashSet<Self>
    where
        Self: Sized,
    {
        let masks = Masked::from(self).other_shifts();
        let mut shifts: HashSet<Self> = HashSet::new();
        shifts.insert(*self);

        for mask in masks {
            shifts.insert(self.fold(&mask));
        }
        shifts
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__store__db__hupresult_tests {
    use super::*;
    use crate::analysis::store::db::sqlite::Connect;
    use crate::arrays::two::Two;
    use crate::util::data::TestData;
    use std::str::FromStr;

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
    fn flip_mode() {
        let hup = HUPResult {
            higher: Bard::SIX_SPADES | Bard::SIX_HEARTS,
            lower: Bard::FIVE_DIAMONDS | Bard::FIVE_CLUBS,
            odds: WinLoseDraw {
                wins: 1_365_284,
                losses: 314_904,
                draws: 32_116,
            },
        };

        let hup_flipped = hup.flip_mode();

        assert_eq!(hup_flipped.higher, Bard::FIVE_DIAMONDS | Bard::FIVE_CLUBS);
    }

    #[test]
    fn matches() {
        let first = HUPResult {
            higher: Bard::SIX_SPADES | Bard::SIX_HEARTS,
            lower: Bard::FIVE_DIAMONDS | Bard::FIVE_CLUBS,
            odds: WinLoseDraw {
                wins: 1_365_284,
                losses: 314_904,
                draws: 32_116,
            },
        };
        let second = HUPResult {
            higher: Bard::SIX_DIAMONDS | Bard::SIX_CLUBS,
            lower: Bard::FIVE_SPADES | Bard::FIVE_HEARTS,
            odds: WinLoseDraw {
                wins: 1_365_284,
                losses: 314_904,
                draws: 32_116,
            },
        };

        assert!(first.matches(&second));
    }

    #[test]
    fn matches_not() {
        let first = HUPResult {
            higher: Bard::ACE_SPADES | Bard::FIVE_HEARTS,
            lower: Bard::FOUR_HEARTS | Bard::TREY_HEARTS,
            odds: WinLoseDraw {
                wins: 1_068_796,
                losses: 632_976,
                draws: 10_532,
            },
        };
        let second = HUPResult {
            higher: Bard::ACE_CLUBS | Bard::FIVE_CLUBS,
            lower: Bard::FOUR_DIAMONDS | Bard::FIVE_CLUBS,
            odds: WinLoseDraw {
                wins: 1_145_595,
                losses: 556_028,
                draws: 10_681,
            },
        };

        assert!(!first.matches(&second));
    }

    #[test]
    fn get_sorted_heads_up() {
        assert_eq!(
            TestData::the_hand_sorted_headsup(),
            TestData::the_hand_as_hup_result().get_sorted_heads_up().unwrap()
        );
    }

    // T♠ T♦ - T♥ 2♦ Type1223d 1010,0110 0000100000000,0000100000001
    // T♠ T♥ - T♣ 2♥ Type1223d 1100,0101 0000100000000,0000100000001
    // T♥ T♣ - T♦ 2♣ Type1223d 0101,0011 0000100000000,0000100000001
    // T♥ T♦ - T♣ 2♦ Type1223d 0110,0011 0000100000000,0000100000001
    // T♠ T♦ - T♣ 2♦ Type1223d 1010,0011 0000100000000,0000100000001
    // T♠ T♣ - T♥ 2♣ Type1223d 1001,0101 0000100000000,0000100000001
    // T♠ T♣ - T♦ 2♣ Type1223d 1001,0011 0000100000000,0000100000001
    // T♠ T♥ - T♦ 2♥ Type1223d 1100,0110 0000100000000,0000100000001
    // T♠ 2♣ - T♥ T♣ Type1223d 1001,0101 0000100000001,0000100000000
    // T♠ 2♦ - T♥ T♦ Type1223d 1010,0110 0000100000001,0000100000000
    // T♠ 2♣ - T♦ T♣ Type1223d 1001,0011 0000100000001,0000100000000
    // T♥ 2♣ - T♦ T♣ Type1223d 0101,0011 0000100000001,0000100000000
    #[test]
    fn fold() {
        let base = HUPResult {
            higher: Two::HAND_TS_2H.bard(),
            lower: Two::HAND_TH_TD.bard(),
            odds: WinLoseDraw {
                wins: 73_828,
                losses: 1_580_550,
                draws: 57_926,
            },
        };
        let masked = Masked::from_str("T♠ 2♣ T♥ T♣").unwrap();

        let folded = base.fold(&masked);

        assert_eq!(folded.higher, Two::HAND_TS_2C.bard());
        assert_eq!(folded.lower, Two::HAND_TH_TC.bard());
        assert_eq!(folded.odds.wins, 73_828);
        assert_eq!(folded.odds.losses, 1_580_550);
        assert_eq!(folded.odds.draws, 57_926);
    }

    #[test]
    fn fold_inverted() {
        let base = HUPResult {
            higher: Two::HAND_TS_2H.bard(),
            lower: Two::HAND_TH_TD.bard(),
            odds: WinLoseDraw {
                wins: 73_828,
                losses: 1_580_550,
                draws: 57_926,
            },
        };
        let masked = Masked::from_str("T♠ T♥ T♣ 2♥").unwrap();

        let folded = base.fold(&masked);

        assert_eq!(folded.higher, Two::HAND_TS_TH.bard());
        assert_eq!(folded.lower, Two::HAND_TC_2H.bard());
        assert_eq!(folded.odds.wins, 1_580_550);
        assert_eq!(folded.odds.losses, 73_828);
        assert_eq!(folded.odds.draws, 57_926);
    }

    /// I'm test driving this one backwards. I do that some time.
    #[test]
    fn display() {
        assert_eq!(
            "6♠ 6♥ 79.73% (1365284) 5♦ 5♣ 18.39% (314904) ties: 1.88% (32116)",
            TestData::the_hand_as_hup_result().to_string()
        );
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
        assert!(HUPResult::exists(&conn, &TestData::the_hand_sorted_headsup()));
        assert!(inserted);
        conn.close().unwrap()
    }

    /// ```
    /// use pkcore::analysis::gto::odds::WinLoseDraw;
    /// use pkcore::analysis::store::db::hup::HUPResult;
    /// use pkcore::bard::Bard;
    /// HUPResult {
    ///     higher: Bard::SIX_SPADES | Bard::SIX_HEARTS,
    ///     lower: Bard::FIVE_DIAMONDS | Bard::FIVE_CLUBS,
    ///     odds: WinLoseDraw {
    ///         wins: 1_365_284,
    ///         losses: 314_904,
    ///         draws: 32_116,
    ///     },
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

    /// This is the worst case edge case, and why you need to watch out for those edge cases and
    /// write tests. You're assumptions are what will kill you. Those you need to test.
    ///
    /// This was a big miss by me, that the shifts will invert the order of the sort, and thus the
    /// results need to be inverted.
    #[test]
    fn suit_shift__shift_suit_up__defect() {
        let base = HUPResult {
            higher: Two::HAND_TS_2H.bard(),
            lower: Two::HAND_TH_TD.bard(),
            odds: WinLoseDraw {
                wins: 73_828,
                losses: 1_580_550,
                draws: 57_926,
            },
        };
        let shiftup = base.shift_suit_up();

        assert_eq!(shiftup.higher, Two::HAND_TS_TH.bard());
        assert_eq!(shiftup.lower, Two::HAND_TC_2S.bard());
        assert_eq!(shiftup.odds.wins, 1_580_550);
        assert_eq!(shiftup.odds.losses, 73_828);
        assert_eq!(shiftup.odds.draws, 57_926);
        assert_eq!(shiftup.shift_suit_down(), base);
    }

    /// These tests are a pain in the ass to setup. Not sure what an easier way to do it is. Slow
    /// and stupid wins the race I guess.
    #[test]
    fn shifty__shifts() {
        let actual = hup1().shifts();

        assert!(actual.contains(&hup1()));
        assert!(actual.contains(&hup2()));
        assert!(actual.contains(&hup3()));
        assert!(actual.contains(&hup4()));
        assert!(actual.contains(&hup5()));
        assert!(actual.contains(&hup6()));

        assert_eq!(actual.len(), 6);
        assert_eq!(hs(), actual);
    }

    /// Test data

    // 7♠ 7♦ (1375342) 6♥ 6♣ (315362) ties: (21600)

    fn hup1() -> HUPResult {
        HUPResult {
            higher: Two::HAND_7D_7C.bard(),
            lower: Two::HAND_6S_6H.bard(),
            odds: WinLoseDraw {
                wins: 1375342,
                losses: 315362,
                draws: 21600,
            },
        }
    }

    fn hup2() -> HUPResult {
        HUPResult {
            higher: Two::HAND_7S_7C.bard(),
            lower: Two::HAND_6H_6D.bard(),
            odds: WinLoseDraw {
                wins: 1375342,
                losses: 315362,
                draws: 21600,
            },
        }
    }

    fn hup3() -> HUPResult {
        HUPResult {
            higher: Two::HAND_7S_7H.bard(),
            lower: Two::HAND_6D_6C.bard(),
            odds: WinLoseDraw {
                wins: 1375342,
                losses: 315362,
                draws: 21600,
            },
        }
    }

    fn hup4() -> HUPResult {
        HUPResult {
            higher: Two::HAND_7H_7D.bard(),
            lower: Two::HAND_6S_6C.bard(),
            odds: WinLoseDraw {
                wins: 1375342,
                losses: 315362,
                draws: 21600,
            },
        }
    }

    fn hup5() -> HUPResult {
        HUPResult {
            higher: Two::HAND_7H_7C.bard(),
            lower: Two::HAND_6S_6D.bard(),
            odds: WinLoseDraw {
                wins: 1375342,
                losses: 315362,
                draws: 21600,
            },
        }
    }

    fn hup6() -> HUPResult {
        HUPResult {
            higher: Two::HAND_7S_7D.bard(),
            lower: Two::HAND_6H_6C.bard(),
            odds: WinLoseDraw {
                wins: 1375342,
                losses: 315362,
                draws: 21600,
            },
        }
    }

    fn v() -> Vec<HUPResult> {
        let v: Vec<HUPResult> = vec![hup1(), hup2(), hup3(), hup4(), hup5(), hup6()];
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
