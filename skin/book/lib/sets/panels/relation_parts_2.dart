// Part of relation_parts.dart (cut there by size, as it was).
part of 'relation_parts.dart';

class _AlongState extends State<AlongPathGadget> {
  final p = [const Offset(.12, .78), const Offset(.3, .12), const Offset(.62, .95), const Offset(.88, .22)];
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 4, title: 'Along Path', jp: 'パスに沿って', tone: Fam.along.c,
        reads: [('Start', _f(p[0].dx)), ('End', _f(p[3].dx)), ('Spacing', '${w.count} · ${_f(w.bias, 1)}')],
        handles: p,
        onChanged: (i, q) => setState(() => p[i] = q),
        paint: (cv, s, h, k, hot, ht, t) {
          final path = Path()
            ..moveTo(h[0].dx, h[0].dy)
            ..cubicTo(h[1].dx, h[1].dy, h[2].dx, h[2].dy, h[3].dx, h[3].dy);
          cv.drawLine(h[0], h[1], k.stroke(k.guide, 1));
          cv.drawLine(h[3], h[2], k.stroke(k.guide, 1));
          k.line(cv, path, a: .9);
          final m = path.computeMetrics().first;
          for (var i = 0; i < w.count; i++) {
            final f = math.pow(i / math.max(1, w.count - 1), w.bias).toDouble();
            k.dot(cv, m.getTangentForOffset(m.length * f)!.position, 2.4 + 3.4 * i / math.max(1, w.count - 1));
          }
        });
  }
}

// ---------------------------------------------------------------- 5 Stagger
class StaggerGadget extends GadgetWidget {
  const StaggerGadget({super.key, super.look, super.size, super.framed, this.count = 8, this.ease = 1.0, this.preview = true});
  final int count;
  final double ease;
  final bool preview;
  @override
  State<StaggerGadget> createState() => _StaggerState();
}

class _StaggerState extends State<StaggerGadget> {
  Offset a = const Offset(.2, .78), b = const Offset(.84, .2);
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 5, title: 'Stagger', jp: '時間差でずらす', tone: Fam.stagger.c, ticking: w.preview,
        reads: [('Offset', _f(_clamp((a.dx - .1) / .8, 0, 1))), ('Direction', '${(math.atan2(b.dy - a.dy, b.dx - a.dx) * 180 / math.pi).round()}°'), ('Range', _f((b - a).distance))],
        handles: [a, b],
        onChanged: (i, p) => setState(() => i == 0 ? a = p : b = p),
        paint: (cv, s, h, k, hot, ht, t) {
          final W = s.width;
          cv.drawPath(Path()..moveTo(.1 * W, .12 * W)..lineTo(.1 * W, .9 * W)..lineTo(.88 * W, .9 * W), k.stroke(N.g26, 1));
          _dash(cv, Path()..moveTo(h[0].dx, h[0].dy)..lineTo(h[1].dx, h[1].dy), k.stroke(k.guide, 1));
          final n = w.count, phase = (t * .9) % 1.8;
          for (var i = 0; i < n; i++) {
            final f = i / math.max(1, n - 1), p = Offset.lerp(h[0], h[1], math.pow(f, w.ease).toDouble())!, r = 2.4 + f * 4.2;
            final lit = w.preview ? math.max(0.0, 1 - (phase - f).abs() / .45) : 0.0;
            k.dot(cv, p, r + lit * 1.4, a: .35 + .65 * f);
            if (lit > 0) cv.drawCircle(p, r + 4 + lit * 2, k.stroke(k.c.withValues(alpha: .5 * lit), 1));
          }
        });
  }
}

// ---------------------------------------------------------------- 6 Scale Distribution
class ScaleGadget extends GadgetWidget {
  const ScaleGadget({super.key, super.look, super.size, super.framed, this.count = 7});
  final int count;
  @override
  State<ScaleGadget> createState() => _ScaleState();
}

