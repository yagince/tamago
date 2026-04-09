mod adult;
mod baby;
mod child;
mod egg;
mod teen;

use super::{Archetype, Stage};

fn name_hash(name: &str) -> usize {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in name.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h as usize
}

pub fn ascii_art(stage: &Stage, archetype: &Option<Archetype>, name: &str) -> String {
    let h = name_hash(name);
    match stage {
        Stage::Egg => egg::EGG[h % egg::EGG.len()].to_string(),
        Stage::Baby => baby::BABY[h % baby::BABY.len()].to_string(),
        Stage::Child => child::CHILD[h % child::CHILD.len()].to_string(),
        Stage::Teen => teen::TEEN[h % teen::TEEN.len()].to_string(),
        Stage::Adult => {
            let arts = match archetype {
                Some(Archetype::Versionist) => &adult::ADULT_VERSIONIST[..],
                Some(Archetype::AiMage) => &adult::ADULT_AIMAGE[..],
                Some(Archetype::CloudDweller) => &adult::ADULT_CLOUD[..],
                Some(Archetype::AncientMage) => &adult::ADULT_ANCIENT[..],
                Some(Archetype::Generalist) | None => &adult::ADULT_GENERALIST[..],
            };
            arts[h % arts.len()].to_string()
        }
    }
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
        let a = ascii_art(&Stage::Baby, &None, "test1");
        let b = ascii_art(&Stage::Baby, &None, "test1");
        assert_eq!(a, b);
    }

    #[test]
    fn different_name_can_differ() {
        let results: Vec<_> = ["a", "b", "c", "d", "e", "f", "g", "h"]
            .iter()
            .map(|n| ascii_art(&Stage::Baby, &None, n))
            .collect();
        let unique: std::collections::HashSet<_> = results.iter().collect();
        assert!(unique.len() > 1, "全部同じ AA になっている");
    }

    #[test]
    fn each_stage_renders() {
        let name = "test";
        assert!(!ascii_art(&Stage::Egg, &None, name).is_empty());
        assert!(!ascii_art(&Stage::Baby, &None, name).is_empty());
        assert!(!ascii_art(&Stage::Child, &None, name).is_empty());
        assert!(!ascii_art(&Stage::Teen, &None, name).is_empty());
        assert!(!ascii_art(&Stage::Adult, &None, name).is_empty());
    }

    #[test]
    fn adult_archetypes_show_title() {
        let art = ascii_art(&Stage::Adult, &Some(Archetype::Versionist), "test");
        assert!(art.contains("Versionist"));
        let art = ascii_art(&Stage::Adult, &Some(Archetype::AiMage), "test");
        assert!(art.contains("AI Mage"));
        let art = ascii_art(&Stage::Adult, &Some(Archetype::CloudDweller), "test");
        assert!(art.contains("CloudDweller"));
        let art = ascii_art(&Stage::Adult, &Some(Archetype::AncientMage), "test");
        assert!(art.contains("AncientMage"));
        let art = ascii_art(&Stage::Adult, &Some(Archetype::Generalist), "test");
        assert!(art.contains("Generalist"));
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
                print!("{}", ascii_art(&stage, &None, name));
            }
            println!("[Adult/Generalist]");
            print!(
                "{}",
                ascii_art(&Stage::Adult, &Some(Archetype::Generalist), name)
            );
            println!();
        }
    }
}
