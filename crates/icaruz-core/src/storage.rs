use rusqlite::Connection;
use std::path::Path;

pub fn init_db(db_path: &Path) -> Result<(), String> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create data dir: {e}"))?;
    }
    let conn = Connection::open(db_path).map_err(|e| format!("Cannot open db: {e}"))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS history(id INTEGER PRIMARY KEY, server TEXT, started_at TEXT, detail TEXT);",
    )
    .map_err(|e| format!("Cannot migrate db: {e}"))?;
    Ok(())
}

pub fn save_setting(db_path: &Path, key: &str, value: &str) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| format!("Cannot open db: {e}"))?;
    conn.execute(
        "INSERT INTO settings(key, value) VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        (key, value),
    )
    .map_err(|e| format!("Cannot save setting: {e}"))?;
    Ok(())
}
