pub mod names;
pub mod render;

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
}

impl Stage {
    pub fn emoji(&self, archetype: &Option<Archetype>) -> &'static str {
        match self {
            Stage::Egg => "🥚",
            Stage::Baby => "🐣",
            Stage::Child => "🐥",
            Stage::Teen => "🐤",
            Stage::Adult => match archetype {
                Some(Archetype::Versionist) => "🐙",
                Some(Archetype::AiMage) => "🧙",
                Some(Archetype::CloudDweller) => "☁️",
                Some(Archetype::AncientMage) => "📜",
                Some(Archetype::Generalist) | None => "🦊",
            },
        }
    }

    fn exp_threshold(&self) -> u64 {
        match self {
            Stage::Egg => 0,
            Stage::Baby => 5_000,
            Stage::Child => 20_000,
            Stage::Teen => 50_000,
            Stage::Adult => 150_000,
        }
    }

    fn next_threshold(&self) -> u64 {
        match self {
            Stage::Egg => 5_000,
            Stage::Baby => 20_000,
            Stage::Child => 50_000,
            Stage::Teen => 150_000,
            Stage::Adult => 300_000,
        }
    }
}

impl PetState {
    pub fn level(&self) -> u64 {
        let base = self.stage.exp_threshold();
        let range = self.stage.next_threshold() - base;
        if range == 0 {
            return 1;
        }
        let progress = self.exp.saturating_sub(base);
        (progress * 99 / range).min(99) + 1
    }

    pub fn emoji(&self) -> &'static str {
        self.stage.emoji(&self.archetype)
    }

    /// 未集計の activity を反映する
    pub fn apply_activities(&mut self, activities: &[crate::storage::ActivityRecord]) {
        for record in activities {
            self.exp += record.exp;
            *self.category_exp.entry(record.cat.clone()).or_insert(0) += record.exp;
            self.last_active = record.ts;
        }
    }

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
    fn apply_activities_accumulates_exp_and_category() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);

        let activities = vec![
            crate::storage::ActivityRecord {
                cmd: "git commit".into(),
                cat: Category::Git,
                exp: 20,
                ts: now,
            },
            crate::storage::ActivityRecord {
                cmd: "cargo build".into(),
                cat: Category::Dev,
                exp: 8,
                ts: now,
            },
            crate::storage::ActivityRecord {
                cmd: "git push".into(),
                cat: Category::Git,
                exp: 18,
                ts: now,
            },
        ];

        pet.apply_activities(&activities);

        assert_eq!(pet.exp, 46);
        assert_eq!(pet.category_exp[&Category::Git], 38);
        assert_eq!(pet.category_exp[&Category::Dev], 8);
        assert_eq!(pet.last_active, now);
    }

    #[test]
    fn apply_empty_activities_changes_nothing() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.apply_activities(&[]);

        assert_eq!(pet.exp, 0);
    }

    #[test]
    fn level_stays_within_bounds() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);

        assert_eq!(pet.level(), 1);

        pet.exp = 2500;
        assert!(pet.level() > 1);
        assert!(pet.level() <= 100);

        // exp が閾値を超えてもクラッシュしない
        pet.exp = 999_999;
        assert!(pet.level() >= 1);
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
