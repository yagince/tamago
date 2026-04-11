use std::time::{SystemTime, UNIX_EPOCH};

const PREFIXES: &[&str] = &[
    "ピカ",
    "モグ",
    "フワ",
    "ゴロ",
    "プチ",
    "ニャ",
    "ポコ",
    "クル",
    "トゲ",
    "ヒノ",
    "ミズ",
    "カゼ",
    "ツキ",
    "ホシ",
    "ユメ",
    "ソラ",
    "モコ",
    "リン",
    "コロ",
    "チビ",
    "ドン",
    "ギガ",
    "メラ",
    "ブル",
    "シャキ",
    "ムク",
    "ハネ",
    "ケム",
    "ノビ",
    "ズル",
    "パチ",
    "ワニ",
    "カメ",
    "ヤミ",
    "ヒカ",
    "ネム",
    "タマ",
    "クモ",
    "ホネ",
    "イワ",
    "テツ",
    "アメ",
    "ルリ",
    "サン",
    "キバ",
    "ツノ",
    "マグ",
    "ヌマ",
    "カイ",
    "ラク",
];

const SUFFIXES: &[&str] = &[
    "リン",
    "チュウ",
    "ドン",
    "プス",
    "ラス",
    "モン",
    "ニャン",
    "マル",
    "ロン",
    "ダマ",
    "ッチ",
    "ノン",
    "バル",
    "グマ",
    "ッピ",
    "ルス",
    "タン",
    "サク",
    "ミー",
    "ゴン",
    "パス",
    "ナイト",
    "ルン",
    "ビー",
    "デス",
    "ダー",
    "ボス",
    "キン",
    "サウ",
    "ドラ",
    "ガメ",
    "リュウ",
    "オー",
    "ラン",
    "ゼル",
    "ムシ",
    "トカ",
    "アラ",
    "フォン",
    "ガル",
    "ドル",
    "バン",
    "テル",
    "カス",
    "ノス",
    "マス",
    "ゼン",
    "ライ",
    "ジン",
    "コウ",
];

pub fn generate_name(engine: Option<&mut dyn crate::llm::TextGenerator>) -> String {
    if let Some(engine) = engine
        && let Some(name) = engine.generate(
            "ターミナルペットの名前を1つだけ考えて。ポケモンっぽいカタカナの名前で、名前だけを出力して。",
            "あなたはターミナルペットの名前を考えるAIです。名前だけを出力してください。",
            10,
        ) {
            let name = name.trim().to_string();
            if !name.is_empty() && name.len() <= 30 {
                return name;
            }
        }
    random_name()
}

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
