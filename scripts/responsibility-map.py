#!/usr/bin/env python3
"""責任を構造で見せる。参照ではなく「いつも一緒に変わるか」で線を引く。

行数は責任の代理でしかない(reference/owned-budget.tsv の冒頭)。
import は「読んでいる」しか言わないが、同じ commit で一緒に変わる file は
同じ責任を分け持っている — 参照が無くてもだ。それを絵にする。

  python3 scripts/responsibility-map.py [--commits 400] [--out <svg>]

物差しの出典: Adam Tornhill の temporal coupling(code-maat / Software Design X-Rays)、
John Lakos の levelization(instability = out/(in+out))。
"""

import argparse
import collections
import itertools
import math
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
HOUSES = [
    ("doc", "motolii/crates/motolii-doc/", "#7fb3d5"),
    ("render", "motolii/crates/motolii-render/", "#e8a87c"),
    ("ui/native", "motolii/ui/native/", "#c39bd3"),
    ("ui/lib", "motolii/ui/lib/", "#85c1a1"),
    ("extensions", "motolii/ui/extensions/", "#d98880"),
]


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


def house_of(path):
    for name, prefix, color in HOUSES:
        if path.startswith(prefix):
            return name, color
    return None, None


def git(*args):
    return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True).stdout


def co_changes(commits, ceiling):
    """同じ commit に現れた file の対を数える。大掃除の commit は結合の証拠にならないので捨てる。"""
    log = git("log", f"-{commits}", "--name-only", "--pretty=format:%H")
    batches, cur = [], []
    for line in log.split("\n"):
        if re.fullmatch(r"[0-9a-f]{40}", line.strip()):
            if cur:
                batches.append(cur)
            cur = []
        elif line.strip():
            p = line.strip()
            if house_of(p)[0] and (p.endswith(".rs") or p.endswith(".dart")) and not is_test(p):
                cur.append(p)
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
    return len(batches), changed, pairs


def measure(path):
    """行数と、そのうち #[cfg(test)] の中にある行数。"""
    f = ROOT / path
    if not f.exists():
        return 0, 0
    lines = f.read_text(errors="replace").split("\n")
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


def layout(nodes, edges, rounds=400, width=1500, height=1000):
    """ばねで置く。近くにある物が一緒に変わる物。"""
    pos = {}
    for i, n in enumerate(nodes):
        angle = 2 * math.pi * i / len(nodes)
        pos[n] = [width / 2 + 420 * math.cos(angle), height / 2 + 300 * math.sin(angle)]
    weight = collections.defaultdict(float)
    for (a, b), w in edges.items():
        weight[(a, b)] = w
    for step in range(rounds):
        cool = 1.0 - step / rounds
        force = {n: [0.0, 0.0] for n in nodes}
        for a, b in itertools.combinations(nodes, 2):
            dx, dy = pos[a][0] - pos[b][0], pos[a][1] - pos[b][1]
            d2 = max(dx * dx + dy * dy, 400.0)
            push = 90000.0 / d2
            force[a][0] += dx * push / math.sqrt(d2)
            force[a][1] += dy * push / math.sqrt(d2)
            force[b][0] -= dx * push / math.sqrt(d2)
            force[b][1] -= dy * push / math.sqrt(d2)
        for (a, b), w in weight.items():
            dx, dy = pos[a][0] - pos[b][0], pos[a][1] - pos[b][1]
            d = max(math.hypot(dx, dy), 1.0)
            pull = (d - 120) * 0.02 * w
            force[a][0] -= dx / d * pull
            force[a][1] -= dy / d * pull
            force[b][0] += dx / d * pull
            force[b][1] += dy / d * pull
        for n in nodes:
            fx, fy = force[n]
            mag = math.hypot(fx, fy)
            if mag > 0:
                cap = min(mag, 30.0) * cool
                pos[n][0] += fx / mag * cap
                pos[n][1] += fy / mag * cap
            pos[n][0] = min(max(pos[n][0], 90), width - 90)
            pos[n][1] = min(max(pos[n][1], 70), height - 70)
    return pos


