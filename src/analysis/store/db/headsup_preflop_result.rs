use crate::analysis::store::db::sqlite::Sqlable;
use crate::arrays::matchups::SortedHeadsUp;
use crate::bard::Bard;
use rusqlite::Connection;

pub struct HUPResult {
    pub higher: Bard,
    pub lower: Bard,
    pub higher_wins: u64,
    pub lower_wins: u64,
    pub ties: u64,
}

impl HUPResult {}

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

    fn insert(conn: &Connection, record: &HUPResult) -> rusqlite::Result<usize> {
        todo!()
    }

    fn insert_many(conn: &Connection, records: Vec<&HUPResult>) -> rusqlite::Result<usize> {
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

    #[test]
    fn sqlable__create_table() {
        let conn = Connect::in_memory_connection().unwrap().connection;
        assert!(HUPResult::create_table(&conn).is_ok())
    }
}
