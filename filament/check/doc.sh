#!/bin/zsh
# Does editing by calls hold? A new document, the calls of check/doc_calls.txt (place three things, keys, a stacking
# way, undo and redo, a still after each stage), then the document opened afresh: its still against the last one made
# before it was closed, number by number. Says what each call took.   check/doc.sh
cd "$(dirname "$0")/.."
rm -f out/doc_check.motolii out/wgpu_kit/out/e_*.png
./edit.sh out/doc_check.motolii < check/doc_calls.txt || exit 1
printf 'at 2.5\nstill out/e_7.png\n' | ./edit.sh out/doc_check.motolii
python3 - <<'PY'
import numpy as np
from PIL import Image
a, b = (np.asarray(Image.open('out/wgpu_kit/out/e_%d.png' % i).convert('RGB'), dtype=int) for i in (6, 7))
d = np.abs(a - b)
print('opened afresh against before closing: mean off %.3f of 255: %s' % (d.mean(), 'the same' if d.mean() < 0.5 else 'NOT the same'))
PY
