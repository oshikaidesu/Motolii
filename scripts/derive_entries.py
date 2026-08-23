#!/usr/bin/env python3
"""「入口が在るか」を実コードから導く(A01 の判定列の機械化)。

2026-08-23: `plan_waves.py` のグラフが**手書きの `責任` 列**に依存していると
MC-1 が指摘した。コードを直しても台帳の判定が「穴」のままならグラフが動かない
= 人の手の抜け漏れが一段上で再発する。そこで**「穴かどうか」を導出**する。

判定規則(発明しない。裁定212 が人手でやったことをそのまま機械化):
    `Intent::X` / pane の `Message::X` が **`next/ui` か `next/shell` の
    非テストコードから参照されていれば「入口あり」**、無ければ「入口ゼロ」。

手書きに残すのは **「なぜ穴か」「どう直すか」「責任」** だけ。
出力は `next/reference/generated/entries.tsv`。
"""
import io, os, re, sys, collections

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
NEXT = os.path.join(ROOT, "next")

def rs_files(*rels):
    for rel in rels:
        for dirpath, _dirs, files in os.walk(os.path.join(NEXT, rel)):
            if "/target/" in dirpath or "/tests" in dirpath:
                continue
            for f in files:
                if f.endswith(".rs"):
                    yield os.path.join(dirpath, f)

# ---- Intent の全枝を store の定義から拾う ----------------------------
intent_src = os.path.join(NEXT, "core/motolii-store/src/document.rs")
text = io.open(intent_src, encoding="utf-8").read()
m = re.search(r'pub enum Intent\s*\{(.*?)\n\}', text, re.S)
variants = re.findall(r'\n    ([A-Z][A-Za-z0-9]*)\s*[\{\(,]', m.group(1)) if m else []
variants = sorted(set(variants))

# ---- 参照を数える(store 自身とテストは入口ではない) -----------------
refs = collections.defaultdict(list)
for path in rs_files("ui", "shell"):
    body = io.open(path, encoding="utf-8", errors="ignore").read()
    rel = os.path.relpath(path, NEXT)
    for v in variants:
        if re.search(rf'\bIntent::{v}\b', body):
            refs[v].append(rel)

os.makedirs(os.path.join(NEXT, "reference/generated"), exist_ok=True)
out = io.open(os.path.join(NEXT, "reference/generated/entries.tsv"), "w", encoding="utf-8")
out.write("kind\tvariant\t判定\t参照元(next/ui・next/shell の非テスト)\n")
zero = []
for v in variants:
    where = refs.get(v, [])
    verdict = "入口あり" if where else "入口ゼロ"
    if not where:
        zero.append(v)
    out.write(f"Intent\t{v}\t{verdict}\t{';'.join(sorted(where))}\n")
out.close()
print(f"Intent {len(variants)}枝 / 入口ゼロ {len(zero)}: {', '.join(zero) if zero else '(なし)'}")

# ---- 台帳との食い違いを報告(腐り検出) ------------------------------
ledger = os.path.join(NEXT, "reference/axis/A01-entry.tsv")
rot = []
if os.path.exists(ledger):
    for line in io.open(ledger, encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        c = line.rstrip("\n").split("\t")
        if len(c) < 6:
            continue
        for v in variants:
            if re.search(rf'`?{v}`?', c[2]) and c[2].strip("`").split("`")[0].startswith(v[:4]):
                said_hole = c[5] == "穴"
                is_hole = v in zero
                if said_hole != is_hole:
                    rot.append((v, "穴" if said_hole else "-", "入口ゼロ" if is_hole else "入口あり"))
                break
if rot:
    print("\n=== 台帳と実コードの食い違い(台帳が古い可能性)")
    for v, said, actual in sorted(set(rot)):
        print(f"  {v}: 台帳は `{said}` / 実コードは `{actual}`")
else:
    print("台帳と実コードの食い違い: なし")
