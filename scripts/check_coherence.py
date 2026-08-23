#!/usr/bin/env python3
"""台帳どうしの食い違いを見つける(ジグソーの噛み合わせ検査)。

2026-08-23 利用者の観察:
> 実装する各要素は独立しておらず蜘蛛の巣のようにマップを構造し、互いに相互補完
> している(まるでジグソーパズル)。訂正が必要な部分は、既に台帳から得れるのでは?

**そのとおり。** 同じ対象は複数の軸に別々の行を持つ — `Intent::X` は A01(入口)・
A02(マウス)・A03(時間軸)・A09(可逆)・A10(予告)に現れる。**軸どうしが同じ対象に
ついて食い違ったら、どちらかが嘘**。ピースを検分しなくても、隣と合わないことで分かる。

今日までの腐り検出は「台帳 対 コード」(`check_evidence.py`/`derive_entries.py`)だった。
これは **台帳 対 台帳**。**リポジトリを読まない**ので安く、選択のバイアスも入らない。

見る食い違い:
  1. 同じ記号について、ある軸が「穴」で別の軸が「穴でない」
  2. `normal-map` の verdict が `採用済` なのに、軸台帳が同じ id を「穴」と言う
  3. `entries.tsv`(実コード由来)が「入口あり」なのに、軸台帳が「穴」と言う
"""
import io, os, re, sys, glob, collections

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
REF = os.path.join(ROOT, "next/reference")
SYM = re.compile(r'`([A-Z][A-Za-z0-9_]*(?:::[A-Za-z0-9_]+)?)`')

def rows(path):
    for line in io.open(path, encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        c = line.rstrip("\n").split("\t")
        if c and c[0] not in ("axis", "kind", "id"):
            yield c

# ---- 1) 記号ごとに、どの軸が何と言っているか -------------------------
claims = collections.defaultdict(list)   # 記号 → [(軸, 穴か, 対象文)]
by_mapid = collections.defaultdict(list) # map_id → [(軸, 穴か)]
for p in sorted(glob.glob(os.path.join(REF, "axis/A*.tsv"))):
    axis = os.path.basename(p)[:-4]
    for c in rows(p):
        if len(c) < 6: continue
        hole = c[5] == "穴"
        for s in set(SYM.findall(c[2])):
            claims[s].append((axis, hole, c[2][:44]))
        if c[1] not in ("-", ""):
            by_mapid[c[1]].append((axis, hole))

out = []
# 規模の軸(一括・量・予算)は他の軸と**両立する**。「Copy は実装されている」と
# 「Copy が複数選択に効かない」は同時に真。矛盾ではないので比較から外す。
SCALE_AXES = {"A04-scale", "A08-bulk", "A11-budget"}

# 食い違い1: 同じ記号を「穴」と「穴でない」で言い合っている
for sym, cs in sorted(claims.items()):
    axes_hole = {a for a, h, _ in cs if h} - SCALE_AXES
    axes_ok   = {a for a, h, _ in cs if not h} - SCALE_AXES
    if axes_hole and axes_ok:
        out.append(("記号", sym,
                    f"穴と言う軸 {sorted(axes_hole)} / 穴でないと言う軸 {sorted(axes_ok)}"))

# 食い違い2: normal-map の verdict と軸台帳
verdict = {}
for c in rows(os.path.join(REF, "normal-map.tsv")):
    if len(c) > 12 and c[0].isdigit():
        verdict[c[0]] = (c[12], c[2])
# **規模の軸は verdict と両立する。** `採用済` は「1つに対して効く」しか言って
# いないので、「複数選択に効かない(A08)」「N倍で壊れる(A04)」と同時に真になれる。
# 導入時にこれを矛盾として8件出したが、**台帳の語彙が粗いだけ**で嘘ではなかった。
for mid, cs in sorted(by_mapid.items()):
    v = verdict.get(mid)
    if not v: continue
    holes = {a for a, h in cs if h} - SCALE_AXES
    if v[0] == "採用済" and holes:
        out.append(("map", f"id{mid} {v[1][:34]}", f"verdict=採用済 だが {sorted(holes)} が穴と言う"))

# 食い違い3: entries.tsv(実コード由来)と軸台帳
ent = os.path.join(REF, "generated/entries.tsv")
if os.path.exists(ent):
    code = {c[1]: c[2] for c in rows(ent) if len(c) > 2}
    for sym, cs in sorted(claims.items()):
        v = code.get(sym)
        holes = {a for a, h, _ in cs if h} - SCALE_AXES
        if v == "入口あり" and holes:
            out.append(("実コード", sym, f"実コードは入口あり だが {sorted(holes)} が穴と言う"))

for kind, what, why in out:
    print(f"[{kind:5s}] {what}\n         {why}")
print(f"\n食い違い {len(out)} 件")
sys.exit(1 if out else 0)
