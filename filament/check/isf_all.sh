#!/bin/zsh
# How many of Vidvox's ISF-Files (research_notes/isf/ISF-Files, MIT) an object takes as its effect, and why the rest do not.
#   ./check/isf_all.sh   (prints the count, then each one not taken with its reason)
cd "$(dirname "$0")/.."
all=(../research_notes/isf/ISF-Files/ISF/*.fs)
doc=$(mktemp -d)
{ echo "add picture $PWD/fx_sample.jpg"; for f in $all; do echo "say 1 fx ${f:a}"; done; } | ./edit.sh $doc/all.motolii > $doc/log.txt 2>&1
no=$(grep -c "is not taken" $doc/log.txt)
echo "$(( ${#all} - no )) of ${#all} taken"
grep "is not taken" $doc/log.txt | sed 's|.*/ISF-Files/ISF/||' | cut -c1-150
