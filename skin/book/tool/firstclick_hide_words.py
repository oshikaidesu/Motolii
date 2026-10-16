# The first-click pictures with every word hidden (reports/firstclick/NN_nowords.png): each word's box filled with the
# colour just beside it, so a control is found by its glyph alone (the window speaks by glyphs, not by explaining text).
#   python3 tool/firstclick_hide_words.py
import json, os
from PIL import Image, ImageDraw

here = os.path.join(os.path.dirname(__file__), '..', '..', '..', 'reports', os.environ.get('FC_DIR', 'firstclick'))  # (FC_DIR=firstclick_b: another set of tasks)
for t in json.load(open(os.path.join(here, 'tasks.json'))):
    im = Image.open(os.path.join(here, t['picture'])).convert('RGB')
    d = ImageDraw.Draw(im)
    for l, top, r, b in t.get('words', []):
        x, y = max(0, int(l) - 3), min(im.height - 1, max(0, int((top + b) / 2)))
        d.rectangle([l - 1, top - 1, r + 1, b + 1], fill=im.getpixel((x, y)))
    im.save(os.path.join(here, t['picture'].replace('.png', '_nowords.png')))
print('hidden words in', len(json.load(open(os.path.join(here, 'tasks.json')))), 'pictures')
