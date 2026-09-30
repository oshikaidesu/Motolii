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

# 2. 履歴は git の仕事。docs/ に reviews/・archive/ など歴史置き場を作らない。
#    decision-index の出典は `git:<sha>:<path>` で、その blob が実在すること。
for d in reviews archive specs spikes mocks mocks-ui samples; do
  [ -n "$(git -C "$ROOT" ls-files "docs/$d")" ] && err "docs/$d が復活している(歴史は git 履歴へ)"
done
while IFS= read -r ref; do
  git -C "$ROOT" cat-file -e "${ref#git:}" 2>/dev/null || err "decision-index の出典が解決しない: $ref"
done < <(grep -oE 'git:[0-9a-f]{7,40}:docs/[A-Za-z0-9_./-]*[A-Za-z0-9_]' "$DOCS/decision-index.md" | sort -u)

# 4. root tombstone と docs/**/*.md のローカルmdリンクが実在すること
# (#fragmentは除去して判定)。必読入口のリンク切れもdocsと同じ失敗にする。
python3 - "$ROOT" <<'PY'
import os, re, subprocess, sys
root = sys.argv[1]
docs = os.path.join(root, 'docs')

# リンク先は**git が追跡しているか**で判定する。ディスクにあるかではない。
# 2026-08-27 の分断以降、作業ツリーは sparse-checkout で `app/` などだけを
# 実体化する — 歴史側(`next/` `crates/` `spikes/`)は追跡されたまま手元に無い。
# 実在確認をディスクに聞くと、その状態で歴史へのリンクが全部「切れている」と
# 誤判定される。**追跡されているファイルへのリンクは、手元に実体が無くても有効**。
try:
    tracked = frozenset(subprocess.run(
        ['git', '-C', root, 'ls-files'],
        capture_output=True, text=True, check=True,
    ).stdout.splitlines())
except (subprocess.CalledProcessError, FileNotFoundError):
    tracked = frozenset()

def link_exists(resolved: str) -> bool:
    if os.path.exists(resolved):
        return True
    rel = os.path.relpath(resolved, root)
    if rel.startswith('..'):
        return False
    # dir へのリンク(末尾スラッシュ無し)も追跡ファイルの接頭辞として拾う
    return rel in tracked or any(t.startswith(rel + '/') for t in tracked)
# npm ci/build/test:visual で docs 配下に現れる生成物・依存 dir へは降下しない
SKIP_DIR_NAMES = frozenset({
    "node_modules", "dist", "test-results", "playwright-report",
})
link_re = re.compile(r'\]\(([^)]+)\)')
fail = False
paths = [os.path.join(root, f) for f in ('AGENTS.md', 'README.md', 'CONTRIBUTING.md', 'motolii/AGENTS.md')]
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
        if not link_exists(resolved):
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
      err "decision-index.md に未定義の状態語彙: 「${w}」(許可: 決定/縮小採用/延期/棄却/撤回/未統一/観察/比較中/停止線)"
    done <<< "$bad"
  fi
else
  err "docs/decision-index.md が存在しない"
fi

if [ $FAIL -eq 0 ]; then
  echo "OK: docs整合チェック全項目通過"
else
  echo "FAILED: 上記を修正する"
fi
exit $FAIL
