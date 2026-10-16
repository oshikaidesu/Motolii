# The first-click test's score: a judge's clicks (answers JSON: {"1": [x, y, "sure"|"unsure"|"lost"], ...} in the picture's
# own pixels) against where each task's right control lies (reports/firstclick/tasks.json, written by integration_test/firstclick_test.dart).
#   python3 tool/firstclick_grade.py answers_a.json [answers_b.json ...]
import json, sys, os

here = os.path.join(os.path.dirname(__file__), '..', '..', '..', 'reports', 'firstclick')
tasks = {str(t['n']): t for t in json.load(open(os.path.join(here, 'tasks.json')))}
pad = 6  # (a click a few pixels off a control's edge is on it)
# (the score is the first clicks right AND sure: a right click the judge was unsure of was found by luck or by elimination)
scores = []
for path in sys.argv[1:]:
    said = json.load(open(path))
    hits = sure = 0
    for n, t in tasks.items():
        a = said.get(n, [])
        x, y = (a + [-1, -1])[:2]
        how = a[2] if len(a) > 2 else 'lost'  # (not said how sure: counted as lost)
        hit = any(l - pad <= x <= r + pad and top - pad <= y <= b + pad for l, top, r, b in [t['rect'], *t.get('also', [])])  # (a task may have more than one right first click)
        hits += hit
        sure += hit and how == 'sure'
        if not (hit and how == 'sure'): print(f'  {"miss" if not hit else how} {n}: {t["task"]}')
    scores.append(sure / len(tasks))
    print(f'{os.path.basename(path)}: {hits} of {len(tasks)} first clicks right, {sure} right and sure ({100 * scores[-1]:.0f}%)')
if scores: print(f'right and sure: {100 * sum(scores) / len(scores):.0f}% (pass line 90%)')
