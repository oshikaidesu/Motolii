#!/usr/bin/env bash
# Rerun(re_renderer, re_video 他)側の部品目録を `next/reference/generated/rerun-parts.tsv`
# へ生成する。gen-inventory.sh と同じ理屈(2026-08-23 利用者裁定「外部に答えはあります」)
# — rustdoc が公式に吐く JSON を読むだけで、自前の構文解析はしない。
#
# ピン止め rev が動いても、このスクリプトを再実行するだけで目録が追従する。
# checkout のパスは `cargo metadata` に毎回引かせる(手打ちしない) — rev が変われば
# checkout のハッシュも変わるので、パスのハードコードは古いまま静かに間違い続ける。
#
# nightly が要る(rustdoc JSON は nightly 限定)。
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
COMPOSITOR_MANIFEST="$ROOT/app/engine/motolii-compositor/Cargo.toml"

# 目録に含める Rerun 側クレート。増やすときはここへ足すだけ。
CRATES=(re_renderer re_video)

OUT_DIR="$(mktemp -d)"
trap 'rm -rf "$OUT_DIR"' EXIT

for crate in "${CRATES[@]}"; do
  manifest=$(cargo metadata --manifest-path "$COMPOSITOR_MANIFEST" --format-version 1 2>/dev/null \
    | python3 -c "
import json, sys
d = json.load(sys.stdin)
for pkg in d['packages']:
    if pkg['name'] == '$crate':
        print(pkg['manifest_path'])
        break
")
  if [ -z "$manifest" ]; then
    echo "warning: $crate が依存解決に見つからない(スキップ)" >&2
    continue
  fi
  echo "doc: $crate ($manifest)"
  RUSTDOCFLAGS="-Z unstable-options --output-format json" \
    cargo +nightly doc --manifest-path "$manifest" --no-deps -q --target-dir "$OUT_DIR" \
    || echo "warning: $crate の rustdoc 生成に失敗(スキップ)" >&2
done

python3 "$ROOT/scripts/inventory_from_rustdoc.py" "$OUT_DIR/doc" \
  > "$ROOT/next/reference/generated/rerun-parts.tsv"
wc -l < "$ROOT/next/reference/generated/rerun-parts.tsv" | xargs echo "rerun-parts.tsv 行数:"
