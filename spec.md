# tamago 設計書

CLIで育てるターミナルペット。ターミナルの使用度と Claude Code の使用量に応じて成長し、育て方で進化が分岐する。

---

## コンセプト

- ターミナルを日常的に使うだけでペットが育つ
- Claude Code のヘビーユーザーほど AI 系に進化する
- Claude Code の statusline に常駐して「相棒感」を演出
- **常駐プロセスなし**。毎回起動→即終了の fire-and-forget 設計
- AA（ASCII アート）がこのツールの核心。halfblock でドット絵風に描画する

---

## 技術スタック

| 項目 | 選定 |
|------|------|
| 言語 | Rust 2024 edition |
| CLI パーサー | clap |
| データストア | JSON ファイル（pet.json + activity.jsonl） |
| シリアライズ | serde + serde_json |
| 排他制御 | flock（nix） |
| 時刻 | chrono |
| ホームディレクトリ | dirs |

### データストア選定理由

SQLite や組み込み KVS ではなく JSON ファイルを採用。

- **tick の速度最優先**: 毎コマンド実行で走るため、JSONL append（~1ms）が最速
- **スキーマ変更が容易**: `#[serde(default)]` でフィールド追加するだけ。マイグレーション不要
- **デバッグ容易**: `cat pet.json` / `jq . activity.jsonl` で中身を確認できる
- **依存最小**: serde_json のみ。C バインディング不要
- **データ量が小さい**: ペット状態は 1 オブジェクト、activity は数 KB 以内

---

## ディレクトリ構成

```
~/.config/tamago/
  pet.json          # ペット状態（1 オブジェクト）
  activity.jsonl    # 未集計のコマンドログ（集計ごとに空になる）
  .lock             # flock 用（pet.json 排他制御）
```

`$XDG_CONFIG_HOME/tamago` を優先。未設定なら `~/.config/tamago`。

---

## クレート構成

```
tamago/
├── Cargo.toml
├── src/
│   ├── main.rs                 # CLI エントリポイント
│   ├── cmd/
│   │   ├── mod.rs              # clap Subcommand 定義 + dispatcher
│   │   ├── init.rs             # 初期セットアップ
│   │   ├── tick.rs             # フックから呼ばれる（内部用）
│   │   ├── show.rs             # tamago デフォルト画面
│   │   ├── status.rs           # statusline 用ワンライナー
│   │   ├── name.rs             # 命名・改名（--ai でClaudeに考えさせる）
│   │   ├── reset.rs            # データリセット
│   │   └── hook.rs             # フックスクリプト出力
│   ├── pet/
│   │   ├── mod.rs              # PetState, Stage, Archetype, Category
│   │   ├── names.rs            # ランダム名生成 / Claude 命名
│   │   └── render/
│   │       ├── render.rs       # select_art, ascii_art, animated_art, colorize_aa, pet_color
│   │       ├── egg.rs          # 卵のバリエーション
│   │       ├── baby.rs         # 幼体のバリエーション
│   │       ├── child.rs        # 子体のバリエーション
│   │       ├── teen.rs         # 青年体のバリエーション
│   │       ├── adult.rs        # 成体 (archetype × variants)
│   │       ├── expression.rs   # 顔の表情を動的に変更（bitmap 往復）
│   │       └── animate.rs      # sparkle デコレーション + 微表情
│   ├── tracker/
│   │   ├── mod.rs
│   │   └── scoring.rs          # コマンド → exp、claude_turn_score
│   └── storage.rs              # JSON ファイル読み書き・flock
├── assets/
│   └── aa/nanobanana-output/   # AA 生成用リファレンス画像
└── tools/
    ├── img2aa.py               # PNG → halfblock AA 変換 (Python + PIL)
    └── img2aa.sh               # 同じく shell 版（ImageMagick + awk）
```

---

## データモデル

### PetState (pet.json)

