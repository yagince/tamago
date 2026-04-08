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

// --- パーツ定義 (全て半角で幅を揃える) ---

// 目: 4文字幅 "L  R"
const EYES: &[&str] = &[
    "o  o", "^  ^", "-  -", "*  *", "@  @", ".  .", "O  O", "u  u", "'  '", "=  =",
];

// 口: 4文字幅 " XX "
const MOUTHS: &[&str] = &[
    " w  ", " v  ", " .  ", " o  ", " u  ", " ~  ", " _  ", " ^  ",
];

// 耳: (左, 右) ペア
const EARS: &[(&str, &str)] = &[
    ("^", "^"),
    ("~", "~"),
    ("*", "*"),
    ("'", "'"),
    ("`", "`"),
    ("/", "\\"),
    ("?", "?"),
    ("+", "+"),
];

// ボディマーク: 2文字幅
const MARKS: &[&str] = &["  ", "<>", "::", "**", "##", "++", "..", "~~", "||", "@@"];

pub fn ascii_art(stage: &Stage, archetype: &Option<Archetype>, name: &str) -> String {
    match stage {
        Stage::Egg => EGG.to_string(),
        Stage::Baby => render_baby(name),
        Stage::Child => render_child(name),
        Stage::Teen => render_teen(name),
        Stage::Adult => render_adult(name, archetype),
    }
}

const EGG: &str = "\
     ___
    /   \\
   | .   |
   |  .  |
    \\___/
";

fn render_baby(name: &str) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let (ear_l, ear_r) = EARS[h.wrapping_add(2) % EARS.len()];

    format!(
        "\
   {ear_l}    {ear_r}
  ( {eyes} )
  ( {mouth} )
   '----'
"
    )
}

fn render_child(name: &str) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let (ear_l, ear_r) = EARS[h.wrapping_add(2) % EARS.len()];
    let mark = pick(MARKS, h, 3);

    format!(
        "\
   {ear_l}    {ear_r}
  ( {eyes} )
  ( {mouth} )
  /| {mark} |\\
   ''  ''
"
    )
}

fn render_teen(name: &str) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let (ear_l, ear_r) = EARS[h.wrapping_add(2) % EARS.len()];
    let mark = pick(MARKS, h, 3);

    format!(
        "\
    {ear_l}    {ear_r}
   ( {eyes} )
   ( {mouth} )
   /|{mark}  |\\
  / | -- | \\
    ''  ''
"
    )
}

fn render_adult(name: &str, archetype: &Option<Archetype>) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let (ear_l, ear_r) = EARS[h.wrapping_add(2) % EARS.len()];
    let mark = pick(MARKS, h, 3);

    let title = match archetype {
        Some(Archetype::Versionist) => "  ~ Versionist ~",
        Some(Archetype::AiMage) => "   ~ AI Mage ~",
        Some(Archetype::CloudDweller) => " ~ CloudDweller ~",
        Some(Archetype::AncientMage) => " ~ AncientMage ~",
        Some(Archetype::Generalist) | None => "  ~ Generalist ~",
    };

    format!(
        "\
     {ear_l}    {ear_r}
    ( {eyes} )
    ( {mouth} )
   --|{mark}  |--
  /  | -- |  \\
  |  '    '  |
  '----------'
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
        let a = ascii_art(&Stage::Baby, &None, "test1");
        let b = ascii_art(&Stage::Baby, &None, "test1");
        assert_eq!(a, b);
    }

    #[test]
    fn different_name_different_art() {
        // ハッシュが異なる名前を使う
        let a = ascii_art(&Stage::Baby, &None, "aaa");
        let b = ascii_art(&Stage::Baby, &None, "zzz");
        assert_ne!(a, b);
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
    }

    #[test]
    fn print_all_stages_for_visual_check() {
        let names = ["ピカドン", "モグリン", "フワッチ"];
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
        }
    }

    #[test]
    fn name_hash_is_deterministic() {
        assert_eq!(name_hash("abc"), name_hash("abc"));
        assert_ne!(name_hash("abc"), name_hash("xyz"));
    }

    #[test]
    fn ears_are_paired() {
        let art = ascii_art(&Stage::Baby, &None, "test");
        // 1行目の耳が対称
        let first_line = art.lines().next().unwrap();
        let trimmed = first_line.trim();
        let first = &trimmed[..1];
        let last = &trimmed[trimmed.len() - 1..];
        // ペアなので左右同じか、/\ のペア
        assert!(
            first == last || (first == "/" && last == "\\"),
            "ears not paired: {first} vs {last}"
        );
    }
}
