#!/bin/bash
# 地図を 2 枚作る。**数で見張るための物**で、絵を描くのが目的ではない。
#
#   依存の地図   — 何にいくら払っていて、そのうち自分で触っているのはどれか
#   責任の地図   — 自分の中身のうち、外の何本のコンセントを挿しているか
#
# 依存の地図は build 済みの dylib を読むので、先に `scripts/motolii-ui.sh native` が要る。
# 出来た HTML は自己完結(data 埋め込み、外部は d3 だけ)。target/ の下なので commit されない。
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
out="$repo/motolii/target/maps"
mkdir -p "$out"
[[ -f "$repo/motolii/target/debug/libmotolii_ui.dylib" ]] || {
  echo 'dylib がありません。先に scripts/motolii-ui.sh native を 1 回。'; exit 1; }
python3 "$repo/scripts/map-dependencies.py"   "$out/dependencies.html"
python3 "$repo/scripts/map-responsibility.py" "$out/responsibility.html"
echo
echo "開く:  open $out/dependencies.html $out/responsibility.html"
