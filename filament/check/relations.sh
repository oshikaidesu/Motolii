#!/bin/zsh
# (a check: relations, behaviours that set things in relation to each other (around, row, follow, push), as films:
# push (a big one goes through, the rest get out of its way), morph (a ring over a row, its strength keyed: row to ring and back), gather (strength keyed 0, 1, 0: scattered things come
# round into a ring and go again), follow (a line of followers each later than the one before), hand (a ball handed
# from one hand to the other), orbit (a moon round a planet round a sun). Then: strength 0 draws as no relation.)
#   check/relations.sh [out dir]      films and a sheet of frames in it (default: a new one)
cd "$(dirname "$0")/.."
out=${1:-$(mktemp -d)}
mkdir -p $out
SIZE=640x360
film() {   # (name, then the calls)
    local name=$1; shift
    rm -f $out/$name.mp4
    print -l -- "$@" "write $out/$name.mp4" | SIZE=$SIZE ./edit.sh $out/$name.motolii | grep -iE "NOT done|not taken" | sed "s/^/$name: /"
    for t in 0.5 1.5 2.5 3.5; do ffmpeg -loglevel error -y -ss $t -i $out/$name.mp4 -frames:v 1 -vf scale=320:-1 $out/${name}_$t.png; done
    magick $out/${name}_{0.5,1.5,2.5,3.5}.png -background '#444' -splice 4x0 +append $out/${name}_row.png
}
small() { calls+=("add shape $1" "set $2 size 0.12" "say $2 fill $3"); }   # (a small shape: kind, its number, colour)
both() { calls+=("set $1 x_by $2" "set $1 y_by $2"); }

# push: thirty-five dots in a grid, a big red one goes through them left to right
calls=("say 0 behind colour" "say 0 background f1ece2" "add behaviour push" "set 1 radius 0.55")
n=2
for gy in -0.6 -0.3 0 0.3 0.6; do for gx in -1.2 -0.8 -0.4 0 0.4 0.8 1.2; do
    small circle $n 2a62c9; calls+=("set $n x $gx"); calls+=("set $n y $gy"); both $n 1; n=$((n + 1))
done; done
small circle $n e5432d; calls+=("set $n size 0.3"); calls+=("key $n x 0 -1.8"); calls+=("key $n x 4 1.8"); calls+=("set 1 leader $n")
film push $calls

# gather: ten shapes scattered, strength 0 -> 1 -> 0 round a ring
calls=("say 0 behind colour" "say 0 background f1ece2" "add behaviour around" "set 1 radius 0.6" "key 1 strength 0 0" "key 1 strength 1.5 1" "key 1 strength 2.5 1" "key 1 strength 4 0")
for k in {2..11}; do
    small $( (( k % 2 )) && echo rect || echo circle ) $k $( (( k % 3 )) && echo 151515 || echo e5432d )
    calls+=("set $k x $(( (k * 37 % 19) / 9.5 - 1 ))" "set $k y $(( (k * 23 % 17) / 9.0 - 0.9 ))"); both $k 1
done
film gather $calls

# follow: a leader goes round, eight followers each 0.12 s later
calls=("say 0 behind colour" "say 0 background f1ece2" "add behaviour follow" "set 1 lag 0.12")
small circle 2 e5432d; calls+=("key 2 x 0 -1.4"); calls+=("key 2 x 1 0.3"); calls+=("key 2 x 2 1.2"); calls+=("key 2 x 3 0"); calls+=("key 2 x 4 -1.4"); calls+=("key 2 y 0 0"); calls+=("key 2 y 1 0.7"); calls+=("key 2 y 2 -0.2"); calls+=("key 2 y 3 -0.7"); calls+=("key 2 y 4 0"); calls+=("set 1 leader 2")
for k in {3..10}; do small circle $k 151515; calls+=("set $k x 0" "set $k y 0"); both $k 1; done
film follow $calls

# hand: two hands go up and down out of step, the ball goes with the left then the right
calls=("say 0 behind colour" "say 0 background f1ece2" "add behaviour follow" "set 1 lag 0")
small rect 2 151515; calls+=("set 2 x -0.8"); calls+=("key 2 y 0 -0.3"); calls+=("key 2 y 1 0.3"); calls+=("key 2 y 2 -0.3"); calls+=("key 2 y 3 0.3"); calls+=("key 2 y 4 -0.3")
small rect 3 151515; calls+=("set 3 x 0.8"); calls+=("key 3 y 0 0.3"); calls+=("key 3 y 1 -0.3"); calls+=("key 3 y 2 0.3"); calls+=("key 3 y 3 -0.3"); calls+=("key 3 y 4 0.3")
small circle 4 e5432d; calls+=("set 4 x 0"); calls+=("set 4 y 0.15"); both 4 1; calls+=("set 1 leader 2"); calls+=("set 1 leader_b 3"); calls+=("key 1 hand 1 0"); calls+=("key 1 hand 2.5 1")
film hand $calls

# orbit: a sun, a planet round it, a moon round the planet
calls=("say 0 behind colour" "say 0 background 151515" "add behaviour around" "add behaviour around")
small circle 3 f7d046; calls+=("set 3 size 0.3"); calls+=("set 1 leader 3"); calls+=("set 1 radius 0.7"); calls+=("key 1 offset 0 0"); calls+=("key 1 offset 4 1")
small circle 4 2a62c9; both 4 1; calls+=("set 2 leader 4"); calls+=("set 2 radius 0.22"); calls+=("key 2 offset 0 0"); calls+=("key 2 offset 4 3")
small circle 5 f1ece2; calls+=("set 5 size 0.06"); both 5 2
film orbit $calls

# morph: six things in a row, a ring over the row; the ring's strength keyed 0, 1, 0: row to ring and back, unbroken
calls=("say 0 behind colour" "say 0 background f1ece2" "add behaviour row" "set 1 gap 0.35" "say 1 members 2 3 4 5 6 7")
for k in {2..7}; do small $( (( k % 2 )) && echo rect || echo circle ) $k $( (( k % 3 )) && echo 151515 || echo e5432d ); calls+=("set $k x $(( (k * 37 % 19) / 9.5 - 1 ))" "set $k y $(( (k * 23 % 17) / 9.0 - 0.9 ))"); done
calls+=("add behaviour around" "set 8 radius 0.55" "say 8 members 2 3 4 5 6 7" "say 8 under 1 1 1 1 1 1" "key 8 strength 0 0" "key 8 strength 1.5 1" "key 8 strength 2.5 1" "key 8 strength 4 0")
for k in {2..7}; do both $k 8; done
film morph $calls

magick $out/{push,gather,follow,hand,orbit,morph}_row.png -background '#444' -splice 0x4 -append $out/relations_sheet.png
# strength 0 is no relation: the same picture as the things alone
print -l "add shape circle" "set 1 size 0.2" "set 1 x 0.4" "add behaviour around" "set 2 strength 0" "still $out/zero.png" "set 1 x_by 2" "set 1 y_by 2" "still $out/zero_related.png" | SIZE=$SIZE ./edit.sh $out/zero.motolii | grep -iE "NOT done"
cmp -s $out/zero.png $out/zero_related.png && echo "ok   strength 0 draws as no relation" || { echo "FAIL strength 0 draws otherwise"; bad=1; }
echo "films and sheet: $out"
(( bad )) && echo FAIL || echo PASS
