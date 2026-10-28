#!/bin/zsh
# How a film is written, as the document says (render_height, render_rate: Alight Motion's export rows): a second of a
# moving rect written as it comes is the stage's size at 60 a second; told 360 high at 30, it is 360 high, its shape
# kept, at 30 (ffprobe reads both).
#   ./check/film_way.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
for mode in plain said; do
    extra=""; [[ $mode == said ]] && extra='set 0 render_height 360\nset 0 render_rate 30\nset 0 render_mbps 2\n'
    printf "add shape rect\nkey 1 x 0 -1\nkey 1 x 1 1\n${extra}write $W/$mode.mp4\n" | ./edit.sh $W/$mode.motolii > /dev/null 2>&1
done
plain=$(ffprobe -v error -select_streams v:0 -show_entries stream=width,height,r_frame_rate -of csv=p=0 $W/plain.mp4)
said=$(ffprobe -v error -select_streams v:0 -show_entries stream=width,height,r_frame_rate -of csv=p=0 $W/said.mp4)
rm -rf $W
w=${plain%%,*} rest=${plain#*,} h=${rest%%,*}
[[ $plain == *,60/1 && $said == $(( (360 * w / h) / 2 * 2 )),360,30/1 ]] && echo "a film is written as the document says: yes ($plain, then $said)" || echo "a film is written as the document says: NO ($plain, then $said)"
