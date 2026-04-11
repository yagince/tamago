use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::ExecutableCommand;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::StreamExt;
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Gauge, Padding, Paragraph};

use chrono::Timelike;
use unicode_width::UnicodeWidthStr;

use crate::llm;
use crate::pet::{Category, PetState, Stage};
use crate::storage::{ActivityRecord, Storage};

const TICK_MS: u64 = 120;
const MESSAGE_DISPLAY_FRAMES: u64 = 83; // ~10秒 (120ms/frame)

fn frame_hash(frame: u64, salt: u64) -> u64 {
    frame.wrapping_add(salt).wrapping_mul(2654435761) >> 16
}

pub async fn run(storage: &Storage, message_interval_secs: u64) {
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

    if let Err(e) = run_tui(storage, &pet, message_interval_secs).await {
        eprintln!("TUI エラー: {e}");
    }
}

const RELOAD_INTERVAL: Duration = Duration::from_secs(5);

fn reload_pet_sync(storage: &Storage) -> Option<(PetState, bool)> {
    let _lock = storage.lock().ok()?;
    let mut pet = storage.load_pet().ok()?;

    let activities = storage.read_and_clear_activities().ok()?;
    if activities.is_empty() {
        return Some((pet, false));
    }

    let now = chrono::Utc::now();
    pet.apply_decay(now);
    let old_stage = pet.stage.clone();
    let old_level = pet.level();
    pet.apply_activities(&activities);
    while pet.try_evolve() {}
    let evolved = pet.stage != old_stage;
    if evolved {
        pet.evolved_at = Some(now);
    }
    let new_level = pet.level();
    let needs_personality = if new_level > old_level {
        pet.apply_level_up_stats(new_level - old_level);
        PetState::should_regenerate_personality(old_level, new_level, evolved)
    } else {
        false
    };
    storage.save_pet(&pet).ok()?;

    Some((pet, needs_personality))
}

fn build_aa(pet: &PetState) -> (Vec<String>, i16, i16) {
    let aa =
        crate::pet::render::ascii_art(&pet.stage, &pet.archetype, &pet.name, pet.hunger, pet.mood);
    let lines: Vec<String> = aa
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    let h = lines.len() as i16;
    let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as i16;
    (lines, w, h)
}

