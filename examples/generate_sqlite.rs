use pkcore::analysis::store::bcm::binary_card_map::BinaryCardMap;
use pkcore::util::data::TestData;
use rusqlite::{named_params, Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open("generated/bcm.db")?;

    create_table(&conn)?;
    insert_bcm(&conn, &TestData::spades_royal_flush_bcm())?;

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

fn select_bcm(conn: &Connection, bc: &Bard) {
    let mut stmt = conn.prepare("SELECT bc, best, rank FROM bcm WHERE bc=:bc?")?;

    stmt.q

    stmt.execute(named_params! {
    ":bc": bcm.bc.as_u64(),
    ":best": bcm.best.as_u64(),
    ":rank": u64::from(bcm.rank)
    })
}
