// Part of timeline_parts.dart (cut there by size, as it was).
part of 'timeline_parts.dart';

class _WorkArea extends StatelessWidget {
  const _WorkArea({super.key, required this.a, required this.b});
  final int a, b;
  @override
  Widget build(BuildContext context) => TpValue<(int, int)>(
        initial: (a, b),
        builder: (context, v, set) {
          final ppf = 6.0, fps = 30, grab = 8.0, top = TL.ruler;
          final x0 = v.$1 * ppf, x1 = v.$2 * ppf;
          int fr(double dx) => (dx / ppf).round();
          Widget handle(double x, bool left) => Positioned(
                left: left ? x - grab : x,
                top: top,
                width: grab,
                height: 14,
                child: MouseRegion(
                  cursor: SystemMouseCursors.resizeLeftRight,
                  child: GestureDetector(
                    onPanUpdate: (d) => left ? set((math.min(v.$1 + fr(d.delta.dx), v.$2 - 2).clamp(0, 118), v.$2)) : set((v.$1, math.max(v.$2 + fr(d.delta.dx), v.$1 + 2).clamp(0, 118))),
                    child: _Grip(left: left),
                  ),
                ),
              );
          return Stack(children: [
            Positioned(top: 0, left: 0, right: 0, height: TL.ruler, child: CustomPaint(painter: _RulerPainter(ppf, fps))),
            Positioned(top: top, left: 0, right: 0, height: 14, child: const ColoredBox(color: N.g13)),
            Positioned(
              top: top,
              left: x0,
              width: x1 - x0,
              height: 14,
              child: MouseRegion(
                cursor: SystemMouseCursors.grab,
                child: GestureDetector(
                  onPanUpdate: (d) {
                    final n = fr(d.delta.dx);
                    final len = v.$2 - v.$1, s = (v.$1 + n).clamp(0, 118 - len);
                    set((s, s + len));
                  },
                  child: const ColoredBox(color: N.g26),
                ),
              ),
            ),
            Positioned(
              top: top + 14,
              left: 0,
              right: 0,
              child: Stack(children: [
                Column(children: [
                  for (var i = 0; i < 3; i++) _band(i, CustomPaint(size: Size.infinite, painter: TpLanePainter(fam: sampleLayers[i + 1].fam, start: sampleLayers[i + 1].start, end: sampleLayers[i + 1].end, keys: sampleLayers[i + 1].keys, ppf: ppf))),
                ]),
                Positioned(left: 0, top: 0, width: x0, height: 3 * TL.height, child: const IgnorePointer(child: ColoredBox(color: Color(0x80000000)))),
                Positioned(left: x1, top: 0, right: 0, height: 3 * TL.height, child: const IgnorePointer(child: ColoredBox(color: Color(0x80000000)))),
              ]),
            ),
            handle(x0, true),
            handle(x1, false),
            Positioned(
              left: 0,
              right: 0,
              top: top + 14 + 3 * TL.height + 8,
              child: Row(children: [
                Text('IN ', style: T.micro(N.g56)),
                Text(tpTime(v.$1, fps), style: T.value()),
                const SizedBox(width: 16),
                Text('OUT ', style: T.micro(N.g56)),
                Text(tpTime(v.$2, fps), style: T.value()),
                const SizedBox(width: 16),
                Text('DUR ', style: T.micro(N.g56)),
                Text(tpTime(v.$2 - v.$1, fps), style: T.value(N.g95)),
              ]),
            ),
          ]);
        },
      );
}

class _Grip extends StatelessWidget {
  const _Grip({required this.left});
  final bool left;
  @override
  Widget build(BuildContext context) => TpHit(
        cursor: SystemMouseCursors.resizeLeftRight,
        builder: (_, h, d) => AnimatedContainer(
          duration: tpFast,
          curve: Curves.easeOut,
          alignment: Alignment.center,
          decoration: BoxDecoration(color: d ? N.g100 : h ? N.g91 : N.g76, borderRadius: BorderRadius.horizontal(left: Radius.circular(left ? 3 : 0), right: Radius.circular(left ? 0 : 3))),
          child: Container(width: 1, height: 6, color: N.g38),
        ),
      );
}

// ---------------------------------------------------------------- 8. ruler

