#!/bin/zsh
# (a check: effects/content_aware_scale.wgsl against the real thing, liblqr (GIMP's and ImageMagick's seam carving), on
# fx_sample.jpg made 40, 60, 80 hundredths as wide, both drawn across the frame again and compared at 128 points: how far
# from liblqr ours is, beside how far the picture left as it is would be. PASS: ours nearer at every width)
#   check/content_aware_scale.sh
cd "$(dirname "$0")/.."
out=$(mktemp -d)
K=js_content_aware_scale
magick fx_sample.jpg -resize 512x512! $out/plain.png
far() { magick compare -metric RMSE "$1" "$2" null: 2>&1 | sed -E 's/.*\(([0-9.e-]+)\).*/\1/'; }
small() { magick "$1" -resize 128x128! "$2"; }
bad=0
for keep in 0.8 0.6 0.4; do
    magick $out/plain.png -liquid-rescale $(( keep * 100 ))x100%! -resize 512x512! $out/real_$keep.png
    printf "add picture $PWD/fx_sample.jpg\nset 1 size 3.6\nsay 0 script $PWD/effects/content_aware_scale.wgsl\nset 0 ${K}_keep $keep\nset 0 ${K}_keep_y 1\nset 0 ${K}_stretch 1\nat 0\nstill $out/ours_$keep.png\n" |
        SIZE=512x512 ./edit.sh $out/doc_$keep.motolii | grep -i "not done\|not taken"
    for f in plain real_$keep ours_$keep; do small $out/$f.png $out/s_$f.png; done
    ours=$(far $out/s_ours_$keep.png $out/s_real_$keep.png) left=$(far $out/s_plain.png $out/s_real_$keep.png)
    near=$(( ours < left ))
    (( near )) || bad=1
    printf "%s keep %s: ours %.3f from liblqr, the picture as it is %.3f\n" "$( (( near )) && echo 'ok  ' || echo FAIL)" $keep $ours $left
done
echo "pictures: $out"
(( bad )) && echo FAIL || echo PASS
