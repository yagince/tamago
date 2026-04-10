use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::{BorderType, Block, Borders, Gauge, Padding, Paragraph};

use crate::pet::{Category, PetState, Stage};
use crate::storage::Storage;

const TICK_MS: u64 = 120;

pub fn run(storage: &Storage) {
    let _lock = storage.lock().expect("ロックの取得に失敗しました");
    let mut pet = storage
        .load_pet()
        .expect("ペットが見つかりません。tamago init を実行してください。");

    let activities = storage
        .read_and_clear_activities()
        .expect("activity の読み込みに失敗しました");

    let now = chrono::Utc::now();
    pet.apply_decay(now);
    let old_stage = pet.stage.clone();
    pet.apply_activities(&activities);
    while pet.try_evolve() {}

    let evolved = pet.stage != old_stage;
    if evolved {
        pet.evolved_at = Some(now);
    }
    storage
        .save_pet(&pet)
        .expect("pet.json の保存に失敗しました");

    drop(_lock);

    if let Err(e) = run_tui(&pet) {
        eprintln!("TUI エラー: {e}");
    }
}

fn run_tui(pet: &PetState) -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let base_aa = crate::pet::render::ascii_art(
        &pet.stage,
        &pet.archetype,
        &pet.name,
        pet.hunger,
        pet.mood,
    );
    let aa_lines: Vec<String> = base_aa.lines().filter(|l| !l.is_empty()).map(String::from).collect();
    let aa_h = aa_lines.len() as i16;
    let aa_w = aa_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as i16;

    let mut state = AnimState::new(aa_w, aa_h, pet.hunger, pet.mood);
    let mut last_tick = Instant::now();

    loop {
        let current_aa = state.current_aa(&aa_lines);
        terminal.draw(|f| draw(f, pet, &current_aa, &state))?;

        let timeout = Duration::from_millis(TICK_MS).saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= Duration::from_millis(TICK_MS) {
            state.tick();
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

// ── Animation state ──────────────────────────────────────────

const SPARKLES: &[char] = &['✦', '✧', '☆', '·', '*', '˚', '°', '⋆'];
const HAPPY_DECOS: &[char] = &['♡', '♪', '✦', '☆', '✧', '·'];
const SAD_DECOS: &[char] = &['·', '.', ' ', ' ', ' ', ' '];

#[derive(Clone, Copy, PartialEq)]
enum MovePhase {
    Walk,
    Idle,
}

struct AnimState {
    x: i16,
    y: i16,
    dx: i16,
    dy: i16,
    aa_w: i16,
    aa_h: i16,
    facing_right: bool,
    frame: u64,
    // まばたき
    blink_next: u64,
    blink_remaining: u64,
    // 移動パターン
    phase: MovePhase,
    phase_timer: u64,
    // ステータス
    hunger: u8,
    mood: u8,
    // スパークル
    sparkles: Vec<(i16, i16, char)>,
}

impl AnimState {
    fn new(aa_w: i16, aa_h: i16, hunger: u8, mood: u8) -> Self {
        Self {
            x: 2,
            y: 1,
            dx: 1,
            dy: 0,
            aa_w,
            aa_h,
            facing_right: true,
            frame: 0,
            blink_next: 25,
            blink_remaining: 0,
            phase: MovePhase::Walk,
            phase_timer: 30,
            hunger,
            mood,
            sparkles: Vec::new(),
        }
    }

    fn tick(&mut self) {
        self.frame += 1;

        // フェーズ切り替え
        self.phase_timer = self.phase_timer.saturating_sub(1);
        if self.phase_timer == 0 {
            match self.phase {
                MovePhase::Walk => {
                    self.phase = MovePhase::Idle;
                    self.phase_timer = 8 + (self.frame % 12); // 8-19フレーム休憩
                    self.dx = 0;
                    self.dy = 0;
                }
                MovePhase::Idle => {
                    self.phase = MovePhase::Walk;
                    self.phase_timer = 15 + (self.frame % 25); // 15-39フレーム歩行
                    self.pick_direction();
                }
            }
        }

        // 歩行中のみ移動
        if self.phase == MovePhase::Walk {
            // ホップ: 3フレーム周期で y を ±1
            let hop = if self.frame % 6 < 3 { 0 } else { 1 };
            let base_y = self.y - if self.frame.wrapping_sub(1) % 6 < 3 { 0 } else { 1 };
            self.x += self.dx;
            self.y = base_y + hop;

            // 壁で反射
            let max_x = 40_i16.saturating_sub(self.aa_w).max(0);
            let max_y = 16_i16.saturating_sub(self.aa_h).max(0);
            if self.x <= 0 {
                self.x = 0;
                self.dx = self.dx.abs().max(1);
            }
            if self.x >= max_x {
                self.x = max_x;
                self.dx = -self.dx.abs().min(-1);
            }
            if self.y <= 0 {
                self.y = 0;
                self.dy = self.dy.abs();
            }
            if self.y >= max_y {
                self.y = max_y;
                self.dy = -self.dy.abs();
            }

            if self.dx > 0 {
                self.facing_right = true;
            } else if self.dx < 0 {
                self.facing_right = false;
            }
        }

        // まばたき
        if self.blink_remaining > 0 {
            self.blink_remaining -= 1;
        } else {
            self.blink_next = self.blink_next.saturating_sub(1);
            if self.blink_next == 0 {
                self.blink_remaining = 3;
                self.blink_next = 30 + (self.frame % 20); // 30-49フレーム後に次のまばたき
            }
        }

        // スパークル更新
        self.update_sparkles();
    }

    fn pick_direction(&mut self) {
        let seed = self.frame.wrapping_mul(2654435761);
        match seed % 8 {
            0 => { self.dx = 1; self.dy = 0; }
            1 => { self.dx = -1; self.dy = 0; }
            2 => { self.dx = 0; self.dy = 1; }
            3 => { self.dx = 0; self.dy = -1; }
            4 => { self.dx = 1; self.dy = 1; }  // 斜め
            5 => { self.dx = -1; self.dy = 1; }
            6 => { self.dx = 1; self.dy = -1; }
            _ => { self.dx = -1; self.dy = -1; }
        }
    }

    fn update_sparkles(&mut self) {
        let min_stat = self.hunger.min(self.mood);
        let decos = if min_stat < 30 {
            SAD_DECOS
        } else if min_stat > 80 {
            HAPPY_DECOS
        } else {
            SPARKLES
        };

        let count = if min_stat > 80 { 3 } else if min_stat > 50 { 2 } else if min_stat > 30 { 1 } else { 0 };

        self.sparkles.clear();
        let seed = self.frame.wrapping_mul(6364136223846793005);
        for i in 0..count {
            let s = seed.wrapping_add(i as u64 * 2971215073);
            let sx = (s % (self.aa_w as u64 + 6)) as i16 - 3 + self.x;
            let sy = (s.wrapping_mul(31) % (self.aa_h as u64 + 2)) as i16 - 1 + self.y;
            let ch = decos[(s / 7 % decos.len() as u64) as usize];
            if ch != ' ' {
                self.sparkles.push((sx, sy, ch));
            }
        }
    }

    fn is_blinking(&self) -> bool {
        self.blink_remaining > 0
    }

    /// 現在のフレーム用の AA 行を生成（まばたき + 反転）
    fn current_aa(&self, base_lines: &[String]) -> Vec<String> {
        let mut lines: Vec<String> = if self.is_blinking() {
            apply_blink(base_lines)
        } else {
            base_lines.to_vec()
        };

        if !self.facing_right {
            lines = lines.iter().map(|l| flip_line(l)).collect();
        }

        lines
    }
}

// ── Blink ────────────────────────────────────────────────────

fn apply_blink(lines: &[String]) -> Vec<String> {
    let mut result: Vec<String> = Vec::with_capacity(lines.len());
    let mut blinked = false;

    for line in lines {
        if !blinked && line.contains('▄') {
            let chars: Vec<char> = line.chars().collect();
            let first_block = chars.iter().position(|&c| matches!(c, '█' | '▀' | '▄'));
            let last_block = chars.iter().rposition(|&c| matches!(c, '█' | '▀' | '▄'));

            if let (Some(first), Some(last)) = (first_block, last_block) {
                if last - first > 4 {
                    let mut new_chars = chars.clone();
                    for i in (first + 1)..last {
                        if new_chars[i] == '▄' {
                            new_chars[i] = ' ';
                        }
                    }
                    result.push(new_chars.into_iter().collect());
                    blinked = true;
                    continue;
                }
            }
        }
        result.push(line.clone());
    }
    result
}

// ── AA flip ──────────────────────────────────────────────────

fn flip_line(line: &str) -> String {
    line.chars()
        .rev()
        .map(|ch| match ch {
            '▌' => '▐',
            '▐' => '▌',
            other => other,
        })
        .collect()
}

// ── Draw ─────────────────────────────────────────────────────

fn draw(f: &mut Frame, pet: &PetState, aa_lines: &[String], state: &AnimState) {
    let size = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(9)])
        .split(size);

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Length(1), Constraint::Percentage(45)])
        .split(outer[0]);

    draw_aa_area(f, main[0], aa_lines, state, pet);
    draw_status(f, main[2], pet);
    draw_category_bars(f, outer[1], pet);
}