class _RulerDemo extends StatelessWidget {
  const _RulerDemo({required this.ppf, required this.fps, required this.compare, required this.grid});
  final double ppf;
  final int fps;
  final bool compare, grid;
  @override
  Widget build(BuildContext context) {
    Widget one(String cap, double z, {bool g = false}) {
      final major = tpMajor(z, fps), fine = tpFine(z, fps, major);
      return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [
          Text(cap, style: T.micro(N.g63)),
          const SizedBox(width: 10),
          Text('labelled every ${major.round()} f   fine every ${fine.round()} f   ${z.toStringAsFixed(1)} px/f', style: T.value(N.g56).copyWith(fontSize: 10)),
        ]),
        const SizedBox(height: 4),
        SizedBox(height: TL.ruler, child: CustomPaint(size: Size.infinite, painter: _RulerPainter(z, fps))),
        if (g) SizedBox(height: 2 * TL.height, child: ColoredBox(color: N.bandA, child: CustomPaint(size: Size.infinite, painter: TpLanePainter(fam: Fam.stagger, showBar: false, ppf: z, fps: fps)))),
        const SizedBox(height: 12),
      ]);
    }

    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      if (compare) one('1/4x', ppf / 4),
      one('NOW', ppf, g: grid),
      if (compare) one('4x', ppf * 4),
    ]);
  }
}

// ---------------------------------------------------------------- 9. curve

const _presets = <String, List<double>>{
  'linear': [0, 0, 1, 1],
  'ease': [.25, .1, .25, 1],
  'ease in': [.42, 0, 1, 1],
  'ease out': [0, 0, .58, 1],
  'ease in-out': [.42, 0, .58, 1],
  'overshoot': [.34, 1.56, .64, 1],
  'anticipate': [.36, -.56, .64, 1],
};

class _CurveEditor extends StatefulWidget {
  const _CurveEditor({super.key, required this.preset, required this.t});
  final String preset;
  final double t;
  @override
  State<_CurveEditor> createState() => _CurveEditorState();
}

class _CurveEditorState extends State<_CurveEditor> {
  late Offset p1 = Offset(_presets[widget.preset]![0], _presets[widget.preset]![1]), p2 = Offset(_presets[widget.preset]![2], _presets[widget.preset]![3]);
  int? drag, hover;
  static const box = 220.0, pad = 20.0;
  static const ymin = -.5, ymax = 1.5;
  Rect get g => const Rect.fromLTWH(pad, pad, box, box);
  Offset px(Offset v) => Offset(g.left + v.dx * g.width, g.bottom - (v.dy - ymin) / (ymax - ymin) * g.height);
  Offset val(Offset p) => Offset(((p.dx - g.left) / g.width).clamp(0.0, 1.0), ymin + (g.bottom - p.dy) / g.height * (ymax - ymin));
  int? near(Offset p) {
    final d1 = (px(p1) - p).distance, d2 = (px(p2) - p).distance;
    final m = math.min(d1, d2);
    return m > 16 ? null : (d1 <= d2 ? 0 : 1);
  }

  void move(Offset p) {
    final v = val(p);
    setState(() => drag == 0 ? p1 = v : p2 = v);
  }

  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        MouseRegion(
          cursor: hover != null || drag != null ? SystemMouseCursors.grab : SystemMouseCursors.basic,
          onHover: (e) => setState(() => hover = near(e.localPosition)),
          onExit: (_) => setState(() => hover = null),
          child: GestureDetector(
            onPanDown: (d) {
              drag = near(d.localPosition);
              if (drag != null) move(d.localPosition);
            },
            onPanUpdate: (d) {
              if (drag != null) move(d.localPosition);
            },
            onPanEnd: (_) => setState(() => drag = null),
            child: Container(
              width: box + 2 * pad,
              height: box + 2 * pad,
              decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(4)),
              child: CustomPaint(painter: _CurvePainter(p1, p2, widget.t, drag ?? hover, drag != null, px)),
            ),
          ),
        ),
        const SizedBox(height: 10),
        Text('cubic-bezier(${p1.dx.toStringAsFixed(2)}, ${p1.dy.toStringAsFixed(2)}, ${p2.dx.toStringAsFixed(2)}, ${p2.dy.toStringAsFixed(2)})', style: T.value(N.g76).copyWith(fontSize: 10)),
      ]);
}

