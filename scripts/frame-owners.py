"""1 コマの時間を持ち主ごとに割る。走っているアプリの VM service に繋ぎ、
Flutter 標準の Timeline を吸い出して 1 枚の表にする。自作の計器は最小限。

  python3 scripts/frame-owners.py ws://127.0.0.1:PORT/TOKEN=/ws [秒]

拾う物(全部 Flutter / Dart VM の標準):
  Frame / Animate / BUILD / LAYOUT / PAINT / POST_FRAME  — SchedulerBinding と RendererBinding
  ext.flutter.profileWidgetBuilds        — widget ごとの build(名前 = 責任の名前)
  ext.flutter.profileRenderObjectLayouts — RenderObject ごとの layout
  ext.flutter.profileRenderObjectPaints  — RenderObject ごとの paint
  ext.flutter.profilePlatformChannels    — channel の往復(符号化は含まれない)
  motolii.*                              — FFI の先(Rust と GPU)。Flutter のどの計器にも映らないので自前
"""
import json, math, statistics, sys, time, websocket


class VM:
    def __init__(self, uri):
        self.ws = websocket.create_connection(uri, timeout=90)
        self.n = 0

    def call(self, method, **params):
        self.n += 1
        ident = str(self.n)
        self.ws.send(json.dumps({"jsonrpc": "2.0", "id": ident, "method": method, "params": params}))
        while True:
            raw = self.ws.recv()
            if isinstance(raw, bytes):
                continue
            msg = json.loads(raw)
            if msg.get("id") == ident:
                if "error" in msg:
                    raise RuntimeError(f"{method}: {msg['error']}")
                return msg["result"]


EXTENSIONS = ("ext.flutter.profileWidgetBuilds", "ext.flutter.profileRenderObjectLayouts",
              "ext.flutter.profileRenderObjectPaints", "ext.flutter.profilePlatformChannels")


def summary(us):
    v = sorted(us)
    p90 = v[math.ceil(len(v) * .9) - 1]
    return f"{statistics.median(v)/1000:8.2f} {p90/1000:8.2f} {v[-1]/1000:8.2f} {len(v):6d} {sum(v)/1000:9.1f}"


def spans(events):
    """(名前 -> 長さ µs の列)。X = 済んだ区間、B/E = 始と終、b/e = 非同期の対。"""
    out, opened = {}, {}
    for e in events:
        name, ph = e.get("name", "?"), e.get("ph")
        if ph == "X":
            out.setdefault(name, []).append(e.get("dur", 0))
        elif ph in ("B", "E"):
            key = (e.get("tid"), name)
            if ph == "B":
                opened.setdefault(key, []).append(e["ts"])
            elif opened.get(key):
                out.setdefault(name, []).append(e["ts"] - opened[key].pop())
        elif ph in ("b", "e"):
            key = (name, e.get("id"))
            if ph == "b":
                opened.setdefault(key, []).append(e["ts"])
            elif opened.get(key):
                out.setdefault(name, []).append(e["ts"] - opened[key].pop())
    return out


def report(events):
    by_name = spans(events)
    threads = {e["tid"]: e["args"]["name"] for e in events
               if e.get("ph") == "M" and e.get("name") == "thread_name"}
    print(f"events={len(events)}  threads={sorted(set(threads.values()))}\n")
    header = f"{'owner (ms)':46}   median      p90      max      n     total"
    frame = ["Frame", "Animate", "BUILD", "LAYOUT", "PAINT", "COMPOSITING",
             "UPDATING COMPOSITING BITS", "SEMANTICS", "POST_FRAME"]
    mine = sorted(n for n in by_name if n.startswith("motolii."))
    raster = [n for n in by_name if "Rasterizer" in n or "SurfaceFrame" in n or "Pipeline" in n]
    channel = [n for n in by_name if "Channel" in n or "platform channel" in n.lower()]
    for title, names in (("1 コマの骨格(UI thread)", frame),
                         ("FFI の先(Rust + GPU): 自前の区間", mine),
                         ("raster thread", raster),
                         ("platform channel(符号化は含まれない)", channel)):
        print(f"--- {title} ---")
        print(header)
        for name in names:
            if name in by_name:
                print(f"{name[:46]:46} {summary(by_name[name])}")
        print()
    print("--- 一番重い持ち主 30(合計順。widget / RenderObject の名前が出る)---")
    print(header)
    skip = set(frame) | set(mine) | set(raster) | set(channel)
    rows = sorted(((n, v) for n, v in by_name.items() if n not in skip),
                  key=lambda kv: -sum(kv[1]))[:30]
    for name, us in rows:
        print(f"{name[:46]:46} {summary(us)}")
    if "Frame" in by_name:
        over = by_name["Frame"]
        print(f"\nFrame > 16.7 ms: {sum(v > 16700 for v in over)} / {len(over)}"
              f"   > 33 ms: {sum(v > 33000 for v in over)}")
        if "Animate" in by_name:
            print(f"Animate の合計 / Frame の合計 = "
                  f"{sum(by_name['Animate'])/max(sum(over),1)*100:.1f}%"
                  "  (Animate ⊂ Frame: Ticker は buildDuration の中)")


def main():
    uri, seconds = sys.argv[1], float(sys.argv[2]) if len(sys.argv) > 2 else 12.0
    out = sys.argv[3] if len(sys.argv) > 3 else "/tmp/motolii-timeline.json"
    vm = VM(uri)
    isolate = vm.call("getVM")["isolates"][0]["id"]
    for ext in EXTENSIONS:
        try:
            vm.call(ext, isolateId=isolate, enabled="true")
        except RuntimeError as e:
            print(f"  {ext}: {e}", file=sys.stderr)
    vm.call("setVMTimelineFlags", recordedStreams=["Dart", "Embedder", "GC"])
    vm.call("clearVMTimeline")
    print(f"--- {seconds:.0f} 秒 記録中: 今すぐ再生を押してください ---", file=sys.stderr)
    time.sleep(seconds)
    timeline = vm.call("getVMTimeline")
    for ext in EXTENSIONS:
        try:
            vm.call(ext, isolateId=isolate, enabled="false")
        except RuntimeError:
            pass
    vm.call("setVMTimelineFlags", recordedStreams=[])
    json.dump(timeline, open(out, "w"))
    print(f"(生の timeline: {out})", file=sys.stderr)
    report(timeline["traceEvents"])


if __name__ == "__main__":
    main()
