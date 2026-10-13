#!/bin/zsh
# The show piece: the poster film, the dance film and the ring film cut together on the bars (each is 8 seconds of 16
# beats, so a cut from one to another at the same moment keeps the beat), 16 seconds, with the drum.
#   ./show.sh  ->  out/show.mp4
cd "$(dirname "$0")"
./poster.sh film || exit 1
./dance.sh film || exit 1
./ring.sh film || exit 1
# which film (0 poster, 1 dance, 2 ring), from which second to which: four seconds each, then two, then one by one
cuts=("0 0 4" "1 4 8" "2 0 2" "0 4 6" "1 0 1" "2 2 3" "0 6 7" "1 1 2")
graph=""; i=0
for c in $cuts; do set -- ${=c}; graph+="[$1:v]trim=$2:$3,setpts=PTS-STARTPTS[v$i];"; i=$((i+1)); done
for j in {0..$((i-1))}; do graph+="[v$j]"; done
graph+="concat=n=$i:v=1:a=0[v];[0:a][0:a]concat=n=2:v=0:a=1[a]"
ffmpeg -loglevel error -y -i out/poster.mp4 -i out/dance.mp4 -i out/ring.mp4 -filter_complex "$graph" -map "[v]" -map "[a]" -c:v libx264 -crf 17 -pix_fmt yuv420p -r 60 -c:a aac -b:a 160k -movflags +faststart out/show.mp4 || exit 1
echo "wrote out/show.mp4"
