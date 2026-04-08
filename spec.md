# tamago 設計書

CLIで育てるターミナルペット。ターミナルの使用度に応じて成長し、育て方で進化が分岐する。

---

## コンセプト

- ターミナルを日常的に使うだけでペットが育つ
- Claude Code のヘビーユーザーほど AI 系に進化する
- statusline に常駐して「相棒感」を演出
- **常駐プロセスなし**。毎回起動→即終了の fire-and-forget 設計

---

## 技術スタック

| 項目 | 選定 |
|------|------|
| 言語 | Rust |
| CLI パーサー | clap |
| データストア | JSON ファイル（pet.json + activity.jsonl） |
| シリアライズ | serde + serde_json |
| 排他制御 | flock（libc） |
| 時刻 | chrono |
| ホームディレクトリ | dirs |

### データストア選定理由

SQLite や組み込み KVS（redb, sled 等）ではなく JSON ファイルを採用。

- **tick の速度最優先**: 毎コマンド実行で走るため、JSONL append（~1ms）が最速
- **スキーマ変更が容易**: `#[serde(default)]` でフィールド追加するだけ。マイグレーション不要
- **デバッグ容易**: `cat pet.json` / `jq . activity.jsonl` で中身を確認できる
- **依存最小**: serde_json のみ。C バインディング不要
- **データ量が小さい**: ペット状態は 1 オブジェクト、activity は 1 日 200 行程度

---

## ディレクトリ構成

```
~/.config/tamago/
  pet.json          # ペット状態（1 オブジェクト）
  activity.jsonl    # コマンドログ（append-only、1 行 1 レコード）
  config.toml       # ユーザー設定
  .lock             # flock 用（tamago 表示時の排他制御）
```

`dirs::config_dir()` で取得。macOS では `~/Library/Application Support/tamago` になるが、
`$XDG_CONFIG_HOME` が設定されていればそちらを優先する。

---

## クレート構成

```
tamago/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI エントリポイント (clap)
│   ├── cmd/
│   │   ├── mod.rs
│   │   ├── init.rs       # 初期セットアップ
│   │   ├── tick.rs       # フックから呼ばれる（内部用）
│   │   ├── show.rs       # メイン表示（tamago）
│   │   ├── status.rs     # statusline 用ワンライナー
│   │   ├── feed.rs       # ごはん
│   │   ├── play.rs       # 遊ぶ
│   │   ├── stats.rs      # 活動統計
│   │   ├── hook.rs       # フックスクリプト出力
│   │   └── sync.rs       # history スキャン（補完）
│   ├── pet/
│   │   ├── mod.rs
│   │   ├── state.rs      # PetState, Stage, Archetype
│   │   ├── evolution.rs  # 進化判定・分岐ロジック
│   │   ├── decay.rs      # hunger/mood 時間減衰
│   │   └── render.rs     # AA 描画・statusline 出力
│   ├── tracker/
│   │   ├── mod.rs
│   │   ├── scoring.rs    # コマンド → 経験値変換
│   │   └── scanner.rs    # shell history スキャン
│   └── storage.rs        # JSON ファイル読み書き・flock
```

---

## データモデル

### pet.json

```json
{
  "name": "クロード",
  "born_at": "2026-04-08T12:00:00Z",
  "stage": "egg",
  "exp": 0,
  "hunger": 100,
  "mood": 100,
  "archetype": null,
  "category_exp": {
    "git": 0,
    "ai": 0,
    "dev": 0,
    "infra": 0,
    "editor": 0,
    "basic": 0,
    "other": 0
  },
  "last_fed": "2026-04-08T12:00:00Z",
  "last_active": "2026-04-08T12:00:00Z",
  "activity_cursor": 0
}
```

- `category_exp`: カテゴリ別の累計 exp。進化分岐の判定に使う
- `activity_cursor`: activity.jsonl の集計済みバイトオフセット
- フィールド追加時は `#[serde(default)]` で後方互換

### activity.jsonl

1 行 1 JSON。append-only。

```jsonl
{"cmd":"git commit -m fix","cat":"git","exp":10,"ts":"2026-04-08T12:01:00Z"}
{"cmd":"cargo build","cat":"dev","exp":8,"ts":"2026-04-08T12:02:00Z"}
{"cmd":"ls","cat":"basic","exp":1,"ts":"2026-04-08T12:03:00Z"}
```

- tick が 1 行ずつ追記する
- tamago（表示）が `activity_cursor` 以降を読んで集計し、cursor を進める
- 定期的に古いログを truncate（Phase 4）

---

## CLI コマンド

