#!/usr/bin/env python3
"""`normal-map.tsv` の `採用済` に裏が取れるかを確かめる。

2026-08-23: 裁定212 が「`normal-map` は4条件を1つも満たさない」と指摘し、
実測として「main が395コミット進む間、採用済227が1行も動かなかった」を挙げていた。
**同日も同じことが起きた**(140コミット進んで採用済229のまま)。

`lottie-coverage` の作法を写す — **`採用済` の行は実在識別子を持ち、試験が確かめる。
自己申告を構造的に不可能にする。** ただし `normal-map` には `evidence` 列が無いので、
**`理由` 列に書かれた識別子**を拾って在庫表(`generated/inventory.tsv`)と照合する。

識別子の拾い方は保守的に:
  - `Type::Variant` / `snake_case_fn` / `CamelCase` を候補にする
  - 在庫表(rustdoc 由来 = コンパイラが解決した実体)に在れば裏が取れたとみなす
  - **1つも当たらない行は「自己申告」**として報告する(嘘とは限らない — 理由の書き方の問題)
"""
import io, os, re, sys, collections

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
REF = os.path.join(ROOT, "next/reference")

inv = set()
for l in io.open(os.path.join(REF, "generated/inventory.tsv"), encoding="utf-8"):
    c = l.split("\t")
    if len(c) > 1:
        inv.add(c[1])

CAND = re.compile(r'\b([A-Z][A-Za-z0-9]+::[A-Za-z0-9_]+|[a-z_][a-z0-9_]{3,}|[A-Z][A-Za-z0-9]{3,})\b')

tot = ok = 0
unbacked = []
for l in io.open(os.path.join(REF, "normal-map.tsv"), encoding="utf-8"):
    c = l.rstrip("\n").split("\t")
    if len(c) < 14 or not c[0].isdigit() or c[12] != "採用済":
        continue
    tot += 1
    hits = {s.split("::")[-1] for s in CAND.findall(c[13])} & inv
    if hits:
        ok += 1
    else:
        unbacked.append((c[0], c[2][:34]))

# **いまの109件を即座に赤にすると作業が止まる**ので、上限を固定して
# 「これ以上増やさない」形にする(裁定215 の owns 柵と同じ手口 —
# 立証不足を合格にしておかないと、緑にするために嘘を書く圧力がかかる)。
BASELINE = 109   # 2026-08-23 の実測。**減らすのはよいが増やしてはいけない**

print(f"採用済 {tot} / 在庫表で裏が取れた {ok} / 自己申告のまま {len(unbacked)}(上限 {BASELINE})")
if unbacked:
    print("\n=== 裏が取れない行(理由に実在識別子が無い)")
    for i, (a, b) in enumerate(unbacked[:15]):
        print(f"  id{a:<5s} {b}")
    if len(unbacked) > 15:
        print(f"  … 他 {len(unbacked)-15} 件")
if len(unbacked) > BASELINE:
    print(f"\n自己申告が {len(unbacked)-BASELINE} 件増えた。"
          f"**`採用済` にするなら `理由` 列へ実在識別子を書くこと**"
          f"(在庫表 `generated/inventory.tsv` に載っている名前)。")
    sys.exit(1)
sys.exit(0)
