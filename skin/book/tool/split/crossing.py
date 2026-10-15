# The private names of a library made of parts that one file declares and another of its files uses (Dart's own
# documentSymbol and references): what has to be public before each part can be a library of its own.
import sys, time, os, re; sys.path.insert(0, os.path.dirname(__file__)); from lsp import Lsp; from plan import root
lib = sys.argv[1]; d = os.path.dirname(lib)
parts = [os.path.normpath(d + '/' + m) for m in re.findall(r"^part '([^']+)';", open(root + '/' + lib).read(), re.M)]
files = [lib] + parts
s = Lsp(root)
for f in files: s.send({'jsonrpc': '2.0', 'method': 'textDocument/didOpen', 'params': {'textDocument': {'uri': 'file://' + root + '/' + f, 'languageId': 'dart', 'version': 1, 'text': open(root + '/' + f).read()}}})
time.sleep(25)
cross = []
for f in files:
    uri = 'file://' + root + '/' + f
    for x in s.call('textDocument/documentSymbol', {'textDocument': {'uri': uri}}):
        if not x['name'].startswith('_'): continue
        rs = s.call('textDocument/references', {'textDocument': {'uri': uri}, 'position': x['selectionRange']['start'], 'context': {'includeDeclaration': False}})
        others = sorted(set(r['uri'].split('/')[-1] for r in rs if r['uri'] != uri))
        if others: cross.append((f, x['name'], others))
for c in cross: print(c[0].split('/')[-1], c[1], '->', ' '.join(c[2]))
print(len(cross), 'private names cross files, of', len(files), 'files')
