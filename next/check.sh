#!/usr/bin/env bash
# ラッパーの規律チェック。やることは2つだけ。
#   1. 各 crate の根(lib.rs / main.rs)が `//! wraps:` か `//! owns:` で始まることを確かめる
#   2. `owns:`(= 上流に無いと主張している箇所)を、行数つきで全部並べる
#
# リンクも索引も検査しない。ラッパーに要るのは「どの上流を包んだか」だけで、
# それはコードの隣にあるのが最も腐りにくい。
# 粒度が crate 単位なのは、`owns:` が「この crate は上流に無い物を実装している」という
# 主張だからである。ファイル単位にすると宣言が増えるだけで、読む人が減る。
set -u
cd "$(dirname "$0")"
fail=0

roots="$(find . \( -name 'lib.rs' -o -name 'main.rs' \) -not -path './target/*' | sort)"

while IFS= read -r f; do
  [ -z "$f" ] && continue
  head1="$(grep -m1 -E '^\s*//!' "$f" || true)"
  case "$head1" in
    *"//! wraps:"*|*"//! owns:"*) ;;
    *) echo "NG: crate の根が wraps:/owns: で始まらない — ${f#./}"; fail=1 ;;
  esac
done <<< "$roots"

echo
echo "=== owns: 上流に無いと主張している箇所(ここだけがレビュー対象) ==="
found=0
while IFS= read -r f; do
  [ -z "$f" ] && continue
  claim="$(grep -m1 -E '^\s*//! owns:' "$f" || true)"
  [ -z "$claim" ] && continue
  found=1
  dir="$(dirname "$f")"
  lines="$(find "$dir" -name '*.rs' -exec cat {} + | wc -l | tr -d ' ')"
  printf '%7s行  %s\n          %s\n' "$lines" "${f#./}" "$(echo "$claim" | sed 's|^\s*//! owns: ||')"
done <<< "$roots"
[ "$found" -eq 0 ] && echo "(なし)"

echo
echo "=== wraps: 上流の薄い口(中身を知りたければ上流を読む) ==="
while IFS= read -r f; do
  [ -z "$f" ] && continue
  claim="$(grep -m1 -E '^\s*//! wraps:' "$f" || true)"
  [ -z "$claim" ] && continue
  printf '          %s\n          %s\n' "${f#./}" "$(echo "$claim" | sed 's|^\s*//! wraps: ||')"
done <<< "$roots"

echo
[ "$fail" -eq 0 ] && echo "OK: wraps/owns marker 全通過" || echo "NG: marker 未記入あり"
exit $fail
