#!/usr/bin/env bash
# 人なら即座に「変だ」と感じる数を、赤緑にする。閾値は reference/hygiene-budget.tsv。
# build しない。数えるだけなので数秒で終わる。
set -u

# commit の瞬間に鳴らす口。staged の中身だけを数え、HEAD と比べる。build も find もしない。
if [ "${1-}" = staged ]; then
    cd "$(dirname "$0")/.." || exit 2
    budget=motolii/reference/hygiene-budget.tsv
    warn=$(awk -F'\t' '$1=="file_lines_warn" {print $2}' "$budget")
    soft=$(awk -F'\t' '$1=="file_lines_soft" {print $2}' "$budget")
    fail=0
    # 数える家は check-hygiene 本体と同じ: doc/render の src、ui/native/src、ui/extensions、ui/lib
    counts() { # 中身を stdin で受け「全体 test」を返す
        awk '
            { total++ }
            intest {
                tl++
                o = gsub(/\{/, "{"); c = gsub(/\}/, "}")
                depth += o - c
                if (o > 0) started = 1
                if (started && depth <= 0) intest = 0
                else if (!started && /;[ \t]*$/) intest = 0
                next
            }
            /^[ \t]*#\[cfg\(test\)\]/ { intest = 1; tl++; depth = 0; started = 0 }
            END { print total+0, tl+0 }
        '
    }
    while IFS= read -r path; do
        case "$path" in
            motolii/crates/*/src/*.rs|motolii/ui/native/src/*.rs|motolii/ui/extensions/*.rs|motolii/ui/lib/*.dart) ;;
            motolii/crates/motolii-render/vism/*.wgsl|motolii/crates/motolii-render/vism/*.fs|motolii/crates/motolii-render/vism/*.frag) ;;
            *) continue ;;
        esac
        set -- $(git show ":$path" 2>/dev/null | counts)
        now=${1:-0}; tl=${2:-0}
        [ "$now" -eq 0 ] && continue
        was=$(git show "HEAD:$path" 2>/dev/null | wc -l | tr -d ' ')
        [ -z "$was" ] && was=0
        note=""
        [ "$tl" -gt 0 ] && note=" — うち test $tl 行"
        if [ "$now" -gt "$soft" ] && [ "$now" -gt "$was" ]; then
            echo "NG: $path = $now 行 (HEAD ${was}、天井 $soft)$note"
            fail=1
        elif [ "$now" -gt "$warn" ] && [ "$now" -gt "$was" ]; then
            echo "warn: $path = $now 行 (HEAD ${was}、目安 $warn)$note"
        fi
    done < <(git diff --cached --name-only --diff-filter=ACMR)
    over_warn=$({ find motolii/crates/*/src motolii/ui/native/src motolii/ui/extensions -name '*.rs' -exec wc -l {} + ; find motolii/ui/lib -name '*.dart' -exec wc -l {} + ; find motolii/crates/motolii-render/vism \( -name '*.wgsl' -o -name '*.fs' -o -name '*.frag' \) -exec wc -l {} + ; } 2>/dev/null | grep -v ' total$' | awk -v w="$warn" '$1>w' | wc -l | tr -d ' ')
    over_soft=$({ find motolii/crates/*/src motolii/ui/native/src motolii/ui/extensions -name '*.rs' -exec wc -l {} + ; find motolii/ui/lib -name '*.dart' -exec wc -l {} + ; find motolii/crates/motolii-render/vism \( -name '*.wgsl' -o -name '*.fs' -o -name '*.frag' \) -exec wc -l {} + ; } 2>/dev/null | grep -v ' total$' | awk -v l="$soft" '$1>l' | wc -l | tr -d ' ')
    echo "いま ${warn}行超 $over_warn 本 / ${soft}行超 $over_soft 本"
    if [ "$fail" -ne 0 ]; then
        cat <<'MSG'
FAILED: 既に天井を超えている file を、さらに伸ばしている。
責任ごとに割るか、motolii/reference/hygiene-budget.tsv の天井を書き換えて commit しろ(履歴に残る)。
MSG
        exit 1
    fi
    exit 0
fi

cd "$(dirname "$0")/../motolii" || exit 2
budget=reference/hygiene-budget.tsv
fail=0
val() { awk -F'\t' -v k="$1" '$1==k {print $2}' "$budget"; }
check() { # name actual
    local limit; limit=$(val "$1")
    if [ -z "$limit" ]; then echo "NG: $1 の天井が $budget に無い"; fail=1; return; fi
    if [ "$2" -gt "$limit" ]; then echo "NG: $1 = $2 > $limit"; fail=1; else echo "ok: $1 = $2 (<= $limit)"; fi
}
target_gb=$( [ -d target ] && du -sk target | awk '{print int($1/1048576)}' || echo 0 )
target_limit=$(val target_gb)
if [ "$target_gb" -gt "$target_limit" ]; then
    echo "WARN: target_gb = $target_gb > $target_limit (live cache は reference/build-artifacts.tsv で分類。総量だけでは消さない)"
else
    echo "ok: target_gb = $target_gb (<= $target_limit)"
fi
incr=$( [ -d target/debug/incremental ] && ls target/debug/incremental | wc -l | tr -d ' ' || echo 0 )
check incremental_sessions "$incr"
# 生きている家とその拡張を数える。退役した実装はGit履歴にある。
sizes() { find crates/*/src ui/native/src ui/extensions -name '*.rs' -exec wc -l {} + ; find ui/lib -name '*.dart' -exec wc -l {} + ; find crates/motolii-render/vism \( -name '*.wgsl' -o -name '*.fs' -o -name '*.frag' \) -exec wc -l {} + ; }
longest=$(sizes | grep -v ' total$' | sort -rn | head -1 | awk '{print $1}')
check longest_file_lines "$longest"
files_over=$(sizes | grep -v ' total$' | awk -v l="$(val file_lines_soft)" '$1>l' | wc -l | tr -d ' ')
check files_over_soft "$files_over"
[ "$fail" -ne 0 ] && sizes | grep -v ' total$' | awk -v l="$(val file_lines_soft)" '$1>l' | sort -rn
[ "$fail" -eq 0 ] && echo "OK: hygiene 全項目通過" || { echo "FAILED"; exit 1; }
