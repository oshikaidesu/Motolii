#!/usr/bin/env python3
"""
blobtrack — the blob-tracking video effect (TouchDesigner "Blob Track" look).

Boxes lock onto what is in the footage, lines connect them, a few boxes turn
into inverted or shifted "windows", white chips read the blobs' real
measurements, and the strokes are screened into the picture.

1. DETECT  the grey frame (at a working resolution whose short side is 540) is
           thresholded four ways: bright, dark, edge-dense, and motion against
           three frames earlier. Each mask is split into connected components;
           a blob's box is its component's own bounding box.
2. TRACK   blobs are matched frame to frame by overlap and centroid distance
           and keep a stable ID; boxes are smoothed so they ride the object.
           Shi-Tomasi corners are followed with pyramidal Lucas-Kanade flow.
3. DRAW    windows, then a white strokes layer (screen-blended) and a chips
           layer (normal-blended).

The method is specified in references/method.md; tuned at 1080x1920 and
scaled to any frame size (k = min(W, H) / 1080 for drawn sizes, detection at
a fixed 540 px short side).

Module:
    from blobtrack import BlobTracker
    bt = BlobTracker(width, height, bright=90, ...)
    out = bt.process(rgb)          # HxWx3 uint8 in, HxWx3 uint8 out
    bt.last_tracks                 # [{id, x, y, w, h, kind, a, age}, ...]

CLI:
    python blobtrack.py INPUT -o OUTPUT.mp4 [--start SEC] [--duration SEC]
                        [--bright 90] ... [--crf 16] [--no-audio]

Notes for anyone porting this (numbers must match for parity):
  * working size = (int(W*S + 0.5), int(H*S + 0.5)), S = 540 / min(W, H),
    grey = cvtColor RGB2GRAY, resized with INTER_LINEAR (skipped if same size);
  * box caps are w <= W*560/1080 and h <= H*760/1920 in output px (the
    teaser's 560x760 at 1080x1920, as fractions of the frame);
  * the contrast term is the population std of the working-res grey over
    rows y..y+h and columns x..x+w inclusive (working px, clipped to the frame);
  * percentiles use linear interpolation (numpy's default);
  * drawn widths are max(1, int(n*k + 0.5)); every other drawn size is n*k.
    That includes the corner ticks: their width is the box width plus
    max(1, int(2*k + 0.5)), i.e. "+2" only at k = 1;
  * Python int() truncates toward zero: the window shift dx, dy are negative
    about half the time, so use Math.trunc, not Math.floor;
  * the ordering relies on stable sorts at three sites -- candidate blobs by
    descending score, the two nearest neighbours of each centroid, and the
    chain of centroids ordered by x -- ties keep insertion (then index) order;
  * rnd() takes sin() of arguments in the tens of thousands; libm results may
    differ in the last ulp between platforms, which only matters when a value
    lands exactly on a threshold (0.45, 0.5, labels, invert, ...);
  * chip text uses printf-style "%.3f" and "%02d"; %.3f rounds exact binary
    ties the way C does, which can differ from JS toFixed in the last digit.
"""
import argparse
import contextlib
import functools
import json
import math
import os
import shutil
import subprocess
import re
import sys
import tempfile

_HERE = os.path.dirname(os.path.abspath(__file__))
_REQS = os.path.join(_HERE, "requirements.txt")

_missing = []
try:
    import numpy as np
except ImportError:
    _missing.append("numpy")
try:
    import cv2
except ImportError:
    _missing.append("opencv-python-headless")
try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    _missing.append("pillow")
if _missing:
    _msg = ("blobtrack needs %s. Install with:\n    pip install -r %s"
            % (", ".join(_missing), _REQS))
    if __name__ == "__main__":
        sys.exit(_msg)
    raise ImportError(_msg)

FONT = os.path.join(_HERE, "JetBrainsMono-Regular.ttf")

