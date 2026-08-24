#!/usr/bin/env bash
# Motolii の在庫表(`next/reference/generated/inventory.tsv`)を生成する。
#
# 2026-08-23 利用者裁定「外部に答えはあります」— 自前の syn 解析(17,488行)を
# 撤去し、**rustdoc が公式に吐く JSON** を読むだけにした。項目の抽出・可視性・
# ファイル位置・doc は全部コンパイラの解決結果なので、テキスト一致の誤検出
# (doc コメント内の言及を呼び手として数える等)が構造的に起きない。
#
# nightly が要る(rustdoc JSON は nightly 限定。active である必要はない)。
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT/next"
RUSTDOCFLAGS="-Z unstable-options --output-format json" \
  cargo +nightly doc --workspace --no-deps -q 2>/dev/null
python3 "$ROOT/scripts/inventory_from_rustdoc.py" "$ROOT/next/target/doc" \
  > "$ROOT/next/reference/generated/inventory.tsv"
wc -l < "$ROOT/next/reference/generated/inventory.tsv" | xargs echo "inventory.tsv 行数:"
python3 "$ROOT/scripts/derive_components.py" "$ROOT"
python3 "$ROOT/scripts/plan_waves.py" "$ROOT"
python3 "$ROOT/scripts/check_responsibility.py" "$ROOT"
