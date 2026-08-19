#!/usr/bin/env bash
# ラッパーの規律チェック。やることは2つだけ。
#   1. 全 .rs の1行目 doc コメントが `//! wraps:` か `//! owns:` で始まることを確かめる
#   2. `owns:`(= 上流に無いと主張している箇所)を全部並べる
# リンクも索引も検査しない。ラッパーに要るのは「どの上流を包んだか」だけである。
set -u
cd "$(dirname "$0")"
fail=0

while IFS= read -r f; do
  head1="$(grep -m1 -E '^\s*//!' "$f" || true)"
  case "$head1" in
    *"//! wraps:"*|*"//! owns:"*) ;;
    *) echo "NG: 1行目の doc コメントが wraps:/owns: で始まらない — $f"; fail=1 ;;
  esac
done < <(find . -name '*.rs' -not -path './target/*')

echo
echo "=== owns: 上流に無いと主張している箇所(ここだけがレビュー対象) ==="
grep -rn --include='*.rs' '//! owns:' . 2>/dev/null | grep -v '/target/' | sed 's|^\./||' || echo "(なし)"

echo
[ "$fail" -eq 0 ] && echo "OK: wraps/owns marker 全通過" || echo "NG: marker 未記入あり"
exit $fail
