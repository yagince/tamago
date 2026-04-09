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
        Stage::Egg => EGG[h % EGG.len()].to_string(),
        Stage::Baby => BABY[h % BABY.len()].to_string(),
        Stage::Child => CHILD[h % CHILD.len()].to_string(),
        Stage::Teen => TEEN[h % TEEN.len()].to_string(),
        Stage::Adult => {
            let arts = match archetype {
                Some(Archetype::Versionist) => &ADULT_VERSIONIST[..],
                Some(Archetype::AiMage) => &ADULT_AIMAGE[..],
                Some(Archetype::CloudDweller) => &ADULT_CLOUD[..],
                Some(Archetype::AncientMage) => &ADULT_ANCIENT[..],
                Some(Archetype::Generalist) | None => &ADULT_GENERALIST[..],
            };
            arts[h % arts.len()].to_string()
        }
    }
}

// ============================================================
// Egg: たまご
// ============================================================

const EGG: &[&str] = &[
    // ノーマル
    "\
\n     ▄███▄\
\n   ██     ██\
\n  █         █\
\n  █         █\
\n   ██     ██\
\n     ▀███▀\n",
    // ヒビ入り
    "\
\n     ▄███▄\
\n   ██   / ██\
\n  █    /    █\
\n  █         █\
\n   ██     ██\
\n     ▀███▀\n",
    // 揺れ
    "\
\n      ▄███▄\
\n    ██     ██\
\n   █         █\
\n   █         █\
\n    ██     ██\
\n      ▀███▀\n",
    // 模様つき
    "\
\n     ▄███▄\
\n   ██ ░ ░ ██\
\n  █  ░ ░ ░  █\
\n  █         █\
\n   ██     ██\
\n     ▀███▀\n",
];

// ============================================================
// Baby: 孵化したて
// ============================================================

const BABY: &[&str] = &[
    // にっこり
    "\
\n     ▄███▄\
\n   █ ●   ● █\
\n   █ ░   ░ █\
\n   █   ▽   █\
\n    ▀█▄▄▄█▀\
\n      █ █\n",
    // きょとん
    "\
\n     ▄███▄\
\n   █ ○   ○ █\
\n   █       █\
\n   █   ·   █\
\n    ▀█▄▄▄█▀\
\n      █ █\n",
    // うれしい
    "\
\n     ▄███▄\
\n   █ ^   ^ █\
\n   █ ░   ░ █\
\n   █  ◡◡   █\
\n    ▀█▄▄▄█▀\
\n      █ █\n",
    // ねむい
    "\
\n     ▄███▄\
\n   █ ─   ─ █\
\n   █ ░   ░ █\
\n   █   ω   █\
\n    ▀█▄▄▄█▀\
\n      █ █\n",
    // びっくり
    "\
\n     ▄███▄\
\n   █ ◉   ◉ █\
\n   █       █\
\n   █   △   █\
\n    ▀█▄▄▄█▀\
\n      █ █\n",
    // もぐもぐ
    "\
\n     ▄███▄\
\n   █ ●   ● █\
\n   █ ░   ░ █\
\n   █  ═══  █\
\n    ▀█▄▄▄█▀\
\n      █ █\n",
    // てれてれ
    "\
\n     ▄███▄\
\n   █ >   < █\
\n   █ ░   ░ █\
\n   █   ▽   █\
\n    ▀█▄▄▄█▀\
\n      █ █\n",
    // わくわく
    "\
\n     ▄███▄\
\n   █ ★   ★ █\
\n   █ ░   ░ █\
\n   █  ▽▽   █\
\n    ▀█▄▄▄█▀\
\n      █ █\n",
    // ドラゴンの子供（卵から孵化）
    "\
\n        ▄██▄\
\n   ▄▀  █ ●  ● █\
\n  ▀▄  █  ░  ░ █\
\n    ~ █   ▽   █\
\n      ▀▄▄█▄▄▀\
\n     ▄▀    ▀▄\
\n     ▀ ▀▀▀▀ ▀\n",
];

// ============================================================
// Child: すこし大きくなった
// ============================================================

const CHILD: &[&str] = &[
    // 元気
    "\
\n     ▄███▄\
\n   █ ●   ● █\
\n   █ ░   ░ █\
\n   █  ▽▽   █\
\n   █       █\
\n    █     █\
\n     █▄ █▄\n",
    // おすまし
    "\
\n     ▄███▄\
\n   █ ◉   ◉ █\
\n   █       █\
\n   █   ─   █\
\n   █       █\
\n    █     █\
\n     █▄ █▄\n",
    // にっこり
    "\
\n     ▄███▄\
\n   █ ^   ^ █\
\n   █ ░   ░ █\
\n   █   ω   █\
\n   █       █\
\n    █     █\
\n     █▄ █▄\n",
    // わくわく
    "\
\n     ▄███▄\
\n   █ ★   ★ █\
\n   █ ░   ░ █\
\n   █   ▽   █\
\n   █  ♪    █\
\n    █     █\
\n     █▄ █▄\n",
];

// ============================================================
// Teen: だいぶ育った
// ============================================================