DEFAULTS = {
    "bright": 90,      # percentile above which a pixel is "bright"
    "dark": 9,         # percentile below which a pixel is "dark"
    "motion": 14,      # grey-level difference vs 3 frames back that counts as motion
    "large": 6,        # quota of large boxes per frame
    "medium": 11,      # quota of medium boxes per frame
    "small": 15,       # quota of small boxes per frame
    "corners": 28,     # optical-flow corner markers
    "labels": 0.55,    # share of boxes that get a chip (large ones always do)
    "invert": 0.18,    # share of boxes whose contents are inverted
    "shift": 0.12,     # share of boxes whose contents are shifted
    "screen": 0.70,    # opacity of the screened strokes layer
    "chip": 0.86,      # opacity of the chips layer
}
_INT_PARAMS = ("large", "medium", "small", "corners")
HELP = {
    "bright": "percentile threshold for bright blobs",
    "dark": "percentile threshold for dark blobs",
    "motion": "grey-level difference vs 3 frames back that counts as motion",
    "large": "max large boxes per frame",
    "medium": "max medium boxes per frame",
    "small": "max small boxes per frame",
    "corners": "optical-flow corner markers",
    "labels": "share of boxes with a label chip (0-1)",
    "invert": "share of boxes whose contents are inverted (0-1)",
    "shift": "share of boxes whose contents are shifted (0-1)",
    "screen": "opacity of the screen-blended strokes (0-1)",
    "chip": "opacity of the label chips (0-1)",
}

_PERCENT = ("bright", "dark")
_SHARES = ("labels", "invert", "shift", "screen", "chip")
MIN_SIDE = 60       # smallest frame side: keeps the 18*k label font at >= 1 px

REF = 1080          # short side the look was tuned at
WORK = 540          # short side detection runs at
RESEED_EVERY = 45   # frames between corner re-seeds
WHITE = (255, 255, 255)


class BlobTrackError(ValueError):
    """A user-facing error: bad parameters, missing tools, unreadable input."""


class OutputExistsError(BlobTrackError):
    """run() refused to replace an existing output file; `path` is that file."""

    def __init__(self, path):
        self.path = path
        super().__init__("%s already exists; pass overwrite=True to replace it" % path)

    def __reduce__(self):                  # pickle/copy rebuild from the constructor args
        return (type(self), (self.path,))


class ParamError(BlobTrackError):
    """A bad effect parameter. `param` is its name, `detail` the rest of the
    message; str() reads "<param> <detail>" (the CLI shows "--<param> <detail>")."""

    def __init__(self, param, detail):
        self.param, self.detail = param, detail
        super().__init__("%s %s" % (param, detail))

    def __reduce__(self):                  # pickle/copy rebuild from the constructor args
        return (type(self), (self.param, self.detail))


def _check_crf(crf):
    """x264 CRF: a whole number 0..51 (bools and non-numbers are refused)."""
    ok = (not isinstance(crf, bool) and isinstance(crf, (int, float, np.integer, np.floating))
          and math.isfinite(crf) and float(crf) == int(crf) and 0 <= crf <= 51)
    if not ok:
        raise ParamError("crf", "must be a whole number 0..51, got %r" % (crf,))
    return int(crf)


def _check_params(cfg):
    for name, val in cfg.items():
        if isinstance(val, bool) or not isinstance(val, (int, float, np.integer, np.floating)):
            raise ParamError(name, "must be a number, got %r" % (val,))
        if not math.isfinite(val):
            raise ParamError(name, "must be a finite number, got %r" % (val,))
        if name in _INT_PARAMS and float(val) != int(val):
            raise ParamError(name, "must be a whole number, got %r" % (val,))
    for name in _PERCENT:
        if not 0 <= cfg[name] <= 100:
            raise ParamError(name, "is a percentile and must be 0..100, got %g" % cfg[name])
    for name in _SHARES:
        if not 0 <= cfg[name] <= 1:
            raise ParamError(name, "must be 0..1, got %g" % cfg[name])
    for name in _INT_PARAMS:
        if not cfg[name] >= 0:
            raise ParamError(name, "must be >= 0, got %g" % cfg[name])
    if not 0 <= cfg["motion"] <= 255:
        raise ParamError("motion", "is a grey-level difference and must be 0..255, got %g" % cfg["motion"])


def rnd(i, salt):
    """Stable per-ID randomness in [0, 1)."""
    x = math.sin(i * 127.1 + salt * 311.7) * 43758.5453
    return x - math.floor(x)


def iou(a, b):
    """Intersection over union of two {x, y, w, h} boxes."""
    ix = max(0, min(a["x"] + a["w"], b["x"] + b["w"]) - max(a["x"], b["x"]))
    iy = max(0, min(a["y"] + a["h"], b["y"] + b["h"]) - max(a["y"], b["y"]))
    inter = ix * iy
    return inter / (a["w"] * a["h"] + b["w"] * b["h"] - inter + 1e-6)


