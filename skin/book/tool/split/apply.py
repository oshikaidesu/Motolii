# Cut one file by the plan (python3 -I tool/split/apply.py lib/...dart, from skin/book; MIN=60 lines a piece), with Dart's own refactors only: a private name that another piece needs is renamed public
# (textDocument/rename), then each piece of MIN lines or more is moved to a file named after it (move_top_level_to_file).
import sys, time, os, re, json, threading; sys.path.insert(0, os.path.dirname(__file__)); from lsp import Lsp; from plan import graph, pieces, root, LIMIT
MIN = int(os.environ.get('MIN', 60))
f = sys.argv[1]; path = root + '/' + f; uri = 'file://' + path
s = Lsp(root); ver = [1]
def opened():
    s.send({'jsonrpc': '2.0', 'method': 'textDocument/didOpen', 'params': {'textDocument': {'uri': uri, 'languageId': 'dart', 'version': ver[0], 'text': open(path).read()}}})
def reload(p):
    ver[0] += 1
    s.send({'jsonrpc': '2.0', 'method': 'textDocument/didChange', 'params': {'textDocument': {'uri': 'file://' + p, 'version': ver[0]}, 'contentChanges': [{'text': open(p).read()}]}})
def put(edit):   # (a workspace edit written to the files)
    touched = []
    for ch in edit.get('documentChanges', []):
        if 'kind' in ch:
            if ch['kind'] == 'create': open(ch['uri'][7:], 'a').close()
            continue
        p = ch['textDocument']['uri'][7:]; text = open(p).read(); lines = text.split('\n')
        def off(pos): return sum(len(l) + 1 for l in lines[:pos['line']]) + pos['character']
        for i, e in sorted(enumerate(ch['edits']), key=lambda ie: (off(ie[1]['range']['start']), off(ie[1]['range']['end']), ie[0]), reverse=True):   # (at one place, the later edit first: they then read in their order)
            a, b = off(e['range']['start']), off(e['range']['end']); text = text[:a] + e['newText'] + text[b:]
        open(p, 'w').write(text); touched.append(p)
    for p in edit.get('changes', {}):
        raise RuntimeError('changes form not handled')
    for p in touched:
        if p != path: s.send({'jsonrpc': '2.0', 'method': 'textDocument/didOpen', 'params': {'textDocument': {'uri': 'file://' + p, 'languageId': 'dart', 'version': 1, 'text': open(p).read()}}})
    reload(path)
def serve(t):
    while t.is_alive() or not s.reqs.empty():
        try: r = s.reqs.get(timeout=1)
        except Exception: continue
        if r['method'] == 'workspace/applyEdit': put(r['params']['edit']); s.answer(r, {'applied': True})
        else: s.answer(r, None)
def snake(n): return re.sub(r'(?<!^)(?=[A-Z])', '_', n.lstrip('_')).lower()
stem = os.path.basename(path)[:-5]
def named(n):   # (a piece's file: the file it came from, then the piece's own words that the first does not say)
    w = snake(n).split('_'); sw = stem.split('_')
    while w and w[0] in sw: w = w[1:]
    return os.path.dirname(path) + '/' + '_'.join(sw + w) + '.dart'
opened(); time.sleep(15)
tops, uses, outside = graph(s, f); out, size = pieces(tops, uses, outside)
own = [p for p in out if p[1] >= MIN]
home = {x: i for i, p in enumerate(own) for x in p[2]}   # (which piece a declaration goes to; not in one: stays)
need = set()
for a, us in uses.items():
    for b in us:
        if b.startswith('_') and home.get(a) != home.get(b): need.add(b)
for name in sorted(need):   # (renamed public: Dart's own rename, every use with it)
    syms = s.call('textDocument/documentSymbol', {'textDocument': {'uri': uri}})
    at = next(x['selectionRange']['start'] for x in syms if x['name'] == name)
    new = name.lstrip('_'); new = new[0].upper() + new[1:] if name[1].isupper() else new
    put(s.call('textDocument/rename', {'textDocument': {'uri': uri}, 'position': at, 'newName': new}))
    print('renamed', name, '->', new)
ren = lambda n: (n.lstrip('_')[0].upper() + n.lstrip('_')[1:] if n[1:2].isupper() else n.lstrip('_')) if n in need else n
for p in own:
    names = [ren(x) for x in p[2]]
    syms = s.call('textDocument/documentSymbol', {'textDocument': {'uri': uri}})
    target = named(ren(p[0]))
    for nm in names:
        syms = s.call('textDocument/documentSymbol', {'textDocument': {'uri': uri}})
        r = next(x['selectionRange'] for x in syms if x['name'] == nm)
        acts = s.call('textDocument/codeAction', {'textDocument': {'uri': uri}, 'range': {'start': r['start'], 'end': r['start']}, 'context': {'diagnostics': []}})
        mv = next(x for x in acts if x.get('kind') == 'refactor.move')
        args = mv['command']['arguments']; args[0]['arguments'] = ['file://' + target]
        t = threading.Thread(target=lambda: s.call('workspace/executeCommand', {'command': mv['command']['command'], 'arguments': args})); t.start(); serve(t)
    print('moved', len(names), 'to', os.path.relpath(target, root), sum(1 for _ in open(target)), 'lines')
written = sorted(set(named(ren(p[0])) for p in own))
for w in written + [path]:
    if w != path: s.send({'jsonrpc': '2.0', 'method': 'textDocument/didOpen', 'params': {'textDocument': {'uri': 'file://' + w, 'languageId': 'dart', 'version': 50, 'text': open(w).read()}}})
    time.sleep(1)
    t = threading.Thread(target=lambda: s.call('workspace/executeCommand', {'command': 'dart.edit.organizeImports', 'arguments': [{'path': w}]})); t.start(); serve(t)
for w in written:   # (its first line: what the piece itself says it is, the first sentence of its own /// words)
    text = open(w).read(); m = re.search(r'^///\s*(.+)$', text, re.M)
    if m:
        say = m.group(1).strip().rstrip('.')
        lines = text.split('\n')
        if lines[0].startswith('//'): lines[0] = '// ' + say + '.'
        else: lines.insert(0, '// ' + say + '.')
        open(w, 'w').write('\n'.join(lines))
print('left', sum(1 for _ in open(path)), 'lines in', f)