def svg(nodes, pos, sizes, tests, edges, changed, meta, width=1500, height=1000):
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">',
        f'<rect width="{width}" height="{height}" fill="#141414"/>',
        '<style>text{font-family:-apple-system,"Helvetica Neue",sans-serif}</style>',
        f'<text x="28" y="40" fill="#e8e8e8" font-size="19">Motolii — 責任の地図</text>',
        f'<text x="28" y="62" fill="#8a8a8a" font-size="12">線 = 同じ commit で一緒に変わった割合（参照ではなく振る舞い）。'
        f'丸の大きさ = 行数、濃い縁 = test の割合。直近 {meta["commits"]} commit。</text>',
    ]
    for (a, b), ratio in sorted(edges.items(), key=lambda kv: kv[1]):
        x1, y1 = pos[a]
        x2, y2 = pos[b]
        alpha = 0.16 + 0.62 * (ratio - 0.45) / 0.55
        out.append(
            f'<line x1="{x1:.0f}" y1="{y1:.0f}" x2="{x2:.0f}" y2="{y2:.0f}" '
            f'stroke="#ffd9a0" stroke-opacity="{alpha:.2f}" stroke-width="{0.7 + 5.0 * (ratio - 0.45):.1f}"/>'
        )
    for n in nodes:
        x, y = pos[n]
        r = 7 + math.sqrt(sizes[n]) * 0.62
        share = tests[n] / sizes[n] if sizes[n] else 0
        _, color = house_of(n)
        out.append(
            f'<circle cx="{x:.0f}" cy="{y:.0f}" r="{r:.1f}" fill="{color}" fill-opacity="0.82" '
            f'stroke="#141414" stroke-width="{1 + 7 * share:.1f}"/>'
        )
        label = n.split("/")[-1]
        out.append(
            f'<text x="{x:.0f}" y="{y + r + 13:.0f}" fill="#d8d8d8" font-size="11" text-anchor="middle">{label}</text>'
        )
        out.append(
            f'<text x="{x:.0f}" y="{y + r + 25:.0f}" fill="#7a7a7a" font-size="9" text-anchor="middle">'
            f'{sizes[n]}行 · {changed[n]}回</text>'
        )
    ly = height - 34
    lx = 28
    for name, _prefix, color in HOUSES:
        out.append(f'<circle cx="{lx}" cy="{ly}" r="7" fill="{color}" fill-opacity="0.82"/>')
        out.append(f'<text x="{lx + 13}" y="{ly + 4}" fill="#9a9a9a" font-size="11">{name}</text>')
        lx += 22 + 9 * len(name)
    out.append("</svg>")
    return "\n".join(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--commits", type=int, default=400)
    ap.add_argument("--min-changes", type=int, default=8, help="この回数より少なく変わった file は描かない")
    ap.add_argument("--min-ratio", type=float, default=0.45, help="この割合より弱い結合は線を引かない")
    ap.add_argument("--batch-ceiling", type=int, default=25, help="これより多くの file を触る commit は数えない")
    ap.add_argument("--out", default="docs/reviews/2026-09-20-responsibility-map.svg")
    args = ap.parse_args()

    total, changed, pairs = co_changes(args.commits, args.batch_ceiling)
    nodes = sorted(f for f, n in changed.items() if n >= args.min_changes)
    if not nodes:
        sys.exit("描くほど動いた file が無い")
    seen = set(nodes)
    edges = {}
    for (a, b), n in pairs.items():
        if a in seen and b in seen:
            ratio = n / min(changed[a], changed[b])
            if ratio >= args.min_ratio and n >= 4:
                edges[(a, b)] = ratio
    sizes, tests = {}, {}
    for n in nodes:
        sizes[n], tests[n] = measure(n)
    nodes = [n for n in nodes if sizes[n] > 0]
    seen = set(nodes)
    edges = {k: v for k, v in edges.items() if k[0] in seen and k[1] in seen}

    pos = layout(nodes, edges)
    out = ROOT / args.out
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(svg(nodes, pos, sizes, tests, edges, changed, {"commits": total}))

    print(f"commit {total} 本 / 描いた file {len(nodes)} / 線 {len(edges)}")
    print(f"→ {args.out}")
    print("\n== 一緒に変わる度合い（上位15）==")
    for (a, b), r in sorted(edges.items(), key=lambda kv: -kv[1])[:15]:
        print(f"  {r * 100:5.0f}%  {a.replace('motolii/', '')} ↔ {b.replace('motolii/', '')}")
    print("\n== 1 file を触ると何 file が付いてくるか（責任の散り具合）==")
    spread = collections.Counter()
    for (a, b) in edges:
        spread[a] += 1
        spread[b] += 1
    for f, n in spread.most_common(10):
        print(f"  {n:2d} 本  {f.replace('motolii/', '')}  ({sizes[f]}行、うち test {tests[f]})")


if __name__ == "__main__":
    main()
