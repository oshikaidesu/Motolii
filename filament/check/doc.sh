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
ln -sf "/System/Library/Fonts/Supplemental/Arial Black.ttf" out/arial_black.ttf   # (a font for the typed words: the system's, by a link)
ln -sf "/System/Library/Fonts/Supplemental/Arial Unicode.ttf" out/arial_unicode.ttf
./edit.sh out/doc_wrong.motolii < check/doc_wrong.txt > out/doc_wrong.txt
grep "NOT done" out/doc_wrong.txt
grep "^  [0-9(]" out/doc_wrong.txt | grep -v "(a number" | diff - check/doc_wrong.expect > /dev/null && echo "after the wrong calls the document is as expected: yes ($(grep -c "NOT done" out/doc_wrong.txt) calls refused)" || echo "after the wrong calls the document is as expected: NO"
# the stretch between two keys goes the way said for it (check/doc_ways.txt: values read back at moments for linear,
# smooth, hold, wiggle, bounce and a way defined there; every way begins and ends at its keys' values; ways that do not,
# or that SQLite cannot work out, are refused): the replies are as check/doc_ways.expect says
rm -f out/doc_ways.motolii
./edit.sh out/doc_ways.motolii < check/doc_ways.txt > out/doc_ways.txt
grep "^  \|NOT done" out/doc_ways.txt | sed 's/, [0-9.]* ms$//' | diff - check/doc_ways.expect > /dev/null && echo "the stretches between keys go as expected: yes ($(grep -c '^  [0-9-]*\.[0-9]*$' out/doc_ways.txt) values read, $(grep -c 'NOT done' out/doc_ways.txt) calls refused)" || echo "the stretches between keys go as expected: NO"
# colours as the files hold them: a picture half white (255), half grey (144), and white typed words, on a document's stage
python3 -c "
from PIL import Image
im = Image.new('RGB', (64, 64), (255, 255, 255)); im.paste((144, 144, 144), (0, 32, 64, 64)); im.save('out/white_grey.png')"
rm -f out/doc_colour.motolii
printf 'add picture ../white_grey.png\nset 1 size 2\nadd text ../arial_black.ttf\nsay 2 fill ffffff\nsay 2 text III\nset 2 y -1.3\nset 2 size 0.4\nstill out/e_colour.png\n' | ./edit.sh out/doc_colour.motolii > /dev/null
python3 - <<'PY'
import numpy as np
from PIL import Image
a, b = (np.asarray(Image.open('out/wgpu_kit/out/e_%d.png' % i).convert('RGB'), dtype=int) for i in (6, 7))
d = np.abs(a - b)
print('opened afresh against before closing: mean off %.3f of 255: %s' % (d.mean(), 'the same' if d.mean() < 0.5 else 'NOT the same'))
c = Image.open('out/wgpu_kit/out/e_colour.png').convert('RGB'); W, H = c.size
white, grey, words = c.getpixel((W // 2, H // 2 - H // 6)), c.getpixel((W // 2, H // 2 + H // 6)), np.asarray(c.crop((0, int(H * 0.87), W, int(H * 0.95)))).reshape(-1, 3).max(axis=0)
print('colours on a document\'s stage: white 255 shown as %s, grey 144 as %s, white words as %s: %s' % (white, grey, tuple(int(v) for v in words), 'as the files hold them' if white == (255, 255, 255) and grey == (144, 144, 144) and min(words) >= 254 else 'NOT as the files hold them'))
PY
