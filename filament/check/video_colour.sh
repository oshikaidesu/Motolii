#!/bin/zsh
# Are a film's colours ours as ffmpeg makes them from the same file? One moment of each awkward clip, ours (the film
# filling the view, nothing done to its colours) beside ffmpeg's, number by number.   check/video_colour.sh [folder]
cd "$(dirname "$0")/.."
W=out/wgpu_kit/out/colour_check
rm -rf $W && mkdir -p $W
KIT_EXTRA="$(pkg-config --cflags --libs gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0)" KIT_SRC=check/video_colour.mm BUILDONLY=1 ./dawn_probe/kit_wgpu.sh colour_check || exit 1
for f in ${1:-out/vid_odd}/*; do
    n=${f:t:r}
    ffmpeg -hide_banner -loglevel error -y -ss 1.0 -i $f -frames:v 1 -vf "scale=w=640:h=640:force_original_aspect_ratio=decrease:force_divisible_by=2:flags=bilinear,format=rgb24" $W/${n}_ref.png
    size=($(python3 -c "from PIL import Image; print(*Image.open('$W/${n}_ref.png').size)"))
    (cd out/wgpu_kit && BACKEND=webgpu perl -e 'alarm 30; exec @ARGV' ../colour_check_wgpu ../../$f 1.0 out/colour_check/${n}_ours.png $size > /dev/null 2>&1)
    echo "$n\t$(ffprobe -v error -select_streams v:0 -show_entries stream=pix_fmt,color_range,color_space,color_transfer -of csv=p=0 $f)"
done > $W/files.tsv
python3 - $W <<'PY'
import sys, numpy as np
from PIL import Image, ImageFilter
W = sys.argv[1]
print('film\twhat the file says\tmean off (of 255)\t99 in 100 within\tpass (mean under 2)')
for line in open(W + '/files.tsv'):
    n, says = line.rstrip('\n').split('\t')
    try:
        a, b = (np.asarray(Image.open(f'{W}/{n}_{k}.png').convert('RGB').filter(ImageFilter.BoxBlur(3)), dtype=float) for k in ('ours', 'ref'))
    except FileNotFoundError:
        print(f'{n}\t{says}\tno picture\t\tNO'); continue
    d = np.abs(a - b)
    print('%s\t%s\t%.2f\t%.1f\t%s' % (n, says, d.mean(), np.percentile(d, 99), 'yes' if d.mean() < 2 else 'NO'))
PY