async fn run_tui(
    storage: &Storage,
    initial_pet: &PetState,
    message_interval_secs: u64,
) -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut pet = initial_pet.clone();
    let (mut aa_lines, aa_w, aa_h) = build_aa(&pet);

    let mut state = AnimState::new(aa_w, aa_h, pet.hunger, pet.mood);

    // ローカル LLM エンジン
    let model_dir = storage.model_dir();
    let llm_engine: Option<Arc<Mutex<llm::LlmEngine>>> = llm::LlmEngine::load(
        &llm::model_path(&model_dir),
        &llm::tokenizer_path(&model_dir),
    )
    .ok()
    .map(|e| Arc::new(Mutex::new(e)));

    let (llm_tx, mut llm_rx) = tokio::sync::mpsc::channel::<String>(8);

    // reload 前に peek した直近 activity を保持（read_and_clear で消える前に）
    let mut recent_activities: Vec<ActivityRecord> = storage.peek_latest_activities(5);

    let mut events = EventStream::new();
    let mut tick_interval = tokio::time::interval(Duration::from_millis(TICK_MS));
    let mut reload_interval = tokio::time::interval(RELOAD_INTERVAL);
    let mut message_interval = tokio::time::interval(Duration::from_secs(message_interval_secs));
    // 初回即発火させるため最初の tick を消費
    tick_interval.tick().await;
    reload_interval.tick().await;
    message_interval.tick().await;

    let mut aa_inner_size = (40u16, 16u16);

    // 初回メッセージ生成
    {
        let activity = recent_activities.first();
        let msg = generate_message(activity, &pet, state.frame);
        state.message = Some(msg);
        state.message_timer = MESSAGE_DISPLAY_FRAMES;

        if llm_engine.is_some() && !recent_activities.is_empty() {
            let cmds: Vec<&str> = recent_activities.iter().map(|r| r.cmd.as_str()).collect();
            spawn_llm_message(&llm_engine, &pet, &cmds, llm_tx.clone());
        }
    }

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                // animation tick
                if state.is_evolving() {
                    state.tick_evolution();
                } else {
                    state.tick(aa_inner_size.0, aa_inner_size.1);
                }

                // draw
                let current_aa = state.current_aa(&aa_lines);
                if state.is_evolving() {
                    terminal.draw(|f| {
                        draw_evolution(f, &state);
                    })?;
                } else {
                    terminal.draw(|f| {
                        aa_inner_size = draw(f, &pet, &current_aa, &state);
                    })?;
                }
            }
            _ = reload_interval.tick() => {
                // reload 前に直近 activity を保持
                let peeked = storage.peek_latest_activities(5);
                if !peeked.is_empty() {
                    recent_activities = peeked;
                }

                let storage_clone = storage.clone();
                if let Ok(Some((mut new_pet, needs_personality))) =
                    tokio::task::spawn_blocking(move || reload_pet_sync(&storage_clone)).await
                {
                    if needs_personality {
                        if let Some(ref engine) = llm_engine {
                            if let Ok(mut eng) = engine.lock() {
                                new_pet.personality =
                                    new_pet.generate_personality(Some(&mut eng));
                            }
                        } else {
                            new_pet.personality = new_pet.generate_personality(None);
                        }
                        let _ = storage.save_pet(&new_pet);
                    }
                    let evolved = new_pet.stage != pet.stage;
                    let aa_changed =
                        new_pet.hunger != pet.hunger || new_pet.mood != pet.mood || evolved;

                    if evolved {
                        let old_stage = pet.stage.clone();
                        state.start_evolution(aa_lines.clone(), &old_stage, &new_pet.stage);
                    }

                    pet = new_pet;
                    state.hunger = pet.hunger;
                    state.mood = pet.mood;
                    if aa_changed {
                        let (new_lines, new_w, new_h) = build_aa(&pet);
                        aa_lines = new_lines;
                        state.aa_w = new_w;
                        state.aa_h = new_h;
                    }
                }
            }
            _ = message_interval.tick() => {
                let activity = recent_activities.first();
                let msg = generate_message(activity, &pet, state.frame);
                state.message = Some(msg);
                state.message_timer = MESSAGE_DISPLAY_FRAMES;

                if llm_engine.is_some() && !recent_activities.is_empty() {
                    let cmds: Vec<&str> = recent_activities.iter().map(|r| r.cmd.as_str()).collect();
                    spawn_llm_message(&llm_engine, &pet, &cmds, llm_tx.clone());
                }
            }
            Some(msg) = llm_rx.recv() => {
                state.message = Some(msg);
                state.message_timer = MESSAGE_DISPLAY_FRAMES;
            }
            Some(Ok(event)) = events.next() => {
                if let Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
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

#[derive(Clone, Copy, PartialEq)]
enum EvolutionPhase {
    Flash,   // 0-5 frame: 画面フラッシュ
    Sparkle, // 6-17 frame: キラキラ + 旧AA
    Reveal,  // 18-33 frame: 新AA + テキスト
    FadeOut, // 34-41 frame: テキスト消え
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
    hunger: u8,
    mood: u8,
    sparkles: Vec<(i16, i16, char)>,
    // セリフ
    message: Option<String>,
    message_timer: u64,
    // 進化演出
    evolution: Option<EvolutionPhase>,
    evolution_timer: u64,
    old_aa: Vec<String>,
    old_stage_name: String,
    new_stage_name: String,
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
            message: None,
            message_timer: 0,
            evolution: None,
            evolution_timer: 0,
            old_aa: Vec::new(),
            old_stage_name: String::new(),
            new_stage_name: String::new(),
        }
    }

    /// 描画領域の実サイズから移動可能範囲を更新して 1 フレーム進める
    fn tick(&mut self, area_w: u16, area_h: u16) {
        self.frame += 1;
        let max_x = (area_w as i16 - self.aa_w).max(0);
        let max_y = (area_h as i16 - self.aa_h).max(0);

        // フェーズ切り替え
        self.phase_timer = self.phase_timer.saturating_sub(1);
        if self.phase_timer == 0 {
            match self.phase {
                MovePhase::Walk => {
                    self.phase = MovePhase::Idle;
                    self.phase_timer = 8 + self.hash(7) % 12;
                }
                MovePhase::Idle => {
                    self.phase = MovePhase::Walk;
                    self.phase_timer = 20 + self.hash(13) % 30;
                    self.pick_direction(max_x, max_y);
                }
            }
        }

        if self.phase == MovePhase::Walk {
            self.x += self.dx;
            self.y += self.dy;

            // 壁で反射 + 新しい方向を選び直す
            let mut bounced = false;
            if self.x <= 0 {
                self.x = 0;
                bounced = true;
            }
            if self.x >= max_x {
                self.x = max_x;
                bounced = true;
            }
            if self.y <= 0 {
                self.y = 0;
                bounced = true;
            }
            if self.y >= max_y {
                self.y = max_y;
                bounced = true;
            }
            if bounced {
                self.pick_direction(max_x, max_y);
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
                self.blink_next = 30 + self.hash(19) % 20;
            }
        }

        self.update_sparkles();

        // セリフタイマー
        if self.message_timer > 0 {
            self.message_timer -= 1;
            if self.message_timer == 0 {
                self.message = None;
            }
        }
    }

    fn start_evolution(&mut self, old_aa: Vec<String>, old_stage: &Stage, new_stage: &Stage) {
        self.old_aa = old_aa;
        self.old_stage_name = format!("{:?}", old_stage);
        self.new_stage_name = format!("{:?}", new_stage);
        self.evolution = Some(EvolutionPhase::Flash);
        self.evolution_timer = 0;
    }

    fn tick_evolution(&mut self) {
        if self.evolution.is_none() {
            return;
        }
        self.evolution_timer += 1;
        self.evolution = match self.evolution_timer {
            0..=5 => Some(EvolutionPhase::Flash),
            6..=17 => Some(EvolutionPhase::Sparkle),
            18..=33 => Some(EvolutionPhase::Reveal),
            34..=41 => Some(EvolutionPhase::FadeOut),
            _ => None,
        };
    }

    fn is_evolving(&self) -> bool {
        self.evolution.is_some()
    }

    fn hash(&self, salt: u64) -> u64 {
        frame_hash(self.frame, salt)
    }

    /// 現在位置から目標地点への方向を設定。壁際にいるときは反対方向を優先。
    fn pick_direction(&mut self, max_x: i16, max_y: i16) {
        let mid_x = max_x / 2;
        let mid_y = max_y / 2;

        // 8パターンの移動方向
        let dirs: [(i16, i16); 8] = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (-1, 1),
            (1, -1),
            (-1, -1),
        ];

        let mut weights = [0u64; 8];
        for (i, &(dx, dy)) in dirs.iter().enumerate() {
            let mut w = 10u64;
            if self.x <= 1 && dx > 0 {
                w += 20;
            }
            if self.x >= max_x - 1 && dx < 0 {
                w += 20;
            }
            if self.y <= 1 && dy > 0 {
                w += 20;
            }
            if self.y >= max_y - 1 && dy < 0 {
                w += 20;
            }
            if (self.x < mid_x && dx > 0) || (self.x > mid_x && dx < 0) {
                w += 5;
            }
            if (self.y < mid_y && dy > 0) || (self.y > mid_y && dy < 0) {
                w += 5;
            }
            weights[i] = w;
        }

        let total: u64 = weights.iter().sum();
        let mut pick = self.hash(31) % total;
        let mut chosen = 0;
        for (i, &w) in weights.iter().enumerate() {
            if pick < w {
                chosen = i;
                break;
            }
            pick -= w;
        }

        let (dx, dy) = dirs[chosen];
        self.dx = dx;
        self.dy = dy;
    }

    fn update_sparkles(&mut self) {
        self.sparkles.clear();

        // 約5秒に1回だけ表示（チカチカ防止）
        if self.hash(53) % 42 != 0 {
            return;
        }

        let min_stat = self.hunger.min(self.mood);
        let decos = if min_stat < 30 {
            SAD_DECOS
        } else if min_stat > 80 {
            HAPPY_DECOS
        } else {
            SPARKLES
        };

        let count = if min_stat > 80 {
            3
        } else if min_stat > 50 {
            2
        } else if min_stat > 30 {
            1
        } else {
            0
        };

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
            // 全行を同じ幅にパディングしてから反転（形が崩れないように）
            let max_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
            lines = lines
                .iter()
                .map(|l| {
                    let pad = max_w - l.chars().count();
                    let padded = format!("{}{}", l, " ".repeat(pad));
                    flip_line(&padded)
                })
                .collect();
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

// ── Speech ───────────────────────────────────────────────────

fn pick_message(candidates: &[&str], frame: u64) -> String {
    let idx = frame_hash(frame, 37) as usize % candidates.len();
    candidates[idx].to_string()
}

fn generate_message(activity: Option<&ActivityRecord>, pet: &PetState, frame: u64) -> String {
    if let Some(record) = activity {
        let candidates: &[&str] = match record.cat {
            Category::Git => &[
                "おつかれ！",
                "またひとつ積み上げたね",
                "えいっ！送信！",
                "git 使いこなしてるね",
                "ふむふむ",
            ],
            Category::Ai => &["ふむふむ...", "なるほどね", "考え中...", "仲間が来た！"],
            Category::Dev => &[
                "ガチャガチャ...できた？",
                "ドキドキ",
                "Rust だね！",
                "かっこいい",
            ],
            Category::Infra => &["コンテナの中は広いなぁ", "☁", "デプロイ！"],
            Category::Editor => &["書いてる書いてる", "集中してるね"],
            Category::Basic => &["ふーん", "...", "♪"],
            Category::Other => &["ふーん", "なにしてるの？", "♪"],
        };
        return pick_message(candidates, frame);
    }

    if pet.hunger < 20 {
        return pick_message(&["おなかすいた...", "なにか食べたい...", "ぐぅ..."], frame);
    }
    if pet.mood < 20 {
        return pick_message(&["さみしい...", "かまって...", "しょんぼり"], frame);
    }

    let hour = chrono::Local::now().hour();
    if hour < 5 {
        return pick_message(&["まだ起きてるの？", "zzz...", "ねむい..."], frame);
    }

    pick_message(&["...", "♪", "〜", "ふふっ", "いい天気だね"], frame)
}

fn spawn_llm_message(
    engine: &Option<Arc<Mutex<llm::LlmEngine>>>,
    pet: &PetState,
    cmds: &[&str],
    tx: tokio::sync::mpsc::Sender<String>,
) {
    let Some(engine) = engine.clone() else {
        return;
    };
    let cmd_list = cmds.join(", ");
    let prompt = format!("ユーザーの直近のコマンド: {cmd_list}。20文字以内で一言リアクション。");

    let personality_hint = if pet.personality.is_empty() {
        String::new()
    } else {
        format!("性格: {}。", pet.personality)
    };

    let system = format!(
        "あなたは「{name}」という名前のLv.{lv}のターミナルペットです。\
        {personality_hint}\
        ステータス: HP{hp} MP{mp} 開発力{dev} 賢さ{wis} おもしろさ{hum} カオスさ{cha}。\
        求められたセリフだけを出力してください。説明や補足は不要です。",
        name = pet.name,
        lv = pet.level(),
        hp = pet.hunger,
        mp = pet.mood,
        dev = pet.dev_power,
        wis = pet.wisdom,
        hum = pet.humor,
        cha = pet.chaos,
    );

    tokio::task::spawn_blocking(move || {
        if let Ok(mut eng) = engine.lock() {
            if let Some(msg) = eng.generate(&prompt, &system, 30) {
                let _ = tx.blocking_send(msg);
            }
        }
    });
}

// ── Evolution cutscene ───────────────────────────────────────

const RAINBOW: &[Color] = &[
    Color::LightRed,
    Color::LightYellow,
    Color::LightGreen,
    Color::LightCyan,
    Color::LightBlue,
    Color::LightMagenta,
];
const EVO_SPARKLES: &[char] = &['★', '✦', '✧', '☆', '⋆', '✦', '♦', '◆'];

fn draw_evolution(f: &mut Frame, state: &AnimState) {
    let area = f.area();
    let phase = match state.evolution {
        Some(p) => p,
        None => return,
    };

    match phase {
        EvolutionPhase::Flash => {
            let brightness = if state.evolution_timer < 3 {
                Color::White
            } else {
                Color::DarkGray
            };
            let block = Block::default().style(Style::default().bg(brightness));
            f.render_widget(block, area);
        }
        EvolutionPhase::Sparkle => {
            let color = RAINBOW[(state.evolution_timer as usize) % RAINBOW.len()];
            let cx = area.width / 2;
            let cy = area.height / 2;

            for (i, line) in state.old_aa.iter().enumerate() {
                let y = cy.saturating_sub(state.old_aa.len() as u16 / 2) + i as u16;
                let x = cx.saturating_sub(line.chars().count() as u16 / 2);
                if y < area.height {
                    f.render_widget(
                        Paragraph::new(line.as_str()).style(Style::default().fg(color)),
                        Rect::new(x, y, line.chars().count() as u16, 1),
                    );
                }
            }

            let seed = state.evolution_timer.wrapping_mul(6364136223846793005);
            for i in 0..12 {
                let s = seed.wrapping_add(i * 2971215073);
                let sx = (s % area.width as u64) as u16;
                let sy = (s.wrapping_mul(31) % area.height as u64) as u16;
                let ch = EVO_SPARKLES[(s / 7 % EVO_SPARKLES.len() as u64) as usize];
                let spark_color = RAINBOW[(s as usize / 3) % RAINBOW.len()];
                f.render_widget(
                    Paragraph::new(ch.to_string()).style(Style::default().fg(spark_color)),
                    Rect::new(sx, sy, 1, 1),
                );
            }
        }
        EvolutionPhase::Reveal => {
            let cy = area.height / 2;

            let msg1 = "✨ しんかした！ ✨";
            let msg2 = format!("{} → {}", state.old_stage_name, state.new_stage_name);

            let color = RAINBOW[(state.evolution_timer as usize) % RAINBOW.len()];

            f.render_widget(
                Paragraph::new(msg1)
                    .style(Style::default().fg(color).bold())
                    .alignment(Alignment::Center),
                Rect::new(0, cy.saturating_sub(1), area.width, 1),
            );
            f.render_widget(
                Paragraph::new(msg2.as_str())
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Center),
                Rect::new(0, cy + 1, area.width, 1),
            );

            let seed = state.evolution_timer.wrapping_mul(2654435761);
            for i in 0..6 {
                let s = seed.wrapping_add(i * 104729);
                let sx = (s % area.width as u64) as u16;
                let sy = (s.wrapping_mul(17) % area.height as u64) as u16;
                let ch = EVO_SPARKLES[(s / 11 % EVO_SPARKLES.len() as u64) as usize];
                f.render_widget(
                    Paragraph::new(ch.to_string()).style(Style::default().fg(Color::Yellow)),
                    Rect::new(sx, sy, 1, 1),
                );
            }
        }
        EvolutionPhase::FadeOut => {
            let cy = area.height / 2;
            let fade = if state.evolution_timer < 38 {
                Color::Gray
            } else {
                Color::DarkGray
            };

            let msg = "✨ しんかした！ ✨";
            f.render_widget(
                Paragraph::new(msg)
                    .style(Style::default().fg(fade))
                    .alignment(Alignment::Center),
                Rect::new(0, cy, area.width, 1),
            );
        }
    }
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

fn draw(f: &mut Frame, pet: &PetState, aa_lines: &[String], state: &AnimState) -> (u16, u16) {
    let size = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(9)])
        .split(size);

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Length(1),
            Constraint::Percentage(45),
        ])
        .split(outer[0]);

    let aa_inner = draw_aa_area(f, main[0], aa_lines, state, pet);
    draw_status(f, main[2], pet);
    draw_category_bars(f, outer[1], pet);
    aa_inner
}