fn draw_aa_area(f: &mut Frame, area: Rect, aa_lines: &[String], state: &AnimState, pet: &PetState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::White))
        .title(format!(" {} ", pet.emoji()))
        .title_alignment(Alignment::Center);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let color = aa_color(&pet.stage);

    // AA 描画
    for (row_idx, line) in aa_lines.iter().enumerate() {
        let y = inner.y + state.y as u16 + row_idx as u16;
        let x = inner.x + state.x as u16;
        if y >= inner.y + inner.height {
            continue;
        }

        let mut col = 0u16;
        for ch in line.chars() {
            if x + col >= inner.x + inner.width {
                break;
            }
            let style = match ch {
                '▀' | '▄' | '█' => Style::default().fg(color),
                ' ' => Style::default(),
                _ => Style::default().fg(Color::Yellow),
            };
            f.render_widget(
                Paragraph::new(ch.to_string()).style(style),
                Rect::new(x + col, y, 1, 1),
            );
            col += 1;
        }
    }

    // スパークル描画
    for &(sx, sy, ch) in &state.sparkles {
        let px = inner.x as i16 + sx;
        let py = inner.y as i16 + sy;
        if px >= inner.x as i16
            && px < (inner.x + inner.width) as i16
            && py >= inner.y as i16
            && py < (inner.y + inner.height) as i16
        {
            f.render_widget(
                Paragraph::new(ch.to_string()).style(Style::default().fg(Color::Yellow)),
                Rect::new(px as u16, py as u16, 1, 1),
            );
        }
    }

    // 休憩中は Zzz 表示
    if state.phase == MovePhase::Idle && state.frame % 8 < 4 {
        let zx = inner.x + state.x as u16 + state.aa_w as u16 + 1;
        let zy = inner.y + state.y as u16;
        if zx + 2 < inner.x + inner.width && zy < inner.y + inner.height {
            let z = if state.frame % 16 < 8 { "z" } else { "Z" };
            f.render_widget(
                Paragraph::new(z).style(Style::default().fg(Color::Cyan)),
                Rect::new(zx, zy, 1, 1),
            );
        }
    }
}