```
tamago              メイン画面（集計 + AA + ステータス表示）
tamago init         初期セットアップ（卵生成 + ガイド表示）
tamago hook --zsh   zsh 用フックスクリプトを stdout に出力
tamago hook --bash  bash 用フックスクリプトを stdout に出力
tamago status       statusline 用ワンライナー（pet.json 読むだけ）
tamago feed         ごはん（hunger 回復）
tamago play         遊ぶ（mood 回復）
tamago name <名前>  命名・改名
tamago stats        活動統計（カテゴリ別の exp 内訳）
tamago sync         history スキャン（手動補完用）
tamago tick --cmd "..." フックから呼ばれる（内部用）
```

---

## フック導入フロー

`tamago init` は卵生成 + ガイド表示のみ。rc ファイルには触らない。

```
$ tamago init
🥚 たまごが生まれました！

次にフックを設定してください:

  tamago hook --zsh  >> ~/.zshrc
  tamago hook --bash >> ~/.bashrc

設定後:
  source ~/.zshrc
```

`tamago hook --zsh` は stdout にフック本体、stderr にガイドを出力:

```zsh
# tamago - terminal pet
_tamago_preexec() { command tamago tick --cmd "$1" &!; }
autoload -Uz add-zsh-hook
add-zsh-hook preexec _tamago_preexec
```

---

## 経験値システム

### コマンドスコアリング

| コマンド | exp | カテゴリ |
|----------|-----|----------|
| `git` (commit/push/rebase等) | 10 | Git |
| `claude` | 15 | Ai |
| `cargo`, `go build` | 8 | Dev |
| `docker`, `kubectl`, `gcloud` | 8 | Infra |
| `emacs`, `emacsclient` | 5 | Editor |
| `vim`, `nvim` | 5 | Editor |
| `make`, `npm`, `yarn`, `bundle` | 5 | Dev |
| `ssh`, `scp`, `rsync` | 4 | Infra |
| `cd`, `ls`, `cat`, `pwd` | 1 | Basic |
| その他 | 2 | Other |

`git` はサブコマンドで追加ボーナス:
- `git commit` → +10
- `git push` → +8
- `git status` → +3

### CC statusline 経由（Phase 2）

CC の statusline スクリプトから `tamago tick --claude-turn` を呼ぶ。
CC が会話更新ごと（300ms スロットル）にスクリプトを実行するため、
ターンごとの経験値が自動で入る。

```bash
# ~/.claude/statusline.sh 内に追記
tamago tick --claude-turn &
```

| イベント | exp | カテゴリ |
|----------|-----|----------|
| `--claude-turn` | 3 | Ai |

---

## データフロー

### tick（毎コマンド実行時）

```
[ユーザーがコマンド実行]
  → preexec 発火
  → tamago tick --cmd "git commit -m fix"  (バックグラウンド &!)
    → scoring: "git commit" → exp=10, category=git
    → activity.jsonl に 1 行 append（flock → write → unlock）
    → 即終了 (目標 < 1ms)
```

**tick は activity.jsonl に append するだけ。pet.json に触らない。**

### tamago（ユーザーが明示的に実行）

```
[ユーザーが tamago を叩く]
  → .lock を flock（排他ロック）
  → pet.json 読み込み
  → activity.jsonl の activity_cursor 以降を読む
  → 未集計分の exp をカテゴリ別に加算
  → activity_cursor を更新
  → hunger/mood の時間減衰計算
  → 進化判定
  → pet.json を atomic write（tmpfile → rename）
  → .lock を unlock
  → AA + ステータス表示
```

flock で排他するため、複数ターミナルから同時に叩いても安全。

### tamago status（statusline 用）

```
[プロンプト表示 or tmux status-interval]
  → pet.json を読む（ロック不要、読み取りのみ）
  → statusline フォーマットして stdout に出力
  → 集計はしない（軽量、< 1ms）
```

---

## 排他制御

```
tick (activity.jsonl への append):
  → activity.jsonl 自体を flock(LOCK_EX)
  → 1 行 write
  → unlock
  → 複数ターミナルからの同時 append を直列化

tamago (pet.json の読み書き):
  → .lock ファイルを flock(LOCK_EX)
  → pet.json 読み → 集計 → 書き戻し
  → unlock
  → 複数ターミナルからの同時実行を直列化

tamago status (pet.json の読み取り):
  → ロックなし
  → pet.json を読むだけ
```

pet.json の書き戻しは **write to tmpfile → rename** で atomic に行う。
status が中途半端な状態を読むことを防ぐ。

---

## 成長・進化

### ステージ遷移