```json
{
  "name": "ピカリン",
  "born_at": "2026-04-08T12:00:00Z",
  "stage": "baby",
  "exp": 12345,
  "hunger": 100,
  "mood": 100,
  "archetype": null,
  "category_exp": {
    "git": 60, "ai": 3525, "dev": 384,
    "infra": 0, "editor": 0, "basic": 0, "other": 82
  },
  "last_fed": "2026-04-08T12:00:00Z",
  "last_active": "2026-04-09T10:30:00Z",
  "evolved_at": null,
  "leveled_up_at": null,
  "last_output_tokens": 485200
}
```

| フィールド | 型 | 用途 |
|------------|-----|------|
| `stage` | enum | Egg / Baby / Child / Teen / Adult |
| `archetype` | Option<enum> | Teen→Adult 時に確定 |
| `category_exp` | HashMap | カテゴリ別累計 exp、進化分岐と show 表示用 |
| `evolved_at` | Option<DateTime> | 進化演出の開始時刻。10 秒間 AA 表示 |
| `leveled_up_at` | Option<DateTime> | レベルアップ演出の開始時刻 |
| `last_output_tokens` | u64 | Claude Code の累積 output_tokens。次回差分計算用 |

新しいフィールドは `#[serde(default)]` で後方互換。

### ActivityRecord (activity.jsonl)

```jsonl
{"cmd":"git commit -m fix","cat":"git","exp":20,"ts":"..."}
{"cmd":"--claude-turn","cat":"ai","exp":15,"ts":"..."}
```

`tick` が 1 行 append。`status` / `show` が **read_and_clear** で全件読んで truncate。
cursor 方式ではなく「集計後に全消し」なので activity.jsonl は常に直近未集計分のみ。

---

## CLI コマンド

```
tamago                  メイン画面（集計 + AA + ステータス + カテゴリバー）
tamago init             初期セットアップ（卵生成 + ガイド表示）
tamago name <名前>      命名・改名
tamago name --ai        Claude に名前を考えさせる
tamago reset            データリセット
tamago status           statusline 用ワンライナー（遅延集計あり）
tamago hook <shell>     zsh / bash / statusline 用フックを stdout に出力
tamago tick ...         内部用（フックから呼ばれる）
  --cmd "..."             コマンド 1 行を記録
  --claude-turn           Claude Code の 1 ターンを記録
  --output-tokens N       累積 output_tokens（差分計算用）
```

---

## フック導入フロー

`tamago init` は卵生成のみ。rc ファイルには触らない。

```
$ tamago init
🥚 たまごが生まれました！

次にフックを設定してください:
  tamago hook zsh  >> ~/.zshrc
  tamago hook bash >> ~/.bashrc
```

Claude Code 側は `statusline-command.sh` に `tamago tick --claude-turn --output-tokens ...` を追記する。

---

## 経験値システム

### シェルコマンド（`tracker::score`）

| コマンド | exp | カテゴリ |
|----------|-----|----------|
| `git commit` | 20 | Git |
| `git push`   | 18 | Git |
| `git status` | 13 | Git |
| `git` (base) | 10 | Git |
| `claude`     | 15 | Ai |
| `cargo`, `go`, `python`, `node`, ... | 8 | Dev |
| `docker`, `kubectl`, `terraform`, ... | 8 | Infra |
| `emacs`, `vim`, `nvim` | 5 | Editor |
| `make`, `npm`, `yarn`, `pip` | 5 | Dev |
| `ssh`, `scp`, `rsync` | 4 | Infra |
| `cd`, `ls`, `cat`, `pwd`, ... | 1 | Basic |
| その他 | 2 | Other |

### Claude Code ターン（`tracker::claude_turn_score`）

```
exp = floor(sqrt(delta_tokens) / 12) + 1
delta_tokens = max(0, current_total - pet.last_output_tokens)
```

- `total_output_tokens` は累積なので、前回差分を `pet.last_output_tokens` に保存
- 平方根ベースの連続関数。短いターンと長いターンで exp が分散
- 代表値: 100→1 / 500→2 / 1000→3 / 10000→9 / 50000→19 / 100000→27 exp

---

## データフロー（2 段構成）

### tick（毎コマンド / 毎ターン、背景）

