# The window tests the test package counts as not passed, read from its own json record (flutter test --file-reporter
# json, the test package's events testStart, testDone, error, print): each one's name and everything it said, as the
# github lines may show it passing.
import json, sys
ev = [json.loads(l) for l in open(sys.argv[1]) if l.startswith('{')]
name = {e['test']['id']: e['test']['name'] for e in ev if e['type'] == 'testStart'}
suite = {e['test']['id']: e['test'].get('suiteID') for e in ev if e['type'] == 'testStart'}
path = {e['suite']['id']: e['suite'].get('path') for e in ev if e['type'] == 'suite'}
done = {e['testID']: e for e in ev if e['type'] == 'testDone'}
for i, n in name.items():
    if i in done and done[i]['result'] == 'success': continue
    print(f"== {path.get(suite.get(i))}: {n}: {done[i]['result'] if i in done else 'never done'}{' (hidden)' if i in done and done[i].get('hidden') else ''}")
    for e in ev:
        if e.get('testID') == i and e['type'] in ('error', 'print'):
            print('   ', (e.get('error') or e.get('message') or '')[:400].replace('\n', '\n    '))
