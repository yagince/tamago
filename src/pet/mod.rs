pub mod names;
pub mod render;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(dead_code)]
pub struct GrowResult {
    pub evolved: bool,
    pub leveled_up: bool,
    pub needs_personality: bool,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl Category {
    /// hunger/mood 回復量 (+hunger, +mood)
    fn recovery(self) -> (u8, u8) {
        match self {
            Category::Git => (1, 1),
            Category::Dev | Category::Infra => (2, 0),
            Category::Ai | Category::Editor => (0, 2),
            Category::Basic => (1, 0),
            Category::Other => (0, 1),
        }
    }
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
    #[serde(default)]
    pub dev_power: u16,
    #[serde(default)]
    pub wisdom: u16,
    #[serde(default)]
    pub humor: u16,
    #[serde(default)]
    pub chaos: u16,
    #[serde(default)]
    pub personality: String,
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
    /// 連続レベル。stage 毎に割り当てレベル数が増え、進化後もリセットしない。
    /// Egg: 1-20 / Baby: 21-60 / Child: 61-130 / Teen: 131-250 / Adult: 251-450
    pub fn level(&self) -> u64 {
        const EGG_LEVELS: u64 = 20;
        const BABY_LEVELS: u64 = 40;
        const CHILD_LEVELS: u64 = 70;
        const TEEN_LEVELS: u64 = 120;
        const ADULT_LEVELS: u64 = 200;

        let (stage_offset, stage_levels) = match self.stage {
            Stage::Egg => (0, EGG_LEVELS),
            Stage::Baby => (EGG_LEVELS, BABY_LEVELS),
            Stage::Child => (EGG_LEVELS + BABY_LEVELS, CHILD_LEVELS),
            Stage::Teen => (EGG_LEVELS + BABY_LEVELS + CHILD_LEVELS, TEEN_LEVELS),
            Stage::Adult => (
                EGG_LEVELS + BABY_LEVELS + CHILD_LEVELS + TEEN_LEVELS,
                ADULT_LEVELS,
            ),
        };

        let base = self.stage.exp_threshold();
        let range = self.stage.next_threshold().saturating_sub(base);
        let within = if range == 0 {
            stage_levels
        } else {
            let progress = self.exp.saturating_sub(base);
            (progress * stage_levels / range).min(stage_levels - 1) + 1
        };
        stage_offset + within
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

        // hunger/mood それぞれの消費時間の大きい方だけ進める。端数は次回へ。
        if mood_chunks > 0 || hunger_chunks > 0 {
            let consumed_min = (mood_chunks * MOOD_CHUNK_MIN).max(hunger_chunks * HUNGER_CHUNK_MIN);
            self.last_decay_at = Some(baseline + chrono::Duration::minutes(consumed_min));
        } else if self.last_decay_at.is_none() {
            // 初回かつ短時間の場合も baseline を確定させる
            self.last_decay_at = Some(now);
        }
    }

    pub fn apply_activities(&mut self, activities: &[crate::storage::ActivityRecord]) {
        let mut hunger_gain: u32 = 0;
        let mut mood_gain: u32 = 0;

        for record in activities {
            self.exp += record.exp;
            *self.category_exp.entry(record.cat).or_insert(0) += record.exp;
            self.last_active = record.ts;

            let (h, m) = record.cat.recovery();
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
            dev_power: 0,
            wisdom: 0,
            humor: 0,
            chaos: 0,
            personality: String::new(),
        }
    }

    const STAT_POINTS_PER_LEVEL: u16 = 5;

    /// アクティビティ集計 → 進化 → レベルアップの一連処理。
    /// show / status 両方から呼ばれる共通ロジック。
    pub async fn grow(
        &mut self,
        now: DateTime<Utc>,
        activities: &[crate::storage::ActivityRecord],
        engine: Option<&mut dyn crate::llm::TextGenerator>,
    ) -> GrowResult {
        self.apply_decay(now);

        let old_stage = self.stage.clone();
        let old_level = self.level();

        self.apply_activities(activities);

        while self.try_evolve() {}

        let evolved = self.stage != old_stage;
        if evolved {
            self.evolved_at = Some(now);
        }

        let new_level = self.level();
        let leveled_up = new_level > old_level;
        if leveled_up {
            self.leveled_up_at = Some(now);
            self.apply_level_up_stats(new_level - old_level);
        }

        let needs_personality = Self::should_regenerate_personality(old_level, new_level, evolved);
        if needs_personality {
            self.personality = self.generate_personality(engine).await;
        }

        GrowResult {
            evolved,
            leveled_up,
            needs_personality,
        }
    }

    pub fn apply_level_up_stats(&mut self, levels_gained: u64) {
        for _ in 0..levels_gained {
            let total: f64 = self.category_exp.values().sum::<u64>() as f64;
            if total == 0.0 {
                let p = Self::STAT_POINTS_PER_LEVEL;
                self.humor += p / 4;
                self.chaos += p / 4;
                self.dev_power += p / 4;
                self.wisdom += p - 3 * (p / 4);
                continue;
            }

            let ratio = |cat: Category| -> f64 {
                *self.category_exp.get(&cat).unwrap_or(&0) as f64 / total
            };

            let weights = [
                ratio(Category::Ai) + ratio(Category::Dev) + ratio(Category::Infra),
                ratio(Category::Ai) + ratio(Category::Dev) + ratio(Category::Editor),
                ratio(Category::Git) + ratio(Category::Editor) + ratio(Category::Basic),
                ratio(Category::Git) + ratio(Category::Infra) + ratio(Category::Other),
            ];
            let w_total: f64 = weights.iter().sum();
            let points = Self::STAT_POINTS_PER_LEVEL;

            let raw: Vec<f64> = weights
                .iter()
                .map(|w| w / w_total * points as f64)
                .collect();
            let floors: Vec<u16> = raw.iter().map(|r| *r as u16).collect();
            let remainders: Vec<f64> = raw
                .iter()
                .zip(&floors)
                .map(|(r, f)| r - *f as f64)
                .collect();
            let mut allocated = floors.clone();
            let remaining = points.saturating_sub(floors.iter().sum::<u16>());
            let mut indices: Vec<usize> = (0..4).collect();
            indices.sort_by(|a, b| remainders[*b].partial_cmp(&remainders[*a]).unwrap());
            for i in 0..remaining as usize {
                allocated[indices[i]] += 1;
            }

            self.dev_power += allocated[0];
            self.wisdom += allocated[1];
            self.humor += allocated[2];
            self.chaos += allocated[3];
        }
    }

    pub fn should_regenerate_personality(old_level: u64, new_level: u64, evolved: bool) -> bool {
        if evolved {
            return true;
        }
        // 10 レベルごと
        old_level / 10 != new_level / 10
    }

    pub async fn generate_personality(
        &self,
        engine: Option<&mut dyn crate::llm::TextGenerator>,
    ) -> String {
        if let Some(engine) = engine
            && let Some(msg) = self.try_llm_personality(engine).await
        {
            return msg;
        }
        self.fallback_personality()
    }

    async fn try_llm_personality(
        &self,
        engine: &mut dyn crate::llm::TextGenerator,
    ) -> Option<String> {
        let mut top_cats: Vec<_> = self.category_exp.iter().collect();
        top_cats.sort_by_key(|(_, v)| std::cmp::Reverse(**v));
        let top3: Vec<String> = top_cats
            .iter()
            .take(3)
            .map(|(k, v)| format!("{:?}:{}", k, v))
            .collect();

        engine
            .generate(
                &format!(
                    "名前:{} Lv.{} 開発力:{} 賢さ:{} おもしろさ:{} カオスさ:{} 得意:{}\n\
                このペットの性格を30文字以内で。",
                    self.name,
                    self.level(),
                    self.dev_power,
                    self.wisdom,
                    self.humor,
                    self.chaos,
                    top3.join(",")
                ),
                "あなたはターミナルペットの性格設定を生成するAIです。\
            求められた性格テキストだけを出力してください。説明や補足は不要です。",
                30,
            )
            .await
    }

    pub(crate) fn fallback_personality(&self) -> String {
        let stats = [
            (self.dev_power, "dev_power"),
            (self.wisdom, "wisdom"),
            (self.humor, "humor"),
            (self.chaos, "chaos"),
        ];
        let max_stat = stats.iter().max_by_key(|(v, _)| *v).unwrap().1;
        let seed = self.name.bytes().fold(0usize, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as usize)
        });