```
[ユーザーがコマンド実行 or Claude ターン終了]
  → tamago tick --cmd "..." &   (or --claude-turn --output-tokens N &)
    → scoring.rs で exp 算出
    → activity.jsonl に 1 行 append（flock → write → unlock）
    → claude-turn の場合のみ pet.json を flock → last_output_tokens 更新
    → 即終了
```

**シェルコマンドの tick は pet.json に触らない。** 目標 <1ms。

### show / status（ユーザー実行 or statusline 更新）

```
[tamago を叩く or CC statusline 更新]
  → pet.json を load
  → activity.jsonl が AGGREGATE_THRESHOLD (512 bytes) を超えていたら
    → .lock を flock
    → apply_decay（hunger/mood 時間減衰）
    → read_and_clear_activities
    → apply_activities（exp / category_exp 加算）
    → 進化判定ループ (try_evolve)
    → 進化・レベルアップなら evolved_at / leveled_up_at = now
    → save_pet
    → unlock
  → AA + ステータスを表示
```

- show は必ず集計、status は閾値超え時のみ集計
- pet.json の書き戻しは `tmpfile → rename` で atomic
- `just_evolved` / `just_leveled_up` の bool ではなく timestamp で管理 → statusline を何度叩いても 10 秒間は演出が続く

---

## 排他制御

| 対象 | lock | 備考 |
|------|------|------|
| `activity.jsonl` append | 自身を `flock(LOCK_EX)` | tick 同士の直列化 |
| `pet.json` 読み書き | `.lock` を `flock(LOCK_EX)` | 集計・進化・ tick (claude-turn) |
| `pet.json` 読み取り（表示のみ） | ロックなし | tmpfile → rename で atomic なのでレースしない |

---

## 成長・進化

### ステージ遷移

| Stage | 必要 exp | 見た目 |
|-------|----------|--------|
| Egg   | 0        | 🥚 |
| Baby  | 5,000    | 🐣 |
| Child | 20,000   | 🐥 |
| Teen  | 50,000   | 🐤 |
| Adult | 150,000  | archetype 絵文字 |

Egg → Teen は単純な exp 閾値。Teen → Adult で archetype が確定する。

### Adult 分岐（Teen → Adult）

`category_exp` の比率で決定。35% 以上の dominant カテゴリがあれば特化型。

| Archetype | 条件 | 絵文字 |
|-----------|------|--------|
| Versionist | Git ≥ 35% | 🐙 |
| AiMage | Ai ≥ 35% | 🧙 |
| CloudDweller | Infra ≥ 35% | ☁️ |
| AncientMage | Editor ≥ 35% | 📜 |
| Generalist | 特化なし | 🦊 |

### レベル

各 stage 内で現在 exp を 1-99 に正規化。stage を跨ぐとリセット。

### hunger / mood 減衰

```
hunger: last_active から 1 時間ごとに -3
mood:   last_active から 2 時間ごとに -2
apply_activities: コマンド 1 件ごとに hunger +1, mood +1（最大 100）
```

hunger/mood が低いと AA の表情が変わる（Normal / Tired / Exhausted）。
`expression.rs` が AA を bitmap に変換 → 目/口を書き換え → AA に戻す。

---

## AA（ASCII アート）

### 構造

`PetArt { art, creature_type, color }` の配列を stage 毎に持つ。`name` の FNV-1a ハッシュで選出。

- `art`: halfblock 文字 (`▀▄█` + space) による多行文字列
- `creature_type`: 「ペンギン」「スライム」等の日本語種族名。status line に `[タコ]` のように表示
- `color`: ANSI fg 色。進化/レベルアップ演出でテーマ色として使う

### 生成パイプライン

1. nanobanana（Gemini CLI 経由）で 32x32 の 1-bit PNG を生成
2. `tools/img2aa.sh` で halfblock AA に変換
3. レビューして `src/pet/render/<stage>.rs` に PetArt として追加

### 現在のラインナップ

- Egg: 11 個（全て `???`）
- Baby: 12 種（スライム、ペンギン、子猫、他）
- Child: 6 種（子鳥、子うさぎ、子猫、子狐、子犬、子スライム）
- Teen: 10 種（子ドラゴン、九尾狐、おばけ、コウモリ、ヘビ、トカゲ、サメ、骸骨、キングスライム、チョウ）
- Adult: 7 種（蛸使い、クリスタル賢者、とんがり帽子、嵐雲、長老、狐剣士、忍び猫）