class _ScaleState extends State<ScaleGadget> {
  double rMin = .02, rMax = .1, curveX = .5;
  static const y0 = .6;
  @override
  Widget build(BuildContext context) {
    final w = widget;
    final ex = math.pow(2, (curveX - .5) * 4).toDouble();
    return _gadget(w, no: 6, title: 'Scale Distribution', jp: '大きさの分布', tone: Fam.stagger.c,
        reads: [('Min', _f(rMin * 10, 1)), ('Max', _f(rMax * 10, 1)), ('Curve', _f(ex))],
        handles: [Offset(.16, y0 - rMin), Offset(.84, y0 - rMax), Offset(curveX, y0)],
        onChanged: (i, p) => setState(() {
              if (i == 0) rMin = _clamp(y0 - p.dy, .01, .1);
              if (i == 1) rMax = _clamp(y0 - p.dy, .02, .1);
              if (i == 2) curveX = _clamp(p.dx, .25, .75);
            }),
        paint: (cv, s, h, k, hot, ht, t) {
          cv.drawLine(Offset(.14 * s.width, y0 * s.width), Offset(.86 * s.width, y0 * s.width), k.stroke(k.guide, 1));
          for (var i = 0; i < w.count; i++) {
            final f = i / math.max(1, w.count - 1), r = _mix(rMin, rMax, math.pow(f, ex).toDouble()) * s.width, p = Offset(_mix(.16, .84, f) * s.width, y0 * s.width);
            if (k.fillA > 0) {
              k.dot(cv, p, r, a: .55 + .3 * f);
            } else {
              k.ring(cv, p, r);
            }
          }
        });
  }
}

// ---------------------------------------------------------------- 7 Rotation
class RotationGadget extends GadgetWidget {
  const RotationGadget({super.key, super.look, super.size, super.framed, this.count = 9, this.jitter = .15});
  final int count;
  final double jitter;
  @override
  State<RotationGadget> createState() => _RotationState();
}

class _RotationState extends State<RotationGadget> {
  double a0 = -.9, a1 = 1.1;
  static const dc = Offset(.5, .32);
  static const dr = .2;
  Offset onDial(double a) => dc + Offset(math.sin(a), -math.cos(a)) * dr;
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 7, title: 'Rotation', jp: '回転の分布', tone: Fam.face.c,
        reads: [('From', '${(a0 * 180 / math.pi).round()}°'), ('To', '${(a1 * 180 / math.pi).round()}°'), ('Random', _f(w.jitter))],
        handles: [onDial(a0), onDial(a1)],
        onChanged: (i, p) => setState(() {
              final v = p - dc, an = math.atan2(v.dx, -v.dy);
              i == 0 ? a0 = an : a1 = an;
            }),
        paint: (cv, s, h, k, hot, ht, t) {
          final W = s.width, c = dc * W, R = dr * W;
          if (k.fillA > 0) {
            cv.drawPath(Path()..moveTo(c.dx, c.dy)..arcTo(Rect.fromCircle(center: c, radius: R), a0 - math.pi / 2, a1 - a0, false)..close(), k.fill(k.c.withValues(alpha: .14)));
          }
          k.ring(cv, c, R, col: k.guide, a: 1, w: 1);
          cv.drawLine(c, h[0], k.stroke(k.guide, 1));
          cv.drawLine(c, h[1], k.stroke(k.guide, 1));
          final rnd = math.Random(7), n = w.count;
          for (var i = 0; i < n; i++) {
            final f = i / math.max(1, n - 1), an = _mix(a0, a1, f) + (rnd.nextDouble() - .5) * w.jitter * math.pi, d = Offset(math.sin(an), -math.cos(an)) * (.07 * W);
            final p = Offset(_mix(.12, .88, f) * W, .8 * W);
            k.line(cv, Path()..moveTo((p - d).dx, (p - d).dy)..lineTo((p + d).dx, (p + d).dy), w: 2);
          }
        });
  }
}

// ---------------------------------------------------------------- 8 Color / Gradient
class ColorGadget extends GadgetWidget {
  const ColorGadget({super.key, super.look, super.size, super.framed, this.ease = 1.0});
  final double ease;
  @override
  State<ColorGadget> createState() => _ColorState();
}