double _bezierY(Offset p1, Offset p2, double x) {
  double bx(double u) => 3 * (1 - u) * (1 - u) * u * p1.dx + 3 * (1 - u) * u * u * p2.dx + u * u * u;
  var lo = 0.0, hi = 1.0;
  for (var i = 0; i < 24; i++) {
    final m = (lo + hi) / 2;
    if (bx(m) < x) {
      lo = m;
    } else {
      hi = m;
    }
  }
  final u = (lo + hi) / 2;
  return 3 * (1 - u) * (1 - u) * u * p1.dy + 3 * (1 - u) * u * u * p2.dy + u * u * u;
}

class _CurvePainter extends CustomPainter {
  const _CurvePainter(this.p1, this.p2, this.t, this.active, this.dragging, this.px);
  final Offset p1, p2;
  final double t;
  final int? active;
  final bool dragging;
  final Offset Function(Offset) px;
  @override
  void paint(Canvas c, Size s) {
    final fill = Paint();
    final a = px(const Offset(0, 0)), b = px(const Offset(1, 1));
    for (var i = 0; i <= 4; i++) {
      final x = (a.dx + (b.dx - a.dx) * i / 4).roundToDouble(), y = (a.dy + (b.dy - a.dy) * i / 4).roundToDouble();
      final edge = i == 0 || i == 4;
      fill.color = edge ? N.glaze15 : N.glaze9;
      c.drawRect(Rect.fromLTWH(x, math.min(a.dy, b.dy) - 20, 1, (a.dy - b.dy).abs() + 40), fill);
      c.drawRect(Rect.fromLTWH(a.dx, y, b.dx - a.dx, 1), fill);
    }
    final stroke = Paint()..style = PaintingStyle.stroke..strokeWidth = 1;
    // handle lines, then the curve over them
    c.drawLine(a, px(p1), stroke..color = N.g56);
    c.drawLine(b, px(p2), stroke..color = N.g56);
    final path = Path()..moveTo(a.dx, a.dy);
    for (var i = 1; i <= 64; i++) {
      final u = i / 64.0;
      final x = 3 * (1 - u) * (1 - u) * u * p1.dx + 3 * (1 - u) * u * u * p2.dx + u * u * u;
      final y = 3 * (1 - u) * (1 - u) * u * p1.dy + 3 * (1 - u) * u * u * p2.dy + u * u * u;
      final o = px(Offset(x, y));
      path.lineTo(o.dx, o.dy);
    }
    c.drawPath(path, stroke..strokeWidth = 1.75..strokeCap = StrokeCap.round..color = tpAccent);
    // the time cursor and the value under it
    final tp = px(Offset(t, _bezierY(p1, p2, t)));
    c.drawRect(Rect.fromLTWH(tp.dx.roundToDouble(), a.dy - 0, 1, 0), fill);
    c.drawLine(Offset(tp.dx, a.dy), Offset(tp.dx, tp.dy), stroke..strokeWidth = 1..color = C.playhead.withValues(alpha: .6));
    c.drawCircle(tp, 2.5, fill..color = C.playhead);
    tpKey(c, a, TL.key / 2 * 1.1, TpKey.normal);
    tpKey(c, b, TL.key / 2 * 1.1, TpKey.normal);
    for (var i = 0; i < 2; i++) {
      final o = px(i == 0 ? p1 : p2), on = active == i;
      c.drawCircle(o, on ? 3.5 : 3, fill..color = on ? tpAccent : N.g95);
      if (dragging && on) c.drawCircle(o, 3.5, Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = N.g100);
    }
  }

  @override
  bool shouldRepaint(_CurvePainter o) => true;
}

// ---------------------------------------------------------------- 10. property lanes

class _PropLanes extends StatefulWidget {
  const _PropLanes({super.key, required this.head, required this.ppf, required this.kind});
  final int head;
  final double ppf;
  final TpInterp kind;
  @override
  State<_PropLanes> createState() => _PropLanesState();
}

