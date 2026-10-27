# A library made of parts turned into libraries of their own (python3 -I tool/split/unpart.py lib/...dart, from skin/book):
# each private name one file declares and another uses is renamed public by Dart's own rename; each part then takes the
# library's imports and imports the library; the library exports each part, so whoever imported it sees the same names.
import sys, time, os, re; sys.path.insert(0, os.path.dirname(__file__)); from lsp import Lsp; from plan import root
lib = sys.argv[1]; d = os.path.dirname(lib); L = root + '/' + lib
parts = re.findall(r"^part '([^']+)';", open(L).read(), re.M)
files = [lib] + [os.path.normpath(d + '/' + p) for p in parts]
s = Lsp(root); ver = {}
def show(f):
    ver[f] = ver.get(f, 0) + 1; m = 'textDocument/didOpen' if ver[f] == 1 else 'textDocument/didChange'
    p = {'textDocument': {'uri': 'file://' + root + '/' + f, 'languageId': 'dart', 'version': ver[f], 'text': open(root + '/' + f).read()}} if ver[f] == 1 else {'textDocument': {'uri': 'file://' + root + '/' + f, 'version': ver[f]}, 'contentChanges': [{'text': open(root + '/' + f).read()}]}
    s.send({'jsonrpc': '2.0', 'method': m, 'params': p})
def put(edit):
    for ch in edit.get('documentChanges', []):
        p = ch['textDocument']['uri'][7:]; text = open(p).read(); lines = text.split('\n')
        def off(pos): return sum(len(l) + 1 for l in lines[:pos['line']]) + pos['character']
        for i, e in sorted(enumerate(ch['edits']), key=lambda ie: (off(ie[1]['range']['start']), ie[0]), reverse=True):
            text = text[:off(e['range']['start'])] + e['newText'] + text[off(e['range']['end']):]
        open(p, 'w').write(text); show(os.path.relpath(p, root))
for f in files: show(f)
time.sleep(25)
taken = set()
for f in files:
    for x in s.call('textDocument/documentSymbol', {'textDocument': {'uri': 'file://' + root + '/' + f}}): taken.add(x['name'])
import subprocess
def errors(): return subprocess.run([os.environ.get('FLUTTER', 'flutter'), 'analyze', 'lib', 'test', 'integration_test'], cwd=root, capture_output=True, text=True).stdout.count('error •')
base = errors(); done = 0
for f in files:
    uri = 'file://' + root + '/' + f
    for x in s.call('textDocument/documentSymbol', {'textDocument': {'uri': uri}}):
        n = x['name']
        if not n.startswith('_'): continue
        rs = s.call('textDocument/references', {'textDocument': {'uri': uri}, 'position': x['selectionRange']['start'], 'context': {'includeDeclaration': False}})
        if not any(r['uri'] != uri for r in rs): continue
        word = os.path.basename(f)[:-5].split('_')[-1].capitalize()
        for new in [n.lstrip('_'), n.lstrip('_') + word, n.lstrip('_') + word + 'Of']:   # (plain, else with its file's last word: a rename that hides or is hidden by a name is taken back)
            if new in taken: continue
            sym = next(y for y in s.call('textDocument/documentSymbol', {'textDocument': {'uri': uri}}) if y['name'] == n)
            try: edit = s.call('textDocument/rename', {'textDocument': {'uri': uri}, 'position': sym['selectionRange']['start'], 'newName': new})
            except RuntimeError: continue
            was = {ch['textDocument']['uri'][7:]: open(ch['textDocument']['uri'][7:]).read() for ch in edit.get('documentChanges', [])}
            put(edit)
            if errors() <= base: break
            for q, t in was.items(): open(q, 'w').write(t); show(os.path.relpath(q, root))   # (taken back as it was)
            time.sleep(2)
        else: raise SystemExit('no public name found for ' + n)
        taken.add(new); done += 1; print('public', n, '->', new)
text = open(L).read()
heads = ''.join(m.group(0) + '\n' for m in re.finditer(r"^import [^;]*;", text, re.M | re.S))
for p in parts:
    P = root + '/' + os.path.normpath(d + '/' + p); t = open(P).read()
    t = re.sub(r"^part of [^;]*;\n", heads + "import '" + os.path.basename(lib) + "';\n", t, count=1, flags=re.M)
    open(P, 'w').write(t)
open(L, 'w').write(re.sub(r"^part '([^']+)';", r"import '\1';\nexport '\1';", text, flags=re.M))   # (the library uses them too)
print(done, 'names made public;', len(parts), 'parts now libraries')
