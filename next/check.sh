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
owns_total=0
while IFS= read -r f; do
  [ -z "$f" ] && continue
  claim="$(grep -m1 -E '^\s*//! owns:' "$f" || true)"
  [ -z "$claim" ] && continue
  found=1
  dir="$(dirname "$f")"
  lines="$(find "$dir" -name '*.rs' -exec cat {} + | wc -l | tr -d ' ')"
  owns_total=$((owns_total + lines))
  printf '%7s行  %s\n          %s\n' "$lines" "${f#./}" "$(echo "$claim" | sed 's|^\s*//! owns: ||')"
done <<< "$roots"
[ "$found" -eq 0 ] && echo "(なし)"
echo
printf '  合計 %s行 — 自前で持っているコードの総量 = 保守の負債。**下がるべき数字**。\n' "$owns_total"
echo '  上がった時は「上流に無い物を作った」か「使わない物を抱えた」かのどちらか。'

echo
echo "=== wraps: 上流の薄い口(中身を知りたければ上流を読む) ==="
while IFS= read -r f; do
  [ -z "$f" ] && continue
  claim="$(grep -m1 -E '^\s*//! wraps:' "$f" || true)"
  [ -z "$claim" ] && continue
  printf '          %s\n          %s\n' "${f#./}" "$(echo "$claim" | sed 's|^\s*//! wraps: ||')"
done <<< "$roots"

# Lottie の地図(= 実質 OSS の AE 解析)をどこまで判断したか。
# 「作る瞬間に読む」だと読み落としが見えないので、先に全語彙を並べて未判定を数える。
if [ -f reference/lottie-coverage.tsv ]; then
  echo
  echo "=== Lottie 地図(AE の意味のうち、まだ向き合っていない量) ==="
  awk -F'\t' '$1 !~ /^#/ && $1 != "group" && $5 != "該当なし" { n[$5]++; total++ }
    END { for (s in n) printf "  %-8s %4d\n", s, n[s];
          printf "  → 未判定 %d / %d\n", n["未判定"]+0, total }' \
    reference/lottie-coverage.tsv
  echo
  echo "=== 発注単位(採用予定 = 残り)==="
  awk -F'\t' '$1 !~ /^#/ && $1 != "group" && $5 == "採用予定" && $8 != "" { n[$8]++ }
    END { for (u in n) printf "  %-16s %3d\n", u, n[u] }' \
    reference/lottie-coverage.tsv | sort -k2 -rn
fi

# 意図束(IB 44束、裁定177 / 2026-08-22)。normal-map.tsv の bundle 列(15列目)が
# 正本 reference/intent-bundles.tsv と噛み合っているかを3点で見る:
#   (a) 採用済/結線待ち/採用予定/保留/拡張の全行に bundle があること(不採用は空欄)
#   (b) 記入された bundle id が intent-bundles.tsv に実在すること
#   (c) 束ごとの行数が intent-bundles.tsv の size 申告と一致すること
if [ -f reference/intent-bundles.tsv ] && [ -f reference/normal-map.tsv ]; then
  echo
  echo "=== 意図束(normal-map bundle 列 ⇔ intent-bundles.tsv)==="
  ib_out="$(awk -F'\t' '
    FNR==NR { if ($1 !~ /^#/ && $1 != "id") size[$1]=$5; next }
    FNR>1 {
      v=$13; b=$15
      if ((v=="採用済"||v=="結線待ち"||v=="採用予定"||v=="保留"||v=="拡張") && b=="")
        printf "NG: bundle 未記入 — id=%s(%s)\n", $1, v
      if (v=="不採用" && b!="")
        printf "NG: 不採用行に bundle — id=%s(%s)\n", $1, b
      if (b!="") { if (!(b in size)) printf "NG: 未定義の bundle id — 行%s が %s を指す\n", $1, b; else n[b]++ }
    }
    END { for (b in size) if (n[b]+0 != size[b]+0)
      printf "NG: 束 %s の行数 %d ≠ size 申告 %s\n", b, n[b]+0, size[b] }
  ' reference/intent-bundles.tsv reference/normal-map.tsv)"
  if [ -n "$ib_out" ]; then
    echo "$ib_out"
    fail=1
  else
    awk -F'\t' '
      FNR==NR { if ($1 !~ /^#/ && $1 != "id") nb++; next }
      FNR>1 && $15!="" { na++ }
      END { printf "  束 %d / 割付 %d行 — 記入完全性・id実在・size一致の3検査 全通過\n", nb, na }
    ' reference/intent-bundles.tsv reference/normal-map.tsv
  fi
else
  echo
  echo "NG: reference/intent-bundles.tsv か reference/normal-map.tsv がない"
  fail=1
fi

echo
[ "$fail" -eq 0 ] && echo "OK: wraps/owns marker 全通過" || echo "NG: marker 未記入あり"
exit $fail