const TEEN: &[&str] = &[
    // クール
    "\
\n      ▄▄▄\
\n    ▄█   █▄\
\n   █ ●   ● █\
\n   █ ░   ░ █\
\n   █   ─   █\
\n   ██     ██\
\n    █     █\
\n    █▀   █▀\n",
    // やんちゃ
    "\
\n    ▄█▄ ▄█▄\
\n   █ ◉   ◉ █\
\n   █ ░   ░ █\
\n   █  ▽▽   █\
\n   ██     ██\
\n    █     █\
\n    █▀   █▀\n",
    // 凛々しい
    "\
\n    ╱▀▀▀▀╲\
\n   █ ▪   ▪ █\
\n   █       █\
\n   █   △   █\
\n   ██     ██\
\n    █     █\
\n    █▀   █▀\n",
    // 元気
    "\
\n      ▄▄▄\
\n    ▄█   █▄\
\n   █ ^   ^ █\
\n   █ ░   ░ █\
\n   █  ◡◡   █\
\n   ██     ██\
\n    █     █\
\n    █▀   █▀\n",
];

// ============================================================
// Adult - Versionist (Git)
// ============================================================

const ADULT_VERSIONIST: &[&str] = &[
    "\
\n    ▄█████▄\
\n   █ ●   ● █\
\n   █ ░   ░ █\
\n   █  ═══  █\
\n   ██ ▓▓▓ ██\
\n    █ ███ █\
\n    █▀   █▀\
\n   🐙 Versionist\n",
    "\
\n    ▄█████▄\
\n   █ ◉   ◉ █\
\n   █ ░   ░ █\
\n   █   △   █\
\n   ██ ▓▓▓ ██\
\n    █ ███ █\
\n    █▀   █▀\
\n   🐙 Versionist\n",
];

// ============================================================
// Adult - AiMage (AI)
// ============================================================

const ADULT_AIMAGE: &[&str] = &[
    "\
\n      ▄▄▄\
\n    ▄█▓▓▓█▄\
\n   █ ★   ★ █\
\n   █ ░   ░ █\
\n   █  ═══  █\
\n   ██ ░░░ ██\
\n    █     █\
\n    █▀   █▀\
\n   🧙 AI Mage\n",
    "\
\n      ▄▄▄\
\n    ▄█▓▓▓█▄\
\n   █ ◆   ◆ █\
\n   █       █\
\n   █   ω   █\
\n   ██ ░░░ ██\
\n    █     █\
\n    █▀   █▀\
\n   🧙 AI Mage\n",
];

// ============================================================
// Adult - CloudDweller (Infra)
// ============================================================

const ADULT_CLOUD: &[&str] = &[
    "\
\n    ░▄███▄░\
\n   █ ○   ○ █\
\n   █       █\
\n   █  ───  █\
\n   ██░░░░░██\
\n    █     █\
\n    █▀   █▀\
\n   ☁  CloudDweller\n",
    "\
\n    ░▄███▄░\
\n   █ ◎   ◎ █\
\n   █ ░   ░ █\
\n   █   △   █\
\n   ██░░░░░██\
\n    █     █\
\n    █▀   █▀\
\n   ☁  CloudDweller\n",
];

// ============================================================
// Adult - AncientMage (Editor)
// ============================================================

const ADULT_ANCIENT: &[&str] = &[
    "\
\n    ▄█▀▀▀█▄\
\n   █ ▫   ▫ █\
\n   █       █\
\n   █  ═══  █\
\n   ██▒▒▒▒▒██\
\n    █     █\
\n    █▀   █▀\
\n   📜 AncientMage\n",
    "\
\n    ▄█▀▀▀█▄\
\n   █ ◆   ◆ █\
\n   █       █\
\n   █   ω   █\
\n   ██▒▒▒▒▒██\
\n    █     █\
\n    █▀   █▀\
\n   📜 AncientMage\n",
];

// ============================================================
// Adult - Generalist
// ============================================================

const ADULT_GENERALIST: &[&str] = &[
    "\
\n    ▄█████▄\
\n   █ ●   ● █\
\n   █ ░   ░ █\
\n   █  ▽▽   █\
\n   ██     ██\
\n    █     █\
\n    █▀   █▀\
\n   🦊 Generalist\n",
    "\
\n    ▄█████▄\
\n   █ ^   ^ █\
\n   █ ░   ░ █\
\n   █   ω   █\
\n   ██     ██\
\n    █     █\
\n    █▀   █▀\
\n   🦊 Generalist\n",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn egg_has_multiple_patterns() {
        assert!(EGG.len() >= 3);
    }

    #[test]
    fn same_name_same_art() {
        let a = ascii_art(&Stage::Baby, &None, "test1");
        let b = ascii_art(&Stage::Baby, &None, "test1");
        assert_eq!(a, b);
    }

    #[test]
    fn different_name_can_differ() {
        // 十分なパターンがあれば異なる名前で異なるAAになりうる
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
    fn print_all_stages_for_visual_check() {
        let names = ["abc", "xyz", "tamago", "pikachu", "ピカボス", "ほげほげ"];
        for name in names {
            println!("=== {name} ===");
            for stage in [Stage::Egg, Stage::Baby, Stage::Child, Stage::Teen] {
                println!("[{stage:?}]");
                print!("{}", ascii_art(&stage, &None, name));
            }
            for arch in [
                Archetype::Versionist,
                Archetype::AiMage,
                Archetype::CloudDweller,
                Archetype::AncientMage,
                Archetype::Generalist,
            ] {
                println!("[Adult/{arch:?}]");
                print!("{}", ascii_art(&Stage::Adult, &Some(arch), name));
            }
            println!();
        }
    }
}
