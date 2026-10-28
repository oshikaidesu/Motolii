#!/bin/zsh
# How a film is written, as the document says (render_height, render_rate: Alight Motion's export rows): a second of a
# moving rect written as it comes is the stage's size at 60 a second; told 360 high at 30, it is 360 high, its shape
# kept, at 30 (ffprobe reads both); as JPEGs, as many pictures as that film has, of its size; as a GIF, 15 a second.
#   ./check/film_way.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
for mode in plain said; do
    extra=""; [[ $mode == said ]] && extra='set 0 render_height 360\nset 0 render_rate 30\nset 0 render_mbps 2\n'
    printf "add shape rect\nkey 1 x 0 -1\nkey 1 x 1 1\n${extra}write $W/$mode.mp4\n" | ./edit.sh $W/$mode.motolii > /dev/null 2>&1
done
plain=$(ffprobe -v error -select_streams v:0 -show_entries stream=width,height,r_frame_rate -of csv=p=0 $W/plain.mp4)
said=$(ffprobe -v error -select_streams v:0 -show_entries stream=width,height,r_frame_rate -of csv=p=0 $W/said.mp4) frames=$(ffprobe -v error -select_streams v:0 -show_entries stream=nb_frames -of csv=p=0 $W/said.mp4)
printf "add shape rect\nkey 1 x 0 -1\nkey 1 x 1 1\nset 0 render_height 360\nset 0 render_rate 30\nwrite $W/each.jpg\n" | ./edit.sh $W/each.motolii > /dev/null 2>&1
printf "add shape rect\nkey 1 x 0 -1\nkey 1 x 1 1\nwrite $W/moving.gif\n" | ./edit.sh $W/gif.motolii > /dev/null 2>&1
gif=$(ffprobe -v error -select_streams v:0 -show_entries stream=codec_name,r_frame_rate -of csv=p=0 $W/moving.gif 2>/dev/null)
jpegs=($W/each_*.jpg(N)) first=$(ffprobe -v error -show_entries stream=width,height -of csv=p=0 $W/each_0001.jpg 2>/dev/null)
w=${plain%%,*} rest=${plain#*,} h=${rest%%,*}
[[ $plain == *,60/1 && $said == $(( (360 * w / h) / 2 * 2 )),360,30/1 && ${#jpegs} == $frames && $first == ${said%,*} && $gif == gif,15/1 ]] && echo "a film is written as the document says: yes ($plain, then $said, ${#jpegs} JPEGs $first, GIF $gif)" || echo "a film is written as the document says: NO ($plain, then $said, ${#jpegs} JPEGs $first, GIF $gif)"
rm -rf $W
