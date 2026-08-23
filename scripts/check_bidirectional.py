#!/usr/bin/env python3
"""台帳と実装を**双方向**で突き合わせる(Lottie の条件2の再演)。

裁定212 が Lottie の4条件の (2) を「**閉じているので網羅が機械判定**」と書いた —
`lottie_coverage.rs` は**双方向**(スキーマに在って表に無い / 表に在ってスキーマに無い)で落とす。

`normal-map` はこれまで**片方向**しか見ていなかった(`採用済` → 実在識別子、裁定229)。
逆向き = **実装に在るのに台帳が要求していない物**。これは**先回りの検出**そのもので、
「段階が要求するまで作らない」(裁定226)の機械的な裏取りになる。

鍵は既にある — `motolii-verbs` の `Verb` が `map_ids` を持つ。
**`map_ids` が空の動詞 = 台帳のどの行も要求していない実装。**
どちらかが正しい: (a) 台帳に行が無い(定義の漏れ) (b) 先回りで作った。
**どちらかを人が決めるが、検出は機械がやる。**
"""
import io, os, re, sys

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
REG = os.path.join(ROOT, "next/ui/motolii-verbs/src/registry.rs")
MAP = os.path.join(ROOT, "next/reference/normal-map.tsv")

src = io.open(REG, encoding="utf-8").read()
verbs = []
for m in re.finditer(r'pub static ([A-Z_0-9]+): Verb = Verb \{(.*?)\n\};', src, re.S):
    body = m.group(2)
    label = re.search(r'label:\s*"([^"]*)"', body)
    ids = re.search(r'map_ids:\s*&\[([^\]]*)\]', body)
    idl = [x.strip() for x in (ids.group(1) if ids else "").split(",") if x.strip()]
    verbs.append((m.group(1), label.group(1) if label else "", idl))

rows = {}
for l in io.open(MAP, encoding="utf-8"):
    c = l.rstrip("\n").split("\t")
    if len(c) > 12 and c[0].isdigit():
        rows[c[0]] = (c[2], c[12])

unclaimed = [(n, lb) for n, lb, ids in verbs if not ids]
dangling = [(n, i) for n, lb, ids in verbs for i in ids if i not in rows]
pending_but_built = [(n, i, rows[i][0], rows[i][1])
                     for n, lb, ids in verbs for i in ids
                     if i in rows and rows[i][1] in ("採用予定", "結線待ち")]

# `map_ids` 空の19件は**判定が要る**(定義の漏れ か 先回り)ので即座に赤にしない。
# 上限を固定して増やさない形にする(裁定229 と同じ手口)。**減らすのはよい。**
UNCLAIMED_BASELINE = 19   # 2026-08-23 の実測

print(f"動詞 {len(verbs)} / map_ids 空 {len(unclaimed)}(上限 {UNCLAIMED_BASELINE}) / "
      f"台帳に無い id {len(dangling)} / 実装済みなのに verdict が採用予定 {len(pending_but_built)}")
if unclaimed:
    print("\n=== 台帳が要求していない実装(定義の漏れ か 先回り)")
    for n, lb in unclaimed:
        print(f"  {n:<22s} {lb}")
if pending_but_built:
    print("\n=== 実装済みなのに台帳が『採用予定』のまま(verdict の遅れ)")
    for n, i, name, v in pending_but_built:
        print(f"  {n:<22s} id{i:<5s} {name[:28]} [{v}]")
if dangling:
    print("\n=== 台帳に存在しない map_id を指している")
    for n, i in dangling:
        print(f"  {n} → id{i}")

bad = []
if len(unclaimed) > UNCLAIMED_BASELINE:
    bad.append(f"台帳が要求していない実装が {len(unclaimed)-UNCLAIMED_BASELINE} 件増えた"
               f"(`map_ids` を書くか、先回りなら作らない)")
if dangling:
    bad.append(f"台帳に存在しない map_id を指す動詞が {len(dangling)} 件")
if pending_but_built:
    bad.append(f"実装済みなのに verdict が採用予定の行が {len(pending_but_built)} 件"
               f"(verdict を追随させる)")
if bad:
    print("\n" + "\n".join("  - " + b for b in bad))
    sys.exit(1)
sys.exit(0)
