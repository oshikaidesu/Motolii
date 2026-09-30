#!/usr/bin/env bash
# The independent police: Semgrep rules (semgrep/motolii.yml) over Rust, Dart and Swift, watching a few boundaries that
# scripts/check-workspace.py and motolii/reference/owned-budget.tsv also watch. It adds no law; a rule that cannot be proven on a
# past bug (semgrep/history.tsv) does not belong here.
#
#   scripts/check-semgrep.sh             rule tests + the tree (raw-GPU rule: only what is new against $BASE)
#   scripts/check-semgrep.sh --history   + each rule red on the commit that had the bug, green on the one that fixed it
# Needs `semgrep` (pip install semgrep); SEMGREP=/path overrides it. BASE defaults to origin/main.
set -euo pipefail
cd "$(dirname "$0")/.."
sg() { "${SEMGREP:-semgrep}" --metrics=off --disable-version-check --quiet "$@"; }
BASE=${BASE:-origin/main}
fail=0

sg --test semgrep/ || fail=1

# hits <rule-suffix|-all> <semgrep json>: how many findings; -all = every rule except the raw-GPU one (legacy, below)
hits() { jq --arg r "$1" '[.results[] | select(if $r == "-all" then (.check_id | endswith("motolii-rust-raw-gpu-infrastructure") | not) else (.check_id | endswith($r)) end)] | length'; }

# All rules but the legacy one must be silent on the tree.
out=$(sg --config semgrep/motolii.yml --json . 2>/dev/null) || true
n=$(hits -all <<<"$out")
if [ "$n" != 0 ]; then
    jq -r '.results[] | "\(.path):\(.start.line): \(.check_id)"' <<<"$out"
    fail=1
fi

# The raw-GPU ceiling is already exceeded by code that predates this check (owned_budget test): judge only what is new.
if git rev-parse -q --verify "$BASE" >/dev/null; then
    base=$(git merge-base HEAD "$BASE")
    wt=$(mktemp -d)
    trap 'git worktree remove --force "$wt" 2>/dev/null || true' EXIT
    git worktree add -q --detach "$wt" HEAD
    out=$(cd "$wt" && sg --config "$OLDPWD/semgrep/motolii.yml" --baseline-commit "$base" --json . 2>/dev/null) || true
    m=$(hits motolii-rust-raw-gpu-infrastructure <<<"$out")
    [ "$m" = 0 ] || { echo "motolii-rust-raw-gpu-infrastructure: $m new hit(s) since $BASE"; fail=1; }
else
    echo "note: $BASE not found; raw-GPU rule skipped"
fi

if [ "${1-}" = --history ]; then
    count() { # count <rev> <rule>: findings of <rule> in the motolii/ tree of <rev>
        local d; d=$(mktemp -d "${TMPDIR:-/tmp}/sgh.XXXXXX")
        git archive "$1" motolii | tar -x -C "$d" --include='motolii/crates/*' --include='motolii/ui/lib/*' --include='motolii/ui/macos/*' --include='motolii/ui/native/src/*' 2>/dev/null
        (cd "$d" && sg --no-git-ignore --config "$OLDPWD/semgrep/motolii.yml" --json . 2>/dev/null) | hits "$2"
        rm -rf "$d"
    }
    while IFS=$'\t' read -r rule mode red green what; do
        case "$rule" in '#'*|'') continue ;; esac
        if [ "$mode" = all ]; then
            r=$(count "$red" "$rule"); g=$(count "$green" "$rule")
            [ "$r" -gt 0 ] && [ "$g" = 0 ] && ok=PASS || { ok=FAIL; fail=1; }
            echo "$ok $rule: red($red)=$r green($green)=$g"
        else
            wt=$(mktemp -d); git worktree add -q --detach "$wt" "$red"
            new=$(cd "$wt" && sg --config "$OLDPWD/semgrep/motolii.yml" --baseline-commit "$green" --json . 2>/dev/null | hits "$rule")
            git worktree remove --force "$wt"
            [ "$new" -gt 0 ] && ok=PASS || { ok=FAIL; fail=1; }
            echo "$ok $rule: $new new hit(s) in $red over $green"
        fi
    done < semgrep/history.tsv
fi

[ "$fail" = 0 ] && echo "semgrep: PASS" || { echo "semgrep: FAIL"; exit 1; }