class _ColorState extends State<ColorGadget> {
  // discrete stops: one hue (Stagger) and greys, plus every family colour on tap
  static final pal = <(String, Color)>[for (final f in Fam.all) (f.name, f.c), ('Grey', N.g44)];
  final pos = [.12, .45, .78];
  final ci = [6, 2, 2];
  Color at(double f) {
    final x = _mix(.12, .88, f);
    var best = 0;
    for (var i = 0; i < 3; i++) {
      if (pos[i] <= x && (pos[best] > x || pos[i] >= pos[best])) best = i;
    }
    return pal[ci[best]].$2;
  }

  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 8, title: 'Color / Gradient', jp: '色の変化', tone: C.mode,
        reads: [('Color', pal[ci[1]].$1), ('Position', _f(pos[1])), ('Ease', _f(w.ease, 1))],
        handles: [for (final p in pos) Offset(p, .66)],
        colors: [for (final i in ci) pal[i].$2],
        onTap: (i) => setState(() => ci[i] = (ci[i] + 1) % pal.length),
        onChanged: (i, p) => setState(() => pos[i] = _clamp(p.dx, .12, .88)),
        paint: (cv, s, h, k, hot, ht, t) {
          final W = s.width, bar = RRect.fromLTRBR(.12 * W, .2 * W, .88 * W, .5 * W, const Radius.circular(6));
          cv.save();
          cv.clipRRect(bar);
          const cells = 16;
          for (var i = 0; i < cells; i++) {
            cv.drawRect(Rect.fromLTRB(bar.left + bar.width * i / cells, bar.top, bar.left + bar.width * (i + 1) / cells + .5, bar.bottom), k.fill(at(math.pow(i / (cells - 1), w.ease).toDouble())));
          }
          cv.restore();
          if (k.quiet) cv.drawRRect(bar.deflate(1), k.stroke(N.g20, 1));
          for (var i = 0; i < 8; i++) {
            k.dot(cv, Offset(_mix(.16, .84, i / 7) * W, .86 * W), 5.5, col: at(math.pow(i / 7, w.ease).toDouble()));
          }
        });
  }
}

// ---------------------------------------------------------------- 9 Curve / Easing
class CurveGadget extends GadgetWidget {
  const CurveGadget({super.key, super.look, super.size, super.framed, this.preset = 1, this.preview = true});
  final int preset;
  final bool preview;
  static const presets = [
    (.33, .33, .67, .67),
    (.42, 0.0, .58, 1.0),
    (.42, 0.0, 1.0, 1.0),
    (0.0, 0.0, .58, 1.0),
    (.34, 1.56, .64, 1.0),
  ];
  static const names = ['Linear', 'In-out', 'In', 'Out', 'Back'];
  @override
  State<CurveGadget> createState() => _CurveState();
}

class _CurveState extends State<CurveGadget> {
  late double x1, y1, x2, y2;
  static const bx0 = .14, bx1 = .86, by0 = .22, by1 = .78;
  @override
  void initState() {
    super.initState();
    _load();
  }

  void _load() {
    final p = CurveGadget.presets[widget.preset.clamp(0, 4)];
    x1 = p.$1;
    y1 = p.$2;
    x2 = p.$3;
    y2 = p.$4;
  }

  @override
  void didUpdateWidget(CurveGadget old) {
    super.didUpdateWidget(old);
    if (old.preset != widget.preset) _load();
  }

