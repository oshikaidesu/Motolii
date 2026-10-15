#!/usr/bin/env python3
# Canvalry's distributions (MIT, Phill T, github.com/phillip-motion/Canvalry-scripts) made recipes as they are: the
# file whole, run by script.cpp's Cavalry stand-ins, with head lines added: a circle copied up to 1000 times, colours,
# each knob (nK, its label from the file's own comment, its default from its code) a track or a check; a 3D one turns
# round once in eight seconds, a phase goes round.
# usage: from_canvalry.py <Canvalry src/distributions> <recipes folder>
import os, re, sys

src, out = sys.argv[1], sys.argv[2]
skip = {"Rose Curve", "Phyllotaxis"}   # (in already: rose_curve.js as it is, sunflower.js the same golden angle)
palettes = ["ff3366 ffd166 3a86ff", "6437ff ff24e0 ffffff", "00f5d4 9b5de5 f15bb5", "ffbe0b fb5607 ff006e", "8ecae6 219ebc ffb703"]
made = []
for k, f in enumerate(sorted(os.listdir(src))):
    if not f.endswith(" Distribution.js"): continue
    title = f[: -len(" Distribution.js")]
    if title in skip: continue
    body = open(os.path.join(src, f)).read()
    labels = {m.group(1): m.group(2).strip() for m in re.finditer(r"^\s*(n\d+)\s+(.+?)\s{2,}|^\s*(n\d+)\s+(\S.*?)\s*\(", body, re.M) if m.group(1)}
    for m in re.finditer(r"^\s*(n\d+)\s+(.+?)\s*\(", body, re.M): labels.setdefault(m.group(1), m.group(2).strip())
    usual = {m.group(1): float(m.group(2)) for m in re.finditer(r"typeof (n\d+) !== \"undefined\"\)\s*\?\s*[^:]*?:\s*(-?[\d.]+)", body)}
    for n in set(re.findall(r"typeof (n\d+) !==", body)) - set(usual):   # (a default its code works out: the comment's, or off)
        said = re.search(rf"^\s*{n}\s+.*?\(default:\s*(-?[\d.]+)", body, re.M)
        usual[n] = float(said.group(1)) if said else 0.0
    knobs, keys, count = [], [], None
    for n in sorted(usual, key=lambda s: int(s[1:])):
        label = labels.get(n, n).replace(",", " ").replace(" degrees", "")
        v = usual[n]
        if re.search(r"Cull|Include", label):
            knobs.append(f"// --check@{n}:{label},{int(v)}"); continue
        if "Count" in label:
            count = n; knobs.append(f"// --track@{n}:{label},1,1000,{int(min(v, 1000))},1"); continue
        if label.startswith("Rotation"):
            knobs.append(f"// --track@{n}:{label},-360,360,{v:g},1")
            if label.endswith("Y"): keys += [f"// key js_{n} 0 0", f"// key js_{n} 8 360"]
            continue
        if label.startswith("Phase"):
            knobs.append(f"// --track@{n}:{label},0,360,{v:g},1"); keys += [f"// key js_{n} 0 0", f"// key js_{n} 8 360"]; continue
        whole = v == int(v) and re.search(r"Rows|Columns|Turns|Revolutions|Arm|Seed|Petal|Depth|Level|Order|Rings|Sides|Count|Lobes|Cusps|Frequency [AB]", label)
        low = 0 if v >= 0 else v * 3
        high = max(abs(v) * 3, 1)
        if "Perspective" in label: low, high = 0, 3000
        knobs.append(f"// --track@{n}:{label},{low:g},{high:g},{v:g},{1 if whole else round(high / 500, 4) or 0.001}")
    name = title.lower().replace(" ", "_")
    if os.path.exists(os.path.join(out, name + ".js")) and not open(os.path.join(out, name + ".js")).read().startswith(f"// {title}: Canvalry's"): name += "_points"
    head = [f"// {title}: Canvalry's {title} Distribution as it is (MIT, Phill T, github.com/phillip-motion/Canvalry-scripts), only these head lines added (filament/recipes/from_canvalry.py)",
            "// make shape circle", f"// say fill {palettes[k % len(palettes)].split()[0]}", "// set size 0.03", "// set copies 1000",
            f"// say copy_fill {palettes[k % len(palettes)]}"] + keys + knobs
    open(os.path.join(out, name + ".js"), "w").write("\n".join(head) + "\n\n" + body)
    made.append(name)
print(len(made), "made:", " ".join(made))
