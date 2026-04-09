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
    /// 進化した時刻。この時刻から一定時間は演出を表示する。
    #[serde(default)]
    pub evolved_at: Option<DateTime<Utc>>,
    /// レベルアップした時刻。同上。
    #[serde(default)]
    pub leveled_up_at: Option<DateTime<Utc>>,
    /// 前回の tick で受け取った累積 output_tokens。差分計算用。
    #[serde(default)]
    pub last_output_tokens: u64,
    /// 前回 decay を計算した時刻。activity の last_active とは独立。
    #[serde(default)]
    pub last_decay_at: Option<DateTime<Utc>>,
}

/// Category 毎の hunger/mood 回復量 (+hunger, +mood)
fn recovery_for(cat: &Category) -> (u8, u8) {
    match cat {
        Category::Git => (1, 1),
        Category::Dev => (2, 0),
        Category::Infra => (2, 0),
        Category::Ai => (0, 2),
        Category::Editor => (0, 2),
        Category::Basic => (1, 0),
        Category::Other => (0, 1),
    }
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

    /// 時間経過による hunger/mood 減衰。
    /// `last_active` とは独立した `last_decay_at` を基準にするため、
    /// activity が連続しても wall clock で減衰が進む。
    /// 粒度は 20 分 (hunger -1) / 60 分 (mood -1)。未消費の端数は次回に持ち越す。
    pub fn apply_decay(&mut self, now: DateTime<Utc>) {
        const HUNGER_CHUNK_MIN: i64 = 20; // -1 per 20 min = -3/h
        const MOOD_CHUNK_MIN: i64 = 60; // -1 per 60 min = -2 / 2h

        let baseline = self.last_decay_at.unwrap_or(now);
        let elapsed_min = (now - baseline).num_minutes().max(0);

        let hunger_chunks = elapsed_min / HUNGER_CHUNK_MIN;
        let mood_chunks = elapsed_min / MOOD_CHUNK_MIN;

        if hunger_chunks > 0 {
            let sub = hunger_chunks.min(u8::MAX as i64) as u8;
            self.hunger = self.hunger.saturating_sub(sub);
        }
        if mood_chunks > 0 {
            let sub = mood_chunks.min(u8::MAX as i64) as u8;
            self.mood = self.mood.saturating_sub(sub);
        }

        // 最大粒度 (60min) を消費した分だけ進める。端数は次回へ。
        // 両方 0 なら進めない（次回まとめて処理）。
        if mood_chunks > 0 || hunger_chunks > 0 {
            // 実際に消費した分だけ timestamp を進める
            let consumed_min = (mood_chunks * MOOD_CHUNK_MIN)
                .max(hunger_chunks * HUNGER_CHUNK_MIN);
            self.last_decay_at = Some(baseline + chrono::Duration::minutes(consumed_min));
        } else if self.last_decay_at.is_none() {
            // 初回かつ短時間の場合も baseline を確定させる
            self.last_decay_at = Some(now);
        }
    }

    /// 未集計の activity を反映する（exp 加算 + カテゴリ別に hunger/mood 回復）。
    /// コマンドのカテゴリで回復対象が変わる:
    ///   Git: hunger +1, mood +1（達成感）
    ///   Dev/Infra: hunger +2（実用）
    ///   Ai/Editor: mood +2（創造・対話）
    ///   Basic: hunger +1
    ///   Other: mood +1
    pub fn apply_activities(&mut self, activities: &[crate::storage::ActivityRecord]) {
        let mut hunger_gain: u32 = 0;
        let mut mood_gain: u32 = 0;

        for record in activities {
            self.exp += record.exp;
            *self.category_exp.entry(record.cat.clone()).or_insert(0) += record.exp;
            self.last_active = record.ts;

            let (h, m) = recovery_for(&record.cat);
            hunger_gain += h as u32;
            mood_gain += m as u32;
        }

        self.hunger = (self.hunger as u32 + hunger_gain).min(100) as u8;
        self.mood = (self.mood as u32 + mood_gain).min(100) as u8;
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
            evolved_at: None,
            leveled_up_at: None,
            last_output_tokens: 0,
            last_decay_at: Some(now),
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

    /// 以前は last_active を基準に decay していたため、activity が続くと
    /// last_active が常に最新になり decay が一切進まない問題があった。
    /// 独立した last_decay_at を持ち、活動の有無に関わらず時間経過で減衰する。
    #[test]
    fn decay_progresses_even_when_activity_is_recent() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);

        // 3 時間後: 直前まで activity があった想定で last_active を最新に
        let later = now + chrono::Duration::hours(3);
        pet.last_active = later;

        pet.apply_decay(later);

        assert!(
            pet.hunger < 100,
            "activity が最新でも hunger は減衰するはず: {}",
            pet.hunger
        );
        assert!(
            pet.mood < 100,
            "activity が最新でも mood は減衰するはず: {}",
            pet.mood
        );
    }

    #[test]
    fn git_recovers_both_stats() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.hunger = 50;
        pet.mood = 50;

        let activities = vec![crate::storage::ActivityRecord {
            cmd: "git commit".into(),
            cat: Category::Git,
            exp: 20,
            ts: now,
        }];
        pet.apply_activities(&activities);

        assert_eq!(pet.hunger, 51);
        assert_eq!(pet.mood, 51);
    }

    #[test]
    fn dev_recovers_hunger_only() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.hunger = 50;
        pet.mood = 50;

        let activities = vec![crate::storage::ActivityRecord {
            cmd: "cargo build".into(),
            cat: Category::Dev,
            exp: 8,
            ts: now,
        }];
        pet.apply_activities(&activities);

        assert_eq!(pet.hunger, 52);
        assert_eq!(pet.mood, 50);
    }

    #[test]
    fn ai_recovers_mood_only() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.hunger = 50;
        pet.mood = 50;

        let activities = vec![crate::storage::ActivityRecord {
            cmd: "--claude-turn".into(),
            cat: Category::Ai,
            exp: 5,
            ts: now,
        }];
        pet.apply_activities(&activities);

        assert_eq!(pet.hunger, 50);
        assert_eq!(pet.mood, 52);
    }

    #[test]
    fn editor_recovers_mood_only() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.hunger = 50;
        pet.mood = 50;

        let activities = vec![crate::storage::ActivityRecord {
            cmd: "vim src".into(),
            cat: Category::Editor,
            exp: 5,
            ts: now,
        }];
        pet.apply_activities(&activities);

        assert_eq!(pet.hunger, 50);
        assert_eq!(pet.mood, 52);
    }

    #[test]
    fn infra_recovers_hunger_only() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.hunger = 50;
        pet.mood = 50;

        let activities = vec![crate::storage::ActivityRecord {
            cmd: "kubectl apply".into(),
            cat: Category::Infra,
            exp: 8,
            ts: now,
        }];
        pet.apply_activities(&activities);

        assert_eq!(pet.hunger, 52);
        assert_eq!(pet.mood, 50);
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
