#!/usr/bin/env python3
"""どの機能が「効いている」かを台帳の相互関係から順位づける。

2026-08-23 利用者の提案:
> 関係値のグラフ化や重み付けまで行った方がいい。**リンク数が多い機能ほど責任が
> 高く、いち早く実装を行うべき**。Google 検索のアルゴリズムです。

そのとおりで、辺は既に台帳にある — **責任ファイル**(同じファイルを直す穴どうし)と
**束**(同じ `bundle` に属する項目どうし)。PageRank をかけると**荷重を受けている物**が出る。

**段階(`plan_steps.py`)とは別の順序**であることに注意:
  - **段階** = 利用者の動線。**次に何をやるか**
  - **ここ** = 構造の荷重。**壊すと何が巻き添えになるか / 直すと何が楽になるか**
段階の中で複数の候補が並んだ時の**優先度**と、**触る時の慎重さ**に使う。
"""
import io, os, re, sys, glob, collections

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
REF = os.path.join(ROOT, "next/reference")
PATH = re.compile(r'(?:next/)?((?:core|engine|ui|shell|probes)/[A-Za-z0-9_./-]+?\.rs)')

def rows(p):
    for line in io.open(p, encoding="utf-8"):
        if line.startswith("#") or not line.strip(): continue
        c = line.rstrip("\n").split("\t")
        if c and c[0] not in ("axis", "kind", "id"): yield c

# ---- 節点と辺を台帳から組む -------------------------------------------
node_axes = collections.defaultdict(set)     # 節点 → 触れている軸
by_file = collections.defaultdict(set)       # 責任ファイル → 節点
for p in sorted(glob.glob(os.path.join(REF, "axis/A*.tsv"))):
    axis = os.path.basename(p)[:-4]
    for c in rows(p):
        if len(c) < 7: continue
        node = c[2][:52]
        node_axes[node].add(axis)
        for f in set(PATH.findall(c[6])):
            by_file[f].add(node)

# **束(bundle)の同居は辺にしない。** 初回はこれを辺にして失敗した —
# B18 のような大きな束は「相互補完」ではなく**ただの分類の粒度**で、同じ袋に
# 入っているだけの項目が互いを持ち上げ合い、上位が「Clear In/Out」で埋まった。
#
# 辺にしてよいのは**実際に一緒に直さないと壊れる関係**だけ:
#   (a) 同じ責任ファイルを名指ししている穴どうし(直す場所が同じ)
#   (b) 同じ対象を複数の軸が指している(観点が重なっている)
edges = collections.defaultdict(set)
for group in by_file.values():
    g = sorted(group)
    if len(g) > 12: continue      # 1ファイルに多数ぶら下がる物は「過積載」であって関係ではない
    for i, a in enumerate(g):
        for b in g[i+1:]:
            edges[a].add(b); edges[b].add(a)

# ---- PageRank(素朴な冪乗法) -----------------------------------------
nodes = sorted(set(edges) | set(node_axes))
n = len(nodes)
rank = {k: 1.0 / n for k in nodes}
for _ in range(30):
    nxt = {k: 0.15 / n for k in nodes}
    for k in nodes:
        outs = edges.get(k, ())
        if not outs:
            for m in nodes: nxt[m] += 0.85 * rank[k] / n
        else:
            share = 0.85 * rank[k] / len(outs)
            for m in outs: nxt[m] += share
    rank = nxt

print(f"節点 {n} / 辺 {sum(len(v) for v in edges.values())//2}\n")
# 荷重 = PageRank × 軸の数(観点の重なり)。**片方だけでは信号が弱い**。
score = {k: rank[k] * 1000 * (1 + len(node_axes.get(k, ()))) for k in rank}
print(f"{'順':>3} {'荷重':>7}  {'軸':>3}  {'辺':>3}  対象")
for i, k in enumerate(sorted(score, key=lambda x: -score[x])[:15], 1):
    print(f"{i:>3} {score[k]:7.2f}  {len(node_axes.get(k,())):>3}  {len(edges.get(k,())):>3}  {k}")
print("\n軸の数が多い節点(複数の観点が同時に指している = 荷重が高い)")
for k, a in sorted(node_axes.items(), key=lambda kv: -len(kv[1]))[:8]:
    print(f"  {len(a)}軸 {sorted(a)}  {k}")
