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

# owns: の根拠 token(裁定215 の施工、2026-08-23 発注)。既定は「借りる」——
# `owns:` を持ってよいのは (a) 意見を名指しできる時 (b) 上流不在を実際に
# 確かめた時だけ。(c) probes/testkit は測定器具として別枠。(d) は「立証が
# 足りない」ことを捏造せず正直に書いた記録(合格扱い)。
# ここは Lottie 地図/Intent 到達可能性の節と同じ**情報表示のみ**(fail させない)
# ——実際に fail する柵本体は
# `core/motolii-testkit/tests/owns_justification_fence.rs`(cargo test で実行)。
echo
echo "=== owns: 根拠 token(裁定215 — (a)/(b)/(c)/(d) の内訳、fail は柵本体側)==="
owns_files="$(find . -name '*.rs' -not -path './target/*' -exec grep -l -E '^\s*//! owns:' {} + 2>/dev/null | sort)"
a=0; b=0; c=0; d=0; none=0; excluded=0
while IFS= read -r f; do
  [ -z "$f" ] && continue
  tok="$(grep -m1 -oE 'OWNS-JUSTIFICATION\([A-Z-]+\)' "$f" || true)"
  case "$f" in
    ./core/motolii-store/*|./core/motolii-eval/*)
      # 別レーン走行中の write-set(このレーンでは触らない) — 未記入は
      # ここで検出されて構わない(むしろ検出すべき、発注書の明記)。
      [ -z "$tok" ] && excluded=$((excluded + 1))
      continue
      ;;
  esac
  case "$tok" in
    *"(A)") a=$((a + 1)) ;;
    *"(B)") b=$((b + 1)) ;;
    *"(C-PROBE)"|*"(C-TESTKIT)") c=$((c + 1)) ;;
    *"(D)") d=$((d + 1)) ;;
    *) none=$((none + 1)) ;;
  esac
done <<< "$owns_files"
printf '  (a) 意見を名指し           %3d\n' "$a"
printf '  (b) 上流不在を実際に確認   %3d\n' "$b"
printf '  (c) 測定器具(probes/testkit、別枠) %3d\n' "$c"
printf '  (d) 立証不足(捏造せず正直に申告・合格) %3d\n' "$d"
printf '  根拠 token 無し            %3d' "$none"
[ "$none" -gt 0 ] && echo '  ← cargo test -p motolii-testkit --test owns_justification_fence が赤い' || echo
printf '  除外(store/eval、別レーン走行中・未記入) %3d\n' "$excluded"

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
#   (a) 採用済/採用予定/保留/拡張の全行に bundle があること(不採用は空欄)
#   (b) 記入された bundle id が intent-bundles.tsv に実在すること
#   (c) 束ごとの行数が intent-bundles.tsv の size 申告と一致すること
if [ -f reference/intent-bundles.tsv ] && [ -f reference/normal-map.tsv ]; then
  echo
  echo "=== 意図束(normal-map bundle 列 ⇔ intent-bundles.tsv)==="
  ib_out="$(awk -F'\t' '
    FNR==NR { if ($1 !~ /^#/ && $1 != "id") size[$1]=$5; next }
    FNR>1 {
      v=$13; b=$15
      if ((v=="採用済"||v=="採用予定"||v=="保留"||v=="拡張") && b=="")
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

# depends/weight 列(2026-08-22 裁定: 台帳の並べ替え軸に「粒の重み」を足す)。
# depends = その行がまだ待っている機構名(空 = 待っていない)。weight = S/M/L の3値。
# ここでの機械検出は1点だけ: **depends が空でない(=まだ機構待ちと申告している)のに
# verdict が 採用済(=もう出来上がったと申告している)行**。これは「顔だけ実装」
# (機構が無いのに UI だけ出た)の逆パターン — 機構が届く前に verdict を採用済へ
# 進めてしまった行を機械的に捕まえる。運用規律: 機構が着地したら、verdict を
# 採用済へ上げるのと**同じコミットで depends を空にする**。空にし忘れたらここが赤くなる。
if [ -f reference/normal-map.tsv ]; then
  echo
  echo "=== depends/weight(裁定: 重みなき並べ替えの是正) ==="
  dw_out="$(awk -F'\t' '
    NR==1 { next }
    {
      v=$13; dep=$16; w=$17
      if (dep!="" && v=="採用済")
        printf "NG: depends が未達機構を指すのに verdict=採用済 — id=%s dep=%s\n", $1, dep
      if (w!="" && w!="S" && w!="M" && w!="L")
        printf "NG: weight が S/M/L 以外 — id=%s weight=%s\n", $1, w
      if (dep!="" && w=="")
        printf "NG: depends があるのに weight が空 — id=%s dep=%s\n", $1, dep
      if (v=="不採用" && w!="")
        printf "NG: 不採用行に weight(実装しないので対象外のはず) — id=%s weight=%s\n", $1, w
      if (dep!="") { withdep++ }
      total++
    }
    END { printf "  __STATS__ %d %d\n", withdep+0, total+0 }
  ' reference/normal-map.tsv)"
  stats_line="$(echo "$dw_out" | grep '__STATS__' || true)"
  ng_lines="$(echo "$dw_out" | grep '^NG:' || true)"
  if [ -n "$ng_lines" ]; then
    echo "$ng_lines"
    fail=1
  else
    withdep="$(echo "$stats_line" | awk '{print $2}')"
    total="$(echo "$stats_line" | awk '{print $3}')"
    printf "  depends 記入 %d / %d行 — 顔だけ実装(採用済なのに機構未達を自己申告)0件・weight値3値のみ・不採用行にweightなし の3検査 全通過\n" "$withdep" "$total"
  fi
else
  echo
  echo "NG: reference/normal-map.tsv がない"
  fail=1
fi

# Intent 到達可能性(2026-08-22 追加発注: 「normal-map の採用済は自己申告」
# 問題を Motolii 自身の閉じたスキーマで埋める)。
#
# 背景: normal-map.tsv の 採用済 は嘘をつける形をしている(実測: 2026-08-22 に
# main が 395 コミット進んでも採用済 227 が丸1日動かなかった)。一方
# lottie-coverage.tsv は嘘をつけない — 採用済 行が evidence 欄に**コード中に
# 実在する識別子**を持ち、cargo test がそれを grep して確かめる(裁定は
# lottie_coverage.rs の `adopted_rows_point_at_real_code` 参照)。
#
# 同じ精度を Motolii 自身の語彙(`Document` への唯一の書き口 `Intent`、
# core/motolii-store/src/document.rs、背骨1)に適用する。ただし cargo は
# 使わない(check.sh の write-set はここだけ・4レーン走行中でコード非改変)ので、
# ここは grep だけで到達可能性を粗く測る——lottie 側の厳密な cargo test の
# **簡易版**であって代替ではない。
#
# 「呼び手が在る」と「入口が在る」は別物。実測(2026-08-22)で見つかった罠:
# `Intent::SetPropertyLink` は `ui/motolii-timeline-pane/src/split.rs` と
# `shell/motolii-shell/src/clipboard.rs` に計2箇所現れるが、**両方とも
# 「既存 layer が既に持っている property source を新しい layer id へ複製する」
# match 腕**(split=layer分割、clipboard=copy/paste)——利用者がこの Intent を
# **新規に**発火させる入口ではなく、そもそも起点が無いので複製もされない
# 死んだ経路になる。単純な呼び手カウントはこれを「到達可能」と誤判定する。
#
# 除外リスト(この2ファイルは「複製/転送経路」であって起点にならないので、
# 出現をカウントしない — 除外の理由をここに明記する):
#   ui/motolii-timeline-pane/src/split.rs      … layer 分割: 既存 intent を
#     両半分へ複製するだけ(新しい intent を発生させない)
#   shell/motolii-shell/src/clipboard.rs       … copy/paste: 既存 layer の
#     property source をそのまま複製するだけ(同上)
# `#[cfg(test)]` mod の中身と `/tests/` 配下・コメント行も除外する(呼び手の
# 「実在」を実コードだけで判定するため)。
#
# 曖昧なら厳しい側(=入口なし)へ倒す設計 — この検査で「入口あり」と出た枝は
# 疑ってよいが、「入口なし」はほぼ確実に本当に無い。
if [ -f core/motolii-store/src/document.rs ]; then
  echo
  echo "=== Intent 到達可能性(枝ごとに UI からの入口があるか、grep 簡易版) ==="
  variants="$(awk '
    /^pub enum Intent \{/ { grab=1; next }
    grab && /^}/ { grab=0 }
    grab {
      line=$0
      trimmed=line; gsub(/^[ \t]+/,"",trimmed)
      if (trimmed ~ /^\/\//) next
      if (trimmed == "") next
      if (match(trimmed, /^[A-Z][A-Za-z0-9]*/)) print substr(trimmed, RSTART, RLENGTH)
    }
  ' core/motolii-store/src/document.rs | sort -u)"

  files="$(find ui shell -name '*.rs' -not -path '*/tests/*' 2>/dev/null)"
  zero_entry=""
  n_variants=0
  while IFS= read -r v; do
    [ -z "$v" ] && continue
    n_variants=$((n_variants + 1))
    total=0
    for f in $files; do
      case "$f" in
        */split.rs|*/clipboard.rs) continue ;;
      esac
      n=$(awk '
        BEGIN { in_test=0; depth=0; pending=0 }
        {
          line=$0
          if (in_test) {
            no=gsub(/\{/,"{",line); nc=gsub(/\}/,"}",line)
            depth += no - nc
            if (depth <= 0) { in_test=0 }
            next
          }
          if (line ~ /#\[cfg\(test\)\]/) { pending=1 }
          trimmed=line
          gsub(/^[ \t]+/,"",trimmed)
          if (pending && trimmed ~ /^(pub[ \t]+)?mod[ \t]+[A-Za-z_0-9]+/) {
            if (line ~ /\{/) { in_test=1; depth=1 }
            pending=0
            next
          }
          if (trimmed ~ /^\/\//) { next }
          print line
        }
      ' "$f" | grep -c "Intent::$v\b")
      total=$((total + n))
    done
    if [ "$total" -eq 0 ]; then
      zero_entry="$zero_entry $v"
    fi
  done <<< "$variants"

  n_zero=0
  for v in $zero_entry; do n_zero=$((n_zero + 1)); done
  printf "  Intent 全%d枝中、実UI入口ゼロ(除外後)= %d枝:\n" "$n_variants" "$n_zero"
  for v in $zero_entry; do
    printf "    入口ゼロ: Intent::%s\n" "$v"
  done
  echo "  (この検査は情報提供のみで fail にはしない — 未実装が悪いのではなく、"
  echo "   台帳がそれを「採用済」と自己申告していないかは上の depends/weight 検査と"
  echo "   normal-map.tsv 本体側の監査が担う)"
  echo
  echo "  限界: この検査が捕まえるのは「Intent 経由の到達可能性」の**類**だけ。"
  echo "  裁定207(量に耐えるか — 縦スクロール・複数選択一括編集等)は Intent の"
  echo "  枝に対応しないので原理的に映らない。この検査が緑(または枝の入口が"
  echo "  揃っている)ことは「完成」を意味しない。"
else
  echo
  echo "NG: core/motolii-store/src/document.rs がない(Intent 到達可能性検査をスキップできない)"
  fail=1
fi

# Inspector 時間軸監査(裁定214: Inspector に映る物は全て時間軸で評価できる、
# 境界は identity/property)。棚卸し本体は
# `docs/reviews/2026-08-23-inspector-time-axis-audit.md`(この check.sh からは
# コード改変なしで届く範囲ではない — 理由は同ファイル §6 参照: Inspector の
# 「欄」は `Intent` enum のような単一の列挙点を持たず、identity/property の
# 判定も裁定214 の文面が要求する意味論的判断で型シグネチャから機械導出できない)。
# ここで機械化できるのは唯一、ドキュメント内の**転記事故**の検出だけ ──
# 「K の一覧(成果物、N件)」という見出しの N と、実際に並ぶ番号付き箇条書きの
# 行数が一致するか(`intent-bundles.tsv`/`normal-map.tsv` の size 列不一致検査と
# 同じ動機)。Lottie coverage 節・Intent 到達可能性節と同じ流儀 ──
# **情報表示のみ、fail させない**(ドキュメントが無ければ黙って節ごと省く)。
if [ -f ../docs/reviews/2026-08-23-inspector-time-axis-audit.md ]; then
  audit_doc=../docs/reviews/2026-08-23-inspector-time-axis-audit.md
  echo
  echo "=== Inspector 時間軸監査(裁定214)— K一覧の自己整合性(情報表示のみ) ==="
  claimed="$(grep -oE 'K の一覧\(成果物、[0-9]+件\)' "$audit_doc" | grep -oE '[0-9]+' || true)"
  actual="$(awk '
    /K の一覧/ { grab=1; next }
    grab && /^## / { grab=0 }
    grab && /^[0-9]+\. / { n++ }
    END { print n+0 }
  ' "$audit_doc")"
  if [ -z "$claimed" ]; then
    echo "  (見出し「K の一覧(成果物、N件)」が見つからない — 転記事故の検査を省く)"
  elif [ "$claimed" != "$actual" ]; then
    echo "  見出しの件数($claimed)と実際の箇条書き行数($actual)が不一致 — 転記事故の疑い"
  else
    echo "  K(乗せるべきなのに乗っていない)の一覧: 見出し記載 $claimed 件 = 実箇条書き $actual 件、一致"
  fi
  echo
  echo "  限界: 拾えるのはドキュメント内の数値と本文のズレ(転記事故)だけ。Inspector"
  echo "  が実際に描いている欄を静的に列挙する検査ではない(理由は監査doc §6)。"
fi

echo
[ "$fail" -eq 0 ] && echo "OK: wraps/owns marker 全通過" || echo "NG: marker 未記入あり"
exit $fail
