#!/usr/bin/env python3
"""rustdoc JSON を1枚の TSV へ畳む(`scripts/gen-inventory.sh` から呼ばれる)。

自前の構文解析はしない — rustdoc が出した `index` を読み替えるだけ。
列: kind / symbol / crate / path:line / vis / doc1行
`callers` 列は**持たない**。呼び手ゼロの検出は `unreachable_pub` + `dead_code`
(コンパイラ)の担当で、ここで数えると今日踏んだ誤検出を再発させる。
"""
import json, sys, glob, os

KIND_KEEP = {"function", "struct", "enum", "variant", "struct_field",
             "constant", "trait", "type_alias", "module"}

rows = []
for f in sorted(glob.glob(os.path.join(sys.argv[1], "*.json"))):
    crate = os.path.basename(f)[:-5].replace("_", "-")
    try:
        d = json.load(open(f, encoding="utf-8"))
    except Exception:
        continue
    for item in d.get("index", {}).values():
        name = item.get("name")
        if not name:
            continue
        inner = item.get("inner") or {}
        kind = next(iter(inner), "?") if isinstance(inner, dict) else "?"
        if kind not in KIND_KEEP:
            continue
        span = item.get("span") or {}
        fn = span.get("filename")
        if not fn:            # 依存側から来た項目には span が無い
            continue
        # trait 実装経由で依存 crate の項目が混ざるので、Motolii 自身の
        # ソース(相対パス)だけを残す。registry/ の絶対パスは全部落とす。
        if os.path.isabs(fn) or "/registry/" in fn or fn.startswith("../"):
            continue
        line = (span.get("begin") or [0])[0]
        vis = item.get("visibility")
        vis = vis if isinstance(vis, str) else "restricted"
        doc = (item.get("docs") or "").strip().split("\n")[0]
        rows.append((kind, name, crate, f"{fn}:{line}", vis, doc))

rows.sort(key=lambda r: (r[2], r[3], r[1]))
print("kind\tsymbol\tcrate\tpath:line\tvis\tdoc1行")
for r in rows:
    print("\t".join(x.replace("\t", " ") for x in r))