class _PropLanesState extends State<_PropLanes> {
  final names = const ['Position X', 'Position Y'];
  final vals = const [[120.0, 480.0, 300.0, 640.0], [80.0, 540.0, 200.0]];
  late final keys = [[8.0, 34.0, 66.0, 104.0], [8.0, 50.0, 82.0]];
  final anim = [true, true];
  (int, int)? sel, hov;

  double valueAt(int l, double f) {
    final k = keys[l], v = vals[l];
    if (!anim[l] || f <= k.first) return v.first;
    if (f >= k.last) return v.last;
    var i = 0;
    while (k[i + 1] < f) {
      i++;
    }
    var u = (f - k[i]) / (k[i + 1] - k[i]);
    if (widget.kind == TpInterp.hold) u = 0;
    if (widget.kind == TpInterp.eased || widget.kind == TpInterp.bezier) u = u * u * (3 - 2 * u);
    return v[i] + (v[i + 1] - v[i]) * u;
  }

  (int, int)? hit(int l, Offset p) {
    (int, int)? best;
    var bd = 9.0;
    for (var i = 0; i < keys[l].length; i++) {
      final d = (keys[l][i] * widget.ppf - p.dx).abs();
      if (d < bd) {
        bd = d;
        best = (l, i);
      }
    }
    return anim[l] ? best : null;
  }

  @override
  Widget build(BuildContext context) => Column(children: [
        Row(children: [
          SizedBox(width: TL.label),
          Expanded(child: SizedBox(height: TL.ruler, child: CustomPaint(painter: _RulerPainter(widget.ppf, 30)))),
        ]),
        for (var l = 0; l < 2; l++)
          _band(
            l,
            Row(children: [
              SizedBox(
                width: TL.label,
                child: Padding(
                  padding: const EdgeInsets.only(left: 22, right: 8),
                  child: Row(children: [
                    TpIconButton(TpGlyph.stopwatch, on: anim[l], onTap: () => setState(() => anim[l] = !anim[l])),
                    const SizedBox(width: 6),
                    Expanded(child: Text(names[l], style: T.name(N.g76), maxLines: 1)),
                    Text(valueAt(l, widget.head.toDouble()).toStringAsFixed(1), style: T.value(N.g95)),
                  ]),
                ),
              ),
              Expanded(
                child: LayoutBuilder(builder: (context, box) {
                  void moveSel(Offset p) {
                    if (sel == null || sel!.$1 != l) return;
                    final i = sel!.$2, k = keys[l];
                    final lo = i == 0 ? 0.0 : k[i - 1] + 1, hi = i == k.length - 1 ? 118.0 : k[i + 1] - 1;
                    setState(() => k[i] = (p.dx / widget.ppf).roundToDouble().clamp(lo, hi));
                  }

                  return MouseRegion(
                    cursor: hov?.$1 == l ? SystemMouseCursors.grab : SystemMouseCursors.basic,
                    onHover: (e) => setState(() => hov = hit(l, e.localPosition)),
                    onExit: (_) => setState(() => hov = null),
                    child: GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onPanDown: (d) => setState(() => sel = hit(l, d.localPosition) ?? sel),
                      onPanUpdate: (d) => moveSel(d.localPosition),
                      child: CustomPaint(
                        size: Size.infinite,
                        painter: _PropLanePainter(keys[l], widget.kind, anim[l], sel?.$1 == l ? sel?.$2 : null, hov?.$1 == l ? hov?.$2 : null, widget.head, widget.ppf),
                      ),
                    ),
                  );
                }),
              ),
            ]),
          ),
      ]);
}

class _PropLanePainter extends CustomPainter {
  const _PropLanePainter(this.keys, this.kind, this.on, this.sel, this.hov, this.head, this.ppf);
  final List<double> keys;
  final TpInterp kind;
  final bool on;
  final int? sel, hov;
  final int head;
  final double ppf;
  @override
  void paint(Canvas c, Size s) {
    tpGrid(c, s, ppf, 30);
    final cy = s.height / 2;
    if (on) {
      c.drawRect(Rect.fromLTRB(keys.first * ppf, cy - .5, keys.last * ppf, cy + .5), Paint()..color = N.g100.withValues(alpha: .7));
      for (var i = 0; i < keys.length - 1; i++) {
        if ((keys[i + 1] - keys[i]) * ppf > 24) tpSeg(c, Offset((keys[i] + keys[i + 1]) / 2 * ppf, cy), kind);
      }
      for (var i = 0; i < keys.length; i++) {
        final k = keys[i].round() == head ? TpKey.atHead : i == sel ? TpKey.selected : TpKey.normal;
        tpKey(c, Offset(keys[i] * ppf, cy), TL.key / 2 * (i == hov ? 1.2 : 1), k);
      }
    }
    c.drawRect(Rect.fromLTWH(head * ppf, 0, 1, s.height), Paint()..color = C.playhead);
  }