        let candidates: &[&str] = match max_stat {
            "dev_power" => &[
                "ストイックな職人気質",
                "黙々とコードを書くタイプ",
                "ものづくりが大好き",
                "効率を求める合理主義者",
            ],
            "wisdom" => &[
                "博識で落ち着いた性格",
                "知的好奇心の塊",
                "本の虫",
                "深く考えるタイプ",
            ],
            "humor" => &[
                "いつも楽しそう",
                "お調子者",
                "みんなを笑わせたい",
                "ムードメーカー",
            ],
            "chaos" => &[
                "予測不能な行動をする",
                "自由奔放",
                "破壊と創造の申し子",
                "常識にとらわれない",
            ],
            _ => &["バランス型の優等生", "なんでもそつなくこなす"],
        };

        candidates[seed % candidates.len()].to_string()
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

        // exp が閾値を超えてもクラッシュしない
        pet.exp = 999_999;
        assert!(pet.level() >= 1);
    }

    /// 進化してもレベルはリセットされず連続する
    #[test]
    fn level_does_not_reset_on_evolution() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);

        // Egg 末期 (Lv.20 相当)
        pet.exp = 4999;
        let before = pet.level();
        assert_eq!(before, 20);

        // Baby へ進化
        pet.exp = 5000;
        assert!(pet.try_evolve());
        assert_eq!(pet.stage, Stage::Baby);

        // レベルはリセットされず、連続して上がる (Egg 20 + Baby 1 = 21)
        let after = pet.level();
        assert!(
            after > before,
            "進化後のレベル({after})は進化前({before})以上でなければならない"
        );
        assert_eq!(after, 21);
    }

    #[test]
    fn stage_level_ranges() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);

        // Egg: 1-20
        pet.exp = 0;
        assert_eq!(pet.level(), 1);
        pet.exp = 4999;
        assert_eq!(pet.level(), 20);

        // Baby: 21-60
        pet.stage = Stage::Baby;
        pet.exp = 5000;
        assert_eq!(pet.level(), 21);
        pet.exp = 19999;
        assert_eq!(pet.level(), 60);

        // Child: 61-130
        pet.stage = Stage::Child;
        pet.exp = 20000;
        assert_eq!(pet.level(), 61);
        pet.exp = 49999;
        assert_eq!(pet.level(), 130);

        // Teen: 131-250
        pet.stage = Stage::Teen;
        pet.exp = 50000;
        assert_eq!(pet.level(), 131);
        pet.exp = 149999;
        assert_eq!(pet.level(), 250);

        // Adult: 251-450
        pet.stage = Stage::Adult;
        pet.exp = 150000;
        assert_eq!(pet.level(), 251);
        pet.exp = 299999;
        assert_eq!(pet.level(), 450);
    }

    #[test]
    fn level_monotonic_across_stages() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);

        let mut prev = pet.level();
        // exp を徐々に増やしながら試す
        for exp in (0..=300_000u64).step_by(500) {
            pet.exp = exp;
            while pet.try_evolve() {}
            let lv = pet.level();
            assert!(
                lv >= prev,
                "level が減少した: exp={exp} stage={:?} lv={lv} prev={prev}",
                pet.stage
            );
            prev = lv;
        }
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
    fn category_recovery_per_type() {
        // (category, expected_hunger, expected_mood)
        let cases = [
            (Category::Git, 51, 51),
            (Category::Dev, 52, 50),
            (Category::Infra, 52, 50),
            (Category::Ai, 50, 52),
            (Category::Editor, 50, 52),
            (Category::Basic, 51, 50),
            (Category::Other, 50, 51),
        ];

        let now = Utc::now();
        for (cat, want_hunger, want_mood) in cases {
            let mut pet = PetState::new("test", now);
            pet.hunger = 50;
            pet.mood = 50;

            let activities = vec![crate::storage::ActivityRecord {
                cmd: "test".into(),
                cat,
                exp: 1,
                ts: now,
            }];
            pet.apply_activities(&activities);

            assert_eq!(pet.hunger, want_hunger, "{:?}: hunger", cat);
            assert_eq!(pet.mood, want_mood, "{:?}: mood", cat);
        }
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

    #[test]
    fn stat_growth_git_heavy() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        *pet.category_exp.get_mut(&Category::Git).unwrap() = 1000;
        pet.apply_level_up_stats(1);
        assert!(pet.chaos > 0 || pet.humor > 0);
        assert_eq!(pet.dev_power + pet.wisdom + pet.humor + pet.chaos, 5);
    }

    #[test]
    fn stat_growth_balanced() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        for cat in [Category::Git, Category::Ai, Category::Dev, Category::Infra] {
            *pet.category_exp.get_mut(&cat).unwrap() = 100;
        }
        pet.apply_level_up_stats(1);
        assert_eq!(pet.dev_power + pet.wisdom + pet.humor + pet.chaos, 5);
    }

    #[test]
    fn stat_growth_zero_exp() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        pet.apply_level_up_stats(1);
        assert_eq!(pet.dev_power + pet.wisdom + pet.humor + pet.chaos, 5);
    }

    #[test]
    fn stat_growth_multiple_levels() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        *pet.category_exp.get_mut(&Category::Ai).unwrap() = 500;
        pet.apply_level_up_stats(10);
        assert_eq!(pet.dev_power + pet.wisdom + pet.humor + pet.chaos, 50);
    }

    #[test]
    fn personality_milestone_check() {
        assert!(PetState::should_regenerate_personality(9, 10, false));
        assert!(PetState::should_regenerate_personality(19, 20, false));
        assert!(!PetState::should_regenerate_personality(10, 11, false));
        assert!(PetState::should_regenerate_personality(5, 6, true));
    }

    #[tokio::test]
    async fn personality_fallback_not_empty() {
        let now = Utc::now();
        let pet = PetState::new("test", now);
        let p = pet.generate_personality(None).await;
        assert!(!p.is_empty());
    }

    #[tokio::test]
    async fn grow_sets_leveled_up_at_on_level_up() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        assert_eq!(pet.level(), 1);
        let activities: Vec<_> = (0..50)
            .map(|_| crate::storage::ActivityRecord {
                cmd: "git commit".into(),
                cat: Category::Git,
                exp: 20,
                ts: now,
            })
            .collect();
        let result = pet.grow(now, &activities, None).await;
        assert!(pet.level() > 1);
        assert!(result.leveled_up);
        assert_eq!(pet.leveled_up_at, Some(now));
    }

    #[tokio::test]
    async fn grow_sets_evolved_at_on_evolution() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        let activities: Vec<_> = (0..250)
            .map(|_| crate::storage::ActivityRecord {
                cmd: "git commit".into(),
                cat: Category::Git,
                exp: 20,
                ts: now,
            })
            .collect();
        let result = pet.grow(now, &activities, None).await;
        assert!(result.evolved);
        assert_eq!(pet.evolved_at, Some(now));
        assert_eq!(pet.stage, Stage::Baby);
    }

    #[tokio::test]
    async fn grow_no_change_with_empty_activities() {
        let now = Utc::now();
        let mut pet = PetState::new("test", now);
        let result = pet.grow(now, &[], None).await;
        assert!(!result.evolved);
        assert!(!result.leveled_up);
        assert_eq!(pet.evolved_at, None);
        assert_eq!(pet.leveled_up_at, None);
    }

    #[test]
    fn serde_backward_compat() {
        // 旧形式の pet.json（新フィールドなし）が読める
        let json = r#"{
            "name": "テスト",
            "born_at": "2026-04-10T00:00:00Z",
            "stage": "egg",
            "exp": 100,
            "hunger": 80,
            "mood": 90,
            "category_exp": {"git": 50, "ai": 30},
            "last_fed": "2026-04-10T00:00:00Z",
            "last_active": "2026-04-10T00:00:00Z"
        }"#;
        let pet: PetState = serde_json::from_str(json).unwrap();
        assert_eq!(pet.dev_power, 0);
        assert_eq!(pet.wisdom, 0);
        assert_eq!(pet.humor, 0);
        assert_eq!(pet.chaos, 0);
        assert_eq!(pet.personality, "");
    }
}
