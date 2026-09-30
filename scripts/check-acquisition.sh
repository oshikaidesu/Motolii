#!/usr/bin/env bash
# Technology Acquisition Gate, at the commit: new code of any size that is a whole new piece (a new source file of 150+ lines in
# the houses) says how it came to be. docs/known-implementation-adoption-model.md §0 is the form and the reasons.
#
#   scripts/check-acquisition.sh --msg FILE     .githooks/commit-msg: the staged diff against the message being written
#   scripts/check-acquisition.sh --range A..B   every commit in the range (CI)
#
# A new file needs one trailer (git's own format, parsed by git interpret-trailers) in the message:
#   Acquisition: Reuse|Wrap|Adapt|Extend|Semantics|Build — <what was found / the owned meaning>
# and Build, which is the last resort, also:  Why-build: <the concrete requirement no existing technology meets>
set -euo pipefail
cd "$(dirname "$0")/.."

HOUSES='^motolii/(crates/[^/]+/src|ui/native/src|ui/lib)/.*\.(rs|dart|swift|wgsl)$'
NOT_CODE='(^|/)(tests?|fixtures|examples|generated)/|_tests?\.(rs|dart)$|tests\.rs$'
MIN_LINES=150
# reasons that are not reasons (model §0): easier, smaller, one more dependency, a different shape
EXCUSE='(simpler|easier|quick|small enough|tiny|few lines|avoid(ing)? (a )?dependenc|no (new )?dependenc|different shape|doesn.?t fit|簡単|小さい|少ない|依存を増やしたくない|dependencyを増やしたくない|形が違う|形が合わない|手軽|面倒)'

fail=0
check() { # check <label> <message> <numstat of added files>
    local label=$1 message=$2 added=$3 big=""
    while IFS=$'\t' read -r plus _ path; do
        [[ "$plus" =~ ^[0-9]+$ ]] || continue
        [ "$plus" -ge "$MIN_LINES" ] || continue
        [[ "$path" =~ $HOUSES ]] || continue
        [[ "$path" =~ $NOT_CODE ]] && continue
        big+="    $path ($plus lines)"$'\n'
    done <<<"$added"
    [ -n "$big" ] || return 0
    # trailers are git's own (git interpret-trailers); only what they must say is ours
    local trailers line
    trailers=$(git stripspace --strip-comments <<<"$message" | git interpret-trailers --parse --unfold)
    line=$(grep -E '^Acquisition: (Reuse|Wrap|Adapt|Extend|Semantics|Build)( |$)' <<<"$trailers" | head -1 || true)
    if [ -z "$line" ] || [ "${#line}" -lt 34 ]; then
        printf '%s: new code without an Acquisition trailer.\n%s' "$label" "$big"
        echo "  Add a trailer:  Acquisition: Reuse|Wrap|Adapt|Extend|Semantics|Build — <existing technology found / owned meaning>"
        echo "  Motolii owns meaning, not technology: search first (docs/known-implementation-adoption-model.md §0)."
        fail=1
        return 0
    fi
    if [[ "$line" =~ ^Acquisition:\ Build ]]; then
        local why
        why=$(grep -E '^Why-build: .{30,}' <<<"$trailers" | head -1 || true)
        if [ -z "$why" ]; then
            printf '%s: Build is the last resort and needs a Why-build line (the concrete requirement no existing technology meets).\n%s' "$label" "$big"
            fail=1
        elif grep -qiE "$EXCUSE" <<<"$why"; then
            printf '%s: that Why-build is not a reason (easier / smaller / no new dependency / a different shape): %s\n' "$label" "$why"
            fail=1
        fi
    fi
}

case "${1-}" in
    --msg)
        message=$(grep -v '^#' "$2" || true)
        check "commit" "$message" "$(git diff --cached --numstat --diff-filter=A)"
        ;;
    --range)
        for c in $(git rev-list --no-merges "$2" 2>/dev/null); do
            check "$(git log -1 --format='%h %s' "$c" | cut -c1-70)" "$(git log -1 --format=%B "$c")" "$(git diff-tree --no-commit-id --numstat -r --diff-filter=A "$c" 2>/dev/null)"
        done
        ;;
    *) echo "usage: $0 --msg FILE | --range A..B"; exit 2 ;;
esac
[ "$fail" = 0 ] && echo "acquisition: PASS" || { echo "acquisition: FAIL"; exit 1; }