  Offset u(double x, double y) => Offset(_mix(bx0, bx1, x), _mix(by1, by0, y));
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 9, title: 'Curve / Easing', jp: '変化のカーブ', tone: C.mode, ticking: w.preview,
        reads: [('Preset', CurveGadget.names[w.preset.clamp(0, 4)]), ('Handle', '${_f(x1)}, ${_f(y1)}'), ('Ease', '${_f(x2)}, ${_f(y2)}')],
        handles: [u(x1, y1), u(x2, y2)],
        onChanged: (i, p) => setState(() {
              final x = _clamp((p.dx - bx0) / (bx1 - bx0), 0, 1), y = _clamp((by1 - p.dy) / (by1 - by0), -.3, 1.3);
              if (i == 0) {
                x1 = x;
                y1 = y;
              } else {
                x2 = x;
                y2 = y;
              }
            }),
        paint: (cv, s, h, k, hot, ht, t) {
          final W = s.width, box = Rect.fromLTRB(bx0 * W, by0 * W, bx1 * W, by1 * W), g = k.stroke(N.g20, 1);
          cv.drawRect(box, g);
          for (var i = 1; i < 4; i++) {
            cv.drawLine(Offset(box.left + box.width * i / 4, box.top), Offset(box.left + box.width * i / 4, box.bottom), g);
            cv.drawLine(Offset(box.left, box.top + box.height * i / 4), Offset(box.right, box.top + box.height * i / 4), g);
          }
          final p0 = u(0, 0) * W, p3 = u(1, 1) * W;
          cv.drawLine(p0, h[0], k.stroke(k.guide, 1));
          cv.drawLine(p3, h[1], k.stroke(k.guide, 1));
          k.line(cv, Path()..moveTo(p0.dx, p0.dy)..cubicTo(h[0].dx, h[0].dy, h[1].dx, h[1].dy, p3.dx, p3.dy), w: 2);
          if (w.preview) {
            final tt = (t * .55) % 2, tri = tt < 1 ? tt : 2 - tt, e = _bez(x1, y1, x2, y2, tri), dp = u(tri, e) * W, rp = Offset(.93 * W, dp.dy);
            cv.drawLine(dp, rp, k.stroke(k.guide, 1));
            k.dot(cv, dp, 3.2, col: N.g95);
            k.dot(cv, rp, 4);
          }
        });
  }
}

// ---------------------------------------------------------------- 10 Noise / Variation
class NoiseGadget extends GadgetWidget {
  const NoiseGadget({super.key, super.look, super.size, super.framed, this.roughness = .45, this.drift = .5});
  final double roughness, drift;
  @override
  State<NoiseGadget> createState() => _NoiseState();
}

class _NoiseState extends State<NoiseGadget> {
  double amp = .17, wl = .3;
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 10, title: 'Noise / Variation', jp: 'ノイズ・ゆらぎ', tone: Fam.scatter.c, ticking: w.drift > 0,
        reads: [('Amount', _f(amp)), ('Scale', _f(wl)), ('Roughness', _f(w.roughness))],
        handles: [Offset(.9, .45 - amp), Offset(.1 + wl, .88)],
        onChanged: (i, p) => setState(() {
              if (i == 0) amp = _clamp(.45 - p.dy, .02, .3);
              if (i == 1) wl = _clamp(p.dx - .1, .1, .7);
            }),
        paint: (cv, s, h, k, hot, ht, t) {
          final W = s.width, mid = .45 * W, per = .3 + .5 * w.roughness;
          var norm = 0.0;
          for (var o = 0; o < 4; o++) {
            norm += math.pow(per, o).toDouble();
          }
          if (k.fillA > 0) cv.drawRect(Rect.fromLTRB(.08 * W, mid - amp * W, .92 * W, mid + amp * W), k.fill(k.c.withValues(alpha: .07)));
          cv.drawLine(Offset(.08 * W, mid), Offset(.92 * W, mid), k.stroke(N.g20, 1));
          final path = Path();
          for (var px = .08 * W; px <= .92 * W; px += 2) {
            var v = 0.0;
            for (var o = 0; o < 4; o++) {
              v += math.pow(per, o) * math.sin(2 * math.pi * (px / W) / (wl / (1 + o * 1.7)) + o * 1.9 + t * w.drift * (1 + o * .6));
            }
            final y = mid + v / norm * amp * W;
            px <= .08 * W ? path.moveTo(px, y) : path.lineTo(px, y);
          }
          k.line(cv, path, w: 1.8);
          final y = .88 * W;
          cv.drawLine(Offset(.1 * W, y), h[1], k.stroke(k.guide, 1));
          cv.drawLine(Offset(.1 * W, y - 4), Offset(.1 * W, y + 4), k.stroke(k.guide, 1));
        });
  }
}

