use rusqlite::Connection;
use crate::analysis::store::db::sqlite::Sqlable;
use crate::arrays::matchups::SortedHeadsUp;
use crate::bard::Bard;

pub struct HUPResult {
    pub higher: Bard,
    pub lower: Bard,
    pub higher_wins: u64,
    pub lower_wins: u64,
    pub ties_wins: u64,
}

impl HUPResult {}

impl Sqlable<HUPResult, SortedHeadsUp> for HUPResult {
    fn create_table(conn: &Connection) -> rusqlite::Result<usize> {
        todo!()
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