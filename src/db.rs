/// SQLite persistence for Age of Agents — auto-save/load game state.

use rusqlite::{Connection, params};

use crate::state::GameWorld;

const SAVE_INTERVAL_TICKS: u64 = 100; // save every ~50 seconds at 2 Hz
const DB_PATH: &str = "age_of_agents.db";

/// Open or create the database.
pub fn open_db(path: Option<&str>) -> Result<Connection, rusqlite::Error> {
    let db_path = path.unwrap_or(DB_PATH);
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS world_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            world_json TEXT NOT NULL,
            saved_at TEXT NOT NULL
        );"
    )?;
    Ok(conn)
}

/// Load the world from the database. Returns None if no save exists.
pub fn load_world(conn: &Connection) -> Result<Option<GameWorld>, String> {
    let result: Result<String, _> = conn.query_row(
        "SELECT world_json FROM world_state WHERE id = 1",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(json) => {
            let world: GameWorld = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to deserialize world: {e}"))?;
            Ok(Some(world))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("DB error: {e}")),
    }
}

/// Save the current world to the database.
pub fn save_world(conn: &Connection, world: &GameWorld) -> Result<(), String> {
    let json = serde_json::to_string(world)
        .map_err(|e| format!("Failed to serialize world: {e}"))?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO world_state (id, world_json, saved_at) VALUES (1, ?1, ?2)",
        params![json, now],
    ).map_err(|e| format!("Failed to save world: {e}"))?;
    Ok(())
}

/// Check if it's time to save based on tick count.
pub fn should_save(tick: u64) -> bool {
    tick > 0 && tick % SAVE_INTERVAL_TICKS == 0
}

/// Background save task — runs in a spawned thread.
pub fn background_save(db_path: String, snapshot: String) {
    std::thread::spawn(move || {
        if let Ok(conn) = Connection::open(&db_path) {
            let now = chrono::Utc::now().to_rfc3339();
            if let Err(e) = conn.execute(
                "INSERT OR REPLACE INTO world_state (id, world_json, saved_at) VALUES (1, ?1, ?2)",
                params![snapshot, now],
            ) {
                eprintln!("[DB] Save failed: {e}");
            } else {
                eprintln!("[DB] World saved at tick");
            }
        }
    });
}