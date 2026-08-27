"""Ableton skin metrology — 2枚の窓画像の対応領域を数値で比べる。

出すのは意見ではなく測定値:
  row_pitch   行のピッチ(pt)。横projection の自己相関ピーク
  cap_height  文字インクの高さ(pt)の最頻値
  ink_pct     インク被覆率(%)。密度の代理
  faces       明度段(領域の主要な面の値、量子化)
  vlines/hlines  1px 級の暗線の本数(継ぎ目の文法)

使い方: python3 skin_metrics.py ref.png ref_region ours.png ours_region
  region = x,y,w,h (pt、画像は 2x 前提)
"""
import sys, collections
from PIL import Image

def load(path): return Image.open(path).convert('L')

def region(im, spec, scale=2):
    x,y,w,h=[int(v) for v in spec.split(',')]
    return im.crop((x*scale,y*scale,(x+w)*scale,(y+h)*scale)), scale

def ink_mask(g, thr=26):
    # 領域の最頻値=地。そこから thr 以上離れた画素=インク
    hist=g.histogram(); bg=max(range(256), key=lambda v:hist[v])
    px=g.load(); W,H=g.size
    return [[abs(px[x,y]-bg)>thr for x in range(W)] for y in range(H)], bg

def row_pitch(mask, scale):
    H=len(mask); W=len(mask[0])
    proj=[sum(r) for r in mask]
    m=sum(proj)/H
    d=[v-m for v in proj]
    best=(0,0.0)
    for lag in range(int(8*scale), int(40*scale)):
        c=sum(d[i]*d[i+lag] for i in range(H-lag))/(H-lag)
        if c>best[1]: best=(lag,c)
    return best[0]/scale

def cap_height(mask, scale):
    # 連続インク行の塊の高さ分布 → 最頻
    runs=[]; run=0
    for r in mask:
        if sum(r)>2: run+=1
        elif run: runs.append(run); run=0
    if run: runs.append(run)
    if not runs: return 0.0
    c=collections.Counter(runs)
    return c.most_common(1)[0][0]/scale

def faces(g):
    hist=g.histogram(); tot=sum(hist)
    # 8階調へ量子化して 3% 以上の段だけ
    q=collections.Counter()
    for v,n in enumerate(hist): q[v//8*8]+=n
    return sorted((v,round(n/tot*100)) for v,n in q.items() if n/tot>0.03)

def lines(g, scale):
    px=g.load(); W,H=g.size
    hist=g.histogram(); bg=max(range(256), key=lambda v:hist[v])
    v=h=0
    for x in range(W):
        col=[px[x,y] for y in range(0,H,4)]
        if sum(1 for c in col if c<bg-18)>len(col)*0.8: v+=1
    for y in range(H):
        row=[px[x,y] for x in range(0,W,4)]
        if sum(1 for c in row if c<bg-18)>len(row)*0.8: h+=1
    return round(v/scale,1), round(h/scale,1)

def report(tag, path, spec):
    im=load(path); g,scale=region(im,spec)
    mask,bg=ink_mask(g)
    ink=sum(sum(r) for r in mask)/(g.size[0]*g.size[1])*100
    vl,hl=lines(g,scale)
    print(f'{tag:8s} bg={bg:3d} ink={ink:4.1f}%  row_pitch={row_pitch(mask,scale):5.1f}pt  cap={cap_height(mask,scale):4.1f}pt  vlines={vl} hlines={hl}')
    print(f'{"":8s} faces={faces(g)}')

if __name__=='__main__':
    report('REF', sys.argv[1], sys.argv[2])
    report('OURS', sys.argv[3], sys.argv[4])
