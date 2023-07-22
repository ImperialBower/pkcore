use crate::analysis::store::db::sqlite::Sqlable;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::wins::Wins;
use crate::Pile;
use rusqlite::{named_params, Connection};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct HUPResult {
    pub higher: Bard,
    pub lower: Bard,
    pub higher_wins: u64,
    pub lower_wins: u64,
    pub ties: u64,
}

impl HUPResult {
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
    ///         &TestData::wins_the_hand()
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
}

impl Display for HUPResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let higher_two = match Two::try_from(self.higher) {
            Ok(t) => t,
            Err(_) => Two::default(),
        };
        let lower_two = match Two::try_from(self.lower) {
            Ok(t) => t,
            Err(_) => Two::default(),
        };
        write!(
            f,
            "{higher_two} ({}) {lower_two} ({}) ties: ({})",
            self.higher_wins, self.lower_wins, self.ties
        )
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

    fn insert(conn: &Connection, hup: &HUPResult) -> rusqlite::Result<usize> {
        log::debug!("HUPResult::insert({})", hup);
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
            ":ties": hup.ties
        })
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
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__store__db__hupresult_tests {
    use super::*;
    use crate::analysis::store::db::sqlite::Connect;
    use crate::util::data::TestData;

    /// I'm test driving this one backwards. I do that some time.
    #[test]
    fn display() {
        assert_eq!(
            "6♠ 6♥ (1365284) 5♦ 5♣ (314904) ties: (32116)",
            TestData::the_hand_as_hup_result().to_string()
        );
    }

    #[test]
    fn sqlable__create_table() {
        let conn = Connect::in_memory_connection().unwrap().connection;
        assert!(HUPResult::create_table(&conn).is_ok())
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
        assert!(HUPResult::insert(&conn, &TestData::the_hand_as_hup_result()).is_ok())
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
    }
}