class BlobTracker:
    """Stateful per-clip effect: feed frames in order to `process`."""

    def __init__(self, width, height, font_path=None, **params):
        unknown = sorted(set(params) - set(DEFAULTS))
        if unknown:
            raise TypeError("unknown parameter(s): %s (known: %s)"
                            % (", ".join(unknown), ", ".join(DEFAULTS)))
        self.W, self.H = int(width), int(height)
        if min(self.W, self.H) < MIN_SIDE:
            raise BlobTrackError("frame is %dx%d; blob tracking needs at least %d px on each side"
                                 % (self.W, self.H, MIN_SIDE))
        cfg = dict(DEFAULTS)
        cfg.update(params)
        _check_params(cfg)
        for k in _INT_PARAMS:
            cfg[k] = int(cfg[k])
        self.cfg = cfg

        short = min(self.W, self.H)
        self.k = short / REF
        self.S = WORK / short
        self.work = (int(self.W * self.S + 0.5), int(self.H * self.S + 0.5))
        self.cap_w = self.W * 560 / 1080
        self.cap_h = self.H * 760 / 1920
        self.label = 18 * self.k
        size = int(self.label) if float(self.label).is_integer() else self.label
        self.font = ImageFont.truetype(font_path or FONT, size)
        self._kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (5, 5))
        self._kernel9 = cv2.getStructuringElement(cv2.MORPH_RECT, (9, 9))

        self.frame = 0          # index of the next frame
        self.tracks = []
        self.next_id = 1
        self.hist = []          # last three working-res greys, oldest first
        self.pts = np.zeros((0, 1, 2), np.float32)
        self.reseeds = []       # frame indices at which corners were (re)seeded
        self.last_tracks = []
        self.last_points = []

    # ------------------------------------------------------------ detect
    def gray(self, rgb):
        g = cv2.cvtColor(rgb, cv2.COLOR_RGB2GRAY)
        if (self.W, self.H) != self.work:
            g = cv2.resize(g, self.work, interpolation=cv2.INTER_LINEAR)
        return g

    def _components(self, mask, amin, amax, kind):
        n, _, st, _ = cv2.connectedComponentsWithStats(mask, connectivity=8)
        S, out = self.S, []
        for i in range(1, n):
            x, y, w, h, a = (int(v) for v in st[i])
            if amin <= a <= amax and w > 3 and h > 3 and w / S <= self.cap_w and h / S <= self.cap_h:
                out.append({"x": x / S, "y": y / S, "w": w / S, "h": h / S,
                            "a": a / (S * S), "kind": kind, "_box": (x, y, w, h)})
        return out

    def detect(self, g, gprev):
        cfg, k5 = self.cfg, self._kernel
        gb = cv2.GaussianBlur(g, (0, 0), 2.2)
        hi = np.percentile(gb, cfg["bright"])
        lo = np.percentile(gb, cfg["dark"])
        blobs = []
        m = cv2.morphologyEx((gb > hi).astype(np.uint8), cv2.MORPH_CLOSE, k5)
        blobs += self._components(m, 40, 90000, "B")
        m = cv2.morphologyEx((gb < lo).astype(np.uint8), cv2.MORPH_CLOSE, k5)
        blobs += self._components(m, 40, 90000, "D")
        e = cv2.Canny(g, 60, 150)
        e = cv2.dilate(e, self._kernel9)
        e = cv2.morphologyEx(e, cv2.MORPH_OPEN, k5)
        blobs += self._components((e > 0).astype(np.uint8), 120, 60000, "E")
        if gprev is not None:
            d = cv2.GaussianBlur(cv2.absdiff(g, gprev), (0, 0), 3)
            m = cv2.dilate((d > cfg["motion"]).astype(np.uint8), k5, iterations=2)
            blobs += self._components(m, 60, 120000, "M")

        for b in blobs:
            x, y, w, h = b.pop("_box")
            patch = g[y:y + h + 1, x:x + w + 1]
            c = float(patch.std()) if patch.size else 0.0
            b["score"] = (2.0 if b["kind"] == "M" else 1.0) * math.sqrt(b["a"]) * (8 + c)
        blobs.sort(key=lambda b: -b["score"])

        big, mid = 230 * self.k, 70 * self.k
        quota = {"L": cfg["large"], "M": cfg["medium"], "S": cfg["small"]}
        picked = []
        for b in blobs:
            side = math.sqrt(b["w"] * b["h"])
            tier = "L" if side > big else ("M" if side > mid else "S")
            if quota[tier] <= 0:
                continue
            if any(iou(b, p) > 0.45 for p in picked):
                continue
            quota[tier] -= 1
            picked.append(b)
        return picked

    # ------------------------------------------------------------- track
    def _update(self, dets):
        norm = 160 * self.k
        live, used = [], set()
        for t in self.tracks:
            best, bs = None, 0.0
            for j, d in enumerate(dets):
                if j in used:
                    continue
                dist = math.hypot(t["x"] + t["w"] / 2 - d["x"] - d["w"] / 2,
                                  t["y"] + t["h"] / 2 - d["y"] - d["h"] / 2)
                s = iou(t, d) + max(0, 1 - dist / norm) * 0.6
                if s > bs:
                    bs, best = s, j
            if best is not None and bs > 0.25:
                d = dets[best]
                used.add(best)
                for key in ("x", "y", "w", "h"):
                    t[key] = t[key] * 0.45 + d[key] * 0.55
                t["a"], t["kind"], t["age"] = d["a"], d["kind"], t["age"] + 1
                live.append(t)
        for j, d in enumerate(dets):
            if j not in used:
                live.append({"id": self.next_id, "x": d["x"], "y": d["y"], "w": d["w"], "h": d["h"],
                             "a": d["a"], "kind": d["kind"], "age": 0})
                self.next_id += 1
        self.tracks = live
        return live

    def _corners(self, g):
        n = self.cfg["corners"]
        if self.frame > 0 and len(self.pts):
            p, st, _ = cv2.calcOpticalFlowPyrLK(self.hist[-1], g, self.pts, None,
                                                winSize=(21, 21), maxLevel=3)
            self.pts = p[st.ravel() == 1].reshape(-1, 1, 2)
        if self.frame == 0 or len(self.pts) < n / 2 or self.frame % RESEED_EVERY == 0:
            p = cv2.goodFeaturesToTrack(g, n, 0.02, 22) if n > 0 else None
            self.pts = p if p is not None else np.zeros((0, 1, 2), np.float32)
            self.reseeds.append(self.frame)
        return [(float(x) / self.S, float(y) / self.S) for x, y in self.pts.reshape(-1, 2)]

    # -------------------------------------------------------------- draw
    def _lw(self, n):
        return max(1, int(n * self.k + 0.5))

    def draw(self, rgb, tracks, pts, frame_no):
        cfg, W, H, k, L = self.cfg, self.W, self.H, self.k, self.label
        base = rgb
        out = rgb.copy()
        # windows: a minority of boxes turn their contents over or slip
        for t in tracks:
            x0, y0 = int(t["x"]), int(t["y"])
            x1, y1 = int(t["x"] + t["w"]), int(t["y"] + t["h"])
            x0, y0, x1, y1 = max(0, x0), max(0, y0), min(W, x1), min(H, y1)
            r = rnd(t["id"], 3)
            if r < cfg["invert"] and (x1 - x0) * (y1 - y0) < (260 * k) ** 2:
                out[y0:y1, x0:x1] = 255 - base[y0:y1, x0:x1]
            elif r < cfg["invert"] + cfg["shift"]:
                dx = int((rnd(t["id"], 4) - 0.5) * 36 * k)
                dy = int((rnd(t["id"], 5) - 0.5) * 36 * k)
                sx = np.clip(x0 + dx, 0, W - (x1 - x0))
                sy = np.clip(y0 + dy, 0, H - (y1 - y0))
                out[y0:y1, x0:x1] = base[sy:sy + (y1 - y0), sx:sx + (x1 - x0)]

        # strokes: always white, so the layer holds only their alpha (ImageDraw
        # overwrites rather than blends, exactly as on a white RGBA layer)
        ov = Image.new("L", (W, H), 0)
        d = ImageDraw.Draw(ov)
        chl = Image.new("RGBA", (W, H), (0, 0, 0, 0))   # chips: normal
        dc = ImageDraw.Draw(chl)
        fnt = self.font
        w1, w2 = self._lw(1), self._lw(2)

        cen = [(t["x"] + t["w"] / 2, t["y"] + t["h"] / 2) for t in tracks]
        drawn = set()
        for i, c in enumerate(cen):
            near = sorted(range(len(cen)),
                          key=lambda j: (cen[j][0] - c[0]) ** 2 + (cen[j][1] - c[1]) ** 2)[1:3]
            for j in near:
                key = (min(i, j), max(i, j))
                if key in drawn:
                    continue
                drawn.add(key)
                d.line([c, cen[j]], fill=150, width=w2)
        order = sorted(range(len(cen)), key=lambda j: cen[j][0])
        for a, b in zip(order, order[1:]):
            if rnd(tracks[a]["id"], 9) < 0.5:
                d.line([cen[a], cen[b]], fill=80, width=w1)

        for n, (px, py) in enumerate(pts):
            s = (6 + int(rnd(n, 11) * 14)) * k
            d.rectangle([px - s / 2, py - s / 2, px + s / 2, py + s / 2], outline=200, width=w1)
            if n % 5 == 0 and cen:
                j = min(range(len(cen)), key=lambda j: (cen[j][0] - px) ** 2 + (cen[j][1] - py) ** 2)
                d.line([(px, py), cen[j]], fill=70, width=w1)

        chips = []
        big, tick, cross = 230 * k, 18 * k, 9 * k
        for t, c in zip(tracks, cen):
            x0, y0, x1, y1 = t["x"], t["y"], t["x"] + t["w"], t["y"] + t["h"]
            side = math.sqrt(t["w"] * t["h"])
            wdt = self._lw(3) if side > big else w2
            d.rectangle([x0, y0, x1, y1], outline=235, width=wdt)
            if side > 90 * k:
                for (cx, cy, sx, sy) in ((x0, y0, 1, 1), (x1, y0, -1, 1), (x0, y1, 1, -1), (x1, y1, -1, -1)):
                    d.line([(cx, cy), (cx + sx * tick, cy)], fill=255, width=wdt + w2)
                    d.line([(cx, cy), (cx, cy + sy * tick)], fill=255, width=wdt + w2)
            if rnd(t["id"], 7) < 0.45:
                d.line([(c[0] - cross, c[1]), (c[0] + cross, c[1])], fill=230, width=w1)
                d.line([(c[0], c[1] - cross), (c[0], c[1] + cross)], fill=230, width=w1)

            if rnd(t["id"], 8) < cfg["labels"] or side > big:
                v = rnd(t["id"], 12)
                if v < 0.34:
                    txt = "ID %02d" % t["id"]
                elif v < 0.67:
                    txt = "%.3f  %.3f" % (c[0] / W, c[1] / H)
                else:
                    txt = "%s%02d  A%05d" % (t["kind"], t["id"], int(t["a"]))
                tw = d.textlength(txt, font=fnt)
                lx, ly = x0, y0 - L - 7 * k
                if ly < 4 * k:
                    ly = y1 + 3 * k
                box = (lx - 4 * k, ly - 4 * k, lx + tw + 12 * k, ly + L + 9 * k)
                if any(not (box[2] < b[0] or box[0] > b[2] or box[3] < b[1] or box[1] > b[3]) for b in chips):
                    continue
                chips.append(box)
                dc.rectangle([lx, ly, lx + tw + 8 * k, ly + L + 5 * k], fill=WHITE + (255,))
                dc.text((lx + 4 * k, ly + 1 * k), txt, font=fnt, fill=(0, 0, 0, 255))

        txt = "TRACK %04d  %02d BLOBS  %02d PTS" % (frame_no, len(tracks), len(pts))
        tw = d.textlength(txt, font=fnt)
        dc.rectangle([26 * k, H - 58 * k, 26 * k + tw + 10 * k, H - 58 * k + L + 7 * k], fill=WHITE + (255,))
        dc.text((31 * k, H - 56 * k), txt, font=fnt, fill=(0, 0, 0, 255))

        return self._composite(out, ov, chl)

    def _composite(self, out, ov, chl):
        """Blend both layers into `out` (in place) and return it.

        SCREEN, not over, for the strokes:  b = b + (1 - b) * stroke * alpha * screen
        normal blend for the chips:         b = b * (1 - ca) + chip * ca

        Only pixels where either layer has alpha are computed. Everywhere else
        both terms are exactly zero and (b/255)*255 + 0.5 truncates back to b,
        so skipping them is bit-identical to blending the whole frame.
        """
        boxes = [bb for bb in (ov.getbbox(), chl.getchannel("A").getbbox()) if bb]
        if not boxes:
            return out
        x0, y0 = min(bb[0] for bb in boxes), min(bb[1] for bb in boxes)
        x1, y1 = max(bb[2] for bb in boxes), max(bb[3] for bb in boxes)
        crop = (x0, y0, x1, y1)
        o8 = np.asarray(ov.crop(crop))
        c8 = np.asarray(chl.crop(crop))
        idx = np.flatnonzero((o8 != 0) | (c8[..., 3] != 0))
        o8 = o8.reshape(-1)[idx]
        c8 = c8.reshape(-1, 4)[idx]
        ys, xs = np.divmod(idx, x1 - x0)
        ys += y0
        xs += x0
        b = out[ys, xs].astype(np.float32)
        b /= 255.0
        s = o8.astype(np.float32)[:, None]      # stroke alpha x stroke colour (1.0)
        s /= 255.0
        s *= self.cfg["screen"]
        t = 1.0 - b
        t *= s
        b += t
        c = c8.astype(np.float32)
        c /= 255.0
        ca = c[:, 3:4] * self.cfg["chip"]
        b *= 1.0 - ca
        c3 = c[:, :3]
        c3 *= ca
        b += c3
        np.clip(b, 0, 1, out=b)
        b *= 255
        b += 0.5
        out[ys, xs] = b.astype(np.uint8)
        return out

    # ----------------------------------------------------------- process
    def process(self, rgb):
        """Treat one RGB frame (HxWx3 uint8); frames must arrive in order."""
        rgb = np.asarray(rgb)
        if rgb.shape != (self.H, self.W, 3) or rgb.dtype != np.uint8:
            raise ValueError("expected a %dx%dx3 uint8 frame, got %s %s"
                             % (self.H, self.W, rgb.shape, rgb.dtype))
        rgb = np.ascontiguousarray(rgb)
        g = self.gray(rgb)
        tracks = self._update(self.detect(g, self.hist[0] if len(self.hist) == 3 else None))
        pts = self._corners(g)
        self.hist = (self.hist + [g])[-3:]
        self.last_tracks = [{key: t[key] for key in ("id", "x", "y", "w", "h", "kind", "a", "age")}
                            for t in tracks]
        self.last_points = pts
        out = self.draw(rgb, tracks, pts, self.frame)
        self.frame += 1
        return out


