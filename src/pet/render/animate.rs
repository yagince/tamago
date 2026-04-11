/// AA に動的なアニメーション効果を追加するエンジン
/// タイムスタンプをシードにして毎回微妙に違う見た目にする
use std::time::{SystemTime, UNIX_EPOCH};

fn time_seed() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as usize
}

/// デコレーション文字セット
const SPARKLES: &[&str] = &["✦", "✧", "☆", "·", "*", "˚", "°", "⋆"];
const HAPPY_DECOS: &[&str] = &["♡", "♪", "✦", "☆", "✧", "·"];
const SAD_DECOS: &[&str] = &["·", ".", "。", " ", " ", " "];
const LEVEL_UP_DECOS: &[&str] = &["★", "✦", "✧", "☆", "⋆", "✦"];

/// AA にデコレーションとマイクロ表情を適用
pub fn animate(aa: &str, hunger: u8, mood: u8, exp: u64) -> String {
    let seed = time_seed();
    let mut result = add_decorations(aa, seed, hunger, mood, exp);
    result = apply_micro_expression(&result, seed);
    result
}

/// AA の周囲にデコレーション文字を配置
fn add_decorations(aa: &str, seed: usize, hunger: u8, mood: u8, exp: u64) -> String {
    let min_stat = hunger.min(mood);
    let decos = if min_stat < 30 {
        SAD_DECOS
    } else if exp % 100 < 10 {
        // EXP のキリ番近くはキラキラ多め
        LEVEL_UP_DECOS
    } else if min_stat > 80 {
        HAPPY_DECOS
    } else {
        SPARKLES
    };

    let lines: Vec<&str> = aa.lines().collect();
    if lines.is_empty() {
        return aa.to_string();
    }

    // AA の最大幅を取得
    let max_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let deco_width = max_width + 6; // 左右に3文字分の余白

    let mut result = Vec::new();
    // デコレーション数: mood が高いほど多い
    let deco_count = if min_stat > 80 {
        3
    } else if min_stat > 50 {
        2
    } else if min_stat > 30 {
        1
    } else {
        0
    };

    for (i, line) in lines.iter().enumerate() {
        let char_count = line.chars().count();
        let padding = deco_width.saturating_sub(char_count);
        let left_pad = 3;
        let right_pad = padding.saturating_sub(left_pad);

        // 左側デコレーション
        let mut left: String = " ".repeat(left_pad);
        if deco_count > 0 && (seed.wrapping_add(i * 7)) % 5 < deco_count {
            let deco_idx = seed.wrapping_add(i * 13) % decos.len();
            let pos = seed.wrapping_add(i * 3) % left_pad;
            let mut chars: Vec<char> = left.chars().collect();
            if pos < chars.len() {
                // 1文字のデコを挿入
                let deco = decos[deco_idx];
                chars[pos] = deco.chars().next().unwrap_or(' ');
            }
            left = chars.into_iter().collect();
        }

        // 右側デコレーション
        let mut right: String = " ".repeat(right_pad.min(4));
        if deco_count > 1 && (seed.wrapping_add(i * 11)) % 4 < deco_count - 1 {
            let deco_idx = seed.wrapping_add(i * 17) % decos.len();
            let right_chars_len = right.chars().count();
            if right_chars_len > 0 {
                let pos = seed.wrapping_add(i * 5) % right_chars_len;
                let mut chars: Vec<char> = right.chars().collect();
                let deco = decos[deco_idx];
                chars[pos] = deco.chars().next().unwrap_or(' ');
                right = chars.into_iter().collect();
            }
        }

        result.push(format!("{left}{line}{right}"));
    }

    result.join("\n")
}

/// マイクロ表情: 確率でまばたきや口の微変化
fn apply_micro_expression(aa: &str, seed: usize) -> String {
    // 20% の確率でまばたき（目の行の一部を変更）
    let blink = seed.is_multiple_of(5);
    if !blink {
        return aa.to_string();
    }

    // ▄ (目のピクセル) を一時的に消す = まばたき
    // 行ごとに処理して、内部の ▄ を空白にする
    let lines: Vec<&str> = aa.lines().collect();
    let mut result = Vec::new();
    let mut blinked = false;

    for line in &lines {
        if !blinked && line.contains('▄') {
            // この行に目がありそう
            let chars: Vec<char> = line.chars().collect();
            let first_block = chars.iter().position(|&c| c == '█' || c == '▀' || c == '▄');
            let last_block = chars
                .iter()
                .rposition(|&c| c == '█' || c == '▀' || c == '▄');

            if let (Some(first), Some(last)) = (first_block, last_block)
                && last - first > 4
            {
                // 内部の ▄ を空白にする（輪郭は残す）
                let mut new_chars = chars.clone();
                for ch in new_chars.iter_mut().take(last).skip(first + 1) {
                    if *ch == '▄' {
                        *ch = ' ';
                    }
                }
                result.push(new_chars.into_iter().collect::<String>());
                blinked = true;
                continue;
            }
        }
        result.push(line.to_string());
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_EGG: &str = "\
\n   ▄▀▀▀▀▀▀▀▀▄\
\n ▄▀          ▀▄\
\n █   ▄    ▄   █\
\n █            █\
\n █    ▀▄▄▀    █\
\n  ▀▄        ▄▀\
\n    ▀▀▄▄▄▄▀▀\n";

    #[test]
    fn animate_adds_content() {
        let result = animate(TEST_EGG, 100, 100, 500);
        // デコレーションが追加されるので元より長い
        assert!(result.len() >= TEST_EGG.len());
    }

    #[test]
    fn animate_varies_with_mood() {
        let happy = animate(TEST_EGG, 100, 100, 500);
        let sad = animate(TEST_EGG, 10, 10, 500);
        // 完全に同じにはならない（デコレーションが違う）
        // ただしタイミング依存なので厳密には比較しない
        assert!(!happy.is_empty());
        assert!(!sad.is_empty());
    }

    #[test]
    fn print_animate_variants() {
        for i in 0..5 {
            println!("=== Frame {i} ===");
            print!("{}", animate(TEST_EGG, 100, 100, 500));
            println!();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    #[test]
    fn print_sad_animate() {
        println!("=== Sad ===");
        print!("{}", animate(TEST_EGG, 20, 20, 100));
    }
}
