use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Option<i64>,
    pub server: String,
    pub port: u16,
    pub started_at: String,
    pub duration_secs: u64,
    pub error: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub server: String,
    pub port: u16,
    pub label: String,
    pub added_at: String,
}

fn ensure_column(conn: &Connection, table: &str, col: &str, ddl: &str) -> Result<(), String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| format!("Cannot inspect db: {e}"))?;
    let cols: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .map_err(|e| format!("Cannot inspect db: {e}"))?
        .flatten()
        .collect();
    if !cols.iter().any(|c| c == col) {
        conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {ddl}"), [])
            .map_err(|e| format!("Cannot migrate db: {e}"))?;
    }
    Ok(())
}

pub fn init_db(db_path: &Path) -> Result<(), String> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create data dir: {e}"))?;
    }
    let conn = Connection::open(db_path).map_err(|e| format!("Cannot open db: {e}"))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS history(
           id INTEGER PRIMARY KEY,
           server TEXT NOT NULL DEFAULT '',
           port INTEGER NOT NULL DEFAULT 7777,
           started_at TEXT NOT NULL DEFAULT '',
           duration_secs INTEGER NOT NULL DEFAULT 0,
           error TEXT NOT NULL DEFAULT '',
           detail TEXT NOT NULL DEFAULT ''
         );
         CREATE TABLE IF NOT EXISTS favorites(
           server TEXT NOT NULL,
           port INTEGER NOT NULL,
           label TEXT NOT NULL DEFAULT '',
           added_at TEXT NOT NULL DEFAULT '',
           PRIMARY KEY(server, port)
         );",
    )
    .map_err(|e| format!("Cannot migrate db: {e}"))?;
    for (col, ddl) in [
        ("port", "port INTEGER NOT NULL DEFAULT 7777"),
        ("duration_secs", "duration_secs INTEGER NOT NULL DEFAULT 0"),
        ("error", "error TEXT NOT NULL DEFAULT ''"),
    ] {
        ensure_column(&conn, "history", col, ddl)?;
    }
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

pub fn get_setting(db_path: &Path, key: &str) -> Result<Option<String>, String> {
    let conn = Connection::open(db_path).map_err(|e| format!("Cannot open db: {e}"))?;
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key=?1")
        .map_err(|e| format!("Cannot read setting: {e}"))?;
    let mut rows = stmt
        .query([key])
        .map_err(|e| format!("Cannot read setting: {e}"))?;
    match rows.next() {
        Ok(Some(r)) => r
            .get::<_, String>(0)
            .map(Some)
            .map_err(|e| format!("Cannot read setting: {e}")),
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Cannot read setting: {e}")),
    }
}

fn row_to_history(r: &Row) -> rusqlite::Result<HistoryEntry> {
    Ok(HistoryEntry {
        id: r.get(0)?,
        server: r.get(1)?,
        port: r.get::<_, i64>(2)? as u16,
        started_at: r.get(3)?,
        duration_secs: r.get::<_, i64>(4)? as u64,
        error: r.get(5)?,
        detail: r.get(6)?,
    })
}

pub fn record_history(db_path: &Path, e: &HistoryEntry) -> Result<i64, String> {
    let conn = Connection::open(db_path).map_err(|e| format!("Cannot open db: {e}"))?;
    conn.execute(
        "INSERT INTO history(server, port, started_at, duration_secs, error, detail)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &e.server,
            e.port as i64,
            &e.started_at,
            e.duration_secs as i64,
            &e.error,
            &e.detail,
        ),
    )
    .map_err(|e| format!("Cannot record history: {e}"))?;
    Ok(conn.last_insert_rowid())
}

pub fn list_history(db_path: &Path, limit: i64) -> Result<Vec<HistoryEntry>, String> {
    let conn = Connection::open(db_path).map_err(|e| format!("Cannot open db: {e}"))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, server, port, started_at, duration_secs, error, detail
             FROM history ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| format!("Cannot read history: {e}"))?;
    let rows = stmt
        .query_map([limit.max(1)], row_to_history)
        .map_err(|e| format!("Cannot read history: {e}"))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("Cannot read history: {e}"))?);
    }
    Ok(out)
}