# ====================================================================== CLI
MP4_AUDIO = {"aac", "mp3", "alac", "ac3", "eac3", "opus", "flac"}


def _need_ffmpeg():
    for tool in ("ffmpeg", "ffprobe"):
        if shutil.which(tool) is None:
            raise BlobTrackError(
                "%s not found on PATH. Install FFmpeg: macOS `brew install ffmpeg`, "
                "Debian/Ubuntu `sudo apt install ffmpeg`, Windows `winget install ffmpeg`." % tool)
    return ffmpeg_version()


def parse_ffmpeg_version(text):
    """(major, minor) from `ffmpeg -version` output; None for git/unknown builds."""
    m = re.search(r"ffmpeg version\s+n?(\d+)\.(\d+)", text)
    return (int(m.group(1)), int(m.group(2))) if m else None


@functools.lru_cache(maxsize=1)
def ffmpeg_version():
    if shutil.which("ffmpeg") is None:
        return None
    r = subprocess.run(["ffmpeg", "-version"], capture_output=True, text=True)
    return parse_ffmpeg_version(r.stdout)


def passthrough_args(version):
    """Keep every decoded frame as is. `-fps_mode` arrived in ffmpeg 5.1; older builds use `-vsync`."""
    if version is not None and version < (5, 1):
        return ["-vsync", "passthrough"]
    return ["-fps_mode", "passthrough"]