fn draw_aa_area(
    f: &mut Frame,
    area: Rect,
    aa_lines: &[String],
    state: &AnimState,
    pet: &PetState,
) -> (u16, u16) {
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

    // 吹き出し
    if let Some(ref msg) = state.message {
        let inner_w = UnicodeWidthStr::width(msg.as_str());
        let bubble_w = inner_w + 4;
        let pet_center_x = inner.x + state.x as u16 + state.aa_w as u16 / 2;
        let bx = pet_center_x.saturating_sub(bubble_w as u16 / 2);
        let bx = bx.min((inner.x + inner.width).saturating_sub(bubble_w as u16));
        let by = (inner.y + state.y as u16).saturating_sub(3);

        if by >= inner.y {
            let bar = "─".repeat(inner_w + 2);
            let top = format!("╭{bar}╮");
            let mid = format!("│ {msg} │");
            let bot = format!("╰{bar}╯");

            for (i, line) in [top, mid, bot].iter().enumerate() {
                let ly = by + i as u16;
                if ly < inner.y + inner.height {
                    f.render_widget(
                        Paragraph::new(line.as_str()).style(Style::default().fg(Color::White)),
                        Rect::new(bx, ly, bubble_w as u16, 1),
                    );
                }
            }
        }
    }

    (inner.width, inner.height)
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

    let stat_max = [pet.dev_power, pet.wisdom, pet.humor, pet.chaos]
        .iter()
        .copied()
        .max()
        .unwrap_or(1)
        .max(1) as usize;

    let lines = vec![
        Line::from(vec![Span::styled(
            &pet.name,
            Style::default().fg(Color::White).bold(),
        )]),
        Line::from(vec![Span::styled(
            format!("[{creature}]"),
            Style::default().fg(Color::LightCyan),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Lv. ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{lv}"),
                Style::default().fg(Color::LightYellow).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("EXP ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", pet.exp),
                Style::default().fg(Color::LightCyan).bold(),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);

    let stats_y = inner.y + 6;
    let stat_rows: &[(&str, usize, usize, Color, Color, Color)] = &[
        (
            "HP",
            pet.hunger as usize,
            100,
            Color::Green,
            Color::Yellow,
            Color::Red,
        ),
        (
            "MP",
            pet.mood as usize,
            100,
            Color::Blue,
            Color::Magenta,
            Color::Red,
        ),
        (
            "開発",
            pet.dev_power as usize,
            stat_max,
            Color::Yellow,
            Color::Yellow,
            Color::Yellow,
        ),
        (
            "賢さ",
            pet.wisdom as usize,
            stat_max,
            Color::Cyan,
            Color::Cyan,
            Color::Cyan,
        ),
        (
            "笑い",
            pet.humor as usize,
            stat_max,
            Color::Green,
            Color::Green,
            Color::Green,
        ),
        (
            "混沌",
            pet.chaos as usize,
            stat_max,
            Color::Magenta,
            Color::Magenta,
            Color::Magenta,
        ),
    ];

    for (i, (label, val, max, high, mid, low)) in stat_rows.iter().enumerate() {
        let y = stats_y + i as u16;
        if y >= inner.y + inner.height {
            break;
        }
        let label_area = Rect::new(inner.x, y, 6, 1);
        f.render_widget(
            Paragraph::new(*label).style(Style::default().fg(*high).bold()),
            label_area,
        );
        let bar_area = Rect::new(inner.x + 6, y, inner.width.saturating_sub(12), 1);
        let ratio = if *max > 0 {
            *val as f64 / *max as f64
        } else {
            0.0
        };
        let pct = (*val * 100).checked_div(*max).unwrap_or(0);
        let color = if pct > 50 {
            *high
        } else if pct > 25 {
            *mid
        } else {
            *low
        };
        f.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
                .ratio(ratio.min(1.0))
                .label(""),
            bar_area,
        );
        let val_area = Rect::new(inner.x + inner.width - 5, y, 5, 1);
        f.render_widget(
            Paragraph::new(format!("{:>4}", val)).alignment(Alignment::Right),
            val_area,
        );
    }

    let personality_y = stats_y + stat_rows.len() as u16 + 1;
    if !pet.personality.is_empty() && personality_y < inner.y + inner.height {
        f.render_widget(
            Paragraph::new(format!("💬 {}", pet.personality))
                .style(Style::default().fg(Color::White).italic()),
            Rect::new(inner.x, personality_y, inner.width, 1),
        );
    }
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
        let ratio = if max > 0 {
            val as f64 / max as f64
        } else {
            0.0
        };

        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(7),
                Constraint::Min(10),
                Constraint::Length(8),
            ])
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
