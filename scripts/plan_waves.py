#!/usr/bin/env python3
"""穴の台帳から「作業割り」を機械的に導く。

利用者裁定(2026-08-23)「並列で一気に進めるべき。ただし人の手では必ず抜け漏れが
生まれる」への答え。**並列計画を人が書かない。**

原理は1つだけ:

    2つの作業項目が並列可能 ⟺ write-set が交わらない

入力(全部 machine-joinable):
  - `next/reference/axis/A*.tsv`  … 穴と `責任`(どのファイルが直すべきか)
  - `next/reference/generated/inventory.tsv` … 実装(記号 → ファイル)
  - `next/reference/normal-map.tsv` … 意味(`map_id` で join)

出力:
  - `next/reference/generated/worklist.tsv` … 穴1件1行(意味・責任ファイルつき)
  - `next/reference/generated/waves.md`     … レーン割りと波

レーンの決め方: **責任ファイルを union-find で束ねる**。1つの穴が複数ファイルを
名指ししたら、それらは同じレーンが持つしかない(= 連結)。連結成分がそのまま
1レーンで、成分どうしは write-set が交わらないので**同時に走れる**。
"""
import csv, glob, io, os, re, sys, collections

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
AXIS = os.path.join(ROOT, "next/reference/axis")
GEN = os.path.join(ROOT, "next/reference/generated")

PATH_RE = re.compile(r'(next/[A-Za-z0-9_./-]+?\.rs)')

def read_tsv(path):
    with io.open(path, encoding="utf-8") as f:
        rows = [l.rstrip("\n").split("\t") for l in f if not l.startswith("#") and l.strip()]
    return rows[0], rows[1:]

# ---- 意味(normal-map) -------------------------------------------------
meaning = {}
hdr, rows = read_tsv(os.path.join(ROOT, "next/reference/normal-map.tsv"))
for r in rows:
    if len(r) > 3 and r[0].isdigit():
        meaning[r[0]] = (r[2], r[3])   # canonical, 意味

# ---- 実装(inventory)— 責任ファイルの実在検査に使う -------------------
known_files = set()
inv = os.path.join(GEN, "inventory.tsv")
if os.path.exists(inv):
    with io.open(inv, encoding="utf-8") as f:
        next(f, None)
        for l in f:
            c = l.split("\t")
            if len(c) > 3:
                known_files.add(c[3].split(":")[0])

# ---- 穴を集める --------------------------------------------------------
items = []
for path in sorted(glob.glob(os.path.join(AXIS, "A*.tsv"))):
    hdr, rows = read_tsv(path)
    for r in rows:
        if len(r) < 7 or r[5] != "穴":
            continue
        files = sorted(set(PATH_RE.findall(r[6])))
        files = [f[len("next/"):] if f.startswith("next/") else f for f in files]
        items.append({
            "axis": r[0], "map_id": r[1], "target": r[2],
            "verdict": r[3], "files": files,
            "overloaded": "※過積載" in r[6],
        })

# ---- union-find でレーンを導く ----------------------------------------
parent = {}
def find(x):
    parent.setdefault(x, x)
    while parent[x] != x:
        parent[x] = parent[parent[x]]; x = parent[x]
    return x
def union(a, b):
    ra, rb = find(a), find(b)
    if ra != rb: parent[ra] = rb

for it in items:
    for f in it["files"]:
        find(f)
    for f in it["files"][1:]:
        union(it["files"][0], f)

lanes = collections.defaultdict(lambda: {"files": set(), "items": []})
homeless = []
for it in items:
    if not it["files"]:
        homeless.append(it); continue
    key = find(it["files"][0])
    lanes[key]["files"].update(it["files"])
    lanes[key]["items"].append(it)

# ---- 出力1: worklist ---------------------------------------------------
os.makedirs(GEN, exist_ok=True)
with io.open(os.path.join(GEN, "worklist.tsv"), "w", encoding="utf-8") as f:
    f.write("lane\taxis\tmap_id\t対象\t判定\t意味(normal-map)\twrite-set\n")
    for key in sorted(lanes, key=lambda k: -len(lanes[k]["items"])):
        lane = lanes[key]
        name = min(lane["files"])
        for it in lane["items"]:
            m = meaning.get(it["map_id"], ("", ""))
            f.write("\t".join([name, it["axis"], it["map_id"], it["target"],
                               it["verdict"], m[1], ";".join(it["files"])]) + "\n")
    for it in homeless:
        m = meaning.get(it["map_id"], ("", ""))
        f.write("\t".join(["(責任ファイル未記入)", it["axis"], it["map_id"],
                           it["target"], it["verdict"], m[1], ""]) + "\n")

# ---- 出力2: waves ------------------------------------------------------
ordered = sorted(lanes.values(), key=lambda l: -len(l["items"]))
out = ["# 作業割り(機械導出)", "",
       f"`scripts/plan_waves.py` が生成。**手で編集しない。**",
       "",
       "原理: **2つの作業項目が並列可能 ⟺ write-set が交わらない**。",
       "責任ファイルを union-find で束ね、連結成分をそのまま1レーンにしている。",
       "成分どうしはファイルが交わらないので**同時に走れる**。", "",
       f"- 穴 {len(items)}件 / レーン {len(ordered)}本 / 責任ファイル未記入 {len(homeless)}件", ""]
missing = sorted({f for l in ordered for f in l["files"]
                  if known_files and f not in known_files})
if missing:
    out += ["## 責任ファイルが実装に見当たらない(要確認)", ""]
    out += [f"- `{m}`" for m in missing] + [""]
out += ["## レーン(重い順)", "",
        "| レーン | 穴 | write-set | 過積載 |", "|---|---|---|---|"]
for l in ordered:
    name = min(l["files"])
    over = "※" if any(i["overloaded"] for i in l["items"]) else ""
    fs = ", ".join(f"`{f}`" for f in sorted(l["files"])[:4])
    if len(l["files"]) > 4: fs += f" ほか{len(l['files'])-4}"
    out.append(f"| `{name}` | {len(l['items'])} | {fs} | {over} |")
if homeless:
    out += ["", "## 責任ファイルが書かれていない穴(発注できない)", ""]
    for it in homeless:
        out.append(f"- {it['axis']} {it['target'][:60]}")
io.open(os.path.join(GEN, "waves.md"), "w", encoding="utf-8").write("\n".join(out) + "\n")
print(f"穴 {len(items)} / レーン {len(ordered)} / 責任未記入 {len(homeless)} / 実在しない責任ファイル {len(missing)}")