### 表情変更

`expression::apply_expression` が condition (Normal/Tired/Exhausted) に応じて AA を改変。
bitmap 往復で目・口を検出して差し替える汎用実装。

### デコレーション

`animate::animate` が mood/exp に応じて sparkle (`✦ ✧ ☆ ♡ ♪ ⋆`) を AA 周囲に散らす。
`render::colorize_aa` が `▀▄█` にテーマ色、sparkle に黄色を適用。

---

## statusline 出力

### 通常時（1 行）

```
🐣 ピカリン [ペンギン] Lv.27 ♥100 🍚100 EXP:8234
```

### 進化時（evolved_at が 10 秒以内）

```
<AA の各行>
(空白行)
🎉 進化！ 🐣 ピカリン [ペンギン] Lv.1 ♥100 🍚100 EXP:5000
```

### レベルアップ時（leveled_up_at が 10 秒以内）

```
<AA の各行>
(空白行)
✨ レベルアップ！ 🐣 ピカリン [ペンギン] Lv.28 ♥100 🍚100 EXP:8456
```

### CC statusline の制約

Claude Code の statusline は行頭の空白（通常スペース・NBSP 含む）を strip する。
対策として空白 ` ` を `\x1b[30m█\x1b[0m`（黒い `█`、非空白だが背景と同化）に置換する。

`▀▄█` 以外の halfblock も east_asian_width が `Ambiguous` だが、
全 cell が非空白文字ならレイアウト計算は一致する。

---

## show の追加出力

ステータス行の下にカテゴリ別 exp バーチャート（降順）:

```
  🧠 AI    ████████████████████ 7425
  🛠️ Dev   █                    512
  ✨ Other █                    142
  🔀 Git   █                    70
  ☁️ Infra                      0
  📝 Edit                       0
  🐚 Basic                      0
```

---

## 進化 / レベルアップ演出の時間管理

- `evolved_at` / `leveled_up_at` は `Option<DateTime<Utc>>`
- アクセス時に `now - ts < CELEBRATION_DURATION_SECS(10)` なら演出表示
- 期限切れの timestamp は次回表示時に lazy cleanup（`None` に戻して save_pet）
- CC statusline が 1 秒間隔で更新しても、10 秒間は演出が続く

---

## 設計原則

1. **AA がすべて** — AA の品質と面白さに最大限こだわる
2. **シェルをブロックしない** — tick は `&` でバックグラウンド、1ms 以内に終了
3. **常駐しない** — デーモンなし。毎回起動→即終了
4. **壊れない** — flock + atomic rename で複数ターミナル安全
5. **rc ファイルを勝手に触らない** — hook コマンドの出力をユーザーがリダイレクト
6. **中身が見える** — cat / jq でデータを直接確認できる

---

## 実装フェーズ

### Phase 1: 動く卵 ✅

- `init` / `tick --cmd` / `show` / `name` / `hook zsh`
- 集計、AA 表示、hunger/mood 減衰

### Phase 2: CC statusline 連携 ✅

- `tick --claude-turn --output-tokens`
- `status` コマンド
- `hook statusline`
- `last_output_tokens` での差分計算

### Phase 3: 成長・進化 ✅

- 5 ステージ × 複数バリエーションの AA
- Adult archetype 判定
- カテゴリ別 exp バー表示
- nanobanana ワークフロー

### Phase 4: 演出・磨き ✅

- 進化 / レベルアップ時に AA をテーマ色付きで表示
- 10 秒間の演出持続
- animate による sparkle デコレーション
- 表情動的変更 (expression.rs)

### Phase 5（未着手）

- `stats` コマンド（詳細統計）
- `sync` コマンド（history スキャン補完）
- activity.jsonl の rotate（現状は read_and_clear で実質不要）
- Homebrew Tap / GitHub Actions リリース
- feed / play（ユーザー判断で保留中）

---

## 配布

現状は `cargo install --path .` でローカルインストール。
Homebrew 対応は未着手。