pub fn add_favorite(db_path: &Path, f: &Favorite) -> Result<(), String> {
    if f.server.trim().is_empty() {
        return Err("Favorite server is empty.".to_string());
    }
    let conn = Connection::open(db_path).map_err(|e| format!("Cannot open db: {e}"))?;
    conn.execute(
        "INSERT INTO favorites(server, port, label, added_at) VALUES(?1, ?2, ?3, ?4)
         ON CONFLICT(server, port) DO UPDATE SET label=excluded.label, added_at=excluded.added_at",
        (&f.server, f.port as i64, &f.label, &f.added_at),
    )
    .map_err(|e| format!("Cannot save favorite: {e}"))?;
    Ok(())
}

pub fn remove_favorite(db_path: &Path, server: &str, port: u16) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| format!("Cannot open db: {e}"))?;
    conn.execute(
        "DELETE FROM favorites WHERE server=?1 AND port=?2",
        (server, port as i64),
    )
    .map_err(|e| format!("Cannot remove favorite: {e}"))?;
    Ok(())
}

pub fn list_favorites(db_path: &Path) -> Result<Vec<Favorite>, String> {
    let conn = Connection::open(db_path).map_err(|e| format!("Cannot open db: {e}"))?;
    let mut stmt = conn
        .prepare("SELECT server, port, label, added_at FROM favorites ORDER BY added_at DESC")
        .map_err(|e| format!("Cannot read favorites: {e}"))?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Favorite {
                server: r.get(0)?,
                port: r.get::<_, i64>(1)? as u16,
                label: r.get(2)?,
                added_at: r.get(3)?,
            })
        })
        .map_err(|e| format!("Cannot read favorites: {e}"))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("Cannot read favorites: {e}"))?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_db(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(name);
        let _ = std::fs::remove_file(&p);
        p
    }

    use std::path::PathBuf;

    #[test]
    fn history_and_favorites_roundtrip() {
        let db = tmp_db("icaruz-storage-p4-test.db");
        init_db(&db).unwrap();
        let id = record_history(
            &db,
            &HistoryEntry {
                id: None,
                server: "127.0.0.1".into(),
                port: 7777,
                started_at: "2026-01-01T00:00:00Z".into(),
                duration_secs: 120,
                error: "".into(),
                detail: "clean exit".into(),
            },
        )
        .unwrap();
        assert!(id > 0);
        let h = list_history(&db, 10).unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].duration_secs, 120);
        add_favorite(
            &db,
            &Favorite {
                server: "127.0.0.1".into(),
                port: 7777,
                label: "Local".into(),
                added_at: "2026-01-01T00:00:00Z".into(),
            },
        )
        .unwrap();
        let f = list_favorites(&db).unwrap();
        assert_eq!(f.len(), 1);
        remove_favorite(&db, "127.0.0.1", 7777).unwrap();
        assert!(list_favorites(&db).unwrap().is_empty());
        let _ = std::fs::remove_file(&db);
    }

    #[test]
    fn migrates_legacy_history_table() {
        let db = tmp_db("icaruz-storage-p4-legacy.db");
        let _ = std::fs::remove_file(&db);
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch(
            "CREATE TABLE history(id INTEGER PRIMARY KEY, server TEXT, started_at TEXT, detail TEXT);",
        )
        .unwrap();
        drop(conn);
        init_db(&db).unwrap();
        let id = record_history(
            &db,
            &HistoryEntry {
                id: None,
                server: "10.0.0.1".into(),
                port: 7777,
                started_at: "2026-01-02T00:00:00Z".into(),
                duration_secs: 5,
                error: "timeout".into(),
                detail: "x".into(),
            },
        )
        .unwrap();
        assert!(id > 0);
        let _ = std::fs::remove_file(&db);
    }
}
