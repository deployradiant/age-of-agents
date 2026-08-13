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
        let schema_version: u32 =
            connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        let legacy_schema = {
            let mut statement = connection.prepare("PRAGMA table_info(world_state)")?;
            let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
            columns
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .any(|column| column == "saved_at")
        };
        let transaction = connection.transaction()?;
        if legacy_schema || schema_version < 4 {
            // The pre-milestone prototype stored a fundamentally different world
            // model; versions through 3 predate typed cargo, building jobs,
            // technologies, and the seven-resource stockpile. Those snapshots cannot
            // be translated safely into the current deterministic world.
            transaction.execute_batch("DROP TABLE IF EXISTS world_state;")?;
        }
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS world_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                world_json TEXT NOT NULL
            );
            PRAGMA user_version = 4;",
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
    fn persistence_round_trips_carried_cargo_and_gather_phase() {
        let path = temporary_db("roundtrip");
        let store = Store::from_path(&path);
        store.initialize().unwrap();
        let mut world = GameWorld::default();
        world.units[0].position = world.resources[0].position;
        world
            .apply_command(crate::game::Command::Gather {
                unit_id: "villager-1".into(),
                resource_id: "tree-1".into(),
            })
            .unwrap();
        world.tick(0.5);
        store.save(&world).unwrap();

        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded, world);
        assert_eq!(
            loaded.units[0].cargo.as_ref().unwrap().amount,
            crate::game::GATHER_RATE * 0.5
        );
        assert!(matches!(
            loaded.units[0].action,
            crate::game::UnitAction::Gather {
                phase: crate::game::GatherPhase::Gathering,
                ..
            }
        ));
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
    fn current_save_without_slice_a_fields_loads_with_intentional_defaults() {
        let path = temporary_db("slice-a-defaults");
        let store = Store::from_path(&path);
        store.initialize().unwrap();
        let original = GameWorld::default();
        let mut legacy = serde_json::to_value(&original).unwrap();
        let object = legacy.as_object_mut().unwrap();
        object.remove("scenario");
        for unit in object["units"].as_array_mut().unwrap() {
            unit.as_object_mut().unwrap().remove("kind");
        }
        for field in ["coal", "timber", "steel", "bricks", "cloth", "rations"] {
            object["stockpile"].as_object_mut().unwrap().remove(field);
        }
        let connection = Connection::open(&path).unwrap();
        connection
            .execute(
                "INSERT INTO world_state (id, world_json) VALUES (1, ?1)",
                params![serde_json::to_string(&legacy).unwrap()],
            )
            .unwrap();
        drop(connection);

        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded, original);
        assert_eq!(loaded.units[0].kind, crate::game::UnitKind::Villager);
        assert_eq!(loaded.scenario, crate::game::ScenarioState::default());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn unknown_persisted_enum_value_is_an_explicit_load_error() {
        let path = temporary_db("unknown-enum");
        let store = Store::from_path(&path);
        store.initialize().unwrap();
        let mut value = serde_json::to_value(GameWorld::default()).unwrap();
        value["resources"][0]["kind"] = serde_json::Value::String("unobtainium".into());
        let connection = Connection::open(&path).unwrap();
        connection
            .execute(
                "INSERT INTO world_state (id, world_json) VALUES (1, ?1)",
                params![serde_json::to_string(&value).unwrap()],
            )
            .unwrap();
        drop(connection);

        let error = store.load().unwrap_err().to_string();
        assert!(error.contains("unknown variant `unobtainium`"), "{error}");
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

    #[test]
    fn version_two_world_is_migrated_before_deserialization() {
        let path = temporary_db("version-two");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE world_state (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    world_json TEXT NOT NULL
                );
                INSERT INTO world_state VALUES (1, '{\"width\":1200,\"height\":800}');
                PRAGMA user_version = 2;",
            )
            .unwrap();
        drop(connection);

        let store = Store::from_path(&path);
        store.initialize().unwrap();
        assert_eq!(store.load().unwrap(), None);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn version_three_world_is_migrated_before_deserialization() {
        let path = temporary_db("version-three");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE world_state (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    world_json TEXT NOT NULL
                );
                INSERT INTO world_state VALUES (1, '{\"width\":2400,\"height\":1600}');
                PRAGMA user_version = 3;",
            )
            .unwrap();
        drop(connection);

        let store = Store::from_path(&path);
        store.initialize().unwrap();
        assert_eq!(store.load().unwrap(), None);
        std::fs::remove_file(path).unwrap();
    }
}
