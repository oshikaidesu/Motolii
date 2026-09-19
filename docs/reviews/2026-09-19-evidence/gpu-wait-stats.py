"""Summarize MOTOLII_JANK logs; only the measured playback interval counts."""
import math
import re
import statistics
import sys


def summary(values):
    values = sorted(values)
    if not values:
        return "—"
    return " / ".join(f"{v / 1000:.2f}" for v in (
        statistics.median(values), values[math.ceil(len(values) * .9) - 1], values[-1]
    ))


for path in sys.argv[1:]:
    lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
    start = next(i for i, line in enumerate(lines) if "verdict=play-start" in line)
    end = next(i for i, line in enumerate(lines[start + 1:], start + 1) if "verdict=play-stop" in line)
    ticks = [dict(re.findall(r"(\w+)=([^ ]+)", line)) for line in lines[start:end]
             if "room=jank total=" in line and "playback=true" in line]
    rendered = [tick for tick in ticks if "Camera" in tick or "User" in tick]
    frames = [dict(re.findall(r"(\w+)=([^ ]+)", line)) for line in lines
              if "room=jank-frame " in line and "playback=true" in line]
    print(path)
    print("stage: median / p90 / max (ms)")
    for key in ("total", "tick", "info", "Camera", "User", "status", "decode", "accept"):
        print(f"{key}: {summary([int(row[key]) for row in rendered if key in row])}")
    for key in ("build", "raster", "total"):
        print(f"Flutter {key}: {summary([int(row[key]) for row in frames])}")
    occupied = sum(int(tick["total"]) for tick in ticks)
    print(f"rendered={len(rendered)} ticks={len(ticks)} frames={len(frames)}")
    print(f"UI occupancy={occupied / 120000:.2f}% ({occupied / 1000:.2f} ms / 12000 ms)")
    for threshold in (16700, 33000):
        print(f"Flutter total > {threshold / 1000} ms: {sum(int(f['total']) > threshold for f in frames)}")
    probes = [line for line in lines if "room=stage-window" in line and "lag=" in line]
    bad = [line for line in probes if not re.search(r"\blag=0\b", line)]
    print(f"window probes={len(probes)} nonzero lag={len(bad)}")
    print("\n".join(line for line in lines if "verdict=play-stop" in line))
