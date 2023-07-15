use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open("generated/bcm.db")?;

    create_table(&conn)?;
    Ok(())
}

fn create_table(conn: &Connection) -> Result<usize> {
    conn.execute(
        "create table if not exists bcm (
            bc integer primary key,
            best integer not null,
            rank integer not null
         )",
        NO_PARAMS,
    )
}

fn insert_bcm(conn: &Connection) {
    
}
