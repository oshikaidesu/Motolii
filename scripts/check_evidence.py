#!/usr/bin/env python3
"""台帳の**証拠が今も実在するか**を全軸へ一律に当てる。

2026-08-23 利用者指摘:
> 「効きそうな所」という部分が少し怖い、まだ人の手が残っている証拠と思います

そのとおりで、「次にどの軸を機械化すると効くか」を supervisor の勘で選んでいた。
**優先順位も導出する。** 軸ごとの規則(入口なら `Intent::X` の参照、等)を書く前に、
**全軸へ一律に当てられる検査**が1つある — 台帳の `証拠` 列に書かれた `file:line` が
**今も実在するか**。

証拠が腐っている行は、判定も腐っている可能性が高い。**腐りの多い軸が、次に
機械化すべき軸**。これで「どこから手を付けるか」が勘から数字になる。

規則は3つだけ:
  1. 参照されたファイルが存在しないなら **腐り**
  2. `file:line` の行番号がファイルの行数を超えているなら **腐り**
  3. 証拠に `` `symbol` `` が書かれていて、その記号がファイル内に無いなら **要確認**
     (改名・移動の疑い。3 は誤検出しうるので腐りとは別に数える)
"""
import io, os, re, sys, glob, collections

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
AXIS = os.path.join(ROOT, "next/reference/axis")
PATH_RE = re.compile(r'(?:next/)?((?:core|engine|ui|shell|probes)/[A-Za-z0-9_./-]+?\.rs)(?::(\d+))?')
# iced fork のソースを指す証拠は Motolii の相対パスと同じ形をしているので、
# 前後の文脈で fork と分かる物は検査対象外にする(上流のパスをこちらの
# 台帳が追随する義理はない — 誤検出になるだけ)。
FORK_HINT = re.compile(r'fork|iced|upstream|widget/src|core/src/(text|event|window)')
SYM_RE = re.compile(r'`([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)`')

lines_cache = {}
def nlines(rel):
    if rel not in lines_cache:
        p = os.path.join(ROOT, "next", rel)
        try:
            lines_cache[rel] = sum(1 for _ in io.open(p, encoding="utf-8", errors="ignore"))
        except OSError:
            lines_cache[rel] = None
    return lines_cache[rel]

body_cache = {}
def body(rel):
    if rel not in body_cache:
        p = os.path.join(ROOT, "next", rel)
        try:
            body_cache[rel] = io.open(p, encoding="utf-8", errors="ignore").read()
        except OSError:
            body_cache[rel] = ""
    return body_cache[rel]

rot = collections.defaultdict(list)     # 軸 → [理由]
suspect = collections.defaultdict(list)
rows_per_axis = collections.Counter()

for path in sorted(glob.glob(os.path.join(AXIS, "A*.tsv"))):
    axis = os.path.basename(path)[:-4]
    for line in io.open(path, encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        c = line.rstrip("\n").split("\t")
        if len(c) < 5 or c[0] == "axis":
            continue
        rows_per_axis[axis] += 1
        evidence = c[4]
        fork_ctx = bool(FORK_HINT.search(evidence))
        for rel, ln in PATH_RE.findall(evidence):
            if fork_ctx and not rel.startswith(("core/motolii", "engine/motolii", "ui/motolii", "shell/motolii", "probes/")):
                continue  # iced fork 側のパス
            n = nlines(rel)
            if n is None:
                rot[axis].append(f"{c[2][:34]} → ファイルが無い: {rel}")
            elif ln and int(ln) > n:
                rot[axis].append(f"{c[2][:34]} → 行が無い: {rel}:{ln}(現在 {n} 行)")
            elif ln:
                for sym in SYM_RE.findall(evidence)[:3]:
                    tail = sym.split("::")[-1]
                    if len(tail) > 3 and tail not in body(rel):
                        suspect[axis].append(f"{c[2][:30]} → `{tail}` が {rel} に無い")
                        break

print(f"{'軸':<18}{'行':>4}{'腐り':>6}{'要確認':>7}")
order = []
for axis in sorted(rows_per_axis):
    r, s = len(rot[axis]), len(set(suspect[axis]))
    order.append((r + s, r, s, axis))
    print(f"{axis:<18}{rows_per_axis[axis]:>4}{r:>6}{s:>7}")
order.sort(reverse=True)
print("\n=== 機械化の優先順(腐り+要確認 の多い順・勘ではない)")
for total, r, s, axis in order:
    if total:
        print(f"  {axis}: 腐り{r} 要確認{s}")
worst = [a for t, r, s, a in order if t]
print("\n=== 明確な腐り(ファイル/行が実在しない)")
for axis in sorted(rot):
    for msg in rot[axis][:6]:
        print(f"  [{axis}] {msg}")
sys.exit(1 if any(rot.values()) else 0)