def _rate(s):
    try:
        n, d = s.split("/")
        return int(n) / int(d) if int(d) else 0.0
    except (ValueError, AttributeError):
        return 0.0


def probe(path):
    """Size (as decoded, after rotation), frame rate and audio codec of a file."""
    _need_ffmpeg()
    r = subprocess.run(["ffprobe", "-v", "error", "-show_streams", "-show_format", "-of", "json", path],
                       capture_output=True, text=True)
    if r.returncode != 0:
        raise BlobTrackError(r.stderr.strip() or "ffprobe could not read %s" % path)
    j = json.loads(r.stdout)
    vids = [s for s in j.get("streams", []) if s.get("codec_type") == "video"
            and not s.get("disposition", {}).get("attached_pic")]
    if not vids:
        raise BlobTrackError("%s: no video stream" % path)
    v = vids[0]
    w, h = int(v["width"]), int(v["height"])
    rot = 0
    for sd in v.get("side_data_list", []):
        if "rotation" in sd:
            rot = int(sd["rotation"])
    if not rot and "rotate" in v.get("tags", {}):
        rot = int(v["tags"]["rotate"])
    if abs(rot) % 180 == 90:
        w, h = h, w
    r_rate, avg = v.get("r_frame_rate", "0/0"), v.get("avg_frame_rate", "0/0")
    rr, ra = _rate(r_rate), _rate(avg)
    # constant frame rate when the nominal and average rates agree to 0.5 %
    cfr = rr > 0 and (ra <= 0 or abs(rr - ra) <= 0.005 * rr)
    rate = r_rate if cfr else avg
    if _rate(rate) <= 0:
        raise BlobTrackError("%s: cannot determine the frame rate" % path)
    auds = [s for s in j.get("streams", []) if s.get("codec_type") == "audio"]
    return {"width": w, "height": h, "rate": rate, "fps": _rate(rate), "cfr": cfr,
            "vindex": int(v["index"]),
            "audio": auds[0]["codec_name"] if auds else None,
            "duration": float(j.get("format", {}).get("duration", 0) or 0)}


