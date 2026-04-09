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

    /// 進化判定。進化したら true を返す。
    pub fn try_evolve(&mut self) -> bool {
        let next = match self.stage {
            Stage::Egg => Stage::Baby,
            Stage::Baby => Stage::Child,
            Stage::Child => Stage::Teen,
            Stage::Teen => Stage::Adult,
            Stage::Adult => return false,
        };

        if self.exp < next.exp_threshold() {
            return false;
        }

        // Teen → Adult は archetype を決定
        if self.stage == Stage::Teen {
            self.archetype = Some(self.determine_archetype());
        }

        self.stage = next;
        true
    }

    fn determine_archetype(&self) -> Archetype {
        let total: u64 = self.category_exp.values().sum();
        if total == 0 {
            return Archetype::Generalist;
        }

        let ratio = |cat: &Category| -> f64 {
            *self.category_exp.get(cat).unwrap_or(&0) as f64 / total as f64
        };

        if ratio(&Category::Git) >= 0.35 {
            Archetype::Versionist
        } else if ratio(&Category::Ai) >= 0.35 {
            Archetype::AiMage
        } else if ratio(&Category::Infra) >= 0.35 {
            Archetype::CloudDweller
        } else if ratio(&Category::Editor) >= 0.35 {
            Archetype::AncientMage
        } else {
            Archetype::Generalist
        }
    }

    /// 時間経過による hunger/mood 減衰
    pub fn apply_decay(&mut self, now: DateTime<Utc>) {
        let hours_since_active = (now - self.last_active).num_hours().max(0) as u8;

        // hunger: 1時間ごとに -3
        let hunger_loss = (hours_since_active / 1).saturating_mul(3);
        self.hunger = self.hunger.saturating_sub(hunger_loss);

        // mood: 2時間ごとに -2
        let mood_loss = (hours_since_active / 2).saturating_mul(2);
        self.mood = self.mood.saturating_sub(mood_loss);
    }

    /// 未集計の activity を反映する（exp + hunger/mood 回復）
    pub fn apply_activities(&mut self, activities: &[crate::storage::ActivityRecord]) {
        let count = activities.len() as u8;
        for record in activities {
            self.exp += record.exp;
            *self.category_exp.entry(record.cat.clone()).or_insert(0) += record.exp;
            self.last_active = record.ts;
        }

        // コマンド実行で回復: 1コマンドにつき hunger+1, mood+1
        self.hunger = self.hunger.saturating_add(count).min(100);
        self.mood = self.mood.saturating_add(count).min(100);
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
    fn evolve_egg_to_baby() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.exp = 5_000;

        let evolved = pet.try_evolve();
        assert!(evolved);
        assert_eq!(pet.stage, Stage::Baby);
    }

    #[test]
    fn no_evolve_below_threshold() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.exp = 4_999;

        let evolved = pet.try_evolve();
        assert!(!evolved);
        assert_eq!(pet.stage, Stage::Egg);
    }

    #[test]
    fn evolve_through_multiple_stages() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.exp = 50_000;

        // 1段ずつ進化
        assert!(pet.try_evolve());
        assert_eq!(pet.stage, Stage::Baby);
        assert!(pet.try_evolve());
        assert_eq!(pet.stage, Stage::Child);
        assert!(pet.try_evolve());
        assert_eq!(pet.stage, Stage::Teen);
        // Teen→Adult は 150_000 必要
        assert!(!pet.try_evolve());
        assert_eq!(pet.stage, Stage::Teen);
    }

    #[test]
    fn evolve_teen_to_adult_git_archetype() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.stage = Stage::Teen;
        pet.exp = 150_000;
        *pet.category_exp.get_mut(&Category::Git).unwrap() = 100_000;

        assert!(pet.try_evolve());
        assert_eq!(pet.stage, Stage::Adult);
        assert_eq!(pet.archetype, Some(Archetype::Versionist));
    }

    #[test]
    fn evolve_teen_to_adult_generalist() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.stage = Stage::Teen;
        pet.exp = 150_000;
        // 均等に分布 → 特化なし
        *pet.category_exp.get_mut(&Category::Git).unwrap() = 30_000;
        *pet.category_exp.get_mut(&Category::Dev).unwrap() = 30_000;
        *pet.category_exp.get_mut(&Category::Ai).unwrap() = 30_000;
        *pet.category_exp.get_mut(&Category::Infra).unwrap() = 30_000;
        *pet.category_exp.get_mut(&Category::Editor).unwrap() = 30_000;

        assert!(pet.try_evolve());
        assert_eq!(pet.stage, Stage::Adult);
        assert_eq!(pet.archetype, Some(Archetype::Generalist));
    }

    #[test]
    fn adult_does_not_evolve_further() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.stage = Stage::Adult;
        pet.exp = 999_999;

        assert!(!pet.try_evolve());
    }

    #[test]
    fn decay_hunger_after_1_hour() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        assert_eq!(pet.hunger, 100);

        let later = now + chrono::Duration::hours(1);
        pet.apply_decay(later);
        assert_eq!(pet.hunger, 97); // -3 per hour
    }

    #[test]
    fn decay_mood_after_2_hours() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        assert_eq!(pet.mood, 100);

        let later = now + chrono::Duration::hours(2);
        pet.apply_decay(later);
        assert_eq!(pet.mood, 98); // -2 per 2 hours
    }

    #[test]
    fn decay_does_not_go_below_zero() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);

        let later = now + chrono::Duration::hours(100);
        pet.apply_decay(later);
        assert_eq!(pet.hunger, 0);
        assert_eq!(pet.mood, 0);
    }

    #[test]
    fn activities_recover_hunger_and_mood() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.hunger = 50;
        pet.mood = 50;

        let activities: Vec<crate::storage::ActivityRecord> = (0..10)
            .map(|i| crate::storage::ActivityRecord {
                cmd: format!("cmd{i}"),
                cat: Category::Dev,
                exp: 8,
                ts: now,
            })
            .collect();

        pet.apply_activities(&activities);
        assert!(pet.hunger > 50);
        assert!(pet.mood > 50);
    }

    #[test]
    fn hunger_and_mood_cap_at_100() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.hunger = 99;
        pet.mood = 99;

        let activities = vec![
            crate::storage::ActivityRecord {
                cmd: "git commit".into(),
                cat: Category::Git,
                exp: 20,
                ts: now,
            };
            50
        ];

        pet.apply_activities(&activities);
        assert!(pet.hunger <= 100);
        assert!(pet.mood <= 100);
    }

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