// ---------------------------------------------------------------- 11 Graph / Link
class GraphGadget extends GadgetWidget {
  const GraphGadget({super.key, super.look, super.size, super.framed, this.strength = .7, this.type = 1, this.pulse = true});
  final double strength;
  final int type;
  final bool pulse;
  static const types = ['Line', 'Curve', 'Step'];
  @override
  State<GraphGadget> createState() => _GraphState();
}

class _GraphState extends State<GraphGadget> {
  final n = [const Offset(.5, .28), const Offset(.2, .55), const Offset(.42, .8), const Offset(.78, .64), const Offset(.84, .26)];
  static const edges = [(0, 1), (0, 3), (1, 2), (2, 3), (0, 4)];
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 11, title: 'Graph / Link', jp: 'つなぐ・関係をつくる', tone: Fam.attach.c, ticking: w.pulse,
        reads: [('Target', 'node 1'), ('Strength', _f(w.strength)), ('Type', GraphGadget.types[w.type.clamp(0, 2)])],
        handles: n,
        onChanged: (i, p) => setState(() => n[i] = p),
        paint: (cv, s, h, k, hot, ht, t) {
          for (var e = 0; e < edges.length; e++) {
            final a = h[edges[e].$1], b = h[edges[e].$2], path = Path()..moveTo(a.dx, a.dy);
            switch (w.type) {
              case 0:
                path.lineTo(b.dx, b.dy);
              case 1:
                final m = (a + b) / 2, d = b - a;
                path.quadraticBezierTo(m.dx - d.dy * .25, m.dy + d.dx * .25, b.dx, b.dy);
              default:
                final mx = (a.dx + b.dx) / 2;
                path
                  ..lineTo(mx, a.dy)
                  ..lineTo(mx, b.dy)
                  ..lineTo(b.dx, b.dy);
            }
            k.line(cv, path, w: .8 + w.strength * 2.2, a: .35 + .5 * w.strength);
            if (w.pulse) {
              final m = path.computeMetrics().first, f = (t * .35 + e * .21) % 1;
              k.dot(cv, m.getTangentForOffset(m.length * f)!.position, 2.2, col: N.g95, a: .9);
            }
          }
          k.ring(cv, h[0], _hr + 5, col: k.guide, a: 1, w: 1);
        });
  }
}

// ---------------------------------------------------------------- 12 Region / Mask
class RegionGadget extends GadgetWidget {
  const RegionGadget({super.key, super.look, super.size, super.framed, this.feather = .4, this.invert = false});
  final double feather;
  final bool invert;
  @override
  State<RegionGadget> createState() => _RegionState();
}

class _RegionState extends State<RegionGadget> {
  late final List<Offset> p = [
    for (var i = 0; i < 6; i++) Offset(.5, .5) + Offset(math.cos(i * math.pi / 3 - math.pi / 2), math.sin(i * math.pi / 3 - math.pi / 2)) * const [.34, .26, .36, .28, .32, .24][i],
  ];
  Path blob(List<Offset> q) {
    final n = q.length, path = Path()..moveTo(q[0].dx, q[0].dy);
    for (var i = 0; i < n; i++) {
      final p0 = q[(i - 1 + n) % n], p1 = q[i], p2 = q[(i + 1) % n], p3 = q[(i + 2) % n], a = p1 + (p2 - p0) / 6, b = p2 - (p3 - p1) / 6;
      path.cubicTo(a.dx, a.dy, b.dx, b.dy, p2.dx, p2.dy);
    }
    return path..close();
  }

