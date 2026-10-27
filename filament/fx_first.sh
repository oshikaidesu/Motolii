#!/bin/zsh
# The values an effect of the library comes with where its own show nothing (Alight Motion's rule: an effect is seen as it
# is put on; where its numbers do nothing it comes with one that does, reports/alight_ui.md "最初の値"). On fx_tiles.sh's
# sample, each screen effect (.fs, .wgsl) is drawn as its file says; one not seen to differ from the sample (PSNR over
# 40 dB, as the window's tiles are told: skin/book/lib/workspace/effect_tiles.dart) has its knobs tried in turn, each from
# its own value toward its highest in tenths, and the first that is plainly seen (under 30 dB) is written to
# effects/first.tsv (script.cpp script_first lays it over the file's own). A transition (a knob named progress, ISF's
# way of saying one) is left as it is: its tile shows it going.   ./fx_first.sh   (Pillow and numpy: Brewfile.checks)
cd "$(dirname "$0")"
dir=$(mktemp -d)
rm -f effects/first.tsv   # (measured from the files' own values)
python3 - "$PWD" "$dir" <<'EOF'
import json, os, re, subprocess, sys
import numpy as np
from PIL import Image
here, out = sys.argv[1], sys.argv[2]

def knobs(f):   # (its numbers as the inside reads them: an ISF file's INPUTS, a .wgsl's head)
    t = open(f, encoding='utf-8', errors='replace').read()
    if f.endswith('.fs'):
        m = re.search(r'/\*(.*?)\*/', t, re.S)
        try: j = json.loads(m.group(1))
        except Exception: return []
        return [(i['NAME'], float(i.get('DEFAULT', 0) or 0), float(i.get('MIN', 0)), float(i.get('MAX', 1)))
                for i in j.get('INPUTS', []) if i.get('TYPE') == 'float' and 'MIN' in i and 'MAX' in i]
    return [(n, float(u), float(lo), float(hi)) for n, lo, hi, u in re.findall(r'^// --track@(\w+):[^,]*,([-\d.]+),([-\d.]+),([-\d.]+)', t, re.M)]

def name(f, k):   # (doc_values.h doc_screen_knob)
    return 'js_' + re.sub(r'[^A-Za-z0-9_]', '_', os.path.splitext(os.path.basename(f))[0]) + '_' + k

def draw(lines):
    head = [f'add picture {here}/fx_sample.jpg', 'set 1 size 3.6', 'key 1 x 0 -0.1', 'key 1 x 2 0.1',
            'add text /System/Library/Fonts/Supplemental/Arial Black.ttf', 'say 2 text Aa', 'say 2 fill ffffff', 'set 2 size 1', 'set 2 y -0.55', 'at 1.4']
    subprocess.run([f'{here}/edit.sh', f'{out}/m{len(lines)}.motolii'], input='\n'.join(head + lines) + '\n', text=True,
                   env=dict(os.environ, SIZE='512x512'), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

def psnr(a, b):
    x = np.asarray(Image.open(a).convert('RGB'), float); y = np.asarray(Image.open(b).convert('RGB'), float)
    e = ((x - y) ** 2).mean()
    return 99.0 if e == 0 else 10 * np.log10(255 ** 2 / e)

files = sorted(os.path.join('effects', f) for f in os.listdir('effects') if f.endswith(('.fs', '.wgsl')))
lines = [f'still {out}/plain.png']
for i, f in enumerate(files):
    lines += [f'say 0 script {here}/{f}', f'still {out}/d{i}.png', 'say 0 script none']
draw(lines)
quiet = [(i, f) for i, f in enumerate(files) if os.path.exists(f'{out}/d{i}.png') and psnr(f'{out}/d{i}.png', f'{out}/plain.png') > 40]
lines, tried = [], []
for i, f in quiet:
    ks = knobs(f)
    if any(k[0] == 'progress' for k in ks): continue
    lines.append(f'say 0 script {here}/{f}')
    for k, usual, lo, hi in ks:
        end = hi if hi != usual else lo
        for s in range(1, 11):
            v = usual + (end - usual) * s / 10
            lines += [f'set 0 {name(f, k)} {v:.6g}', f'still {out}/t{len(tried)}.png']
            tried.append((f, k, v))
        lines.append(f'set 0 {name(f, k)} {usual:.6g}')
    lines.append('say 0 script none')
draw(lines)
first, seen = {}, set()
for n, (f, k, v) in enumerate(tried):
    if f in first or not os.path.exists(f'{out}/t{n}.png'): continue
    if psnr(f'{out}/t{n}.png', f'{out}/plain.png') < 30: first[f] = (k, v)
with open('effects/first.tsv', 'w') as w:
    w.write('# fx_first.sh: the values these come with, where their own show nothing (file, knob, value)\n')
    for f, (k, v) in sorted(first.items()): w.write(f'{os.path.basename(f)}\t{k}\t{v:.6g}\n')
left = [f for i, f in quiet if f not in first and not any(k[0] == 'progress' for k in knobs(f))]
print(f'{len(files)} screen effects; {len(quiet)} showed nothing as their files say; {len(first)} given a value that shows; '
      f'{sum(any(k[0] == "progress" for k in knobs(f)) for i, f in quiet)} transitions left going; none found for {len(left)}: ' + ', '.join(os.path.basename(f) for f in left))
EOF
rm -rf "$dir"
