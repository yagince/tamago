use super::{Archetype, Stage};

/// 名前からハッシュ値を生成（個体ごとに固定の見た目にする）
fn name_hash(name: &str) -> usize {
    name.bytes().fold(0usize, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as usize)
    })
}

fn pick<'a>(parts: &[&'a str], hash: usize, salt: usize) -> &'a str {
    parts[hash.wrapping_add(salt) % parts.len()]
}

// --- パーツ定義 ---

const EYES: &[&str] = &[
    "o  o", "^  ^", "-  -", "*  *", "◉  ◉", "@  @", "●  ●", "°  °", "~  ~", ">  <",
];

const MOUTHS: &[&str] = &[" ▽ ", " ω ", " ∀ ", " □ ", " ー ", " ◇ ", " ▿ ", " △ "];

const EAR_L: &[&str] = &["^", "?", "~", "*", "/", "♪", "'", "`", "⌐", "¬"];
const EAR_R: &[&str] = &["^", "?", "~", "*", "\\", "♪", "'", "`", "⌐", "¬"];

const BODY_MARKS: &[&str] = &["  ", "♡ ", "★ ", "♦ ", "◆ ", "♠ ", "● ", "◎ ", "✦ ", "☆ "];

const FEET: &[&str] = &["||||", "/\\/\\", "⌇⌇⌇⌇", "∪∪", "vvvv", "~~~~", "^^^^", "ωω"];

const TEEN_TAILS: &[&str] = &["~", "⌁", "♪", "★", "∿", "⚡", "♦", "✦"];

const ADULT_DECORATIONS: &[&str] = &["♛", "⚔", "✧", "⚡", "♦", "☀", "❖", "✿", "⊕", "△"];

pub fn ascii_art(stage: &Stage, archetype: &Option<Archetype>, name: &str) -> String {
    match stage {
        Stage::Egg => EGG.to_string(),
        Stage::Baby => render_baby(name),
        Stage::Child => render_child(name),
        Stage::Teen => render_teen(name),
        Stage::Adult => render_adult(name, archetype),
    }
}

const EGG: &str = r"
    ___
   /   \
  |     |
  |     |
   \___/
";

fn render_baby(name: &str) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let ear_l = pick(EAR_L, h, 2);
    let ear_r = pick(EAR_R, h, 3);
    let feet = pick(FEET, h, 4);

    format!(
        r"
   {ear_l}__{ear_r}
  ({eyes})
  ({mouth})
   {feet}
"
    )
}

fn render_child(name: &str) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let mark = pick(BODY_MARKS, h, 5);

    format!(
        r"
  \({eyes})/
   ({mouth})
   |{mark}|
  / \  / \
"
    )
}

fn render_teen(name: &str) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let ear_l = pick(EAR_L, h, 2);
    let ear_r = pick(EAR_R, h, 3);
    let tail = pick(TEEN_TAILS, h, 6);

    format!(
        r"
   /{ear_l}_{ear_r}\
  ( {eyes} )
  ( {mouth} )
  /|    |\{tail}
 (_|    |_)
"
    )
}

fn render_adult(name: &str, archetype: &Option<Archetype>) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let ear_l = pick(EAR_L, h, 2);
    let ear_r = pick(EAR_R, h, 3);
    let deco = pick(ADULT_DECORATIONS, h, 7);
    let mark = pick(BODY_MARKS, h, 5);

    let title = match archetype {
        Some(Archetype::Versionist) => "🐙 Versionist",
        Some(Archetype::AiMage) => "🧙 AI Mage",
        Some(Archetype::CloudDweller) => "☁️  Cloud Dweller",
        Some(Archetype::AncientMage) => "📜 Ancient Mage",
        Some(Archetype::Generalist) | None => "🦊 Generalist",
    };

    format!(
        r"
    {deco}
   /{ear_l}_{ear_r}\
  ( {eyes} )
  ( {mouth} )
  /|{mark}|\
 / |    | \
  {title}
"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn egg_is_fixed() {
        let a = ascii_art(&Stage::Egg, &None, "aaa");
        let b = ascii_art(&Stage::Egg, &None, "bbb");
        assert_eq!(a, b);
    }

    #[test]
    fn same_name_same_art() {
        let a = ascii_art(&Stage::Baby, &None, "ピカドン");
        let b = ascii_art(&Stage::Baby, &None, "ピカドン");
        assert_eq!(a, b);
    }

    #[test]
    fn different_name_different_art() {
        let a = ascii_art(&Stage::Baby, &None, "ピカドン");
        let b = ascii_art(&Stage::Baby, &None, "モグリン");
        assert_ne!(a, b);
    }

    #[test]
    fn each_stage_renders() {
        let name = "テスト";
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
    }

    #[test]
    fn name_hash_is_deterministic() {
        assert_eq!(name_hash("abc"), name_hash("abc"));
        assert_ne!(name_hash("abc"), name_hash("xyz"));
    }
}