| Stage | 必要 exp | 追加条件 | 見た目 |
|-------|----------|----------|--------|
| Egg   | 0        | -        | 🥚     |
| Baby  | 50       | -        | 🐣     |
| Child | 200      | hunger 平均 70 以上 | 🐥     |
| Teen  | 500      | -        | 🐤     |
| Adult | 1500     | カテゴリ比率で分岐 | 後述   |

Egg → Baby → Child → Teen は単純な exp 閾値で進化。
Teen → Adult のみ分岐判定が入る。

### 進化分岐（Teen → Adult）

category_exp の比率で決定。35% 以上の dominant カテゴリがあれば特化型に進化。

| Archetype | 条件 | 見た目 |
|-----------|------|--------|
| Versionist（バージョンの守護者） | Git ≥ 35% | 🐙 |
| AiMage（AI使い） | Ai ≥ 35% | 🧙 |
| CloudDweller（雲の住人） | Infra ≥ 35% | ☁️ |
| AncientMage（古の魔術師） | Editor ≥ 35% | 📜 |
| Generalist（万能型） | 特化なし | 🦊 |

### hunger / mood 減衰

```
hunger: 最終 feed 時刻から 1 時間ごとに -3
mood:   最終 activity 時刻から 2 時間ごとに -2

hunger = 0 かつ 48 時間放置 → mood さらに -20
```

---

## statusline 出力フォーマット

```
通常:     🥚 クロード Lv.3 ♥80 🍚92
Adult:    🧙 クロード Lv.24 ♥95 🍚78
空腹:     💀 クロード Lv.12 ♥30 🍚12
```

Lv = stage 内の相対レベル（exp ベースで算出）

---

## activity.jsonl の肥大化対策

1 日 200 コマンド × 30 日 = ~600KB。短期的には問題ない。
Phase 4 で以下を実装:

- `tamago` 実行時に cursor 以前の古いログを truncate
- または月次で rotate（activity.jsonl.1 等）

---

## 開発フェーズ

### Phase 1: 動く卵（1-2 日）

- `init`: ~/.config/tamago/ 作成 + pet.json 生成 + ガイド表示
- `hook --zsh`: preexec フックスクリプト出力
- `tick --cmd`: activity.jsonl に append
- `tamago`: 集計 + 表示（AA + ステータス）
- `feed` / `play`: hunger/mood 回復
- `name`: 命名

### Phase 2: CC statusline 連携（1-2 日）

- `tick --claude-turn`: CC ターン記録
- `status`: statusline 用ワンライナー
- CC statusline.sh への組み込みガイド

### Phase 3: 成長・進化（2-3 日）

- Stage 遷移ロジック
- カテゴリ別統計集計（category_exp）
- 進化分岐判定
- Archetype 別 AA
- `stats` コマンド

### Phase 4: 磨き・配布（1-2 日）

- `hook --bash`: bash 対応
- `sync`: history スキャン（tick 漏れ補完）
- activity.jsonl の truncate / rotate
- statusline カスタムフォーマット
- `tamago` 表示時の進化演出
- hunger/mood 減衰の調整
- GitHub Actions リリースワークフロー（マルチプラットフォームビルド）
- Homebrew Tap 作成・Formula 自動更新

---

## 配布

### インストール方法（ユーザー向け）

```bash
# Cargo（Rust ユーザー）
cargo install tamago

# Homebrew（Phase 4 以降で対応）
# brew tap <owner>/tamago
# brew install tamago
```

### Homebrew Tap（Phase 4 以降）

別リポジトリ `<owner>/homebrew-tamago` を作成。
GitHub Actions でタグ push 時にマルチプラットフォームビルド → Release にアップロード → Formula 自動更新。
詳細は実装時に設計する。

### ビルドターゲット

| ターゲット | OS | Arch | 備考 |
|------------|-----|------|------|
| aarch64-apple-darwin | macOS | Apple Silicon | メイン |
| x86_64-apple-darwin | macOS | Intel | |
| x86_64-unknown-linux-musl | Linux | x86_64 | 静的リンク |
| aarch64-unknown-linux-musl | Linux | ARM64 | 静的リンク |

Linux は musl で静的リンク。glibc バージョン依存を避ける。

---

## 設計原則

1. **シェルをブロックしない** — tick は &! でバックグラウンド、1ms 以内に終了
2. **常駐しない** — デーモンなし。毎回起動→即終了
3. **壊れない** — flock + atomic rename で複数ターミナル安全
4. **rc ファイルを勝手に触らない** — hook コマンドの出力をユーザーがリダイレクト
5. **段階的に育てる** — Phase 1 で最小限動くものを作り、機能を積む
6. **中身が見える** — cat / jq でデータを直接確認できる