def _span(start, duration):
    a = []
    if start:
        a += ["-ss", "%.6f" % start]
    if duration:
        a += ["-t", "%.6f" % duration]
    return a


def read_frames(path, info, start=None, duration=None):
    """Yield decoded RGB frames (HxWx3 uint8) one at a time from an ffmpeg pipe.

    Callers that may stop before the last frame should wrap the generator in
    contextlib.closing(...) (as run() does), so the ffmpeg process is closed
    and reaped at once rather than whenever the generator is garbage-collected.
    """
    W, H = info["width"], info["height"]
    cmd = (["ffmpeg", "-v", "error", "-nostdin"] + _span(start, duration)
           + ["-i", path, "-map", "0:%d" % info["vindex"]])
    if not info["cfr"]:
        cmd += ["-vf", "fps=%s" % info["rate"]]
    cmd += passthrough_args(ffmpeg_version()) + ["-f", "rawvideo", "-pix_fmt", "rgb24", "-"]
    err = tempfile.TemporaryFile()
    p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=err)
    n = W * H * 3
    try:
        while True:
            buf = p.stdout.read(n)
            if len(buf) < n:
                break
            yield np.frombuffer(buf, np.uint8).reshape(H, W, 3)
    finally:
        p.stdout.close()
        rc = p.wait()
        err.seek(0)
        msg = err.read().decode(errors="replace").strip()
        err.close()
    if rc != 0:
        raise BlobTrackError(msg or "ffmpeg failed decoding %s" % path)


