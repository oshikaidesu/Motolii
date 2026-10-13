#!/usr/bin/env python3
# lottie3d.py: makes a Lottie file's third dimension readable through ThorVG as it is.
# ThorVG (1.1.2) reads a Lottie file's 3D layers but keeps only what is flat: the third number of position, anchor
# and size is dropped as the file is read, the turns about x and y and the orientation are worked out and flattened,
# a camera's zoom is not read. Nothing of that can be asked back. So before the file goes to ThorVG, every number
# that would be lost is written out once more on helper Null layers, as plain 2D positions, which ThorVG does keep
# and work out (keys, easing, expressions on single numbers). motion.h reads them back and puts the layer together.
#   ./lottie3d.py in.json out.json
# For the layer with index N (Lottie's "ind") of the main composition, the helpers are named:
#   "N.p" position x, y       "N.a" anchor x, y        "N.s" size x, y          "N.o" orientation x, y
#         (and, for a layer that turns to face its way along its path, turned as ThorVG turns it: about z, by the path's x, y)
#   "N.z" position z, anchor z                "N.w" size z, orientation z
#   "N.r" turn about x, about y               "N.q" turn about z, camera zoom  "N.h" skew, skew axis
#   "N.k" parent's index + 1 (0: none), kind (0 flat, 1 a 3D layer, 2 a camera that looks at a point: its "anchor" is that point, 3 a camera that does not)
#   "<layer's name>.i" its index            "N.m" (for a flat layer that 3D layers stand under) the flat layer's place
# What cannot be carried: a 3D position keyed along a curved path is split into x, y (curved as it was, measured flat)
# and z (straight): keep exact by keying its dimensions apart in After Effects (Separate Dimensions). Expressions on
# a whole position, anchor, size or orientation stay with x, y only. 3D layers inside precompositions are not written out.
import copy, json, sys

def fixed(v):
    return {"a": 0, "k": v}

def one(prop, i, default):
    """the i-th number of a property, as a property of one number (its keys and easing kept)"""
    if prop is None:
        return fixed(default)
    k = prop.get("k")
    if not prop.get("a"):
        return fixed(k[i] if isinstance(k, list) and i < len(k) else (default if isinstance(k, list) else k))
    def pick(v):
        return [v[i] if isinstance(v, list) and i < len(v) else (v[0] if isinstance(v, list) else v)]
    keys = []
    for key in k:
        n = {"t": key["t"]}
        for f in ("s", "e"):
            if f in key:
                n[f] = [key[f][i] if i < len(key[f]) else default]
        if key.get("h"):
            n["h"] = 1
        for f in ("i", "o"):
            if f in key:
                n[f] = {"x": pick(key[f]["x"]), "y": pick(key[f]["y"])}
        keys.append(n)
    out = {"a": 1, "k": keys}
    if "x" in prop and not isinstance(prop.get("k", [{}])[0].get("s", [0]) if prop.get("a") else 0, list):
        out["x"] = prop["x"]
    return out

def single(prop, default):
    """a property of one number (a turn, a zoom) as it is, or a fixed default"""
    return copy.deepcopy(prop) if prop is not None else fixed(default)

def main(src, dst):
    film = json.load(open(src))
    layers = film.get("layers", [])
    first, last = film.get("ip", 0), film.get("op", 0)
    spare = [max([l.get("ind", 0) for l in layers] + [0]) + 1000]
    helpers = []

    def helper(name, position, parent=None, facing=0):
        spare[0] += 1
        h = {"ddd": 0, "ind": spare[0], "ty": 3, "nm": name, "sr": 1, "ip": first, "op": last, "st": 0, "ao": facing,
             "ks": {"o": fixed(100), "r": fixed(0), "p": position, "a": fixed([0, 0, 0]), "s": fixed([100, 100, 100])}}
        if parent is not None:
            h["parent"] = parent
        helpers.append(h)

    def pair(name, x, y):
        helper(name, {"s": True, "x": x, "y": y})

    deep = {l["ind"] for l in layers if "ind" in l and (l.get("ddd") or l.get("ty") == 13)}
    if not deep:
        json.dump(film, open(dst, "w"), separators=(",", ":"))
        return
    above = {l.get("parent") for l in layers if l.get("ind") in deep}   # layers that 3D layers stand under
    for l in layers:
        n = l.get("ind")
        if n is None:
            continue
        ks = l.get("ks", {})
        kind = (2 if "a" in ks else 3) if l.get("ty") == 13 else (1 if n in deep else 0)
        if kind or n in above:
            pair("%d.k" % n, fixed(l.get("parent", -1) + 1), fixed(kind))
        if kind == 0:
            if n in above:
                helper("%d.m" % n, fixed([0, 0, 0]), parent=n)
            continue
        if l.get("nm"):
            pair(l["nm"] + ".i", fixed(n), fixed(0))
        p = ks.get("p")
        if p is not None and p.get("s") is True:   # position keyed dimension by dimension
            helper("%d.p" % n, {"s": True, "x": single(p.get("x"), 0), "y": single(p.get("y"), 0)})
            pz = single(p.get("z"), 0)
        else:
            helper("%d.p" % n, copy.deepcopy(p) if p is not None else fixed([0, 0, 0]), facing=l.get("ao", 0))
            pz = one(p, 2, 0)
        helper("%d.a" % n, copy.deepcopy(ks["a"]) if "a" in ks else fixed([0, 0, 0]))
        helper("%d.s" % n, copy.deepcopy(ks["s"]) if "s" in ks else fixed([100, 100, 100]))
        helper("%d.o" % n, copy.deepcopy(ks["or"]) if "or" in ks else fixed([0, 0, 0]))
        pair("%d.z" % n, pz, one(ks.get("a"), 2, 0))
        pair("%d.w" % n, one(ks.get("s"), 2, 100), one(ks.get("or"), 2, 0))
        pair("%d.r" % n, single(ks.get("rx"), 0), single(ks.get("ry"), 0))
        pair("%d.q" % n, single(ks.get("rz", ks.get("r")), 0), single(l.get("pe"), 0))
        if "sk" in ks or "sa" in ks:
            pair("%d.h" % n, single(ks.get("sk"), 0), single(ks.get("sa"), 0))
    film["layers"] = layers + helpers
    json.dump(film, open(dst, "w"), separators=(",", ":"))

if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
