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

# 1. reviews/ の全ファイルが reviews/README.md の索引に登録されていること
for f in "$DOCS"/reviews/*.md; do
  b="$(basename "$f")"
  [ "$b" = "README.md" ] && continue
  grep -q "$b" "$DOCS/reviews/README.md" || err "reviews索引に未登録: docs/reviews/$b"
done

# 2. docs/README.md ファイルマップにリンク先の重複行がないこと
# (要旨セル内の相互参照は正当なので、表の先頭セルのリンクだけを見る)
dups=$(grep -oE '^\| \[[^]]+\]\((reviews/[^)#]+\.md)\)' "$DOCS/README.md" \
  | grep -oE 'reviews/[^)#]+\.md' | sort | uniq -d)
if [ -n "$dups" ]; then
  while IFS= read -r d; do
    err "docs/README.md に重複掲載: ${d#](}"
  done <<< "$dups"
fi

# 3. AGENTS.md と docs/**/*.md のローカルmdリンクが実在すること
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

# 4. decision-index.md の状態語彙が固定集合に収まっていること
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

# 5. UI表示・起動の入口が、成果物用語正本を参照地図より先に通ること
python3 - "$ROOT" <<'PY'
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
agents = (root / "AGENTS.md").read_text(encoding="utf-8")
readme = (root / "docs/README.md").read_text(encoding="utf-8")
terms = (root / "docs/ui-artifact-terminology.md").read_text(encoding="utf-8")
fail = False

def require(condition, message):
    global fail
    if not condition:
        print(f"NG: {message}")
        fail = True

agent_terms = agents.find("docs/ui-artifact-terminology.md")
agent_map = agents.find("docs/ui-reference-map.md")
require(agent_terms >= 0, "AGENTS.md のM3必読動線にUI成果物用語正本がない")
require(agent_map >= 0, "AGENTS.md のM3必読動線にUI参照地図がない")
require(
    agent_terms >= 0 and agent_map >= 0 and agent_terms < agent_map,
    "AGENTS.md はUI成果物用語正本をUI参照地図より先に読ませること",
)
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

# 6. M5 Render Contributionの入口と停止済み証拠templateが再発注不能なこと
python3 - "$ROOT" <<'PY'
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
route_name = "2026-07-29-m5-render-contribution-design-closure-map.md"
route = root / "docs/reviews" / route_name
entry_files = (
    root / "AGENTS.md",
    root / "docs/README.md",
    root / "docs/specs/README.md",
    root / "docs/implementation-ledger.md",
    root / "docs/specs/M5-3d-and-post.md",
)
retired_templates = (
    "2026-07-29-m5-render-contribution-contract-draft.md",
    "2026-07-29-m5-rerun-render-contribution-evidence.md",
    "2026-07-29-m5-render-phase-primary-source-evidence.md",
    "2026-07-29-m5-render-contribution-boundary-facts-v3.md",
    "2026-07-29-m5-rerun-observation-transcription-v3.md",
    "2026-07-29-m5-bevy-observation-transcription-v3.md",
    "2026-07-29-m5-unreal-observation-transcription-v3.md",
)
fail = False

def require(condition, message):
    global fail
    if not condition:
        print(f"NG: {message}")
        fail = True

route_text = route.read_text(encoding="utf-8")
require("## 0. Start Here" in route_text, "M5 Render Contribution締結地図にStart Hereがない")

for path in entry_files:
    text = path.read_text(encoding="utf-8")
    require(route_name in text, f"{path.relative_to(root)} からM5締結地図へ到達できない")

for name in retired_templates:
    path = root / "docs/reviews" / name
    text = path.read_text(encoding="utf-8")
    require("再発注禁止" in text, f"停止済みM5証拠templateに再発注禁止がない: {name}")
    require("状態: **比較中**" not in text, f"停止済みM5証拠templateが比較中のまま: {name}")
    require("未実行" not in text, f"実行済みM5証拠templateが未実行のまま: {name}")

sys.exit(1 if fail else 0)
PY
[ $? -ne 0 ] && FAIL=1

if [ $FAIL -eq 0 ]; then
  echo "OK: docs整合チェック全項目通過"
else
  echo "FAILED: 上記を修正するか、意図的なら該当規則を docs/reviews/README.md の登録規則ごと改訂する"
fi
exit $FAIL
