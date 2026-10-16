# The first-click test's score, as ISO 9186's comprehension test counts a symbol: per task, the share of judges whose
# first click lands on a right control, its lower bound (Wilson, 90%), how much the judges agree (the share on the
# part most of them pressed), and what a random click on the window's parts would get (the baseline).
# A task passes when its lower bound is at least 67% (ISO 9186); the window passes when 85% of its tasks pass.
#   python3 tool/firstclick_grade.py answers_1.json answers_2.json ...   (each {"1": [x, y], ...} in the picture's pixels)
import json, math, os, sys
from collections import Counter

here = os.path.join(os.path.dirname(__file__), '..', '..', '..', 'reports', os.environ.get('FC_DIR', 'firstclick'))  # (FC_DIR=firstclick_b: another set of tasks)
tasks = json.load(open(os.path.join(here, 'tasks.json')))
said = [json.load(open(p)) for p in sys.argv[1:]]
pad = 6  # (a click a few pixels off a control's edge is on it)


def inside(x, y, r, p=pad):
    return r[0] - p <= x <= r[2] + p and r[1] - p <= y <= r[3] + p


def wilson_low(k, n, z=1.645):  # (90%, one side)
    if n == 0: return 0.0
    p = k / n
    return (p + z * z / (2 * n) - z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n))) / (1 + z * z / n)


passed = 0
print(f'{len(said)} judges')
for t in tasks:
    right = [t['rect'], *t.get('also', [])]
    parts = t.get('parts', [])
    clicks = [a[str(t['n'])][:2] for a in said if str(t['n']) in a]
    k = sum(any(inside(x, y, r) for r in right) for x, y in clicks)
    def part(x, y):  # (the smallest part a click lands in)
        hit = [i for i, r in enumerate(parts) if inside(x, y, r, 0)]
        return min(hit, key=lambda i: (parts[i][2] - parts[i][0]) * (parts[i][3] - parts[i][1])) if hit else -1
    agree = Counter(part(x, y) for x, y in clicks).most_common(1)[0][1] / len(clicks) if clicks else 0
    good = [i for i, r in enumerate(parts) if any(inside((r[0] + r[2]) / 2, (r[1] + r[3]) / 2, q, 0) or inside((q[0] + q[2]) / 2, (q[1] + q[3]) / 2, r, 0) for q in right)]
    base = max(1, len(good)) / max(1, len(parts))
    low = wilson_low(k, len(clicks))
    passed += low >= .67
    print(f'{t["n"]:2} {"pass" if low >= .67 else "    "} right {k}/{len(clicks)}  low {100 * low:3.0f}%  agree {100 * agree:3.0f}%  random {100 * base:4.1f}%  {t["task"]}')
print(f'tasks passed: {passed} of {len(tasks)} ({100 * passed / len(tasks):.0f}%, pass line 85%)')
