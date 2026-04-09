/// AA のビットマップ往復で表情を動的に変更するエンジン

/// 2値ピクセルグリッド
struct Grid {
    width: usize,
    height: usize,
    pixels: Vec<bool>,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![false; width * height],
        }
    }

    fn get(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            false
        }
    }

    fn set(&mut self, x: usize, y: usize, val: bool) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = val;
        }
    }
}

/// AA 文字列 → ピクセルグリッド
fn aa_to_grid(aa: &str) -> Grid {
    let lines: Vec<&str> = aa.lines().filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return Grid::new(0, 0);
    }

    let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let height = lines.len() * 2;

    let mut grid = Grid::new(width, height);

    for (row, line) in lines.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let y = row * 2;
            match ch {
                '█' => {
                    grid.set(col, y, true);
                    grid.set(col, y + 1, true);
                }
                '▀' => {
                    grid.set(col, y, true);
                }
                '▄' => {
                    grid.set(col, y + 1, true);
                }
                _ => {}
            }
        }
    }

    grid
}

/// ピクセルグリッド → AA 文字列
fn grid_to_aa(grid: &Grid) -> String {
    let mut lines = Vec::new();

    for y in (0..grid.height).step_by(2) {
        let mut line = String::new();
        for x in 0..grid.width {
            let top = grid.get(x, y);
            let bot = grid.get(x, y + 1);
            match (top, bot) {
                (true, true) => line.push('█'),
                (true, false) => line.push('▀'),
                (false, true) => line.push('▄'),
                (false, false) => line.push(' '),
            }
        }
        lines.push(line);
    }

    let lines: Vec<String> = lines.iter().map(|l| l.trim_end().to_string()).collect();
    format!("\n{}\n", lines.join("\n"))
}

/// 各行で輪郭の内部にある孤立ピクセルを検出する
/// 「左端の filled から右端の filled の間で、両隣が空白のピクセル」= 内部特徴
fn find_interior_features(grid: &Grid) -> Vec<(usize, usize)> {
    let mut features = Vec::new();

    for y in 0..grid.height {
        // 行内の filled ピクセルの位置を収集
        let filled: Vec<usize> = (0..grid.width).filter(|&x| grid.get(x, y)).collect();
        if filled.len() < 3 {
            continue; // 輪郭だけ
        }

        let left_edge = filled[0];
        let right_edge = filled[filled.len() - 1];

        // 内部 = 左端/右端の輪郭から離れたピクセル
        for &x in &filled {
            if x <= left_edge + 1 || x >= right_edge - 1 {
                continue; // 輪郭に隣接
            }
            // 左右に空白があるか確認（孤立チェック）
            let left_empty = !grid.get(x - 1, y);
            let right_empty = !grid.get(x + 1, y);
            if left_empty || right_empty {
                features.push((x, y));
            }
        }
    }

    features
}

/// 内部特徴を目と口に分類
fn classify_features(features: &[(usize, usize)]) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    if features.is_empty() {
        return (vec![], vec![]);
    }

    let avg_y = features.iter().map(|p| p.1).sum::<usize>() / features.len();

    let eyes: Vec<(usize, usize)> = features.iter().filter(|p| p.1 <= avg_y).copied().collect();
    let mouth: Vec<(usize, usize)> = features.iter().filter(|p| p.1 > avg_y).copied().collect();

    (eyes, mouth)
}

