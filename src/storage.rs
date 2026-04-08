use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
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
        let base_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .expect("home directory not found")
                    .join(".config")
            })
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

    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
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

    /// activity.jsonl を全て読み込み、ファイルを空にする。
    /// activity.jsonl を flock してから読み→truncate するため、tick との競合を防ぐ。
    pub fn read_and_clear_activities(&self) -> io::Result<Vec<ActivityRecord>> {
        let path = self.activity_file();
        if !path.exists() {
            return Ok(vec![]);
        }

        let file = OpenOptions::new().read(true).write(true).open(&path)?;
        let locked = Flock::lock(file, FlockArg::LockExclusive)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.1))?;

        let mut records = Vec::new();
        let reader = io::BufReader::new(&*locked);
        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            if let Ok(record) = serde_json::from_str::<ActivityRecord>(&line) {
                records.push(record);
            }
        }

        // truncate で中身を空にする
        locked.set_len(0)?;

        Ok(records)
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
    fn read_and_clear_returns_all_records() {
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

        let records = storage.read_and_clear_activities().unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].cmd, "cmd0");
        assert_eq!(records[2].cmd, "cmd2");
    }

    #[test]
    fn read_and_clear_empties_file() {
        let (_dir, storage) = setup();
        storage.ensure_dir().unwrap();

        let record = ActivityRecord {
            cmd: "test".into(),
            cat: Category::Basic,
            exp: 1,
            ts: Utc::now(),
        };
        storage.append_activity(&record).unwrap();
        storage.read_and_clear_activities().unwrap();

        // ファイルは空になっている
        let content = fs::read_to_string(storage.activity_file()).unwrap();
        assert!(content.is_empty());

        // 2回目は空
        let records = storage.read_and_clear_activities().unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn read_and_clear_no_file() {
        let (_dir, storage) = setup();
        let records = storage.read_and_clear_activities().unwrap();
        assert!(records.is_empty());
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
