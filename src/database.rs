use rusqlite::{Connection, params};

use crate::{error, config};

pub(crate) struct TimeDTO {
    pub(crate) last_seen_local: u64,
    pub(crate) last_seen: u32,
}

pub(crate) async fn init_database(config: &mut config::Config) -> error::Result<()> {
    let conn = Connection::open(config.database_path.clone())
        .or(Err(error::new("could not open fencer.db".to_string())))?;

    conn.execute("CREATE TABLE IF NOT EXISTS timestamps (device TEXT PRIMARY KEY, last_seen_local INTEGER, last_seen INTEGER)", [])
        .or(Err(error::new("could not create timestamps table".to_string())))?;

    conn.execute("CREATE TABLE IF NOT EXISTS restarts (device TEXT PRIMARY KEY, counter INTEGER)", [])
        .or(Err(error::new("could not create timestamps table".to_string())))?;

    conn.close()
        .or(Err(error::new("Could not close database".to_string())))?;

    Ok(())
}

pub(crate) async fn get_times(database_path: String, device: String) -> error::Result<TimeDTO> {
    let conn = Connection::open(database_path)
        .or(Err(error::new("could not open fencer.db".to_string())))?;

    let timedto_obj: rusqlite::Result<TimeDTO> = conn.query_row("SELECT last_seen_local, last_seen FROM timestamps WHERE device = ?1", 
        params![device], |row| {
            let last_seen_local = row.get(0)?;
            let last_seen = row.get(1)?;

            Ok(TimeDTO {
                last_seen_local,
                last_seen,
            })
        });

    conn.close()
        .or(Err(error::new("Could not close database".to_string())))?;

    Ok(timedto_obj.unwrap_or({
        TimeDTO {
            last_seen_local: 0,
            last_seen: 0,
        }
    }))
}

pub(crate) async fn store_times(database_path: String, device: String, last_seen_local: u64, last_seen: u32) -> error::Result<()> {
    let conn = Connection::open(database_path)
        .or(Err(error::new("could not open fencer.db".to_string())))?;

    conn.execute("INSERT OR REPLACE INTO timestamps(device, last_seen_local, last_seen) VALUES (?1, ?2, ?3)", 
        params![device, last_seen_local, last_seen])
        .or(Err(error::new("could not insert or replace new timestamp".to_string())))?;

    conn.close()
        .or(Err(error::new("Could not close database".to_string())))?;

    Ok(())
}

pub(crate) async fn get_restarts(database_path: String, device: String) -> error::Result<u16> {
    let conn = Connection::open(database_path)
        .or(Err(error::new("could not open fencer.db".to_string())))?;

    let counter: rusqlite::Result<u16> = conn.query_row("SELECT counter FROM restarts WHERE device = ?1", 
        params![device], |row| {
            let counter = row.get(0)?;
            Ok(counter)
        });

    conn.close()
        .or(Err(error::new("Could not close database".to_string())))?;

    Ok(counter.unwrap_or(0))
}

pub(crate) async fn store_restarts(database_path: String, device: String, counter: u16) -> error::Result<()> {
    let conn = Connection::open(database_path)
        .or(Err(error::new("could not open fencer.db".to_string())))?;

    conn.execute("INSERT OR REPLACE INTO restarts(device, counter) VALUES (?1, ?2)", 
        params![device, counter])
        .or(Err(error::new("could not insert or replace new restart counter".to_string())))?;

    conn.close()
        .or(Err(error::new("Could not close database".to_string())))?;

    Ok(())
}