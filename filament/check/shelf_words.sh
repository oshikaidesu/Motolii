#!/bin/zsh
# An ISF effect's shelf word: VIDVOX's own words for a kind said as the others say it ("Geometry Adjustment" Geometry,
# "Halftone Effect" Halftone), one chip a kind, as the shelf already reads "Distortion Effect" Distort; a transition on
# Transition, as After Effects keeps them (isf.cpp; the
# 50,000-effects table in reports/discoverability.md: category spellings that disagree).
#   ./check/shelf_words.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
said=$(printf "shelf\n" | ./edit.sh $W/s.motolii 2>/dev/null | awk -F'\t' '$2 ~ /\.fs$/ {print $3}' | sort -u)
odd=$(echo "$said" | grep -c " Effect$\| Adjustment$\|	")
all=$(printf "shelf\n" | ./edit.sh $W/s.motolii 2>/dev/null)
geo=$(echo "$all" | awk -F'\t' '$1 == "3d Rotate" {print $3}') wipe=$(echo "$all" | awk -F'\t' '$1 == "Circle Open" {print $3}') moves=$(echo "$all" | awk -F'\t' '$3 == "Transition"' | wc -l | tr -d ' ')
[ "$odd" = 0 ] && [ "$geo" = Geometry ] && [ "$wipe" = Transition ] && [ "$moves" -ge 60 ] && echo "an ISF effect's shelf word is one a kind: yes" || echo "an ISF effect's shelf word is one a kind: NO ($odd words still with Effect or Adjustment; 3d Rotate on $geo, Geometry asked; Circle Open on $wipe, Transition asked; $moves transitions, 60 or more asked)"
rm -rf $W
