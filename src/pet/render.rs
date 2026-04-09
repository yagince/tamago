mod adult;
mod animate;
mod baby;
mod child;
mod egg;
mod expression;
mod teen;

use super::{Archetype, Stage};

pub struct PetArt {
    pub art: &'static str,
    pub creature_type: &'static str,
}

fn name_hash(name: &str) -> usize {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in name.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h as usize
}

/// hunger/mood の最小値で状態を判定
#[derive(Debug, PartialEq)]
pub enum Condition {
    Normal,    // > 70
    Tired,     // 30-70
    Exhausted, // < 30
}

pub fn condition(hunger: u8, mood: u8) -> Condition {
    let min = hunger.min(mood);
    if min < 30 {
        Condition::Exhausted
    } else if min <= 70 {
        Condition::Tired
    } else {
        Condition::Normal
    }
}

fn select_art(stage: &Stage, archetype: &Option<Archetype>, name: &str) -> &'static PetArt {
    let h = name_hash(name);
    match stage {
        Stage::Egg => &egg::EGG[h % egg::EGG.len()],
        Stage::Baby => &baby::BABY[h % baby::BABY.len()],
        Stage::Child => &child::CHILD[h % child::CHILD.len()],
        Stage::Teen => &teen::TEEN[h % teen::TEEN.len()],
        Stage::Adult => {
            let arts: &[PetArt] = match archetype {
                Some(Archetype::Versionist) => &adult::ADULT_VERSIONIST,
                Some(Archetype::AiMage) => &adult::ADULT_AIMAGE,
                Some(Archetype::CloudDweller) => &adult::ADULT_CLOUD,
                Some(Archetype::AncientMage) => &adult::ADULT_ANCIENT,
                Some(Archetype::Generalist) | None => &adult::ADULT_GENERALIST,
            };
            &arts[h % arts.len()]
        }
    }
}

pub fn ascii_art(
    stage: &Stage,
    archetype: &Option<Archetype>,
    name: &str,
    hunger: u8,
    mood: u8,
) -> String {
    let pet_art = select_art(stage, archetype, name);
    let cond = condition(hunger, mood);
    expression::apply_expression(pet_art.art, &cond)
}

pub fn creature_type(stage: &Stage, archetype: &Option<Archetype>, name: &str) -> &'static str {
    select_art(stage, archetype, name).creature_type
}

/// show 用: 表情変更 + デコレーション + マイクロアニメーション
pub fn animated_art(
    stage: &Stage,
    archetype: &Option<Archetype>,
    name: &str,
    hunger: u8,
    mood: u8,
    exp: u64,
) -> String {
    let base = ascii_art(stage, archetype, name, hunger, mood);
    animate::animate(&base, hunger, mood, exp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn egg_has_patterns() {
        assert!(egg::EGG.len() >= 1);
    }

    #[test]
    fn same_name_same_art() {
        let a = ascii_art(&Stage::Baby, &None, "test1", 100, 100);
        let b = ascii_art(&Stage::Baby, &None, "test1", 100, 100);
        assert_eq!(a, b);
    }

    #[test]
    fn different_name_can_differ() {
        let results: Vec<_> = ["a", "b", "c", "d", "e", "f", "g", "h"]
            .iter()
            .map(|n| ascii_art(&Stage::Baby, &None, n, 100, 100))
            .collect();
        let unique: std::collections::HashSet<_> = results.iter().collect();
        assert!(unique.len() > 1, "全部同じ AA になっている");
    }

    #[test]
    fn each_stage_renders() {
        let name = "test";
        assert!(!ascii_art(&Stage::Egg, &None, name, 100, 100).is_empty());
        assert!(!ascii_art(&Stage::Baby, &None, name, 100, 100).is_empty());
        assert!(!ascii_art(&Stage::Child, &None, name, 100, 100).is_empty());
        assert!(!ascii_art(&Stage::Teen, &None, name, 100, 100).is_empty());
        assert!(!ascii_art(&Stage::Adult, &None, name, 100, 100).is_empty());
    }

    #[test]
    fn adult_archetypes_show_title() {
        let art = ascii_art(
            &Stage::Adult,
            &Some(Archetype::Versionist),
            "test",
            100,
            100,
        );
        assert!(art.contains("Versionist"));
        let art = ascii_art(&Stage::Adult, &Some(Archetype::AiMage), "test", 100, 100);
        assert!(art.contains("AI Mage"));
        let art = ascii_art(
            &Stage::Adult,
            &Some(Archetype::CloudDweller),
            "test",
            100,
            100,
        );
        assert!(art.contains("CloudDweller"));
        let art = ascii_art(
            &Stage::Adult,
            &Some(Archetype::AncientMage),
            "test",
            100,
            100,
        );
        assert!(art.contains("AncientMage"));
        let art = ascii_art(
            &Stage::Adult,
            &Some(Archetype::Generalist),
            "test",
            100,
            100,
        );
        assert!(art.contains("Generalist"));
    }

    #[test]
    fn low_hunger_shows_tired_aa() {
        let normal = ascii_art(&Stage::Baby, &None, "test", 100, 100);
        let tired = ascii_art(&Stage::Baby, &None, "test", 50, 100);
        assert_ne!(normal, tired);
    }

    #[test]
    fn very_low_shows_exhausted_aa() {
        let tired = ascii_art(&Stage::Baby, &None, "test", 50, 100);
        let exhausted = ascii_art(&Stage::Baby, &None, "test", 10, 100);
        assert_ne!(tired, exhausted);
    }

    #[test]
    fn condition_thresholds() {
        assert_eq!(condition(100, 100), Condition::Normal);
        assert_eq!(condition(71, 71), Condition::Normal);
        assert_eq!(condition(70, 100), Condition::Tired);
        assert_eq!(condition(100, 50), Condition::Tired);
        assert_eq!(condition(29, 100), Condition::Exhausted);
        assert_eq!(condition(100, 0), Condition::Exhausted);
    }

    #[test]
    fn name_hash_is_deterministic() {
        assert_eq!(name_hash("abc"), name_hash("abc"));
        assert_ne!(name_hash("abc"), name_hash("xyz"));
    }

    #[test]
    fn print_all_stages_for_visual_check() {
        let names = [
            "abc",
            "xyz",
            "tamago",
            "pikachu",
            "moglin",
            "test123",
            "hello",
            "world",
            "rust",
            "claude",
            "ピカボス",
            "ほげほげ",
            "これはどうだ？",
            "モグリン",
            "フワッチ",
        ];
        for name in names {
            println!("=== {name} ===");
            for stage in [Stage::Egg, Stage::Baby, Stage::Child, Stage::Teen] {
                println!("[{stage:?}]");
                print!("{}", ascii_art(&stage, &None, name, 100, 100));
            }
            println!("[Adult/Generalist]");
            print!(
                "{}",
                ascii_art(&Stage::Adult, &Some(Archetype::Generalist), name, 100, 100)
            );
            println!();
        }
    }
}
