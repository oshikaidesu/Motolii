# A plain LSP client for Dart's own language server: send a request, read the answer.
import json, subprocess, os, sys, threading, queue
DART = os.path.expanduser('~/rust_ae/Motolii/.tools/flutter/bin/dart')
class Lsp:
    def __init__(self, root):
        self.p = subprocess.Popen([DART, 'language-server', '--protocol=lsp', '--client-id=split'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, cwd=root)
        self.n = 0; self.q = {}; self.notes = queue.Queue(); self.reqs = queue.Queue()
        threading.Thread(target=self._read, daemon=True).start()
        self.call('initialize', {'processId': os.getpid(), 'rootUri': 'file://' + root, 'capabilities': {'workspace': {'applyEdit': True, 'workspaceEdit': {'documentChanges': True, 'resourceOperations': ['create', 'rename', 'delete']}}, 'textDocument': {'documentSymbol': {'hierarchicalDocumentSymbolSupport': True}, 'codeAction': {'codeActionLiteralSupport': {'codeActionKind': {'valueSet': ['', 'quickfix', 'refactor', 'refactor.move', 'source']}}}}, 'experimental': {'supportsDartTextDocumentContentProvider': False}}, 'initializationOptions': {}})
        self.send({'jsonrpc': '2.0', 'method': 'initialized', 'params': {}})
    def send(self, m):
        b = json.dumps(m).encode(); self.p.stdin.write(b'Content-Length: %d\r\n\r\n' % len(b) + b); self.p.stdin.flush()
    def _read(self):
        f = self.p.stdout
        while True:
            h = {}
            while True:
                l = f.readline()
                if not l: return
                l = l.decode().strip()
                if not l: break
                k, v = l.split(':', 1); h[k.lower()] = v.strip()
            m = json.loads(f.read(int(h['content-length'])))
            if 'id' in m and 'method' in m: self.reqs.put(m)
            elif 'id' in m: self.q[m['id']].put(m)
            else: self.notes.put(m)
    def call(self, method, params, timeout=120):
        self.n += 1; i = self.n; self.q[i] = queue.Queue()
        self.send({'jsonrpc': '2.0', 'id': i, 'method': method, 'params': params})
        m = self.q[i].get(timeout=timeout)
        if 'error' in m: raise RuntimeError(m['error'])
        return m['result']
    def answer(self, req, result): self.send({'jsonrpc': '2.0', 'id': req['id'], 'result': result})
