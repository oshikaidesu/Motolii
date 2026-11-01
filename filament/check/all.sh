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
run absent_shapes.sh
run absent_blurred.sh
run clip_matte.sh
run fog_near.sh
run film_way.sh
run volume_keys.sh
run motion_blur.sh
run camera_switch.sh
run camera_parent.sh
run camera_motion.sh
run fx_out_keys.sh
run fx_transition.sh
run fx_colour.sh
run fx_sheet.sh
run fx_world.sh
run deep.sh
run world_view.sh
run solid_copies.sh
run fx_reach.sh
run fx_chains.sh
run fx_moved.sh
run extract.sh
run shelf_words.sh
run pack.sh
run letters_move.sh
run ramp_stops.sh
run ramp_blend.sh
run blend_reads.sh
twin=$(./interp_twin.sh 2>&1); print -r -- "$twin" | grep -E ': the same|differ'   # (the eases the same as the past repository's own, number by number)
print -r -- "$twin" | grep -q 'not checked' && print "skip interp_twin.sh (the past repository is private)" || (( $(print -r -- "$twin" | grep -c ': the same') >= 1 )) || { print "FAIL interp_twin.sh (its last lines:)"; print -r -- "$twin" | tail -25 | sed 's/^/    /'; bad=1; }
rel=$(./relations.sh 2>&1); print -r -- "relations: ${${(f)rel}[-1]}"   # (relations, behaviours between things: PASS or FAIL at its end)
[[ ${${(f)rel}[-1]} == PASS ]] || { print "FAIL relations.sh"; bad=1; }
taken=$(./isf_all.sh 2>&1 | head -1); print -r -- "ISF-Files: $taken"
(( ${${(s: :)taken}[1]:-0} >= 318 )) || { print "FAIL isf_all.sh (README.md says 318)"; bad=1; }
(( bad )) && { print "some checks failed"; exit 1; }
print "all checks held"
