# Every call from the window into the inside (skin/BRIDGE.md): where it is and which of the bridge's ways it goes by.
#   python3 -I tool/bridge_sites.py lib out.tsv     (D the document's picture, M the moment's, Q asked, S read once, W a change)
import os, re, sys, collections
ROOT = sys.argv[1]
KIND = {
 # D: the document's, made again when it changes
 **{k:'D' for k in 'things word words cards values moving range shown reading fxTitles keyMoments thingKeyMoments thingStretches lasts seconds work marks guides fits ways wayNames thingEffects kept counters rate trouble list answerWhy looping keyingNow point size shaping busy now drawnAt frames brokenCount broken spent label picture whole'.split()},
 # M: at the moment shown, made with each picture
 **{k:'M' for k in 'shows keyed frame seen points handles'.split()},
 # Q: asked of another moment or a long thing: asked, answered later
 **{k:'Q' for k in 'path loudness loudnessOver strikes wayAt wayOf wayNumbers setting thumbnail thumbnailAt fontTags kindOf still write'.split()},
 # S: the folders, read once
 **{k:'S' for k in 'recipes effects animators'.split()},
 # W: a change
 **{k:'W' for k in 'add remove set say key keyMany unkey way hold take carry letGo put keyHere stay resize sequence setWork mark guide raise make define effect later undo redo pick desk play at keying together forget watch close open'.split()},
}
# value(thing, name, at): M when at is the moment shown, Q otherwise
recv = r'(?:\b(?:m|mm|inside|_inside|motolii)\b|\binside\b|\.inside)[!?]?'
pat = re.compile(recv + r'\??\.(\w+)\b(\(|)')
rows = []
for d, _, fs in os.walk(ROOT):
    if '/build' in d or '/.dart_tool' in d: continue
    for f in fs:
        if not f.endswith('.dart') or f == 'motolii.dart': continue
        p = os.path.join(d, f); rel = os.path.relpath(p, ROOT)
        lines = open(p, encoding='utf-8').read().split('\n')
        fn = '?'
        for i, l in enumerate(lines, 1):
            m = re.match(r'\s*(?:@override\s*)?(?:static\s+)?(?:[\w<>?,\s\[\]().]+\s)?(\w+)\s*\([^;]*\)\s*(?:async\s*)?(?:=>|\{)', l)
            if m and m.group(1) not in ('if','for','while','switch','catch','return','setState'): fn = m.group(1)
            for mm in pat.finditer(l):
                name = mm.group(1)
                if name == 'value':
                    k = 'M/Q'
                elif name in KIND: k = KIND[name]
                else: continue
                rows.append((rel, i, fn, name, k))
by = collections.Counter(r[4] for r in rows)
print('sites', len(rows), dict(by))
print('files', len({r[0] for r in rows}))
with open(sys.argv[2], 'w') as o:
    for r in sorted(rows): o.write('\t'.join(map(str, r)) + '\n')
c = collections.Counter((r[3], r[4]) for r in rows)
for (n, k), v in sorted(c.items(), key=lambda x: (x[0][1], -x[1])): print(f'{k}\t{n}\t{v}')
