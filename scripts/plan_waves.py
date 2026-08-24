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
  - `next/reference/generated/waves.md`     … 意味レーンとWIRE結線

レーンの決め方: **意味責任ファイルを union-find で束ねる**。1つの穴が複数ファイルを
名指ししたら、それらは同じレーンが持つしかない(= 連結)。連結成分がそのまま
1レーンで、成分どうしは write-set が交わらないので**同時に走れる**。
ただし `//! responsibility: wire` を宣言したファイルは意味責任から外し、別の
直列なWIRE結線として数える。Shell rootを全機能の意味責任へ連結しないためである。
"""
import csv, glob, io, os, re, sys, collections

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
AXIS = os.path.join(ROOT, "next/reference/axis")
GEN = os.path.join(ROOT, "next/reference/generated")

PATH_RE = re.compile(r'(next/[A-Za-z0-9_./-]+?\.rs)')
WIRE_RE = re.compile(r'^\s*//!\s*responsibility:\s*wire\s*$', re.MULTILINE)
EXTERNAL_RE = re.compile(r'fork側|上流|外部|Motolii内では直せない')

def discover_wire_files(root):
    """コード自身が宣言したWIREファイルを返す。別の手書き台帳を増やさない。"""
    wire = set()
    next_root = os.path.join(root, "next")
    for dirpath, dirnames, filenames in os.walk(next_root):
        dirnames[:] = [name for name in dirnames if name != "target" and not name.startswith(".")]
        for filename in filenames:
            if not filename.endswith(".rs"):
                continue
            path = os.path.join(dirpath, filename)
            with io.open(path, encoding="utf-8", errors="ignore") as f:
                if WIRE_RE.search(f.read()):
                    wire.add(os.path.relpath(path, next_root).replace(os.sep, "/"))
    return wire

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

wire_files = discover_wire_files(ROOT)

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
            "semantic_files": [f for f in files if f not in wire_files],
            "wire_files": [f for f in files if f in wire_files],
            "external": bool(EXTERNAL_RE.search(r[6])),
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
    for f in it["semantic_files"]:
        find(f)
    for f in it["semantic_files"][1:]:
        union(it["semantic_files"][0], f)

lanes = collections.defaultdict(lambda: {"files": set(), "items": []})
wire_only = []
external = []
homeless = []
for it in items:
    if not it["semantic_files"]:
        if it["wire_files"]:
            wire_only.append(it)
        elif it["external"]:
            external.append(it)
        else:
            homeless.append(it)
        continue
    key = find(it["semantic_files"][0])
    lanes[key]["files"].update(it["semantic_files"])
    lanes[key]["items"].append(it)

# ---- 出力1: worklist ---------------------------------------------------
os.makedirs(GEN, exist_ok=True)
def write_row(handle, values):
    # 空列を末尾のtabのまま出すと、生成物のdiff検査が意味の無い空欄で赤くなる。
    # TSVのjoin性は `-` を空値の記号にして保つ。
    handle.write("\t".join(value or "-" for value in values) + "\n")

with io.open(os.path.join(GEN, "worklist.tsv"), "w", encoding="utf-8") as f:
    f.write("lane\taxis\tmap_id\t対象\t判定\t意味(normal-map)\tsemantic-write-set\twire-set\n")
    for key in sorted(lanes, key=lambda k: -len(lanes[k]["items"])):
        lane = lanes[key]
        name = min(lane["files"])
        for it in lane["items"]:
            m = meaning.get(it["map_id"], ("", ""))
            write_row(f, [name, it["axis"], it["map_id"], it["target"],
                         it["verdict"], m[1], ";".join(it["semantic_files"]),
                         ";".join(it["wire_files"])])
    for it in sorted(wire_only, key=lambda item: (item["axis"], item["target"])):
        m = meaning.get(it["map_id"], ("", ""))
        write_row(f, ["(WIRE結線)", it["axis"], it["map_id"], it["target"],
                     it["verdict"], m[1], "", ";".join(it["wire_files"])])
    for it in homeless:
        m = meaning.get(it["map_id"], ("", ""))
        write_row(f, ["(責任ファイル未記入)", it["axis"], it["map_id"],
                     it["target"], it["verdict"], m[1], "", ""])
    for it in external:
        m = meaning.get(it["map_id"], ("", ""))
        write_row(f, ["(外部依存)", it["axis"], it["map_id"],
                     it["target"], it["verdict"], m[1], "", ""])

# ---- 出力2: waves ------------------------------------------------------
ordered = sorted(lanes.values(), key=lambda l: -len(l["items"]))
wire_required = [it for it in items if it["wire_files"]]
out = ["# 作業割り(機械導出)", "",
       f"`scripts/plan_waves.py` が生成。**手で編集しない。**",
       "",
       "原理: **意味componentのwrite-setが交わらない作業項目は同時に走れる**。",
       "`//! responsibility: wire` を持つShell rootは意味レーンから除外し、",
       "最後に1本のWIRE結線で接続する。これで結線の共有と意味の所有を混ぜない。", "",
       f"- 穴 {len(items)}件 / 意味レーン {len(ordered)}本 / WIRE結線 1本",
       f"- WIRE宣言 {len(wire_files)}ファイル / WIRE関与 {len(wire_required)}件(結線だけ {len(wire_only)}件) / 外部依存 {len(external)}件 / 責任ファイル未記入 {len(homeless)}件", ""]
# 実在検査は**ファイルシステムで**行う。inventory.tsv は rustdoc が
# 公開した項目しか載せないので、中身が全部 pub(crate)/private なモジュールは
# 載らない — それを「見当たらない」と報告すると偽陽性になる。
missing = sorted({f for l in ordered for f in l["files"]
                  if not os.path.exists(os.path.join(ROOT, "next", f))
                  and not os.path.exists(os.path.join(ROOT, f))})
if missing:
    out += ["## 責任ファイルが実装に見当たらない(要確認)", ""]
    out += [f"- `{m}`" for m in missing] + [""]
out += ["## 意味レーン(重い順)", "",
        "WIRE結線はここへ含めない。意味componentの実装後にsupervisorがまとめて接続する。", "",
        "| レーン | 穴 | semantic write-set | WIRE | 過積載 |", "|---|---:|---|---|---|"]
for l in ordered:
    name = min(l["files"])
    over = "※" if any(i["overloaded"] for i in l["items"]) else ""
    fs = ", ".join(f"`{f}`" for f in sorted(l["files"])[:4])
    if len(l["files"]) > 4: fs += f" ほか{len(l['files'])-4}"
    wire = "※" if any(i["wire_files"] for i in l["items"]) else ""
    out.append(f"| `{name}` | {len(l['items'])} | {fs} | {wire} | {over} |")
if wire_files:
    out += ["", "## WIRE結線(直列)", "",
            "意味レーンが完成した後、Shell rootへ結線する。WIREファイルは意味レーンを連結しない。", "",
            "| WIREファイル | 責任参照 | 判定 |", "|---|---:|---|"]
    by_wire = collections.Counter(f for it in wire_required for f in it["wire_files"])
    for f in sorted(wire_files):
        count = by_wire[f]
        out.append(f"| `{f}` | {count} | WIRE |")
if homeless:
    out += ["", "## 責任ファイルが書かれていない穴(発注できない)", ""]
    for it in homeless:
        out.append(f"- {it['axis']} {it['target'][:60]}")
if external:
    out += ["", "## 外部依存(このrepoのwrite-setへ入れない)", "",
            "外部上流の欠如はMotoliiの意味componentへ偽の責任を割り当てない。", ""]
    for it in external:
        out.append(f"- {it['axis']} {it['target'][:60]}")
io.open(os.path.join(GEN, "waves.md"), "w", encoding="utf-8").write("\n".join(out) + "\n")
print(f"穴 {len(items)} / 意味レーン {len(ordered)} / WIRE結線 {1 if wire_required else 0} / 外部依存 {len(external)} / 責任未記入 {len(homeless)} / 実在しない責任ファイル {len(missing)}")
