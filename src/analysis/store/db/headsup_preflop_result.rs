use crate::analysis::store::db::sqlite::Sqlable;
use crate::arrays::matchups::SortedHeadsUp;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::PKError;
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

impl HUPResult {}

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
        todo!()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__store__db__hupresult_tests {
    use super::*;
    use crate::analysis::store::db::sqlite::Connect;
    use crate::util::data::TestData;

    #[test]
    fn display() {
        assert_eq!("", TestData::the_hand_as_hup_result().to_string());
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
}