  @override
  Widget build(BuildContext context) {
    final w = widget;
    const tone = N.g76;
    return _gadget(w, no: 12, title: 'Region / Mask', jp: '範囲で制限', tone: tone,
        reads: [('Shape', '6 points'), ('Feather', _f(w.feather)), ('Invert', w.invert ? 'on' : 'off')],
        handles: p,
        onChanged: (i, q) => setState(() => p[i] = q),
        paint: (cv, s, h, k, hot, ht, t) {
          final path = blob(h), fw = w.feather * .14 * s.width;
          if (k.fillA > 0) cv.drawPath(path, k.fill(tone.withValues(alpha: .1)));
          if (fw > 1) cv.drawPath(path, k.stroke(tone.withValues(alpha: .08), fw * 2));
          k.line(cv, path, col: tone, a: .85);
          final poly = <Offset>[for (final m in path.computeMetrics()) for (var d = 0.0; d < m.length; d += m.length / 72) m.getTangentForOffset(d)!.position];
          for (var i = 0; i < 8; i++) {
            for (var j = 0; j < 8; j++) {
              final q = Offset(_mix(.12, .88, i / 7), _mix(.12, .88, j / 7)) * s.width, inside = path.contains(q);
              var d = double.infinity;
              for (final e in poly) {
                d = math.min(d, (e - q).distance);
              }
              var amt = fw > 1 ? _clamp(.5 + (inside ? d : -d) / (2 * fw), 0, 1) : (inside ? 1.0 : 0.0);
              if (w.invert) amt = 1 - amt;
              k.dot(cv, q, 1.8 + 1.7 * amt, col: N.g95, a: .14 + .8 * amt);
            }
          }
        });
  }
}

// ---------------------------------------------------------------- 13 Repeat / Grid
class RepeatGadget extends GadgetWidget {
  const RepeatGadget({super.key, super.look, super.size, super.framed, this.cols = 5, this.rows = 3, this.jitter = .25});
  final int cols, rows;
  final double jitter;
  @override
  State<RepeatGadget> createState() => _RepeatState();
}

class _RepeatState extends State<RepeatGadget> {
  Offset a = const Offset(.2, .3), b = const Offset(.8, .7);
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 13, title: 'Repeat / Grid', jp: '繰り返す', tone: N.g76,
        reads: [('Count', '${w.cols} × ${w.rows}'), ('Spacing', _f((b.dx - a.dx) / math.max(1, w.cols - 1))), ('Jitter', _f(w.jitter))],
        handles: [a, b],
        colors: [C.mode, N.g76],
        onChanged: (i, p) => setState(() {
              if (i == 0) {
                final d = b - a;
                a = _cl(p, .08, .6);
                b = _cl(a + d, .3, .94);
              } else {
                b = _cl(Offset(math.max(p.dx, a.dx + .12), math.max(p.dy, a.dy + .08)), .3, .94);
              }
            }),
        paint: (cv, s, h, k, hot, ht, t) {
          final rnd = math.Random(5), sp = Offset((h[1].dx - h[0].dx) / math.max(1, w.cols - 1), (h[1].dy - h[0].dy) / math.max(1, w.rows - 1));
          if (k.fillA > 0) cv.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(h[0], h[1]).inflate(10), const Radius.circular(6)), k.fill(N.g76.withValues(alpha: .06)));
          for (var r = 0; r < w.rows; r++) {
            for (var c = 0; c < w.cols; c++) {
              final j = Offset(rnd.nextDouble() - .5, rnd.nextDouble() - .5) * (w.jitter * math.min(sp.dx, sp.dy) * .8);
              if (r == 0 && c == 0) continue;
              if (r == w.rows - 1 && c == w.cols - 1) continue;
              k.dot(cv, h[0] + Offset(sp.dx * c, sp.dy * r) + j, 3, col: N.g91, a: .75);
            }
          }
        });
  }
}

// ---------------------------------------------------------------- 14 Follow
class FollowGadget extends GadgetWidget {
  const FollowGadget({super.key, super.look, super.size, super.framed, this.lag = .35, this.trail = 36});
  final double lag;
  final int trail;
  @override
  State<FollowGadget> createState() => _FollowState();
}
