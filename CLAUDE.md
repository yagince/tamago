# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

tamago は CLI ターミナルペット。シェルの preexec フックで毎コマンド実行時に経験値を記録し、ペットが成長・進化する。常駐プロセスなしの fire-and-forget 設計。

## Build & Test Commands

```bash
cargo build              # ビルド
cargo run                # 実行（= tamago メイン画面）
cargo run -- <subcmd>    # サブコマンド実行（例: cargo run -- init）
cargo test               # 全テスト実行
cargo test <test_name>   # 単一テスト実行
cargo test --features metal -- --ignored  # LLM 推論テスト（Metal GPU、要モデルDL済み）
cargo clippy             # lint
cargo fmt                # フォーマット
cargo fmt -- --check     # フォーマットチェック
```

Rust edition は **2024**。

## Architecture

### データフロー（2段構成）

1. **tick（毎コマンド）**: preexec フックからバックグラウンドで呼ばれ、`activity.jsonl` に1行 append するだけ。pet.json に触らない。目標 < 1ms。
2. **tamago（明示実行）**: `activity.jsonl` の未集計分を読み、pet.json に集計・書き戻し、AA + ステータスを表示。flock で排他制御。

### モジュール構成

- `cmd/` — CLI サブコマンド（init, tick, show, status, feed, play, stats, hook, sync）
- `pet/` — ペット状態管理（state, evolution, decay, render）
- `tracker/` — コマンドスコアリング（scoring）、シェル履歴スキャン（scanner）
- `storage.rs` — JSON ファイル読み書き・flock・atomic write

### データストア

`~/.config/tamago/` 配下に JSON ファイル:
- `pet.json` — ペット状態（1オブジェクト）。書き戻しは tmpfile → rename で atomic
- `activity.jsonl` — コマンドログ（append-only）。tick が追記、tamago が集計
- `.lock` — flock 用排他ファイル

### 排他制御

- `activity.jsonl` への append: ファイル自体を `flock(LOCK_EX)`
- `pet.json` の読み書き: `.lock` ファイルを `flock(LOCK_EX)`
- `status` コマンド: ロックなし（読み取りのみ）

### 技術スタック

clap (CLI), serde + serde_json (シリアライズ), chrono (時刻), dirs (ホームディレクトリ), libc (flock)