  @override
  bool shouldRepaint(_PropLanePainter o) => true;
}

// ---------------------------------------------------------------- 11. zoom

class _ZoomSlider extends StatelessWidget {
  const _ZoomSlider({super.key, required this.fit, required this.start});
  final double fit, start;
  static double toPpf(double t) => .5 * math.pow(80, t);
  static double fromPpf(double p) => (math.log(p / .5) / math.log(80)).clamp(0.0, 1.0);
  @override
  Widget build(BuildContext context) => TpValue<double>(
        initial: fromPpf(start),
        builder: (context, t, set) {
          final fitT = fromPpf(fit);
          final atFit = (t - fitT).abs() < .01;
          return Row(children: [
            TpIconButton(TpGlyph.minus, size: 22, onTap: () => set((t - .08).clamp(0.0, 1.0))),
            const SizedBox(width: 2),
            Expanded(
              child: LayoutBuilder(builder: (context, box) {
                void drag(Offset p) => set(((p.dx - 6) / (box.maxWidth - 12)).clamp(0.0, 1.0));
                return MouseRegion(
                  cursor: SystemMouseCursors.resizeLeftRight,
                  child: GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onPanDown: (d) => drag(d.localPosition),
                    onPanUpdate: (d) => drag(d.localPosition),
                    child: SizedBox(height: 22, child: CustomPaint(painter: _ZoomTrack(t, fitT))),
                  ),
                );
              }),
            ),
            const SizedBox(width: 2),
            TpIconButton(TpGlyph.plus, size: 22, onTap: () => set((t + .08).clamp(0.0, 1.0))),
            const SizedBox(width: 8),
            SizedBox(width: 58, child: Text('${toPpf(t).toStringAsFixed(1)} px/f', style: T.value(N.g76).copyWith(fontSize: 10), maxLines: 1)),
            const SizedBox(width: 6),
            TpHit(
              onTap: () => set(fitT),
              builder: (_, h, d) => AnimatedContainer(
                duration: tpFast,
                curve: Curves.easeOut,
                height: 22,
                padding: const EdgeInsets.symmetric(horizontal: 6),
                decoration: BoxDecoration(color: d ? N.g20 : h ? N.g15 : const Color(0x00262626), borderRadius: BorderRadius.circular(4)),
                child: Row(mainAxisSize: MainAxisSize.min, children: [
                  TpGlyphView(TpGlyph.fit, atFit ? tpAccent : h ? N.g95 : N.g63, size: 11),
                  const SizedBox(width: 4),
                  Text('FIT', style: T.micro(atFit ? N.g95 : N.g76)),
                ]),
              ),
            ),
          ]);
        },
      );
}

class _ZoomTrack extends CustomPainter {
  const _ZoomTrack(this.t, this.fitT);
  final double t, fitT;
  @override
  void paint(Canvas c, Size s) {
    final cy = s.height / 2, w = s.width - 12, x = 6 + w * t, p = Paint();
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(0, cy - 1, s.width, cy + 1), const Radius.circular(1)), p..color = N.g20);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(0, cy - 1, x, cy + 1), const Radius.circular(1)), p..color = N.g56);
    c.drawCircle(Offset(x, cy), 6, p..color = N.g95);
  }

  @override
  bool shouldRepaint(_ZoomTrack o) => o.t != t || o.fitT != fitT;
}

// ---------------------------------------------------------------- 12. transport

class _Transport extends StatefulWidget {
  const _Transport({super.key, required this.playing, required this.loop, required this.fps, required this.total});
  final bool playing, loop;
  final int fps, total;
  @override
  State<_Transport> createState() => _TransportState();
}
