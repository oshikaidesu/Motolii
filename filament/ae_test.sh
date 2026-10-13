#!/bin/zsh
# Is an After Effects file played by ae.mm the same picture as ThorVG draws alone? Every example film ThorVG ships
# (third_party/thorvg_example/res/lottie, After Effects' own exports) at three moments, ours (ThorVG on Filament's
# device, shown by Filament) against ae_ref (ThorVG on the CPU, nothing else), number by number.
#   ./ae_test.sh            ->  out/wgpu_kit/out/ae_test.tsv, a line for each film; the sum at the end
# A film passes if its colours are off by less than 2 of 255 on average and fewer than 1 in 100 of its points are far off (32 or more).
set -e
cd "$(dirname "$0")"
F=third_party/filament_src
T=third_party/thorvg_src
BUILDONLY=1 ./dawn_probe/kit_wgpu.sh ae
clang++ -std=c++20 -O2 -I $T/inc ae_ref.cpp $T/build_dawn/src/libthorvg-1.a -L $F/out/wgpu_install/lib/arm64 -lwebgpu_dawn \
    -framework Cocoa -framework Metal -framework QuartzCore -framework IOSurface -framework IOKit -framework CoreVideo -o out/ae_ref
cd out/wgpu_kit
W=out/ae_test
rm -rf $W && mkdir -p $W
for f in ${@:-../../third_party/thorvg_example/res/lottie/*.json}; do
    n=${f:t:r}
    BACKEND=webgpu ../ae_wgpu $f stills $W/${n}_ours 0.1 0.5 0.9 > /dev/null 2>&1 || { echo "$n\tours failed"; continue }
    size=($(python3 -c "from PIL import Image; print(*Image.open('$W/${n}_ours_0.png').size)"))
    ../ae_ref $f $size $W/${n}_ref 0.1 0.5 0.9 || { echo "$n\tref failed"; continue }
done | tee out/ae_test.failed
python3 - $W <<'PY'
import sys, glob, os
import numpy as np
from PIL import Image
W = sys.argv[1]
rows = []
for ours in sorted(glob.glob(W + '/*_ours_0.png')):
    n = os.path.basename(ours)[:-len('_ours_0.png')]
    mean = far = 0.0
    try:
        for i in range(3):
            a = np.asarray(Image.open(f'{W}/{n}_ours_{i}.png').convert('RGB'), dtype=np.int16)
            b = np.asarray(Image.open(f'{W}/{n}_ref_{i}.ppm').convert('RGB'), dtype=np.int16)
            d = np.abs(a - b)
            mean = max(mean, float(d.mean()))
            far = max(far, 100.0 * float((d.max(axis=2) >= 32).mean()))
    except FileNotFoundError:
        continue
    rows.append((n, a.shape[1], a.shape[0], mean, far, mean < 2.0 and far < 1.0))
rows.sort(key=lambda r: -r[4])
with open('out/ae_test.tsv', 'w') as f:
    f.write('film\twide\thigh\tworst mean off (of 255)\tworst far off (%)\tpass\n')
    for r in rows: f.write('%s\t%d\t%d\t%.3f\t%.3f\t%s\n' % (r[0], r[1], r[2], r[3], r[4], 'yes' if r[5] else 'NO'))
ok = sum(r[5] for r in rows)
print('films compared %d, pass %d, fail %d' % (len(rows), ok, len(rows) - ok))
for r in rows[:12]: print('  %-34s mean %.3f  far %.3f %%  %s' % (r[0], r[3], r[4], 'pass' if r[5] else 'FAIL'))
PY