fn draw_status(f: &mut Frame, area: Rect, pet: &PetState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::White))
        .title(" STATUS ")
        .title_alignment(Alignment::Center)
        .padding(Padding::new(2, 2, 1, 1));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let creature = crate::pet::render::creature_type(&pet.stage, &pet.archetype, &pet.name);
    let lv = pet.level();

    let lines = vec![
        Line::from(vec![
            Span::styled(&pet.name, Style::default().fg(Color::White).bold()),
        ]),
        Line::from(vec![
            Span::styled(format!("[{creature}]"), Style::default().fg(Color::LightCyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Lv. ", Style::default().fg(Color::White)),
            Span::styled(format!("{lv}"), Style::default().fg(Color::LightYellow).bold()),
        ]),
        Line::from(vec![
            Span::styled("EXP ", Style::default().fg(Color::White)),
            Span::styled(format!("{}", pet.exp), Style::default().fg(Color::LightCyan).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("HP  ", Style::default().fg(Color::LightGreen).bold()),
            hp_bar(pet.hunger),
            Span::styled(format!(" {}", pet.hunger), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("MP  ", Style::default().fg(Color::LightBlue).bold()),
            mp_bar(pet.mood),
            Span::styled(format!(" {}", pet.mood), Style::default().fg(Color::White)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn hp_bar(val: u8) -> Span<'static> {
    let filled = (val as usize * 10 / 100).min(10);
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(10 - filled));
    let color = if val > 50 {
        Color::Green
    } else if val > 25 {
        Color::Yellow
    } else {
        Color::Red
    };
    Span::styled(bar, Style::default().fg(color))
}

fn mp_bar(val: u8) -> Span<'static> {
    let filled = (val as usize * 10 / 100).min(10);
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(10 - filled));
    let color = if val > 50 {
        Color::Blue
    } else if val > 25 {
        Color::Magenta
    } else {
        Color::Red
    };
    Span::styled(bar, Style::default().fg(color))
}

fn draw_category_bars(f: &mut Frame, area: Rect, pet: &PetState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::White))
        .title(" SKILLS ")
        .title_alignment(Alignment::Center)
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let cats: &[(Category, &str, Color)] = &[
        (Category::Git, "Git  ", Color::Cyan),
        (Category::Ai, "AI   ", Color::Magenta),
        (Category::Dev, "Dev  ", Color::Green),
        (Category::Infra, "Infra", Color::Blue),
        (Category::Editor, "Edit ", Color::Yellow),
        (Category::Basic, "Basic", Color::White),
        (Category::Other, "Other", Color::Gray),
    ];

    let max = pet.category_exp.values().copied().max().unwrap_or(0).max(1);

    let mut sorted: Vec<_> = cats.iter().collect();
    sorted.sort_by_key(|(cat, _, _)| std::cmp::Reverse(*pet.category_exp.get(cat).unwrap_or(&0)));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1); sorted.len()])
        .split(inner);

    for (i, (cat, label, color)) in sorted.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let val = *pet.category_exp.get(cat).unwrap_or(&0);
        let ratio = if max > 0 { val as f64 / max as f64 } else { 0.0 };

        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(7), Constraint::Min(10), Constraint::Length(8)])
            .split(rows[i]);

        f.render_widget(
            Paragraph::new(*label).style(Style::default().fg(*color)),
            row[0],
        );
        f.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(*color).bg(Color::DarkGray))
                .ratio(ratio)
                .label(""),
            row[1],
        );
        f.render_widget(
            Paragraph::new(format!("{val:>6}")).alignment(Alignment::Right),
            row[2],
        );
    }
}

fn aa_color(stage: &Stage) -> Color {
    match stage {
        Stage::Egg => Color::White,
        Stage::Baby => Color::LightGreen,
        Stage::Child => Color::LightCyan,
        Stage::Teen => Color::LightMagenta,
        Stage::Adult => Color::LightYellow,
    }
}
