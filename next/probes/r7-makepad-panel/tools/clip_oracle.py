"""窓の実画素から「箱の縁で切れているグリフ」を数える。

widget の内部を覗かない — 箱の矩形(/d)と描かれた画素(/g)だけを見る。
縁の行/列に、箱の中の地色から十分離れた画素(=インク)が乗っていたら、
そのグリフは箱の外へ続いていた、と読む。

Label は除く: makepad は Label の矩形に実寸を返さない(「MOTOLII」が 10x10)。
"""
import json, sys, subprocess, collections
from PIL import Image

port = sys.argv[1]
base = f'http://127.0.0.1:{port}'
def get(p): return subprocess.run(['curl','-s',base+p],capture_output=True,text=True).stdout

dump = get('/d')
im = Image.open(json.loads(get('/g'))['png']).convert('RGB')
W, H = im.size
SCALE = 2
SKIP_TYPES = {'Label'}
EDGE_INK = 3      # 縁にこれだけインクが乗ったら切れていると読む
EDGE_FULL = 0.60  # 縁のほぼ全部がインク = 箱の境界が丸めで隣の色に読めているだけ(偽陽性)
DIFF = 28         # 地色からこれだけ離れたらインク

rows = []
for line in dump.split('\n'):
    p = line.split()
    if len(p) < 8: continue
    try: x, y, w, h = (float(v) for v in p[-4:])
    except ValueError: continue
    rows.append((p[-6], p[-5], x, y, w, h))

def edge_ink(x, y, w, h):
    X, Y, Wd, Hd = int(x*SCALE), int(y*SCALE), int(w*SCALE), int(h*SCALE)
    if Wd < 12 or Hd < 12 or X < 0 or Y < 0 or X+Wd > W or Y+Hd > H: return None
    ins = collections.Counter()
    for yy in range(Y+2, Y+Hd-2):
        for xx in range(X+2, X+Wd-2): ins[im.getpixel((xx, yy))] += 1
    if not ins: return None
    bg = ins.most_common(1)[0][0]
    ink = lambda xx, yy: max(abs(im.getpixel((xx, yy))[i]-bg[i]) for i in range(3)) > DIFF
    return {'top':    sum(ink(xx, Y)       for xx in range(X, X+Wd)),
            'bottom': sum(ink(xx, Y+Hd-1)  for xx in range(X, X+Wd)),
            'left':   sum(ink(X, yy)       for yy in range(Y, Y+Hd)),
            'right':  sum(ink(X+Wd-1, yy)  for yy in range(Y, Y+Hd))}

checked, hits = 0, []
for name, ty, x, y, w, h in rows:
    if ty in SKIP_TYPES or w > 60 or h > 60: continue
    e = edge_ink(x, y, w, h)
    if e is None: continue
    checked += 1
    span = {'top': int(w*SCALE), 'bottom': int(w*SCALE), 'left': int(h*SCALE), 'right': int(h*SCALE)}
    bad = {k: v for k, v in e.items() if EDGE_INK <= v < span[k]*EDGE_FULL}
    if bad: hits.append((name, ty, (x, y, w, h), bad))

print(f'調べた箱 {checked} / 縁で切れている {len(hits)}')
for name, ty, rect, bad in sorted(hits, key=lambda r: -sum(r[3].values())):
    print(f'  {name:18s} {ty:10s} {rect}  {bad}')
sys.exit(1 if hits else 0)
