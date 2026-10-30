# Starts Blender in the background as a geometry-node calculator and times 120 frames of "send a knob value,
# get the mesh back" for each tree: the whole round trip, Blender's evaluation, and the mesh read-out.
import socket, struct, subprocess, sys, time, statistics, os

BLENDER = '/Applications/Blender.app/Contents/MacOS/Blender'
here = os.path.dirname(os.path.abspath(__file__))

def run(tree, port):
    p = subprocess.Popen([BLENDER, '-b', '--factory-startup', '--python', os.path.join(here, 'server.py'), '--', str(port), tree],
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    t_start = time.perf_counter()
    for line in p.stdout:
        if 'READY' in line: break
    ready = time.perf_counter() - t_start
    c = socket.create_connection(('127.0.0.1', port)); c.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
    f = c.makefile('rb')
    trips, evals, reads = [], [], []
    for i in range(130):
        t0 = time.perf_counter()
        c.sendall(struct.pack('<f', 0.2 + 0.01 * i))
        v, tr, ev, rd = struct.unpack('<iiff', f.read(16))
        f.read(v * 12 + tr * 12)
        if i >= 10: trips.append((time.perf_counter() - t0) * 1e3); evals.append(ev); reads.append(rd)
    c.close(); p.kill(); p.wait()
    med = statistics.median
    p95 = lambda xs: sorted(xs)[int(len(xs) * .95)]
    print(f'{tree:12s} verts {v:7d} tris {tr:7d} | start {ready:5.2f} s | round trip median {med(trips):6.2f} ms p95 {p95(trips):6.2f} ms | eval {med(evals):6.2f} ms | read-out {med(reads):6.2f} ms', flush=True)

for i, tree in enumerate(sys.argv[1:] or ['grid100', 'grid316', 'scatter1000', 'scatter5000']):
    run(tree, 47100 + i)
