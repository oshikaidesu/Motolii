#!/bin/zsh
# The joined checks each end with expectSameAfterReopen where it holds: put it before every _close, run, take it out of
# those that still differ, and list what differs (from skin/book: tool/reopen_sweep.sh).
cd "${0:A:h}/.."
FL="${FLUTTER:-flutter}"
T=test/workspace_joined_test.dart; LOG=${REOPEN_LOG:-$(mktemp)}
perl -0pi -e 's/^ +await expectSameAfterReopen\(t\);\n//mg; s/^( +)await _close\(t\);/$1await expectSameAfterReopen(t);\n$1await _close(t);/mg' $T
$FL test $T > $LOG 2>&1
grep -qE "^[0-9:]+ \+[0-9]+.*(All tests passed|Some tests failed)" $LOG || { echo "the checks did not run:"; tail -20 $LOG; git checkout -q -- $T; exit 1; }   # (a check that never ran says nothing differs)
grep "^REOPEN	" $LOG | cut -f2 | sort -u > $LOG.red
python3 - "$T" "$LOG.red" <<'P'
import re, sys
red = [l.strip() for l in open(sys.argv[2]) if l.strip()]
s = open(sys.argv[1]).read(); out = []
for part in re.split(r"(?=\n  testWidgets\()", s):
    m = re.match(r"\n  testWidgets\('((?:[^'\\]|\\.)*)'", part)
    name = m.group(1).replace("\\'", "'") if m else None
    if name and any(name.split(':')[0] == r or name.startswith(r) for r in red): part = part.replace("    await expectSameAfterReopen(t);\n", "")
    out.append(part)
open(sys.argv[1], 'w').write(''.join(out))
P
echo "held: $(grep -c expectSameAfterReopen $T), differ: $(wc -l < $LOG.red | tr -d ' ')"
grep "^REOPEN	" $LOG | cut -f2 | sort | uniq -c | sort -rn | cut -c1-100
