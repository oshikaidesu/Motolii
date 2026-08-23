#!/usr/bin/env python3
"""「普通の動画ソフトにする」残作業を台帳から割り振る。

2026-08-23 利用者指摘:
> 今は私自身が監督に仕事を提案していますが。**事実上の普通の動画ソフトの台帳が
> あるのでそこからタスクの割り振りを機械化できないか**という話です。

そのとおりで、定義も割り振りの材料も既に台帳にある。発明しない:

  - **何が残っているか** = `normal-map.tsv` の `verdict` が `採用予定`/`結線待ち` の行
  - **どこが持つか**     = 同 `bundle` 列(B01〜B52)→ `intent-bundles.tsv` の `home`
  - **どの順か**         = `freq`(4製品中いくつに在るか)降順。裁定158 の最優先キューと同じ

`home` は日本語の場所名(「Inspector blendタブ」等)なので、**語で crate へ引く**。
引けない束は `(家が未決)` として出す — **推測で割り当てない**(嘘の割当てを作らない)。
"""
import io, os, re, sys, collections

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
REF = os.path.join(ROOT, "next/reference")

HOME_TO_CRATE = [
    ("Inspector",   "ui/motolii-inspector-pane"),
    ("Timeline",    "ui/motolii-timeline-pane"),
    ("Browser",     "ui/motolii-browser-pane"),
    ("Viewer",      "ui/motolii-stage-pane"),
    ("キャンバス",   "ui/motolii-stage-pane"),
    ("Stage",       "ui/motolii-stage-pane"),
    ("Preferences", "ui/motolii-settings-pane"),
    ("Settings",    "ui/motolii-settings-pane"),
    ("Export",      "ui/motolii-export-pane"),
    ("Render Queue","ui/motolii-export-pane"),
    ("メニューバー", "ui/motolii-menubar"),
    ("右クリック",   "ui/motolii-menubar"),
    ("パネルタブ",   "shell/motolii-shell"),
    ("Window",      "shell/motolii-shell"),
    ("ワークスペース","shell/motolii-shell"),
    ("shortcut",    "ui/motolii-keymap"),
    ("キーボード",   "ui/motolii-keymap"),
]

def rows(path):
    with io.open(path, encoding="utf-8") as f:
        head = None
        for line in f:
            if line.startswith("#") or not line.strip():
                continue
            c = line.rstrip("\n").split("\t")
            if head is None:
                head = c; continue
            yield c

bundles = {}
for c in rows(os.path.join(REF, "intent-bundles.tsv")):
    if len(c) >= 4:
        bundles[c[0]] = {"name": c[1], "signature": c[2], "home": c[3]}

def crate_for(home):
    for key, crate in HOME_TO_CRATE:
        if key in home:
            return crate
    return None

lanes = collections.defaultdict(lambda: {"items": [], "homes": set(), "bundles": set()})
unhoused = []
for c in rows(os.path.join(REF, "normal-map.tsv")):
    if len(c) < 16:
        continue
    verdict, bundle = c[12], c[14]
    if verdict not in ("採用予定", "結線待ち"):
        continue
    b = bundles.get(bundle)
    crate = crate_for(b["home"]) if b else None
    item = {"id": c[0], "name": c[2], "meaning": c[3], "freq": c[8],
            "verdict": verdict, "bundle": bundle,
            "bundle_name": b["name"] if b else "", "home": b["home"] if b else ""}
    if crate is None:
        unhoused.append(item); continue
    lanes[crate]["items"].append(item)
    lanes[crate]["homes"].add(b["home"])
    lanes[crate]["bundles"].add(f'{bundle} {b["name"]}')

os.makedirs(os.path.join(REF, "generated"), exist_ok=True)
with io.open(os.path.join(REF, "generated/backlog.tsv"), "w", encoding="utf-8") as f:
    f.write("crate\tbundle\t束の名前\tmap_id\tfreq\t項目\t意味\tverdict\n")
    for crate in sorted(lanes, key=lambda k: -len(lanes[k]["items"])):
        for it in sorted(lanes[crate]["items"], key=lambda i: (-int(i["freq"] or 0), i["id"])):
            f.write("\t".join([crate, it["bundle"], it["bundle_name"], it["id"],
                               it["freq"], it["name"], it["meaning"], it["verdict"]]) + "\n")
    for it in sorted(unhoused, key=lambda i: (-int(i["freq"] or 0), i["id"])):
        f.write("\t".join(["(家が未決)", it["bundle"], it["bundle_name"], it["id"],
                           it["freq"], it["name"], it["meaning"], it["verdict"]]) + "\n")

order = sorted(lanes.items(), key=lambda kv: -len(kv[1]["items"]))
out = ["# 残作業の割り振り(機械導出)", "",
       "`scripts/plan_backlog.py` が生成。**手で編集しない。**", "",
       "**「普通の動画ソフト」の定義は `normal-map.tsv` が持っている。** 残っているのは",
       "`verdict` が `採用予定`/`結線待ち` の行で、どの pane が持つかは `bundle` →",
       "`intent-bundles.tsv` の `home` から引ける。順番は `freq`(4製品中いくつに在るか)降順。",
       "", f"- 残 {sum(len(v['items']) for v in lanes.values()) + len(unhoused)}件 / "
       f"crate {len(order)}本 / 家が未決 {len(unhoused)}件", "",
       "## crate ごと(重い順)— **crate が違えば同時に走れる**", "",
       "| crate | 残 | freq≥2 | 束 |", "|---|---|---|---|"]
for crate, v in order:
    hi = sum(1 for i in v["items"] if int(i["freq"] or 0) >= 2)
    bs = ", ".join(sorted(v["bundles"])[:3])
    if len(v["bundles"]) > 3: bs += f" ほか{len(v['bundles'])-3}"
    out.append(f"| `{crate}` | {len(v['items'])} | {hi} | {bs} |")
out += ["", "## 最優先(freq≥2 — 4製品中2つ以上が持つ = 普通度が高い)", "",
        "| crate | id | freq | 項目 | 意味 |", "|---|---|---|---|---|"]
top = [(c, i) for c, v in lanes.items() for i in v["items"] if int(i["freq"] or 0) >= 2]
top += [("(家が未決)", i) for i in unhoused if int(i["freq"] or 0) >= 2]
for c, i in sorted(top, key=lambda x: -int(x[1]["freq"] or 0)):
    out.append(f"| `{c}` | {i['id']} | {i['freq']} | {i['name']} | {i['meaning'][:38]} |")
if unhoused:
    out += ["", f"## 家が未決 {len(unhoused)}件(束が `home` を持たない/語で引けない)", "",
            "**推測で割り当てていない。** 束の `home` を決めるのが先。", ""]
    for b, n in collections.Counter(f'{i["bundle"]} {i["bundle_name"]}' for i in unhoused).most_common(10):
        out.append(f"- {b}: {n}件")
io.open(os.path.join(REF, "generated/backlog.md"), "w", encoding="utf-8").write("\n".join(out) + "\n")
print(f"残 {sum(len(v['items']) for v in lanes.values()) + len(unhoused)} / crate {len(order)} / 家が未決 {len(unhoused)}")
