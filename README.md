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
- **対話できる** — `tamago chat` や TUI 下部の入力欄からペットと会話（+50 EXP）
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

### 対話

| コマンド | 説明 |
|----------|------|
| `tamago chat "<メッセージ>"` | ペットに話しかけて返事をもらう。+50 EXP。`tamago show` 起動中なら吹き出しにも表示される |

TUI (`tamago show`) の最下部には常時チャット入力欄があります:

- 文字を入力 → **Enter** で送信
- **Backspace** で削除
- **Esc** でバッファクリア（空なら終了）
- **Ctrl+C / Ctrl+D** で終了

### シェルフック

| コマンド | 説明 |
|----------|------|
| `tamago hook zsh` | zsh 用フックスクリプトを stdout に出力 |
| `tamago hook bash` | bash 用フックスクリプトを stdout に出力 |
| `tamago hook statusline` | Claude Code statusline 用スクリプトを stdout に出力 |

### LLM バックエンド

デフォルトは `claude`（Claude CLI）。

| コマンド | 説明 |
|----------|------|
| `tamago llm show` | 現在の LLM バックエンドを表示 |
| `tamago llm claude` | Claude CLI に切り替え（デフォルト、要 Claude Code） |
| `tamago llm local` | ローカル LLM (candle + Qwen2.5) に切り替え（初回はモデル DL） |
| `tamago llm none` | LLM を無効化（テンプレフォールバックのみ） |
| `tamago llm device` | 推論に使うデバイス (GPU/CPU) を表示 |

#### ローカル LLM の GPU サポート

| 環境 | バックエンド | Homebrew |
|------|------------|----------|
| macOS (Apple Silicon / Intel) | Metal | ✅ 有効 |
| Linux | CUDA | ❌ CPU のみ（自前ビルドで有効化） |

> [!NOTE]
> **GPU が無い環境では `claude` か `none` を推奨。** CPU でローカル LLM を回すと推論が数十秒かかり、TUI の応答性が悪くなります。
>
> ```bash
> tamago llm claude  # Claude CLI を使う（要 Claude Code）
> tamago llm none    # LLM 無効、固定テンプレートのみ
> ```

##### macOS

Homebrew 版は Metal feature を有効化済み。追加設定不要:

```bash
tamago llm device
# コンパイル時 GPU feature: metal
# 推論デバイス (ランタイム): Metal
```

##### Linux + CUDA

Homebrew / cargo install のデフォルトビルドは musl 静的リンクのため CPU 推論のみ。GPU を使うには自前ビルドが必要:

**事前要件:**
- NVIDIA GPU + ドライバ（`nvidia-smi` が動くこと）
- [CUDA Toolkit](https://developer.nvidia.com/cuda-downloads) 12.x（`nvcc` が PATH にあること）
- cuBLAS（CUDA Toolkit に同梱）

**NVIDIA ドライバのインストール:**

```bash
# Ubuntu / Debian
sudo ubuntu-drivers autoinstall   # 推奨ドライバを自動選択
# または明示版: sudo apt install nvidia-driver-550
sudo reboot

# Arch
sudo pacman -S nvidia

# Fedora / RHEL
sudo dnf install akmod-nvidia   # RPM Fusion 必要
```

再起動後 `nvidia-smi` で GPU が見えることを確認。

**CUDA Toolkit のインストール:**

ディストロ公式パッケージはバージョンが古い場合があるため、[NVIDIA 公式](https://developer.nvidia.com/cuda-downloads) の手順推奨:

```bash
# 例: Ubuntu 22.04 / CUDA 12.4 の場合
wget https://developer.download.nvidia.com/compute/cuda/12.4.0/local_installers/cuda-repo-ubuntu2204-12-4-local_12.4.0-550.54.14-1_amd64.deb
sudo dpkg -i cuda-repo-ubuntu2204-12-4-local_*.deb
sudo cp /var/cuda-repo-ubuntu2204-12-4-local/cuda-*-keyring.gpg /usr/share/keyrings/
sudo apt update
sudo apt install cuda-toolkit-12-4

# PATH 設定（~/.bashrc 等に）
export PATH=/usr/local/cuda/bin:$PATH
export LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH
```

Arch なら `sudo pacman -S cuda` で PATH も自動。

**確認:**

```bash
nvidia-smi              # ドライバ & GPU 認識
nvcc --version          # CUDA Toolkit
```

両方動くことを確認してから次へ進んでください。

**ビルド:**

```bash
cargo install --git https://github.com/yagince/tamago.git --features cuda
```

**動作確認:**

```bash
tamago llm device
# コンパイル時 GPU feature: cuda
# 推論デバイス (ランタイム): CUDA
```

`推論デバイス: CPU` と出る場合はドライバ未検出 or CUDA Toolkit のバージョン不一致の可能性。`nvidia-smi` と `nvcc --version` の対応バージョンを確認してください。

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
- `chat_feed.jsonl` — `tamago chat` の返答を TUI にブリッジする一時キュー（TUI が読むと切り詰め）
- `tamago.log` — 実行ログ（LLM タイムアウト等）。5000 行でローテーション

## ライセンス

MIT
