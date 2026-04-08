use super::{Archetype, Stage};

fn name_hash(name: &str) -> usize {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in name.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h as usize
}

const SALTS: &[u64] = &[0, 7, 13, 19, 29, 37, 43, 53];

fn pick_one<'a>(parts: &[&'a str], hash: usize, salt: usize) -> &'a str {
    let s = SALTS[salt % SALTS.len()];
    let shifted = hash.wrapping_shr((s as u32) % 64);
    parts[shifted % parts.len()]
}

fn pick_pair<'a>(parts: &[(&'a str, &'a str)], hash: usize, salt: usize) -> (&'a str, &'a str) {
    let s = SALTS[salt % SALTS.len()];
    let shifted = hash.wrapping_shr((s as u32) % 64);
    parts[shifted % parts.len()]
}

// --- 1文字パーツ ---

const EYES: &[(&str, &str)] = &[
    ("o", "o"),
    ("^", "^"),
    ("-", "-"),
    ("*", "*"),
    ("@", "@"),
    (".", "."),
    ("O", "O"),
    ("u", "u"),
    ("'", "'"),
    ("=", "="),
    ("0", "0"),
    ("x", "x"),
    ("T", "T"),
    ("~", "~"),
    (">", "<"),
    ("$", "$"),
    ("p", "q"),
    ("d", "b"),
    ("n", "n"),
    ("v", "v"),
];

const BLUSH: &[&str] = &[
    "  ", "''", "..", "~~", "::", "``", "><", "//", "--", "**", "<>", "##", "||", "^^", "==", "@@",
];

const MOUTHS: &[(&str, &str)] = &[
    ("w", "w"),
    ("v", "v"),
    (".", "."),
    ("o", "o"),
    ("u", "u"),
    ("~", "~"),
    ("_", "_"),
    ("^", "^"),
    ("3", "3"),
    ("m", "m"),
    ("-", "-"),
    ("n", "n"),
    ("<", ">"),
    ("=", "="),
    ("d", "d"),
    ("D", "D"),
    ("[", "]"),
    ("{", "}"),
    ("(", ")"),
    ("/", "\\"),
];

const EARS: &[(&str, &str)] = &[
    ("^", "^"),
    ("*", "*"),
    ("/", "\\"),
    ("+", "+"),
    ("v", "v"),
    ("(", ")"),
    (">", "<"),
    ("@", "@"),
    ("!", "!"),
    ("$", "$"),
    ("~", "~"),
    (")", "("),
    ("d", "b"),
    ("Y", "Y"),
    ("}", "{"),
    ("=", "="),
    ("Ψ", "Ψ"),
    ("Ω", "Ω"),
    ("λ", "λ"),
    ("Σ", "Σ"),
    ("Д", "Д"),
    ("Ж", "Ж"),
    ("π", "π"),
    ("δ", "δ"),
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

const MARKS: &[(&str, &str)] = &[
    (" ", " "),
    ("<", ">"),
    (":", ":"),
    ("*", "*"),
    ("#", "#"),
    ("+", "+"),
    (".", "."),
    ("~", "~"),
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

struct Parts {
    el: &'static str,  // 左耳
    er: &'static str,  // 右耳
    le: &'static str,  // 左目
    re: &'static str,  // 右目
    bl: &'static str,  // 目の間装飾 (2文字)
    mol: &'static str, // 口左
    mor: &'static str, // 口右
    cl: &'static str,  // 左頬
    cr: &'static str,  // 右頬
    ml: &'static str,  // 左マーク
    mr: &'static str,  // 右マーク
    ft: &'static str,  // 足 (8文字)
}

fn pick_parts(name: &str) -> Parts {
    let h = name_hash(name);
    let (el, er) = pick_pair(EARS, h, 0);
    let (le, re) = pick_pair(EYES, h, 1);
    let bl = pick_one(BLUSH, h, 2);
    let (mol, mor) = pick_pair(MOUTHS, h, 3);
    let (cl, cr) = pick_pair(CHEEKS, h, 4);
    let (ml, mr) = pick_pair(MARKS, h, 5);
    let ft = pick_one(FEET, h, 6);

    Parts {
        el,
        er,
        le,
        re,
        bl,
        mol,
        mor,
        cl,
        cr,
        ml,
        mr,
        ft,
    }
}

//  Baby:
//   el      er        耳: 位置3と8
//  cl le bl re cr     顔: 位置2-9 (頬1+目1+装飾2+目1+頬1 = 8)
//  cl  momo  cr       口: 位置2-9
//  feet                足: 位置2-9

fn render_baby(name: &str) -> String {
    let p = pick_parts(name);
    format!(
        "\n  {el}      {er}\n  {cl} {le}{bl}{re} {cr}\n  {cl}  {mol}{mor}  {cr}\n  {ft}\n",
        el = p.el,
        er = p.er,
        cl = p.cl,
        cr = p.cr,
        le = p.le,
        re = p.re,
        bl = p.bl,
        mol = p.mol,
        mor = p.mor,
        ft = p.ft,
    )
}

fn render_child(name: &str) -> String {
    let p = pick_parts(name);
    format!(
        "\n  {el}      {er}\n  {cl} {le}{bl}{re} {cr}\n  {cl}  {mol}{mor}  {cr}\n  |  {ml}{mr}  |\n  {ft}\n",
        el = p.el,
        er = p.er,
        cl = p.cl,
        cr = p.cr,
        le = p.le,
        re = p.re,
        bl = p.bl,
        mol = p.mol,
        mor = p.mor,
        ml = p.ml,
        mr = p.mr,
        ft = p.ft,
    )
}

fn render_teen(name: &str) -> String {
    let p = pick_parts(name);
    format!(
        "\n   {el}      {er}\n   {cl} {le}{bl}{re} {cr}\n   {cl}  {mol}{mor}  {cr}\n   /| {ml}{mr}  |\\\n  / '-----' \\\n",
        el = p.el,
        er = p.er,
        cl = p.cl,
        cr = p.cr,
        le = p.le,
        re = p.re,
        bl = p.bl,
        mol = p.mol,
        mor = p.mor,
        ml = p.ml,
        mr = p.mr,
    )
}

fn render_adult(name: &str, archetype: &Option<Archetype>) -> String {
    let p = pick_parts(name);
    let title = match archetype {
        Some(Archetype::Versionist) => "  ~Versionist~",
        Some(Archetype::AiMage) => "   ~AI Mage~",
        Some(Archetype::CloudDweller) => " ~CloudDweller~",
        Some(Archetype::AncientMage) => " ~AncientMage~",
        Some(Archetype::Generalist) | None => "  ~Generalist~",
    };
    format!(
        "\n    {el}      {er}\n    {cl} {le}{bl}{re} {cr}\n    {cl}  {mol}{mor}  {cr}\n  --|  {ml}{mr}  |--\n  /  '------'  \\\n  |            |\n  '------------'\n  {title}\n",
        el = p.el,
        er = p.er,
        cl = p.cl,
        cr = p.cr,
        le = p.le,
        re = p.re,
        bl = p.bl,
        mol = p.mol,
        mor = p.mor,
        ml = p.ml,
        mr = p.mr,
        title = title,
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
