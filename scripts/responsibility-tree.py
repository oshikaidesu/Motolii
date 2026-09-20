#!/usr/bin/env python3
"""責任を file 構造の上に描く。

木を見れば分かる形にする。行の長さ = 行数、暗い部分 = test、
右の数字 = その file を触ると一緒に変わる file の本数(直近の git 履歴から)。
散っている責任は「小さいのに連れが多い」行として現れる。

  python3 scripts/responsibility-tree.py [--commits 400] [--out <svg>]
"""

import argparse
import collections
import itertools
import math
import pathlib
import re
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent.parent
HOUSES = [
    ("doc — 契約", "motolii/crates/motolii-doc/src", "#7fb3d5"),
    ("render — 描く側", "motolii/crates/motolii-render/src", "#e8a87c"),
    ("ui/lib — 窓", "motolii/ui/lib", "#85c1a1"),
    ("ui/native — 接続", "motolii/ui/native/src", "#c39bd3"),
    ("extensions — 拡張", "motolii/ui/extensions", "#d98880"),
    ("vism — 棚(WGSL)", "motolii/crates/motolii-render/vism", "#d4b483"),
]
WARN, SOFT = 600, 800


TEST_PATH = re.compile(
    r"(^|/)tests?/|_tests?\.rs$|_test\.dart$|_contracts?\.rs$|_regressions?\.rs$|_probe\.rs$|_diagnostic\.rs$|/fixture\.rs$"
)


def is_test(path):
    """test の file は結合の証拠にならない。

    inline の `#[cfg(test)]` は path では見分けられないが、2026-09-20 に長い file の
    test を隣の file へ出したので、以後 test だけの commit は実装 file を触らない。
    それ以前の履歴では、実装と test の共変化がそのまま結合として出る(M の調査で
    block_program ↔ blocks が 86% → production だけなら 60% だった例)。
    """
    return bool(TEST_PATH.search(path))


def git(*args):
    return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True).stdout


def spread(commits, ceiling=25, min_ratio=0.45):
    """いつも一緒に変わる相手の数。参照ではなく振る舞いで測る(temporal coupling)。"""
    log = git("log", f"-{commits}", "--name-only", "--pretty=format:%H")
    batches, cur = [], []
    for line in log.split("\n"):
        if re.fullmatch(r"[0-9a-f]{40}", line.strip()):
            if cur:
                batches.append(cur)
            cur = []
        elif line.strip().endswith((".rs", ".dart")) and line.startswith("motolii/"):
            if not is_test(line.strip()):
                cur.append(line.strip())
    if cur:
        batches.append(cur)
    changed, pairs = collections.Counter(), collections.Counter()
    for batch in batches:
        batch = sorted(set(batch))
        if len(batch) > ceiling:
            continue
        for f in batch:
            changed[f] += 1
        for a, b in itertools.combinations(batch, 2):
            pairs[(a, b)] += 1
    partners = collections.defaultdict(list)
    for (a, b), n in pairs.items():
        if changed[a] >= 5 and changed[b] >= 5 and n >= 4:
            ratio = n / min(changed[a], changed[b])
            if ratio >= min_ratio:
                partners[a].append((ratio, b))
                partners[b].append((ratio, a))
    return changed, partners


def measure(path):
    lines = path.read_text(errors="replace").split("\n")
    total, test, depth, started, inside = len(lines), 0, 0, False, False
    for line in lines:
        if inside:
            test += 1
            depth += line.count("{") - line.count("}")
            if "{" in line:
                started = True
            if started and depth <= 0:
                inside = False
            elif not started and line.rstrip().endswith(";"):
                inside = False
        elif line.lstrip().startswith("#[cfg(test)]"):
            inside, test, depth, started = True, test + 1, 0, False
    return total, test


