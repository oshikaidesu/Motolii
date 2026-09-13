#!/usr/bin/env python3
"""画の骨格の表を作る: KanjiVG(かな・漢字、CC BY-SA 3.0)と Relief SingleLine(欧文、OFL 1.1)の
筆順付きの線を、1 字あたり数十 byte の二進表 `strokes.bin` に落とす。

    python3 scripts/strokes/build_strokes.py <kanjivg repo> <Relief-SingleLine repo> <out dir>

書式(little endian): "MSTK" u8 version=1 u32 count、続いて codepoint 順に
    u32 codepoint, u8 strokes, { u8 points, { u8 x, u8 y }* }*
点は em の箱を 0..255 に写した物(y は下向き)。使う側は骨格の箱を字の箱へ合わせ直すので絶対位置は要らない。
"""
import os, re, struct, sys, plistlib
import xml.etree.ElementTree as ET

MAX_POINTS = 10

def cubic(p0, c1, c2, p1, n=8):
    return [((1-t)**3*p0[0] + 3*(1-t)**2*t*c1[0] + 3*(1-t)*t**2*c2[0] + t**3*p1[0],
             (1-t)**3*p0[1] + 3*(1-t)**2*t*c1[1] + 3*(1-t)*t**2*c2[1] + t**3*p1[1]) for t in [i/n for i in range(1, n+1)]]

def svg_path(d):
    toks = re.findall(r'[MmCcSsLl]|-?\d*\.?\d+', d)
    pts, cur, cmd, prev = [], None, None, None
    i = 0
    def num():
        nonlocal i; v = float(toks[i]); i += 1; return v
    while i < len(toks):
        if toks[i].isalpha(): cmd = toks[i]; i += 1
        if cmd in 'Mm':
            p = (num(), num()); cur = p if cmd == 'M' or cur is None else (cur[0]+p[0], cur[1]+p[1])
            pts.append(cur); prev = None; cmd = 'L' if cmd == 'M' else 'l'
        elif cmd in 'Ll':
            p = (num(), num()); cur = p if cmd == 'L' else (cur[0]+p[0], cur[1]+p[1]); pts.append(cur); prev = None
        elif cmd in 'CcSs':
            if cmd in 'Cc':
                c1 = (num(), num()); c2 = (num(), num()); p = (num(), num())
                if cmd == 'c': c1, c2, p = (cur[0]+c1[0], cur[1]+c1[1]), (cur[0]+c2[0], cur[1]+c2[1]), (cur[0]+p[0], cur[1]+p[1])
            else:
                c2 = (num(), num()); p = (num(), num())
                if cmd == 's': c2, p = (cur[0]+c2[0], cur[1]+c2[1]), (cur[0]+p[0], cur[1]+p[1])
                c1 = (2*cur[0]-prev[0], 2*cur[1]-prev[1]) if prev else cur
            pts.extend(cubic(cur, c1, c2, p)); prev = c2; cur = p
    return pts

def resample(pts, n):
    if len(pts) < 2: return pts
    seg = [((pts[i+1][0]-pts[i][0])**2 + (pts[i+1][1]-pts[i][1])**2) ** 0.5 for i in range(len(pts)-1)]
    total = sum(seg)
    if total <= 0: return [pts[0], pts[-1]]
    out, acc, k = [pts[0]], 0.0, 0
    for j in range(1, n):
        s = total * j / (n-1)
        while k < len(seg)-1 and acc + seg[k] < s: acc += seg[k]; k += 1
        u = 0.0 if seg[k] == 0 else (s-acc)/seg[k]
        out.append((pts[k][0] + (pts[k+1][0]-pts[k][0])*u, pts[k][1] + (pts[k+1][1]-pts[k][1])*u))
    return out

def kanjivg(repo):
    out = {}
    for name in sorted(os.listdir(os.path.join(repo, 'kanji'))):
        if not re.fullmatch(r'[0-9a-f]{5}\.svg', name): continue  # 変種(-Kaisho など)は外す
        cp = int(name[:5], 16)
        svg = open(os.path.join(repo, 'kanji', name), encoding='utf-8').read()
        strokes = [svg_path(d) for d in re.findall(r'<path[^>]*\bd="([^"]*)"', svg)]
        strokes = [[(x/109*255, y/109*255) for x, y in resample(s, MAX_POINTS)] for s in strokes if len(s) >= 2]
        if strokes: out[cp] = strokes
    return out

def relief(repo):
    ufo = os.path.join(repo, 'sources', 'Relief-SingleLine.ufo')
    upm = 1000.0
    contents = plistlib.load(open(os.path.join(ufo, 'glyphs', 'contents.plist'), 'rb'))
    out = {}
    for gname, fn in contents.items():
        root = ET.parse(os.path.join(ufo, 'glyphs', fn)).getroot()
        cps = [int(u.get('hex'), 16) for u in root.findall('unicode')]
        outline = root.find('outline')
        if not cps or outline is None: continue
        strokes = []
        for contour in outline.findall('contour'):
            pts = [(float(p.get('x')), float(p.get('y')), p.get('type')) for p in contour.findall('point')]
            poly, cur, offs = [], None, []
            for x, y, t in pts:
                p = (x, y)
                if t is None: offs.append(p); continue
                if cur is None or t == 'move': poly.append(p); cur = p; offs = []; continue
                if t == 'line' or not offs: poly.append(p)
                else: poly.extend(cubic(cur, offs[0], offs[-1], p))
                cur = p; offs = []
            if len(poly) >= 2:
                # y を下向きに、em の箱(0..upm)を 0..255 に。上が ascender 側。
                strokes.append([(min(max(x/upm*255, 0), 255), min(max((upm - y - 200)/upm*255, 0), 255)) for x, y in resample(poly, MAX_POINTS)])
        if strokes:
            for cp in cps: out.setdefault(cp, strokes)
    return out

def main():
    kvg, rel, out_dir = sys.argv[1:4]
    table = {}
    table.update(relief(rel))
    table.update(kanjivg(kvg))  # かな・漢字は KanjiVG が正
    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, 'strokes.bin'), 'wb') as f:
        f.write(b'MSTK' + struct.pack('<BI', 1, len(table)))
        for cp in sorted(table):
            strokes = table[cp][:255]
            f.write(struct.pack('<IB', cp, len(strokes)))
            for s in strokes:
                f.write(struct.pack('<B', len(s)))
                for x, y in s: f.write(struct.pack('<BB', int(round(x)), int(round(y))))
    print(f'{len(table)} characters -> {os.path.getsize(os.path.join(out_dir, "strokes.bin"))} bytes')

if __name__ == '__main__':
    main()