def _encode_filters(version):
    """Video filters for the encoder: even size, BT.709 conversion and tags.

    setparams (with its colour options) first shipped in FFmpeg 4.1, so it is
    left out on older builds; the -x264-params tags still apply there.
    """
    vf = "crop=trunc(iw/2)*2:trunc(ih/2)*2,scale=out_color_matrix=bt709:out_range=tv"
    if version is None or version >= (4, 1):
        vf += ",setparams=color_primaries=bt709:color_trc=bt709:colorspace=bt709:range=tv"
    return vf


def _new_file_mode(dst):
    """Permissions for the output: the existing file's, else 0666 minus umask."""
    if os.path.exists(dst):
        return os.stat(dst).st_mode & 0o7777
    mask = os.umask(0)
    os.umask(mask)
    return 0o666 & ~mask


def run(src, dst, start=None, duration=None, crf=16, audio=True, params=None, progress=None,
        overwrite=False):
    """Apply the effect to a whole file; returns the number of frames written.

    The video is written to a temporary file beside `dst` and renamed onto it
    only on success, so a failed run never leaves a partial file or touches an
    existing one. An existing `dst` is refused unless `overwrite` is true.
    """
    if os.path.realpath(src) == os.path.realpath(dst) or (
            os.path.exists(src) and os.path.exists(dst) and os.path.samefile(src, dst)):
        raise BlobTrackError("output would overwrite the input")
    if os.path.isdir(dst):
        raise BlobTrackError("%s is a directory" % dst)
    if os.path.exists(dst) and not overwrite:
        raise OutputExistsError(dst)
    crf = _check_crf(crf)
    info = probe(src)
    W, H = info["width"], info["height"]
    bt = BlobTracker(W, H, **(params or {}))
    with_audio = audio and info["audio"] is not None

    try:
        fd, tmp = tempfile.mkstemp(dir=os.path.dirname(os.path.abspath(dst)),
                                   prefix="." + os.path.basename(dst) + ".", suffix=".part")
        os.close(fd)
    except OSError as e:
        raise BlobTrackError("cannot write next to %s: %s" % (dst, e.strerror or e))

    cmd = ["ffmpeg", "-v", "error", "-y", "-nostdin",
           "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", "%dx%d" % (W, H), "-r", info["rate"], "-i", "-"]
    if with_audio:
        cmd += _span(start, duration) + ["-i", src]
    cmd += ["-map", "0:v:0"] + (["-map", "1:a:0"] if with_audio else [])
    # Tag BT.709 on the frames (setparams) and in the H.264 VUI (x264-params).
    # The -color_* options alone are not enough: ffmpeg 7 takes colour
    # properties from the frames, and scale leaves primaries/transfer unset.
    cmd += ["-vf", _encode_filters(ffmpeg_version()),
            "-c:v", "libx264", "-preset", "medium", "-crf", str(crf), "-pix_fmt", "yuv420p",
            "-x264-params", "colorprim=bt709:transfer=bt709:colormatrix=bt709:range=tv",
            "-colorspace", "bt709", "-color_primaries", "bt709", "-color_trc", "bt709", "-color_range", "tv"]
    if with_audio:
        cmd += ["-c:a", "copy"] if info["audio"] in MP4_AUDIO else ["-c:a", "aac", "-b:a", "192k"]
    cmd += ["-movflags", "+faststart", "-f", "mp4", tmp]

    remaining = max(0.0, info["duration"] - (start or 0)) if info["duration"] > 0 else None
    span = min(duration, remaining) if duration and remaining is not None else (duration or remaining or 0.0)
    total = int(round(span * info["fps"]))
    n, done = 0, False
    try:
        err = tempfile.TemporaryFile()
        enc = subprocess.Popen(cmd, stdin=subprocess.PIPE, stderr=err)
        try:
            with contextlib.closing(read_frames(src, info, start, duration)) as frames:
                for f in frames:
                    enc.stdin.write(bt.process(f).tobytes())
                    n += 1
                    if progress:
                        progress(n, total)
        except BrokenPipeError:
            pass
        finally:
            try:
                enc.stdin.close()
            except BrokenPipeError:
                pass
            rc = enc.wait()
            err.seek(0)
            msg = err.read().decode(errors="replace").strip()
            err.close()
        if rc != 0:
            raise BlobTrackError(msg or "ffmpeg failed encoding %s" % dst)
        if n == 0:
            raise BlobTrackError("%s: no frames decoded (check the start and duration)" % src)
        try:
            os.chmod(tmp, _new_file_mode(dst))
            os.replace(tmp, dst)
        except OSError as e:
            raise BlobTrackError("cannot write %s: %s" % (dst, e.strerror or e)) from e
        done = True
    finally:
        if not done and os.path.exists(tmp):
            os.remove(tmp)
    if total >= 10 and n < 0.9 * total:
        sys.stderr.write("blobtrack: warning: decoded %d frames, expected about %d; "
                         "the input may be truncated\n" % (n, total))
    return n


