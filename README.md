# 🥚 tamago

CLIで育てるターミナルペット。シェルの使用度に応じて成長し、育て方で進化が分岐する。

```
      ▄▀▀▀▀▀▀▀▀▄
    ▄▀          ▀▄
    █   ▄    ▄   █
    █            █
    █    ▀▄▄▀    █
     ▀▄        ▄▀
       ▀▀▄▄▄▄▀▀

🥚 ピカリン [???] Lv.8 ♥100 🍚100
```

## 特徴

- **ターミナルを使うだけで育つ** — preexec フックで毎コマンド経験値を記録
- **Claude Code 連携** — Claude との会話量に応じて AI カテゴリの経験値が増加
- **進化分岐** — Git / AI / Dev / Infra / Editor のカテゴリ比率で Adult の姿が変わる
- **ratatui TUI** — `tamago show` でドラクエ風のアニメーション付きステータス画面
- **動的表情** — hunger/mood に応じて AA のビットマップを解析し目・口を動的変更
- **Claude が喋る** — TUI で Claude CLI がペットのセリフを生成
- **常駐プロセスなし** — 毎回起動→即終了の fire-and-forget 設計

## インストール

### Homebrew

```bash
brew tap yagince/tamago
brew install tamago
```

### Cargo

```bash
cargo install --git https://github.com/yagince/tamago.git
```

## セットアップ

```bash
# 1. 初期化（卵生成）
tamago init

# 2. シェルフックを設定
tamago hook zsh >> ~/.zshrc
source ~/.zshrc

# bash の場合
tamago hook bash >> ~/.bashrc
```

### Claude Code statusline 連携（任意）

```bash
tamago hook statusline >> ~/.claude/statusline-command.sh
```

## コマンドリファレンス

### 基本

| コマンド | 説明 |
|----------|------|
| `tamago` | メイン画面（集計 + AA + ステータス）。引数なしのデフォルト動作 |
| `tamago init` | 初期セットアップ。卵を生成してガイドを表示 |
| `tamago reset` | 全データを削除して初期化しなおす |
| `tamago update` | 最新バージョンに自己更新 |

### 表示

| コマンド | 説明 |
|----------|------|
| `tamago show` | ドラクエ風 TUI アニメーション画面（Claude がセリフを生成） |
| `tamago show --message-interval <秒>` | セリフの更新間隔を指定（デフォルト: 30 秒） |
| `tamago status` | statusline 用ワンライナー（emoji + 名前 + 種族 + Lv + mood + hunger + EXP） |

### 命名

| コマンド | 説明 |
|----------|------|
| `tamago name <名前>` | 指定名に改名 |
| `tamago name --ai` | LLM に名前を考えさせる |

### シェルフック

| コマンド | 説明 |
|----------|------|
| `tamago hook zsh` | zsh 用フックスクリプトを stdout に出力 |
| `tamago hook bash` | bash 用フックスクリプトを stdout に出力 |
| `tamago hook statusline` | Claude Code statusline 用スクリプトを stdout に出力 |

### LLM バックエンド

| コマンド | 説明 |
|----------|------|
| `tamago llm show` | 現在の LLM バックエンドを表示 |
| `tamago llm local` | ローカル LLM (candle + Qwen2.5) に切り替え（初回はモデル DL） |
| `tamago llm claude` | Claude CLI に切り替え |
| `tamago llm none` | LLM を無効化（テンプレフォールバックのみ） |

### Claude Code スキル

| コマンド | 説明 |
|----------|------|
| `tamago skill install` | Claude Code Agent Skill をユーザーグローバルにインストール |
| `tamago skill install --project` | プロジェクトローカル (`.claude/skills/`) にインストール |

インストール後、Claude Code で「ペットを見せて」「pet status」などで呼び出せます。

## 成長・進化

| Stage | 必要 EXP | 見た目 |
|-------|---------|--------|
| Egg   | 0       | 🥚 卵  |
| Baby  | 5,000   | 🐣 孵化 |
| Child | 20,000  | 🐥 成長 |
| Teen  | 50,000  | 🐤 変化 |
| Adult | 150,000 | 分岐   |

### Adult の進化分岐

| Archetype | 条件 | 例 |
|-----------|------|-----|
| Versionist | Git ≥ 35% | 🐙 蛸使い |
| AiMage | AI ≥ 35% | 🧙 クリスタル賢者 |
| CloudDweller | Infra ≥ 35% | ☁ 嵐雲 |
| AncientMage | Editor ≥ 35% | 📜 長老 |
| Generalist | 特化なし | 🦊 狐剣士 |

## データ

`~/.config/tamago/` に保存:

- `pet.json` — ペット状態
- `activity.jsonl` — コマンドログ（append-only）

## ライセンス

MIT
