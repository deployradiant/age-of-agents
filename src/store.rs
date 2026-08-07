use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};

use crate::game::GameWorld;

const DEFAULT_DB_PATH: &str = "age_of_agents.db";

type StoreResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn configured() -> Self {
        Self::from_path(
            std::env::var_os("AGE_OF_AGENTS_DB")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(DEFAULT_DB_PATH)),
        )
    }

    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn initialize(&self) -> StoreResult<()> {
        let mut connection = Connection::open(&self.path)?;
        let legacy_schema = {
            let mut statement = connection.prepare("PRAGMA table_info(world_state)")?;
            let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
            columns
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .any(|column| column == "saved_at")
        };
        let transaction = connection.transaction()?;
        if legacy_schema {
            // The pre-milestone prototype stored a fundamentally different world
            // model. Its snapshot cannot be translated into this smaller game.
            transaction.execute_batch("DROP TABLE world_state;")?;
        }
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS world_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                world_json TEXT NOT NULL
            );
            PRAGMA user_version = 1;",
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn load(&self) -> StoreResult<Option<GameWorld>> {
        let connection = Connection::open(&self.path)?;
        let json: Option<String> = connection
            .query_row(
                "SELECT world_json FROM world_state WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        json.map(|value| serde_json::from_str(&value).map_err(Into::into))
            .transpose()
    }

    pub fn save(&self, world: &GameWorld) -> StoreResult<()> {
        let json = serde_json::to_string(world)?;
        let connection = Connection::open(&self.path)?;
        connection.execute(
            "INSERT INTO world_state (id, world_json) VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET world_json = excluded.world_json",
            params![json],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn temporary_db(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "age-of-agents-{name}-{}-{nonce}.db",
            std::process::id()
        ))
    }

    #[test]
    fn persistence_round_trip() {
        let path = temporary_db("roundtrip");
        let store = Store::from_path(&path);
        store.initialize().unwrap();
        let mut world = GameWorld::default();
        world.stockpile.wood = 13.0;
        store.save(&world).unwrap();
        assert_eq!(store.load().unwrap(), Some(world));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn configured_path_is_used_for_every_save() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = temporary_db("configured");
        let old_value = std::env::var_os("AGE_OF_AGENTS_DB");
        // SAFETY: this module serializes its only environment-mutating test, and
        // the value is restored before releasing the lock.
        unsafe { std::env::set_var("AGE_OF_AGENTS_DB", &path) };

        let store = Store::configured();
        store.initialize().unwrap();
        store.save(&GameWorld::default()).unwrap();
        assert_eq!(store.path(), path);
        assert!(path.exists());

        match old_value {
            Some(value) => {
                // SAFETY: guarded and restored as described above.
                unsafe { std::env::set_var("AGE_OF_AGENTS_DB", value) };
            }
            None => {
                // SAFETY: guarded and restored as described above.
                unsafe { std::env::remove_var("AGE_OF_AGENTS_DB") };
            }
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn corrupt_snapshot_is_an_error() {
        let path = temporary_db("corrupt");
        let store = Store::from_path(&path);
        store.initialize().unwrap();
        let connection = Connection::open(&path).unwrap();
        connection
            .execute(
                "INSERT INTO world_state (id, world_json) VALUES (1, 'not json')",
                [],
            )
            .unwrap();
        drop(connection);

        assert!(store.load().is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn legacy_schema_is_migrated_to_an_empty_current_store() {
        let path = temporary_db("legacy");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE world_state (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    world_json TEXT NOT NULL,
                    saved_at TEXT NOT NULL
                );
                INSERT INTO world_state VALUES (1, '{\"legacy\":true}', '2026-08-01');",
            )
            .unwrap();
        drop(connection);

        let store = Store::from_path(&path);
        store.initialize().unwrap();
        assert_eq!(store.load().unwrap(), None);
        store.save(&GameWorld::default()).unwrap();
        assert!(store.load().unwrap().is_some());
        std::fs::remove_file(path).unwrap();
    }
}