def _crf(v):
    """argparse type for --crf; errors read "argument --crf: must be ..."."""
    try:
        n = int(v)
    except ValueError:
        n = v
    try:
        return _check_crf(n)
    except ParamError as e:
        raise argparse.ArgumentTypeError(e.detail)


def main(argv=None):
    ap = argparse.ArgumentParser(
        prog="blobtrack.py",
        description="Apply the blob-tracking effect to a video. Output is H.264 MP4 with the input's audio.")
    ap.add_argument("input", help="input video (anything ffmpeg can read)")
    ap.add_argument("-o", "--output", required=True, help="output .mp4 path")
    ap.add_argument("--start", type=float, default=None, help="start time in seconds")
    ap.add_argument("--duration", type=float, default=None, help="duration in seconds")
    for name, val in DEFAULTS.items():
        ap.add_argument("--" + name, type=int if name in _INT_PARAMS else float, default=val,
                        help="%s (default %s)" % (HELP[name], val))
    ap.add_argument("--crf", type=_crf, default=16, help="x264 quality 0-51, lower is better (default 16)")
    ap.add_argument("--no-audio", action="store_true", help="drop the audio track")
    ap.add_argument("--overwrite", action="store_true", help="replace OUTPUT if it already exists")
    a = ap.parse_args(argv)
    for opt in ("start", "duration"):
        v = getattr(a, opt)
        if v is not None and v < 0:
            ap.error("--%s must be >= 0" % opt)
    params = {k: getattr(a, k) for k in DEFAULTS}

    tty = sys.stderr.isatty()

    def progress(n, total):
        if tty and (n % 10 == 0 or n == total):
            sys.stderr.write("\r%d/%d frames" % (n, total) if total else "\r%d frames" % n)
            sys.stderr.flush()

    try:
        n = run(a.input, a.output, a.start, a.duration, a.crf, not a.no_audio, params, progress,
                overwrite=a.overwrite)
    except (BlobTrackError, ValueError, OSError) as e:
        if isinstance(e, OutputExistsError):
            e = "%s already exists; pass --overwrite to replace it" % e.path
        elif isinstance(e, ParamError):
            e = "--%s %s" % (e.param, e.detail)
        if tty:
            sys.stderr.write("\n")
        sys.stderr.write("blobtrack: %s\n" % e)
        return 1
    if tty:
        sys.stderr.write("\n")
    print("%s: %d frames" % (a.output, n))
    return 0


if __name__ == "__main__":
    sys.exit(main())
