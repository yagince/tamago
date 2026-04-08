use std::fs;
use std::io;
use std::path::PathBuf;

use crate::pet::PetState;

const PET_FILE: &str = "pet.json";

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
    fn save_pet_is_atomic() {
        let (_dir, storage) = setup();
        let pet = PetState::new("テスト", Utc::now());
        storage.save_pet(&pet).unwrap();

        // tmp ファイルが残っていないこと
        let tmp = storage.pet_file().with_extension("json.tmp");
        assert!(!tmp.exists());
    }
}
