#!/usr/bin/env python3
"""残作業を「原材料が在るか」で重みづける(台帳の合体)。

2026-08-23 利用者の問い:
> 台帳がふたつあって変だ。合体できないか。この普通の動画ソフトの台帳は、恐らく
> タスク量の目安としてウェイト化できる。**原材料でできるかどうか**、機械化は可能か?

**可能。** 2つの台帳は「何を作るか」(`normal-map`)と「何を持っているか」(在庫表)で、
**合体点は識別子の照合**。ただし `採用予定` の行は**定義上まだ識別子を持たない**ので、
直接は照合できない。そこで**束(bundle)を経由する**:

    同じ束で既に `採用済` になった行が使っている識別子 = その束の原材料

材料が揃っている束の残り = **繋ぐだけ(安い)**。
材料がゼロの束の残り = **意味から要る(高い)**。

これは推定であって保証ではない(束の粒度に依存する)。**数えられる形にするのが目的**。
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
bundles = {}
for l in io.open(os.path.join(REF, "intent-bundles.tsv"), encoding="utf-8"):
    if l.startswith("#"): continue
    c = l.rstrip("\n").split("\t")
    if len(c) >= 2 and c[0] not in ("id",):
        bundles[c[0]] = c[1]

mat = collections.defaultdict(set)      # 束 → 実在が確認できた材料
pending = collections.Counter()         # 束 → 残り件数
done = collections.Counter()
freq_hi = collections.Counter()
for l in io.open(os.path.join(REF, "normal-map.tsv"), encoding="utf-8"):
    c = l.rstrip("\n").split("\t")
    if len(c) < 15 or not c[0].isdigit(): continue
    b = c[14] or "(束なし)"
    if c[12] == "採用済":
        done[b] += 1
        mat[b] |= {s.split("::")[-1] for s in CAND.findall(c[13])} & inv
    elif c[12] in ("採用予定", "結線待ち"):
        pending[b] += 1
        if (c[8] or "0").isdigit() and int(c[8]) >= 2:
            freq_hi[b] += 1

rows = []
for b, n in pending.items():
    m = len(mat.get(b, ()))
    rows.append((b, bundles.get(b, ""), n, done[b], m, freq_hi[b]))

# 安い順 = 材料が多く残りが少ない / 高い順 = 材料ゼロで残りが多い
print(f"{'束':<6}{'残':>4}{'済':>4}{'材料':>5}{'freq≥2':>7}  名前")
print("--- 材料が在る(繋ぐ側。安い) ---")
for r in sorted([r for r in rows if r[4] > 0], key=lambda r: (-r[4], r[2]))[:10]:
    print(f"{r[0]:<6}{r[2]:>4}{r[3]:>4}{r[4]:>5}{r[5]:>7}  {r[1][:30]}")
print("--- 材料がゼロ(意味から要る。高い) ---")
for r in sorted([r for r in rows if r[4] == 0], key=lambda r: -r[2])[:10]:
    print(f"{r[0]:<6}{r[2]:>4}{r[3]:>4}{r[4]:>5}{r[5]:>7}  {r[1][:30]}")
cheap = sum(r[2] for r in rows if r[4] > 0)
exp   = sum(r[2] for r in rows if r[4] == 0)
print(f"\n残 {cheap+exp} = 材料が在る {cheap} / 材料ゼロ {exp}")
