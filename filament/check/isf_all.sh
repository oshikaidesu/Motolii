#!/bin/zsh
# How many of Vidvox's ISF-Files (MIT) an object takes as its effect, and why the rest do not. ISF-Files at the commit
# counted with, fetched once into third_party/ISF-Files (or MOTOLII_ISF_DIR, a copy of one's own).
#   ./check/isf_all.sh   (prints the count, then each one not taken with its reason)
cd "$(dirname "$0")/.."
I=${MOTOLII_ISF_DIR:-third_party/ISF-Files}
[ -d $I/ISF ] || { mkdir -p $I && git init -q $I && git -C $I -c core.hooksPath=/dev/null fetch -q --depth 1 https://github.com/Vidvox/ISF-Files 395072d48b3ce7351ccb20a5fda54470591324df && git -C $I checkout -q FETCH_HEAD; }
all=($I/ISF/*.fs)
doc=$(mktemp -d)
{ echo "add picture $PWD/fx_sample.jpg"; for f in $all; do echo "say 1 fx ${f:a}"; done; } | ./edit.sh $doc/all.motolii > $doc/log.txt 2>&1
no=$(grep -c "is not taken" $doc/log.txt)
echo "$(( ${#all} - no )) of ${#all} taken"
grep "is not taken" $doc/log.txt | sed 's|.*/ISF-Files/ISF/||' | cut -c1-150
