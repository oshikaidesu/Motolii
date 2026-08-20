#!/usr/bin/env bash
# docs整合チェッカー: 台帳の抜け・重複・リンク切れを機械検証する。
# 根拠: 2026-07-19 docs体系化(入口台帳から36件のreview文書が欠落し、
# 既決事項が逆引きできず旧仕様が混在した再発防止)。
# 使い方: scripts/check-docs.sh   (リポジトリルートから)
set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DOCS="$ROOT/docs"
FAIL=0

err() { echo "NG: $1"; FAIL=1; }

# 1. root AGENTS.md は古い参照を壊さないためのtombstoneとして存在だけ確認する。
# 内容へrepo固有のagent policyは課さない。
[ -f "$ROOT/AGENTS.md" ] || err "root AGENTS.md tombstoneがない"

# 旧監督ループ正本は短い撤回tombstoneに固定し、規則集として復活させない。
RETIRED_SUPERVISION="$DOCS/reviews/2026-07-25-opus-spark-grok-supervision-loop-decision.md"
retired_supervision_bytes="$(LC_ALL=C wc -c < "$RETIRED_SUPERVISION" | tr -d '[:space:]')"
if [ "$retired_supervision_bytes" -gt 6000 ]; then
  err "撤回済み監督ループ文書が ${retired_supervision_bytes} bytes。歴史本文を現行treeへ復元しないこと"
fi
grep -Fq '状態: **撤回**' "$RETIRED_SUPERVISION" \
  || err "旧監督ループ文書が撤回状態でない"
grep -Fq '2026-08-03-runner-independent-supervision-decision.md' "$RETIRED_SUPERVISION" \
  || err "旧監督ループ文書に現行runner非依存正本への移動先がない"

# 2. reviews/ の全ファイルが reviews/README.md の索引に登録されていること
for f in "$DOCS"/reviews/*.md; do
  b="$(basename "$f")"
  [ "$b" = "README.md" ] && continue
  grep -q "$b" "$DOCS/reviews/README.md" || err "reviews索引に未登録: docs/reviews/$b"
done

# 3. docs/README.md ファイルマップにリンク先の重複行がないこと
# (要旨セル内の相互参照は正当なので、表の先頭セルのリンクだけを見る)
dups=$(grep -oE '^\| \[[^]]+\]\((reviews/[^)#]+\.md)\)' "$DOCS/README.md" \
  | grep -oE 'reviews/[^)#]+\.md' | sort | uniq -d)
if [ -n "$dups" ]; then
  while IFS= read -r d; do
    err "docs/README.md に重複掲載: ${d#](}"
  done <<< "$dups"
fi

# 4. root tombstone と docs/**/*.md のローカルmdリンクが実在すること
# (#fragmentは除去して判定)。必読入口のリンク切れもdocsと同じ失敗にする。
python3 - "$ROOT" <<'PY'
import os, re, sys
root = sys.argv[1]
docs = os.path.join(root, 'docs')
# npm ci/build/test:visual で docs 配下に現れる生成物・依存 dir へは降下しない
SKIP_DIR_NAMES = frozenset({
    "node_modules", "dist", "test-results", "playwright-report",
})
link_re = re.compile(r'\]\(([^)]+)\)')
fail = False
paths = [os.path.join(root, 'AGENTS.md')]
for dirpath, dirnames, files in os.walk(docs):
    dirnames[:] = [d for d in dirnames if d not in SKIP_DIR_NAMES]
    for name in files:
        if name.endswith('.md'):
            paths.append(os.path.join(dirpath, name))
for path in paths:
    text = open(path, encoding='utf-8').read()
    for target in link_re.findall(text):
        if target.startswith(('http://', 'https://', 'mailto:', '#')):
            continue
        target = target.split('#')[0].strip()
        if not target:
            continue
        resolved = os.path.normpath(os.path.join(os.path.dirname(path), target))
        if not os.path.exists(resolved):
            rel = os.path.relpath(path, root)
            print(f"NG: リンク切れ {rel} -> {target}")
            fail = True
sys.exit(1 if fail else 0)
PY
[ $? -ne 0 ] && FAIL=1

# 5. decision-index.md の状態語彙が固定集合に収まっていること
if [ -f "$DOCS/decision-index.md" ]; then
  bad=$(awk -F'|' '/^\|/ && NF>=6 && $2 !~ /主題|---/ {
    gsub(/^[ \t]+|[ \t]+$/, "", $4);
    if ($4 !~ /^(決定|縮小採用|延期|棄却|撤回|未統一|観察|比較中|停止線)$/) print $4
  }' "$DOCS/decision-index.md" | sort -u)
  if [ -n "$bad" ]; then
    while IFS= read -r w; do
      err "decision-index.md に未定義の状態語彙: 「$w」(許可: 決定/縮小採用/延期/棄却/撤回/未統一/観察/比較中/停止線)"
    done <<< "$bad"
  fi
else
  err "docs/decision-index.md が存在しない"
fi

# 6. UI表示・起動のdocs入口が成果物用語正本を保持すること
python3 - "$ROOT" <<'PY'
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
readme = (root / "docs/README.md").read_text(encoding="utf-8")
terms = (root / "docs/ui-artifact-terminology.md").read_text(encoding="utf-8")
fail = False

def require(condition, message):
    global fail
    if not condition:
        print(f"NG: {message}")
        fail = True

require(
    "[ui-artifact-terminology.md](ui-artifact-terminology.md)" in readme,
    "docs/README.md の読む順序にUI成果物用語正本がない",
)
require(
    "## 表示・起動要求の必須ルート" in terms,
    "UI成果物用語正本に表示・起動要求の必須ルートがない",
)
require(
    "最も近いもの」を自動で起動しない" in terms,
    "UI成果物用語正本に代替起動の負例がない",
)

sys.exit(1 if fail else 0)
PY
[ $? -ne 0 ] && FAIL=1

if [ $FAIL -eq 0 ]; then
  echo "OK: docs整合チェック全項目通過"
else
  echo "FAILED: 上記を修正するか、意図的なら該当規則を docs/reviews/README.md の登録規則ごと改訂する"
fi
exit $FAIL
