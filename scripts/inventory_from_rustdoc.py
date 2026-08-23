#!/usr/bin/env python3
"""rustdoc JSON を1枚の TSV へ畳む(`scripts/gen-inventory.sh` から呼ばれる)。

自前の構文解析はしない — rustdoc が出した `index` を読み替えるだけ。
列: kind / symbol / crate / path:line / vis / doc1行 / **署名**

**署名を持たせるのが肝**(2026-08-23 利用者指摘): 「今はそれぞれ API など関数が見えないから
適当に作って cargo をして、やっぱりあったんだと確かめて繋ぎ直す」。**それは cargo を
"検索"に使っている**。cargo が本来担うのは網羅性と借用で、**「その関数は在るか・引数は何か」は
表で引ける**。表を先に引けば、書き直しの往復が消える。
`callers` 列は**持たない**。呼び手ゼロの検出は `unreachable_pub` + `dead_code`
(コンパイラ)の担当で、ここで数えると今日踏んだ誤検出を再発させる。
"""
import json, sys, glob, os

KIND_KEEP = {"function", "struct", "enum", "variant", "struct_field",
             "constant", "trait", "type_alias", "module"}


def ty(t, depth=0):
    """rustdoc JSON の型表現を人が読める1行へ畳む(完全な再現は狙わない)。"""
    if t is None or depth > 4:
        return "_"
    if isinstance(t, str):
        return t
    if not isinstance(t, dict):
        return "_"
    if "primitive" in t:
        return t["primitive"]
    if "generic" in t:
        return t["generic"]
    if "resolved_path" in t:
        return t["resolved_path"].get("path", "_").split("::")[-1]
    if "borrowed_ref" in t:
        b = t["borrowed_ref"]
        return ("&mut " if b.get("is_mutable") else "&") + ty(b.get("type"), depth + 1)
    if "slice" in t:
        return f"[{ty(t['slice'], depth+1)}]"
    if "array" in t:
        return f"[{ty(t['array'].get('type'), depth+1)}; N]"
    if "tuple" in t:
        return "(" + ", ".join(ty(x, depth + 1) for x in t["tuple"]) + ")"
    if "qualified_path" in t:
        return t["qualified_path"].get("name", "_")
    if "impl_trait" in t or "dyn_trait" in t:
        return "impl/dyn"
    return "_"


def signature(inner):
    fn = inner.get("function") or {}
    sig = fn.get("sig") or {}
    ins = ", ".join(f"{n}: {ty(t)}" for n, t in (sig.get("inputs") or []))
    out = sig.get("output")
    return f"({ins})" + (f" -> {ty(out)}" if out else "")

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
        sig = signature(inner) if kind == "function" else ""
        rows.append((kind, name, crate, f"{fn}:{line}", vis, doc, sig))

rows.sort(key=lambda r: (r[2], r[3], r[1]))
print("kind\tsymbol\tcrate\tpath:line\tvis\tdoc1行\t署名")
for r in rows:
    print("\t".join(x.replace("\t", " ") for x in r))
