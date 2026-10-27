#!/bin/zsh
# The checks a fresh clone can run, one after another; each says yes or NO in its own words, and this fails (exit 1)
# if any says NO, stops, or the ISF count falls below what README.md says. Needs the inside built (edit.sh).
#   ./check/all.sh
cd "$(dirname "$0")"
bad=0
run() {   # (a check, and the words that mean it held)
    local out; out=$(./$1 2>&1); local code=$?
    print -r -- "$out" | grep -E ': (yes|NO)( |$|\()'
    if (( code )) || print -r -- "$out" | grep -q -E ': NO( |$|\()'; then print "FAIL $1 (its last lines:)"; print -r -- "$out" | tail -25 | sed 's/^/    /'; bad=1; fi
}
run anime_ways.sh
run glass_shift.sh
run doc.sh
twin=$(./interp_twin.sh 2>&1); print -r -- "$twin" | grep -E ': the same|differ'   # (the eases the same as the past repository's own, number by number)
(( $(print -r -- "$twin" | grep -c ': the same') >= 2 )) || { print "FAIL interp_twin.sh (its last lines:)"; print -r -- "$twin" | tail -25 | sed 's/^/    /'; bad=1; }
rel=$(./relations.sh 2>&1); print -r -- "relations: ${${(f)rel}[-1]}"   # (relations, behaviours between things: PASS or FAIL at its end)
[[ ${${(f)rel}[-1]} == PASS ]] || { print "FAIL relations.sh"; bad=1; }
taken=$(./isf_all.sh 2>&1 | head -1); print -r -- "ISF-Files: $taken"
(( ${${(s: :)taken}[1]:-0} >= 318 )) || { print "FAIL isf_all.sh (README.md says 318)"; bad=1; }
(( bad )) && { print "some checks failed"; exit 1; }
print "all checks held"
