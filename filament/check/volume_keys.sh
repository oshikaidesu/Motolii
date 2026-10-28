#!/bin/zsh
# A film's Volume keyed (Alight Motion's Volume with its keys): 0 at the start, 1 two seconds on. Written, its sound is
# far quieter early than the same film written unkeyed, and about as loud late (ffmpeg's asendcmd telling its volume
# each 1/30 s, as the document reads the keys); heard while playing, the same reading (heard.h heard_loud).
#   ./check/volume_keys.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
for mode in plain keyed; do
    keys=''; [[ $mode == keyed ]] && keys='key 1 volume 0 0\nkey 1 volume 2 1\n'
    printf "add film $PWD/check/beats.mp4\n${keys}write $W/$mode.mp4\n" | ./edit.sh $W/$mode.motolii > /dev/null 2>&1
done
loud() { ffmpeg -v info -i $W/$1.mp4 -vn -af "atrim=$2:$(( $2 + 0.4 )),volumedetect" -f null - 2>&1 | awk '/mean_volume/ {print $5}'; }   # (cut out by the filter: a seek lands where it can)
early=$(( $(loud keyed 0.1) - $(loud plain 0.1) )) late=$(( $(loud keyed 1.9) - $(loud plain 1.9) ))
(( early < -9 && late > -3 )) && echo "a keyed volume is heard as its keys say: yes (early $(printf %.1f $early) dB, late $(printf %.1f $late) dB against unkeyed)" || echo "a keyed volume is heard as its keys say: NO (early $(printf %.1f $early) dB, late $(printf %.1f $late) dB against unkeyed)"
rm -rf $W
