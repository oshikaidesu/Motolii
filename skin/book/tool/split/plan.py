# The plan: a file's top-level declarations as a tree of who alone needs whom (the dominator tree of its use graph,
# from everything seen from outside), cut into pieces of at most LIMIT lines: a piece is a declaration and all that
# only it needs, unless that is too long, then its children are pieces of their own.
import sys, time, os, json; sys.path.insert(0, os.path.dirname(__file__)); from lsp import Lsp
LIMIT = int(os.environ.get('LIMIT', 400))
root = os.path.expanduser('~/motolii/skin/book')
def graph(s, f):
    uri = 'file://' + root + '/' + f
    syms = s.call('textDocument/documentSymbol', {'textDocument': {'uri': uri}})
    tops = [(x['name'], x['range']['start']['line'], x['range']['end']['line'], x['selectionRange']['start']) for x in syms]
    def owner(line): return next((t[0] for t in tops if t[1] <= line <= t[2]), None)
    uses = {t[0]: set() for t in tops}; outside = set()
    for name, a, b, at in tops:
        for r in s.call('textDocument/references', {'textDocument': {'uri': uri}, 'position': at, 'context': {'includeDeclaration': False}}):
            if r['uri'] != uri: outside.add(name); continue
            o = owner(r['range']['start']['line'])
            if o and o != name: uses[o].add(name)
    return tops, uses, outside
def dominators(nodes, succ, start):
    order, seen = [], set()
    def dfs(n):
        seen.add(n)
        for m in succ[n]:
            if m not in seen: dfs(m)
        order.append(n)
    dfs(start); order.reverse(); index = {n: i for i, n in enumerate(order)}
    pred = {n: [] for n in order}
    for n in order:
        for m in succ[n]:
            if m in pred: pred[m].append(n)
    idom = {start: start}
    def meet(a, b):
        while a != b:
            while index[a] > index[b]: a = idom[a]
            while index[b] > index[a]: b = idom[b]
        return a
    changed = True
    while changed:
        changed = False
        for n in order[1:]:
            ps = [p for p in pred[n] if p in idom]
            new = ps[0]
            for p in ps[1:]: new = meet(p, new)
            if idom.get(n) != new: idom[n] = new; changed = True
    return idom
def pieces(tops, uses, outside):
    size = {t[0]: t[2] - t[1] + 1 for t in tops}
    R = '<outside>'
    succ = {**{k: sorted(v) for k, v in uses.items()}, R: [t[0] for t in tops if not t[0].startswith('_') or t[0] in outside]}
    used = set(x for v in uses.values() for x in v)
    succ[R] += [t[0] for t in tops if t[0] not in used and t[0] not in succ[R]]   # (nothing uses it here: seen from outside, or dead)
    idom = dominators(list(size) + [R], succ, R)
    kids = {}
    for n, d in idom.items():
        if n != R: kids.setdefault(d, []).append(n)
    def total(n): return size.get(n, 0) + sum(total(k) for k in kids.get(n, []))
    out = []
    def cut(n):   # (n and what only it needs: one piece if it fits, else n alone and its children cut)
        if total(n) <= LIMIT or not kids.get(n): out.append((n, total(n), sub(n)))
        else:
            out.append((n, size[n], [n]))
            for k in kids[n]: cut(k)
    def sub(n): return [n] + [x for k in kids.get(n, []) for x in sub(k)]
    for k in kids.get(R, []): cut(k)
    return out, size
if __name__ == '__main__':
    files = sys.argv[1:]; s = Lsp(root)
    for f in files: s.send({'jsonrpc': '2.0', 'method': 'textDocument/didOpen', 'params': {'textDocument': {'uri': 'file://' + root + '/' + f, 'languageId': 'dart', 'version': 1, 'text': open(root + '/' + f).read()}}})
    time.sleep(15)
    for f in files:
        tops, uses, outside = graph(s, f); out, size = pieces(tops, uses, outside)
        big = [p for p in out if p[1] > LIMIT]
        small = sum(p[1] for p in out if p[1] <= LIMIT)
        print(f'{f}: {sum(size.values())} lines in declarations; {len(out)} pieces; too long alone: ' + ', '.join(f'{p[0]} {p[1]}' for p in big))
        for p in sorted(out, key=lambda p: -p[1])[:6]: print(f'     {p[1]:5d} {p[0]} (+{len(p[2]) - 1})')
