use crate::analysis::store::bcm::binary_card_map::BC_RANK_HASHMAP;
use crate::analysis::store::db::sqlite::Sqlable;
use crate::arrays::five::Five;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::arrays::seven::Seven;
use crate::bard::Bard;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::wins::Wins;
use crate::{Pile, SuitShift};
use rusqlite::{named_params, Connection};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
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

impl From<SortedHeadsUp> for HUPResult {
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
    fn from(shu: SortedHeadsUp) -> Self {
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
            }
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

impl SuitShift for HUPResult {
    fn shift_suit_down(&self) -> Self {
        let mut shu = match SortedHeadsUp::try_from(self) {
            Ok(s) => s.shift_suit_down(),
            Err(_) => SortedHeadsUp::default(),
        };
        shu = shu.shift_suit_down();
        HUPResult {
            higher: shu.higher_as_bard(),
            lower: shu.lower_as_bard(),
            higher_wins: self.higher_wins,
            lower_wins: self.lower_wins,
            ties: self.ties,
        }
    }

    fn shift_suit_up(&self) -> Self {
        let mut shu = match SortedHeadsUp::try_from(self) {
            Ok(s) => s.shift_suit_up(),
            Err(_) => SortedHeadsUp::default(),
        };
        shu = shu.shift_suit_up();
        HUPResult {
            higher: shu.higher_as_bard(),
            lower: shu.lower_as_bard(),
            higher_wins: self.higher_wins,
            lower_wins: self.lower_wins,
            ties: self.ties,
        }
    }

    fn opposite(&self) -> Self {
        let mut shu = match SortedHeadsUp::try_from(self) {
            Ok(s) => s.opposite(),
            Err(_) => SortedHeadsUp::default(),
        };
        shu = shu.opposite();
        HUPResult {
            higher: shu.higher_as_bard(),
            lower: shu.lower_as_bard(),
            higher_wins: self.higher_wins,
            lower_wins: self.lower_wins,
            ties: self.ties,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__store__db__hupresult_tests {
    use super::*;
    use crate::analysis::store::db::sqlite::Connect;
    use crate::arrays::two::Two;
    use crate::util::data::TestData;

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
        let actual = HUPResult::from(TestData::the_hand_sorted_headsup());

        assert_eq!(actual, TestData::the_hand_as_hup_result());
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

    #[test]
    fn suit_shift__shift_suit_down() {
        let hup = TestData::the_hand_as_hup_result();
        let mut shifted = hup.shift_suit_down();
        for _ in 0..3 {
            shifted = shifted.shift_suit_down();
        }

        assert_eq!(hup, shifted);
    }

    #[test]
    fn suit_shift__shift_suit_up() {
        let hup = TestData::the_hand_as_hup_result();
        let mut shifted = hup.shift_suit_up();
        for _ in 0..3 {
            shifted = shifted.shift_suit_up();
        }

        assert_eq!(hup, shifted);
    }
}