/// condition に応じてグリッド上の顔パーツを書き換え
fn modify_features(
    grid: &mut Grid,
    eyes: &[(usize, usize)],
    mouth: &[(usize, usize)],
    cond: &super::Condition,
) {
    match cond {
        super::Condition::Normal => {}
        super::Condition::Tired => {
            // 目の上半分を消す → 半目
            if !eyes.is_empty() {
                let min_y = eyes.iter().map(|p| p.1).min().unwrap();
                let max_y = eyes.iter().map(|p| p.1).max().unwrap();
                let mid = (min_y + max_y) / 2;
                for &(x, y) in eyes {
                    if y <= mid {
                        grid.set(x, y, false);
                    }
                }
            }
        }
        super::Condition::Exhausted => {
            // 目を全部消して×に
            if !eyes.is_empty() {
                let min_x = eyes.iter().map(|p| p.0).min().unwrap();
                let max_x = eyes.iter().map(|p| p.0).max().unwrap();
                let min_y = eyes.iter().map(|p| p.1).min().unwrap();
                let max_y = eyes.iter().map(|p| p.1).max().unwrap();

                for &(x, y) in eyes {
                    grid.set(x, y, false);
                }

                // 目の領域の中央に×を描く
                let _center_x = (min_x + max_x) / 2;
                let center_y = (min_y + max_y) / 2;
                let half_x = (max_x - min_x) / 4 + 1;
                let half_y = (max_y - min_y) / 4 + 1;
                for i in 0..=half_x.max(half_y) {
                    // 左目の×
                    let lx = min_x + (max_x - min_x) / 4;
                    if lx + i < grid.width && center_y + i < grid.height {
                        grid.set(lx.saturating_sub(i), center_y.saturating_sub(i), true);
                        grid.set(lx + i, center_y.saturating_sub(i), true);
                        grid.set(
                            lx.saturating_sub(i),
                            (center_y + i).min(grid.height - 1),
                            true,
                        );
                        grid.set(lx + i, (center_y + i).min(grid.height - 1), true);
                    }
                    // 右目の×
                    let rx = max_x - (max_x - min_x) / 4;
                    if rx + i < grid.width && center_y + i < grid.height {
                        grid.set(rx.saturating_sub(i), center_y.saturating_sub(i), true);
                        grid.set(rx + i, center_y.saturating_sub(i), true);
                        grid.set(
                            rx.saturating_sub(i),
                            (center_y + i).min(grid.height - 1),
                            true,
                        );
                        grid.set(rx + i, (center_y + i).min(grid.height - 1), true);
                    }
                }
            }

            // 口を波線に
            if !mouth.is_empty() {
                let min_x = mouth.iter().map(|p| p.0).min().unwrap();
                let max_x = mouth.iter().map(|p| p.0).max().unwrap();
                let center_y = mouth.iter().map(|p| p.1).sum::<usize>() / mouth.len();

                for &(x, y) in mouth {
                    grid.set(x, y, false);
                }

                for x in min_x..=max_x {
                    let offset = if (x - min_x) % 2 == 0 { 0 } else { 1 };
                    let y = center_y.saturating_sub(offset);
                    if y < grid.height {
                        grid.set(x, y, true);
                    }
                }
            }
        }
    }
}

/// AA に表情変更を適用
pub fn apply_expression(aa: &str, cond: &super::Condition) -> String {
    if matches!(cond, super::Condition::Normal) {
        return aa.to_string();
    }

    let mut grid = aa_to_grid(aa);
    if grid.width == 0 || grid.height == 0 {
        return aa.to_string();
    }

    let features = find_interior_features(&grid);
    if features.is_empty() {
        return aa.to_string();
    }

    let (eyes, mouth) = classify_features(&features);
    modify_features(&mut grid, &eyes, &mouth, cond);
    grid_to_aa(&grid)
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
    fn aa_to_grid_roundtrip() {
        let grid = aa_to_grid(TEST_EGG);
        let result = grid_to_aa(&grid);
        assert_eq!(TEST_EGG, result);
    }

    #[test]
    fn normal_condition_unchanged() {
        let result = apply_expression(TEST_EGG, &crate::pet::render::Condition::Normal);
        assert_eq!(TEST_EGG, result);
    }

    #[test]
    fn tired_changes_aa() {
        let result = apply_expression(TEST_EGG, &crate::pet::render::Condition::Tired);
        assert_ne!(TEST_EGG, result);
    }

    #[test]
    fn exhausted_changes_aa() {
        let result = apply_expression(TEST_EGG, &crate::pet::render::Condition::Exhausted);
        assert_ne!(TEST_EGG, result);
    }

    #[test]
    fn finds_interior_features() {
        let grid = aa_to_grid(TEST_EGG);
        let features = find_interior_features(&grid);
        assert!(!features.is_empty(), "内部特徴が見つからない");
    }

    #[test]
    fn print_expression_variants() {
        println!("=== Normal ===");
        print!(
            "{}",
            apply_expression(TEST_EGG, &crate::pet::render::Condition::Normal)
        );
        println!("=== Tired ===");
        print!(
            "{}",
            apply_expression(TEST_EGG, &crate::pet::render::Condition::Tired)
        );
        println!("=== Exhausted ===");
        print!(
            "{}",
            apply_expression(TEST_EGG, &crate::pet::render::Condition::Exhausted)
        );

        let slime = "\
\n   ▄▄▀▀▀▀▀▀▄▄\
\n ▄ ▄████████▄ ▄\
\n▄▀████████████▀▄\
\n████ ██████ ████\
\n███████▀▀███████\
\n▀██████████████▀\
\n ▀▀██████████▀▀\n";

        println!("=== Slime Normal ===");
        print!(
            "{}",
            apply_expression(slime, &crate::pet::render::Condition::Normal)
        );
        println!("=== Slime Tired ===");
        print!(
            "{}",
            apply_expression(slime, &crate::pet::render::Condition::Tired)
        );
        println!("=== Slime Exhausted ===");
        print!(
            "{}",
            apply_expression(slime, &crate::pet::render::Condition::Exhausted)
        );
    }
}
