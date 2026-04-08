use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use nix::fcntl::{Flock, FlockArg};
use serde::{Deserialize, Serialize};

use crate::pet::{Category, PetState};

const PET_FILE: &str = "pet.json";
const ACTIVITY_FILE: &str = "activity.jsonl";

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub cmd: String,
    pub cat: Category,
    pub exp: u64,
    pub ts: DateTime<Utc>,
}

pub struct Storage {
    base_dir: PathBuf,
}

impl Default for Storage {
    fn default() -> Self {
        let base_dir = dirs::config_dir()
            .expect("config directory not found")
            .join("tamago");
        Self { base_dir }
    }
}

impl Storage {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn ensure_dir(&self) -> io::Result<()> {
        fs::create_dir_all(&self.base_dir)
    }

    pub fn pet_file(&self) -> PathBuf {
        self.base_dir.join(PET_FILE)
    }

    pub fn pet_exists(&self) -> bool {
        self.pet_file().exists()
    }

    pub fn save_pet(&self, pet: &PetState) -> io::Result<()> {
        let path = self.pet_file();
        let tmp = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(pet)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&tmp, &json)?;
        fs::rename(&tmp, &path)?;
        Ok(())
    }

    pub fn load_pet(&self) -> io::Result<PetState> {
        let data = fs::read_to_string(self.pet_file())?;
        serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn activity_file(&self) -> PathBuf {
        self.base_dir.join(ACTIVITY_FILE)
    }

    pub fn append_activity(&self, record: &ActivityRecord) -> io::Result<()> {
        let mut line = serde_json::to_string(record)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        line.push('\n');

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.activity_file())?;
        let mut locked = Flock::lock(file, FlockArg::LockExclusive)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.1))?;
        locked.write_all(line.as_bytes())?;
        // drop で自動 unlock
        Ok(())
    }

    fn lock_file(&self) -> PathBuf {
        self.base_dir.join(".lock")
    }

    pub fn lock(&self) -> io::Result<Flock<std::fs::File>> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.lock_file())?;
        Flock::lock(file, FlockArg::LockExclusive)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Storage) {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path());
        (dir, storage)
    }

    #[test]
    fn ensure_dir_creates_directory() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("a").join("b");
        let storage = Storage::new(&nested);
        storage.ensure_dir().unwrap();
        assert!(nested.exists());
    }

    #[test]
    fn pet_exists_returns_false_when_no_file() {
        let (_dir, storage) = setup();
        assert!(!storage.pet_exists());
    }

    #[test]
    fn save_and_load_pet() {
        let (_dir, storage) = setup();
        let pet = PetState::new("テスト", Utc::now());
        storage.save_pet(&pet).unwrap();

        assert!(storage.pet_exists());

        let loaded = storage.load_pet().unwrap();
        assert_eq!(loaded.name, "テスト");
    }

    #[test]
    fn append_activity_creates_jsonl() {
        let (_dir, storage) = setup();
        storage.ensure_dir().unwrap();

        let record = ActivityRecord {
            cmd: "git commit -m fix".into(),
            cat: Category::Git,
            exp: 20,
            ts: Utc::now(),
        };
        storage.append_activity(&record).unwrap();

        let content = fs::read_to_string(storage.activity_file()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);

        let parsed: ActivityRecord = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed.cmd, "git commit -m fix");
        assert_eq!(parsed.cat, Category::Git);
        assert_eq!(parsed.exp, 20);
    }

    #[test]
    fn append_activity_appends_multiple_lines() {
        let (_dir, storage) = setup();
        storage.ensure_dir().unwrap();

        for i in 0..3 {
            let record = ActivityRecord {
                cmd: format!("cmd{i}"),
                cat: Category::Basic,
                exp: 1,
                ts: Utc::now(),
            };
            storage.append_activity(&record).unwrap();
        }

        let content = fs::read_to_string(storage.activity_file()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn save_pet_is_atomic() {
        let (_dir, storage) = setup();
        let pet = PetState::new("テスト", Utc::now());
        storage.save_pet(&pet).unwrap();

        // tmp ファイルが残っていないこと
        let tmp = storage.pet_file().with_extension("json.tmp");
        assert!(!tmp.exists());
    }
}
