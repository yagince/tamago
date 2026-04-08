use super::{Archetype, Stage};

fn name_hash(name: &str) -> usize {
    // FNV-1a hash for better distribution with multibyte chars
    let mut h: u64 = 0xcbf29ce484222325;
    for b in name.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h as usize
}

const SALTS: &[u64] = &[0, 7, 13, 19, 29, 37, 43, 53];

fn pick<'a>(parts: &[&'a str], hash: usize, salt: usize) -> &'a str {
    let s = SALTS[salt % SALTS.len()];
    let shifted = hash.wrapping_shr((s as u32) % 64);
    parts[shifted % parts.len()]
}

// --- パーツ定義 ---

const EYES: &[&str] = &[
    " o  o ", " ^  ^ ", " -  - ", " *  * ", " @  @ ", " .  . ", " O  O ", " u  u ", " '  ' ",
    " =  = ", " o''o ", " o..o ", " ^~~^ ", " *``* ", " @::@ ", " o><o ", " ^//^ ", " -<<- ",
    " .~~. ", " O::O ",
];

const MOUTHS: &[&str] = &[
    "  ww  ", "  vv  ", "  ..  ", "  oo  ", "  uu  ", "  ~~  ", "  __  ", "  ^^  ", "  33  ",
    "  mm  ", "  --  ", "  nn  ", "  <>  ", " =vv= ", "  dd  ", " www  ",
];

// 耳: 全て8文字幅（顔と同じ幅）。インデントはテンプレート側で制御
const EARS: &[(&str, &str)] = &[
    (" ^    ^ ", " ~    ~ "),
    (" *    * ", " '    ' "),
    (" /    \\ ", " ?    ? "),
    (" +    + ", " `    ` "),
    (" v    v ", " n    n "),
    (" (    ) ", " o    o "),
    (" >    < ", " <    > "),
    (" @    @ ", " #    # "),
    (" !    ! ", " i    i "),
    (" $    $ ", " &    & "),
    (" ~    ~ ", " ^    ^ "),
    (" )    ( ", " (    ) "),
    (" d    b ", " q    p "),
    (" Y    Y ", " T    T "),
    (" }    { ", " {    } "),
    (" =    = ", " -    - "),
    (" Ψ    Ψ ", " ψ    ψ "),
    (" Ω    Ω ", " ω    ω "),
    (" λ    λ ", " Λ    Λ "),
    (" Σ    Σ ", " σ    σ "),
    (" Д    Д ", " Л    Л "),
    (" Ж    Ж ", " Ф    Ф "),
    (" π    π ", " τ    τ "),
    (" δ    δ ", " θ    θ "),
];

const CHEEKS: &[(&str, &str)] = &[
    ("(", ")"),
    ("+", "+"),
    ("-", "-"),
    ("[", "]"),
    ("{", "}"),
    ("<", ">"),
    ("|", "|"),
    ("/", "\\"),
    (":", ":"),
    ("!", "!"),
    ("~", "~"),
    ("*", "*"),
    ("#", "#"),
    ("@", "@"),
    ("$", "$"),
    ("&", "&"),
];

const FEET: &[&str] = &[
    "'------'",
    "d      b",
    "|_||||_|",
    "()    ()",
    "\\_/  \\_/",
    "m      m",
    "`------`",
    "v      v",
    "J      L",
    "\\|    |/",
    "/|    |\\",
    "~------~",
    "||    ||",
    "^------^",
    "L      J",
    "o      o",
];

const MARKS: &[&str] = &[
    "      ", " <>   ", " ::   ", " **   ", " ##   ", " ++   ", " ..   ", " ~~   ",
];

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
\n    ___\
\n   /   \\\
\n  | .   |\
\n  |  .  |\
\n   \\___/\n";

fn pick_ears(hash: usize, salt: usize) -> (&'static str, &'static str) {
    let s = SALTS[salt % SALTS.len()];
    let shifted = hash.wrapping_shr((s as u32) % 64);
    EARS[shifted % EARS.len()]
}

fn pick_cheeks(hash: usize, salt: usize) -> (&'static str, &'static str) {
    let s = SALTS[salt % SALTS.len()];
    let shifted = hash.wrapping_shr((s as u32) % 64);
    CHEEKS[shifted % CHEEKS.len()]
}

fn render_baby(name: &str) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let (ear_l, _) = pick_ears(h, 2);
    let (cl, cr) = pick_cheeks(h, 3);
    let feet = pick(FEET, h, 4);

    format!("\n  {ear_l}\n  {cl}{eyes}{cr}\n  {cl}{mouth}{cr}\n  {feet}\n")
}

fn render_child(name: &str) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let (ear_l, _) = pick_ears(h, 2);
    let (cl, cr) = pick_cheeks(h, 3);
    let mark = pick(MARKS, h, 5);
    let feet = pick(FEET, h, 4);

    format!("\n  {ear_l}\n  {cl}{eyes}{cr}\n  {cl}{mouth}{cr}\n  |{mark}|\n  {feet}\n")
}

fn render_teen(name: &str) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let (_, ear_r) = pick_ears(h, 2);
    let (cl, cr) = pick_cheeks(h, 3);
    let mark = pick(MARKS, h, 5);

    format!(
        "\n   {ear_r}\n   {cl}{eyes}{cr}\n   {cl}{mouth}{cr}\n   /|{mark}|\\\n  / '------' \\\n"
    )
}

fn render_adult(name: &str, archetype: &Option<Archetype>) -> String {
    let h = name_hash(name);
    let eyes = pick(EYES, h, 0);
    let mouth = pick(MOUTHS, h, 1);
    let (_, ear_r) = pick_ears(h, 2);
    let (cl, cr) = pick_cheeks(h, 3);
    let mark = pick(MARKS, h, 5);

    let title = match archetype {
        Some(Archetype::Versionist) => "  ~Versionist~",
        Some(Archetype::AiMage) => "   ~AI Mage~",
        Some(Archetype::CloudDweller) => " ~CloudDweller~",
        Some(Archetype::AncientMage) => " ~AncientMage~",
        Some(Archetype::Generalist) | None => "  ~Generalist~",
    };

    format!(
        "\n    {ear_r}\n    {cl}{eyes}{cr}\n    {cl}{mouth}{cr}\n  ---|{mark}|---\n  /  '------'  \\\n  |            |\n  '------------'\n  {title}\n"
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
        let a = ascii_art(&Stage::Baby, &None, "aaa");
        let b = ascii_art(&Stage::Baby, &None, "zzz");
        assert_ne!(a, b);
    }

    #[test]
    fn japanese_names_produce_different_art() {
        let a = ascii_art(&Stage::Baby, &None, "ピカボス");
        let b = ascii_art(&Stage::Baby, &None, "ほげほげ");
        let c = ascii_art(&Stage::Baby, &None, "これはどうだ？");
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
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
