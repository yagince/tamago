pub mod names;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Egg,
    Baby,
    Child,
    Teen,
    Adult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Archetype {
    Versionist,
    AiMage,
    CloudDweller,
    AncientMage,
    Generalist,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Git,
    Ai,
    Dev,
    Infra,
    Editor,
    Basic,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetState {
    pub name: String,
    pub born_at: DateTime<Utc>,
    pub stage: Stage,
    pub exp: u64,
    pub hunger: u8,
    pub mood: u8,
    #[serde(default)]
    pub archetype: Option<Archetype>,
    pub category_exp: HashMap<Category, u64>,
    pub last_fed: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    #[serde(default)]
    pub activity_cursor: u64,
}

impl PetState {
    pub fn new(name: impl Into<String>, now: DateTime<Utc>) -> Self {
        let mut category_exp = HashMap::new();
        for cat in [
            Category::Git,
            Category::Ai,
            Category::Dev,
            Category::Infra,
            Category::Editor,
            Category::Basic,
            Category::Other,
        ] {
            category_exp.insert(cat, 0);
        }

        Self {
            name: name.into(),
            born_at: now,
            stage: Stage::Egg,
            exp: 0,
            hunger: 100,
            mood: 100,
            archetype: None,
            category_exp,
            last_fed: now,
            last_active: now,
            activity_cursor: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pet_is_egg_with_full_stats() {
        let now = Utc::now();
        let pet = PetState::new("たまご", now);

        assert_eq!(pet.name, "たまご");
        assert_eq!(pet.stage, Stage::Egg);
        assert_eq!(pet.exp, 0);
        assert_eq!(pet.hunger, 100);
        assert_eq!(pet.mood, 100);
        assert_eq!(pet.archetype, None);
        assert_eq!(pet.born_at, now);
        assert_eq!(pet.category_exp.len(), 7);
        assert!(pet.category_exp.values().all(|&v| v == 0));
    }

    #[test]
    fn pet_state_roundtrip_json() {
        let now = Utc::now();
        let pet = PetState::new("クロード", now);
        let json = serde_json::to_string_pretty(&pet).unwrap();
        let restored: PetState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.name, "クロード");
        assert_eq!(restored.stage, Stage::Egg);
        assert_eq!(restored.exp, pet.exp);
    }
}
