# Named top-level declarations moved to a file by Dart's own move_top_level_to_file (python3 -I tool/split/move.py from.dart to.dart Name...).
import sys, time, os, threading; sys.path.insert(0, os.path.dirname(__file__)); from lsp import Lsp; from plan import root
src, dst, names = root + '/' + sys.argv[1], root + '/' + sys.argv[2], sys.argv[3:]
s = Lsp(root); uri = 'file://' + src; v = [1]
s.send({'jsonrpc': '2.0', 'method': 'textDocument/didOpen', 'params': {'textDocument': {'uri': uri, 'languageId': 'dart', 'version': 1, 'text': open(src).read()}}})
time.sleep(15)
def put(edit):
    for ch in edit.get('documentChanges', []):
        if 'kind' in ch: open(ch['uri'][7:], 'a').close(); continue
        p = ch['textDocument']['uri'][7:]; text = open(p).read(); lines = text.split('\n')
        def off(pos): return sum(len(l) + 1 for l in lines[:pos['line']]) + pos['character']
        for i, e in sorted(enumerate(ch['edits']), key=lambda ie: (off(ie[1]['range']['start']), ie[0]), reverse=True):
            text = text[:off(e['range']['start'])] + e['newText'] + text[off(e['range']['end']):]
        open(p, 'w').write(text)
    v[0] += 1; s.send({'jsonrpc': '2.0', 'method': 'textDocument/didChange', 'params': {'textDocument': {'uri': uri, 'version': v[0]}, 'contentChanges': [{'text': open(src).read()}]}})
for n in names:
    r = next(x['selectionRange'] for x in s.call('textDocument/documentSymbol', {'textDocument': {'uri': uri}}) if x['name'] == n)
    mv = next(a for a in s.call('textDocument/codeAction', {'textDocument': {'uri': uri}, 'range': {'start': r['start'], 'end': r['start']}, 'context': {'diagnostics': []}}) if a.get('kind') == 'refactor.move')
    args = mv['command']['arguments']; args[0]['arguments'] = ['file://' + dst]
    t = threading.Thread(target=lambda: s.call('workspace/executeCommand', {'command': mv['command']['command'], 'arguments': args})); t.start()
    while t.is_alive() or not s.reqs.empty():
        try: q = s.reqs.get(timeout=1)
        except Exception: continue
        if q['method'] == 'workspace/applyEdit': put(q['params']['edit']); s.answer(q, {'applied': True})
        else: s.answer(q, None)
    print('moved', n)
t = threading.Thread(target=lambda: s.call('workspace/executeCommand', {'command': 'dart.edit.organizeImports', 'arguments': [{'path': dst}]})); t.start()
while t.is_alive() or not s.reqs.empty():
    try: q = s.reqs.get(timeout=1)
    except Exception: continue
    if q['method'] == 'workspace/applyEdit': put(q['params']['edit']); s.answer(q, {'applied': True})
    else: s.answer(q, None)
