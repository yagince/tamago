use std::time::{SystemTime, UNIX_EPOCH};

const PREFIXES: &[&str] = &[
    "ピカ", "モグ", "フワ", "ゴロ", "プチ", "ニャ", "ポコ", "クル", "トゲ", "ヒノ",
    "ミズ", "カゼ", "ツキ", "ホシ", "ユメ", "ソラ", "モコ", "リン", "コロ", "チビ",
    "ドン", "ギガ", "メラ", "ブル", "シャキ",
];

const SUFFIXES: &[&str] = &[
    "リン", "チュウ", "ドン", "プス", "ラス", "モン", "ニャン", "マル", "ロン", "ダマ",
    "ッチ", "ノン", "バル", "グマ", "ッピ", "ルス", "タン", "サク", "ミー", "ゴン",
    "パス", "ナイト", "ルン", "ビー", "デス",
];

pub fn random_name() -> String {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let prefix = PREFIXES[(seed as usize) % PREFIXES.len()];
    let suffix = SUFFIXES[((seed / 997) as usize) % SUFFIXES.len()];
    format!("{prefix}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_name_is_not_empty() {
        let name = random_name();
        assert!(!name.is_empty());
    }

    #[test]
    fn name_combines_prefix_and_suffix() {
        let name = random_name();
        let has_prefix = PREFIXES.iter().any(|p| name.starts_with(p));
        let has_suffix = SUFFIXES.iter().any(|s| name.ends_with(s));
        assert!(has_prefix, "名前がプレフィックスで始まっていない: {name}");
        assert!(has_suffix, "名前がサフィックスで終わっていない: {name}");
    }
}