def collect(prefix):
    root = ROOT / prefix
    files = sorted(p for p in root.rglob("*") if p.suffix in (".rs", ".dart", ".wgsl", ".fs", ".frag") and p.is_file())
    out = []
    for p in files:
        rel = p.relative_to(root)
        total, test = measure(p)
        out.append((str(rel), total, test, str(p.relative_to(ROOT))))
    return out


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--commits", type=int, default=400)
    ap.add_argument("--out", default="docs/reviews/2026-09-20-responsibility-tree.svg")
    args = ap.parse_args()

    changed, partners = spread(args.commits)

    ROW, INDENT, BAR_X, BAR_MAX, SCALE = 15, 11, 430, 430, 0.165
    W = 1180
    rows = []  # (kind, depth, text, total, test, n_partners, path)
    for title, prefix, color in HOUSES:
        entries = collect(prefix)
        if not entries:
            continue
        total_lines = sum(e[1] for e in entries)
        rows.append(("house", 0, f"{title}", total_lines, sum(e[2] for e in entries), 0, prefix, color))
        shown_dirs = set()
        for rel, total, test, full in entries:
            parts = rel.split("/")
            for d in range(len(parts) - 1):
                key = "/".join(parts[: d + 1])
                if key not in shown_dirs:
                    shown_dirs.add(key)
                    kids = [e for e in entries if e[0].startswith(key + "/")]
                    rows.append(("dir", d + 1, parts[d] + "/", sum(k[1] for k in kids),
                                 sum(k[2] for k in kids), 0, key, color))
            n = len({b for _, b in partners.get(full, [])})
            rows.append(("file", len(parts), parts[-1], total, test, n, full, color))
        rows.append(("gap", 0, "", 0, 0, 0, "", color))

    H = 96 + ROW * len(rows) + 40
    o = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
        f'<rect width="{W}" height="{H}" fill="#151515"/>',
        '<style>text{font-family:ui-monospace,"SF Mono",Menlo,monospace}</style>',
        '<text x="26" y="38" fill="#ececec" font-size="17" font-family="-apple-system,sans-serif">'
        'Motolii — file 構造に描いた責任</text>',
        f'<text x="26" y="60" fill="#8c8c8c" font-size="11" font-family="-apple-system,sans-serif">'
        f'棒 = 行数（暗い方は test）・{WARN} 行で黄・{SOFT} 行で赤。'
        f'右の ● の数 = その file を触ると一緒に変わる file の本数（直近 {args.commits} commit の振る舞い、参照ではない）。</text>',
        f'<line x1="26" y1="76" x2="{W - 26}" y2="76" stroke="#2c2c2c"/>',
    ]
    y = 96
    for kind, depth, text, total, test, n, path, color in rows:
        if kind == "gap":
            y += ROW
            continue
        x = 26 + depth * INDENT
        if kind == "house":
            o.append(f'<rect x="20" y="{y - 11}" width="{W - 40}" height="{ROW}" fill="{color}" fill-opacity="0.09"/>')
            o.append(f'<text x="{x}" y="{y}" fill="{color}" font-size="12" font-family="-apple-system,sans-serif">'
                     f'{esc(text)}</text>')
            o.append(f'<text x="{BAR_X - 12}" y="{y}" fill="#8c8c8c" font-size="10" text-anchor="end">{total} 行</text>')
            y += ROW
            continue
        if kind == "dir":
            o.append(f'<text x="{x}" y="{y}" fill="#9a9a9a" font-size="10.5">{esc(text)}</text>')
            o.append(f'<text x="{BAR_X - 12}" y="{y}" fill="#5e5e5e" font-size="9" text-anchor="end">{total}</text>')
            y += ROW
            continue

        ink = "#e05b5b" if total > SOFT else ("#e8b45b" if total > WARN else "#cfcfcf")
        o.append(f'<text x="{x}" y="{y}" fill="{ink}" font-size="10.5">{esc(text)}</text>')
        o.append(f'<text x="{BAR_X - 12}" y="{y}" fill="#6e6e6e" font-size="9" text-anchor="end">{total}</text>')
        w = min(total * SCALE, BAR_MAX)
        tw = min(test * SCALE, w)
        o.append(f'<rect x="{BAR_X}" y="{y - 8}" width="{w:.1f}" height="9" fill="{color}" fill-opacity="0.72" rx="1.5"/>')
        if tw > 0.6:
            o.append(f'<rect x="{BAR_X + w - tw:.1f}" y="{y - 8}" width="{tw:.1f}" height="9" '
                     f'fill="#151515" fill-opacity="0.55" rx="1.5"/>')
        for line in (WARN, SOFT):
            lx = BAR_X + line * SCALE
            o.append(f'<line x1="{lx:.0f}" y1="{y - 9}" x2="{lx:.0f}" y2="{y + 2}" stroke="#4a4a4a" stroke-width="0.6"/>')
        dx = BAR_X + BAR_MAX + 26
        for i in range(min(n, 10)):
            heat = "#ff9d4d" if n >= 5 else ("#d0a06a" if n >= 3 else "#7d7d7d")
            o.append(f'<circle cx="{dx + i * 9}" cy="{y - 4}" r="3.1" fill="{heat}"/>')
        if n >= 3:
            best = sorted(partners.get(path, []), reverse=True)[:1]
            if best:
                ratio, other = best[0]
                o.append(f'<text x="{dx + 10 * 9 + 6}" y="{y}" fill="#6e6e6e" font-size="8.5">'
                         f'{ratio * 100:.0f}% {esc(other.split("/")[-1])}</text>')
        y += ROW

    o.append(f'<text x="26" y="{H - 16}" fill="#6e6e6e" font-size="10" font-family="-apple-system,sans-serif">'
             f'散っている責任は「棒が短いのに ● が多い行」— 小さい file が多くの file を連れてくる。'
             f'python3 scripts/responsibility-tree.py で引き直す。</text>')
    o.append("</svg>")
    out = ROOT / args.out
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(o))

    over_soft = [r for r in rows if r[0] == "file" and r[3] > SOFT]
    over_warn = [r for r in rows if r[0] == "file" and WARN < r[3] <= SOFT]
    scattered = sorted([r for r in rows if r[0] == "file" and r[5] >= 3], key=lambda r: (-r[5], r[3]))
    print(f"→ {args.out}")
    print(f"{SOFT} 行超 {len(over_soft)} 本 / {WARN}〜{SOFT} 行 {len(over_warn)} 本")
    for r in sorted(over_soft, key=lambda r: -r[3]):
        print(f"  {r[3]:5d} 行 (test {r[4]:4d})  {r[6]}")
    print("\n責任が散っている file（小さくても連れが多い物が本命）:")
    for r in scattered[:12]:
        print(f"  ● {r[5]:2d} 本  {r[3]:5d} 行  {r[6]}")


if __name__ == "__main__":
    main()
