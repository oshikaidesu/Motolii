#!/bin/zsh
# A document packed to be taken elsewhere (Alight Motion's project package; After Effects' Collect Files): its two
# pictures of one name from two folders and a film, copied into the package beside the document's copy; opened there,
# from that folder alone, every thing is made of what it holds (nothing "could not be made").
#   ./check/pack.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d) F=$PWD
mkdir -p $W/a $W/b && cp fx_sample.jpg $W/a/photo.jpg && cp fx_sample.jpg $W/b/photo.jpg && cp check/beats.mp4 $W/a/clip.mp4
said=$(printf 'add picture %s/a/photo.jpg\nadd picture %s/b/photo.jpg\nadd film %s/a/clip.mp4\npack %s/pk trip\n' $W $W $W $W | ./edit.sh $W/src.motolii 2>&1 | grep "files")
mv $W/a $W/gone_a && mv $W/b $W/gone_b   # (the first places gone: only the package's own are there to find)
opened=$(printf 'still %s/pk/look.png\n' $W | (cd $W/pk && $F/edit.sh $W/pk/trip.motolii 2>&1) | grep -c "NOT")
[[ $said == *"3 files"* && $opened == 0 && -f $W/pk/photo.jpg && -f "$W/pk/photo 2.jpg" && -f $W/pk/clip.mp4 ]] && echo "a document packed opens elsewhere whole: yes" || echo "a document packed opens elsewhere whole: NO ($said; $opened refused there)"
rm -rf $W
