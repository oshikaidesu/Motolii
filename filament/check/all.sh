#!/bin/zsh
# The checks a fresh clone can run, one after another; each says yes or NO in its own words, and this fails (exit 1)
# if any says NO, stops, or the ISF count falls below what README.md says. Needs the inside built (edit.sh).
#   ./check/all.sh
cd "$(dirname "$0")"
bad=0
run() {   # (a check, and the words that mean it held)
    local out; out=$(./$1 2>&1); local code=$?
    print -r -- "$out" | grep -E ': (yes|NO)( |$|\()'
    if (( code )) || print -r -- "$out" | grep -q -E ': NO( |$|\()'; then print "FAIL $1"; bad=1; fi
}
run anime_ways.sh
run glass_shift.sh
if [ -f ../out/vid/happy.mp4 ]; then run doc.sh   # (it reads a film kept on this machine only)
else print "skip doc.sh (needs filament/out/vid/happy.mp4)"; fi
taken=$(./isf_all.sh 2>&1 | head -1); print -r -- "ISF-Files: $taken"
(( ${${(s: :)taken}[1]:-0} >= 318 )) || { print "FAIL isf_all.sh (README.md says 318)"; bad=1; }
(( bad )) && { print "some checks failed"; exit 1; }
print "all checks held"
