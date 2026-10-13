#!/bin/zsh
# Does editing by calls hold? A new document, the calls of check/doc_calls.txt (place three things, keys, a stacking
# way, undo and redo, a still after each stage), then the document opened afresh: its still against the last one made
# before it was closed, number by number. Then the wrong calls. Says what each call took.   check/doc.sh
cd "$(dirname "$0")/.."
rm -f out/doc_check.motolii out/wgpu_kit/out/e_*.png
./edit.sh out/doc_check.motolii < check/doc_calls.txt || exit 1
printf 'at 2.5\nstill out/e_7.png\n' | ./edit.sh out/doc_check.motolii
# what goes wrong for a user (check/doc_wrong.txt: a value changed again and again and undone past the beginning, files
# that are not there, things and names that are not there, values the stage cannot show): every such call is refused
# with its reason, and the document ends as check/doc_wrong.expect says
rm -f out/doc_wrong.motolii
./edit.sh out/doc_wrong.motolii < check/doc_wrong.txt > out/doc_wrong.txt
grep "NOT done" out/doc_wrong.txt
grep "^  [0-9(]" out/doc_wrong.txt | grep -v "(a number" | diff - check/doc_wrong.expect > /dev/null && echo "after the wrong calls the document is as expected: yes ($(grep -c "NOT done" out/doc_wrong.txt) calls refused)" || echo "after the wrong calls the document is as expected: NO"
python3 - <<'PY'
import numpy as np
from PIL import Image
a, b = (np.asarray(Image.open('out/wgpu_kit/out/e_%d.png' % i).convert('RGB'), dtype=int) for i in (6, 7))
d = np.abs(a - b)
print('opened afresh against before closing: mean off %.3f of 255: %s' % (d.mean(), 'the same' if d.mean() < 0.5 else 'NOT the same'))
PY
