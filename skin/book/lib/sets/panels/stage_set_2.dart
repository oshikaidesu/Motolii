// Part of stage_set.dart (cut there by size, as it was).
part of 'stage_set.dart';

class _PivotDemoState extends State<_PivotDemo> {
  static const _w = 200.0, _h = 120.0;
  static const _o = Offset(160, 100); // layer top-left on the stage
  Offset pv = const Offset(_w / 2, _h / 2), raw = const Offset(_w / 2, _h / 2);
  bool dragging = false;
  bool snapped = false;

  void move(Offset d) {
    raw += d;
    var p = Offset(raw.dx.clamp(0.0, _w), raw.dy.clamp(0.0, _h));
    snapped = false;
    if (widget.snap) {
      for (final fx in [0.0, .5, 1.0]) {
        for (final fy in [0.0, .5, 1.0]) {
          final t = Offset(fx * _w, fy * _h);
          if ((p - t).distance < 8) {
            p = t;
            snapped = true;
          }
        }
      }
    }
    setState(() => pv = p);
  }

  @override
  Widget build(BuildContext context) {
    final world = _o + pv;
    return _stage([
      Positioned(
        left: _o.dx,
        top: _o.dy,
        width: _w,
        height: _h,
        child: Transform.rotate(
          angle: widget.angle * math.pi / 180,
          origin: pv - const Offset(_w / 2, _h / 2),
          child: Stack(fit: StackFit.expand, children: [
            DecoratedBox(decoration: BoxDecoration(border: Border.all(color: N.g95))),
            if (dragging) CustomPaint(painter: _SnapPointsPainter(pv)),
          ]),
        ),
      ),
      Positioned(
        left: world.dx - 14,
        top: world.dy - 14,
        width: 28,
        height: 28,
        child: MouseRegion(
          cursor: SystemMouseCursors.grab,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanStart: (_) => setState(() {
              dragging = true;
              raw = pv;
            }),
            onPanUpdate: (d) => move(d.delta),
            onPanEnd: (_) => setState(() => dragging = false),
            child: CustomPaint(painter: _PivotPainter(widget.look, widget.radius, dragging, snapped)),
          ),
        ),
      ),
      Positioned(left: 12, bottom: 12, child: Readout([('X', '${(pv.dx / _w * 100).round()}%', axisX), ('Y', '${(pv.dy / _h * 100).round()}%', axisY)], look: widget.look)),
    ]);
  }
}

class _SnapPointsPainter extends CustomPainter {
  _SnapPointsPainter(this.pv);
  final Offset pv;
  @override
  void paint(Canvas c, Size s) {
    for (final fx in [0.0, .5, 1.0]) {
      for (final fy in [0.0, .5, 1.0]) {
        c.drawRect(Rect.fromCenter(center: Offset(fx * s.width, fy * s.height), width: 3, height: 3), Paint()..color = N.g100.withValues(alpha: .7));
      }
    }
  }

  @override
  bool shouldRepaint(_SnapPointsPainter o) => false;
}

class _PivotPainter extends CustomPainter {
  _PivotPainter(this.look, this.r, this.hot, this.snapped);
  final Look look;
  final double r;
  final bool hot, snapped;
  @override
  void paint(Canvas c, Size s) {
    final m = s.center(Offset.zero);
    final color = snapped ? accent : N.g100;
    final arm = r + 7;
    if (look == Look.glow) c.drawCircle(m, r + 3, Paint()..color = accent.withValues(alpha: hot ? .5 : .3)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
    if (look != Look.quiet) {
      legible(c, Path()..addOval(Rect.fromCircle(center: m, radius: r)), color, w: 1.5);
      if (look == Look.glow) c.drawCircle(m, r, Paint()..color = accent.withValues(alpha: .35));
    }
    for (final d in const [Offset(1, 0), Offset(-1, 0), Offset(0, 1), Offset(0, -1)]) {
      legibleLine(c, m + d * (look == Look.quiet ? 2 : r + 1.5), m + d * arm, color);
    }
    c.drawCircle(m, 1.5, Paint()..color = color);
  }

  @override
  bool shouldRepaint(_PivotPainter o) => true;
}

// ---------------------------------------------------------------- 6 guides and rulers

class _GuidesDemo extends StatefulWidget {
  const _GuidesDemo({required this.look, required this.zoom, required this.pointerTick});
  final Look look;
  final double zoom;
  final bool pointerTick;
  @override
  State<_GuidesDemo> createState() => _GuidesDemoState();
}

class _GuidesDemoState extends State<_GuidesDemo> {
  static const ruler = 20.0;
  static const origin = Offset(36, 36);
  List<double> vg = [480, 1440], hg = [270];
  Offset? ptr;
  (bool, int)? drag; // (vertical guide?, index)

  double get z => widget.zoom;

  void setG(bool v, int i, double screen) => setState(() => (v ? vg : hg)[i] = ((screen - (v ? origin.dx : origin.dy)) / z));

  void end(bool v, int i, double screen) {
    if (screen < ruler) setState(() => (v ? vg : hg).removeAt(i));
    setState(() => drag = null);
  }

  Widget guide(bool v, int i, Size box) {
    final g = (v ? vg : hg)[i];
    final s = (v ? origin.dx : origin.dy) + g * z;
    final line = Stack(fit: StackFit.expand, children: [
      Center(child: Container(width: v ? 3 : null, height: v ? null : 3, color: shade(.30))),
      Center(child: Container(width: v ? 1 : null, height: v ? null : 1, color: widget.look == Look.quiet ? accent.withValues(alpha: .6) : accent)),
    ]);
    return Positioned(
      left: v ? s - 4 : ruler,
      top: v ? ruler : s - 4,
      width: v ? 9 : box.width - ruler,
      height: v ? box.height - ruler : 9,
      child: MouseRegion(
        cursor: v ? SystemMouseCursors.resizeColumn : SystemMouseCursors.resizeRow,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onPanStart: (_) => setState(() => drag = (v, i)),
          onPanUpdate: (d) => setG(v, i, s + (v ? d.delta.dx : d.delta.dy)),
          onPanEnd: (_) => end(v, i, s),
          child: line,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: LayoutBuilder(builder: (context, box) {
          final sz = box.biggest;
          final tx = ptr != null && widget.pointerTick ? ptr!.dx : null, ty = ptr != null && widget.pointerTick ? ptr!.dy : null;
          return MouseRegion(
            onHover: (e) => setState(() => ptr = e.localPosition),
            onExit: (_) => setState(() => ptr = null),
            child: Stack(children: [
              Positioned.fill(child: ColoredBox(color: N.g07)),
              Positioned(left: origin.dx, top: origin.dy, width: 1920 * z, height: 1080 * z, child: const SampleArt()),
              for (var i = 0; i < vg.length; i++) guide(true, i, sz),
              for (var i = 0; i < hg.length; i++) guide(false, i, sz),
              Positioned(left: ruler, top: 0, right: 0, height: ruler, child: GestureDetector(
                onVerticalDragStart: (d) => setState(() { hg.add((d.localPosition.dy - origin.dy) / z); drag = (false, hg.length - 1); }),
                onVerticalDragUpdate: (d) => setG(false, hg.length - 1, d.localPosition.dy),
                onVerticalDragEnd: (_) => end(false, hg.length - 1, 0),
                child: CustomPaint(painter: _RulerPainter(true, z, origin.dx - ruler, tx == null ? null : tx - ruler)),
              )),
              Positioned(left: 0, top: ruler, bottom: 0, width: ruler, child: GestureDetector(
                onHorizontalDragStart: (d) => setState(() { vg.add((d.localPosition.dx - origin.dx) / z); drag = (true, vg.length - 1); }),
                onHorizontalDragUpdate: (d) => setG(true, vg.length - 1, d.localPosition.dx),
                onHorizontalDragEnd: (_) => end(true, vg.length - 1, 0),
                child: CustomPaint(painter: _RulerPainter(false, z, origin.dy - ruler, ty == null ? null : ty - ruler)),
              )),
              Positioned(left: 0, top: 0, width: ruler, height: ruler, child: ColoredBox(color: N.g10)),
              if (drag != null)
                Positioned(
                  left: drag!.$1 ? origin.dx + vg[drag!.$2] * z + 10 : 32,
                  top: drag!.$1 ? 28 : origin.dy + hg[drag!.$2] * z + 10,
                  child: IgnorePointer(child: Readout([(drag!.$1 ? 'X' : 'Y', (drag!.$1 ? vg[drag!.$2] : hg[drag!.$2]).round().toString(), drag!.$1 ? axisX : axisY)], look: widget.look)),
                ),
            ]),
          );
        }),
      );
}

class _RulerPainter extends CustomPainter {
  _RulerPainter(this.horizontal, this.zoom, this.origin, this.pointer);
  final bool horizontal;
  final double zoom, origin;
  final double? pointer;
  @override
  void paint(Canvas c, Size s) {
    final len = horizontal ? s.width : s.height, thick = horizontal ? s.height : s.width;
    c.drawRect(Offset.zero & s, Paint()..color = N.g10);
    final edge = Paint()..color = N.g20;
    if (horizontal) {
      c.drawRect(Rect.fromLTWH(0, thick - 1, len, 1), edge);
    } else {
      c.drawRect(Rect.fromLTWH(thick - 1, 0, 1, len), edge);
    }
    const steps = [10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0];
    final step = steps.firstWhere((v) => v * zoom >= 60, orElse: () => 1000);
    final minor = step / 5;
    final k0 = ((0 - origin) / zoom / minor).floor(), k1 = ((len - origin) / zoom / minor).ceil();
    final tick = Paint()..color = N.g44;
    for (var k = k0; k <= k1; k++) {
      final pos = origin + k * minor * zoom;
      if (pos < 0 || pos > len) continue;
      final major = k % 5 == 0;
      final l = major ? 6.0 : 3.0;
      if (horizontal) {
        c.drawRect(Rect.fromLTWH(pos, thick - 1 - l, 1, l), tick);
      } else {
        c.drawRect(Rect.fromLTWH(thick - 1 - l, pos, l, 1), tick);
      }
      if (major) {
        final label = (k * minor).round().toString();
        final st = T.value(N.g63).copyWith(fontSize: 9.5);
        final tw = (TextPainter(text: TextSpan(text: label, style: st), textDirection: TextDirection.ltr)..layout()).width;
        if (pos + 3 + tw > len - 8) continue; // keep 8px clear of the edge
        if (horizontal) {
          paintText(c, label, Offset(pos + 3, 3), st);
        } else {
          c.save();
          c.translate(3, pos - 3);
          c.rotate(-math.pi / 2);
          paintText(c, label, Offset.zero, st);
          c.restore();
        }
      }
    }
    if (pointer != null && pointer! >= 0 && pointer! <= len) {
      final p = Paint()..color = N.g100;
      if (horizontal) {
        c.drawRect(Rect.fromLTWH(pointer!, 0, 1, thick), p);
      } else {
        c.drawRect(Rect.fromLTWH(0, pointer!, thick, 1), p);
      }
    }
  }

  @override
  bool shouldRepaint(_RulerPainter o) => true;
}

// ---------------------------------------------------------------- 7 grid and safe area

enum _Safe { both, action, title }

class _OverlaysDemo extends StatelessWidget {
  const _OverlaysDemo({required this.look, required this.grid, required this.density, required this.safe, required this.mode});
  final Look look;
  final bool grid, safe;
  final int density;
  final _Safe mode;
  @override
  Widget build(BuildContext context) => SpLive<(bool, int, bool)>(
        initial: (grid, density, safe),
        builder: (context, v, set) => ClipRRect(borderRadius: BorderRadius.circular(4), child: Stack(fit: StackFit.expand, children: [
          const ColoredBox(color: N.g07),
          Positioned(left: 0, right: 0, top: 0, bottom: 44, child: ClipRect(child: Stack(fit: StackFit.expand, children: [
            const SampleArt(),
            if (v.$1) Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _GridPainter([64.0, 32.0, 16.0][v.$2])))),
            if (v.$3) Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _SafePainter(mode)))),
          ]))),
          Positioned(
            left: 8,
            bottom: 8,
            child: Plate(look: look, child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
              ChromeButton(glyph: Glyph.grid, size: 24, look: look, on: v.$1, tip: 'Grid', keys: "⌘'", tipSide: AxisDirection.up, onTap: () => set((!v.$1, v.$2, v.$3))),
              const SizedBox(width: 2),
              AnimatedOpacity(duration: kFast, curve: Curves.easeOut, opacity: v.$1 ? 1 : .4, child: MiniSeg(items: const ['64', '32', '16'], index: v.$2, onChanged: (i) => set((v.$1, i, v.$3)))),
              const VDivider(),
              ChromeButton(glyph: Glyph.safe, size: 24, look: look, on: v.$3, tip: 'Safe area', tipSide: AxisDirection.up, onTap: () => set((v.$1, v.$2, !v.$3))),
            ])),
          ),
        ])),
      );
}

class _GridPainter extends CustomPainter {
  _GridPainter(this.step);
  final double step;
  @override
  void paint(Canvas c, Size s) {
    // DESIGN.md: one line style, white 9% between and 16% at the major step; every 4th line is major.
    final minor = Paint()..color = N.g100.withValues(alpha: .09), major = Paint()..color = N.g100.withValues(alpha: .16);
    var i = 0;
    for (var x = 0.0; x < s.width; x += step, i++) {
      c.drawRect(Rect.fromLTWH(x, 0, 1, s.height), i % 4 == 0 ? major : minor);
    }
    i = 0;
    for (var y = 0.0; y < s.height; y += step, i++) {
      c.drawRect(Rect.fromLTWH(0, y, s.width, 1), i % 4 == 0 ? major : minor);
    }
  }

  @override
  bool shouldRepaint(_GridPainter o) => o.step != step;
}

class _SafePainter extends CustomPainter {
  _SafePainter(this.mode);
  final _Safe mode;
  @override
  void paint(Canvas c, Size s) {
    void frame(double inset, String label, {bool right = false}) {
      final r = Rect.fromLTRB(s.width * inset, s.height * inset, s.width * (1 - inset), s.height * (1 - inset));
      legible(c, Path()..addRect(r), N.g100.withValues(alpha: .55));
      // action label sits in the top-left corner, title label in the top-right, so they never meet
      final tp = TextPainter(text: TextSpan(text: label, style: T.micro(N.g91)), textDirection: TextDirection.ltr)..layout();
      paintText(c, label, right ? r.topRight + Offset(-8 - tp.width, 8) : r.topLeft + const Offset(8, 8), T.micro(N.g91));
    }

    if (mode != _Safe.title) frame(.035, 'ACTION 93%');
    if (mode != _Safe.action) frame(.05, 'TITLE 90%', right: true);
    final m = s.center(Offset.zero);
    legibleLine(c, m + const Offset(-5, 0), m + const Offset(5, 0), N.g100.withValues(alpha: .55));
    legibleLine(c, m + const Offset(0, -5), m + const Offset(0, 5), N.g100.withValues(alpha: .55));
  }

  @override
  bool shouldRepaint(_SafePainter o) => o.mode != mode;
}

// ---------------------------------------------------------------- 8 camera frustum

class _FrustumDemo extends StatefulWidget {
  const _FrustumDemo({required this.look, required this.fov, required this.near, required this.far});
  final Look look;
  final double fov, near, far;
  @override
  State<_FrustumDemo> createState() => _FrustumDemoState();
}

class _FrustumDemoState extends State<_FrustumDemo> {
  double camX = 0;
  int sel = 1;
  static const layers = [(-2.2, 3.0, 2.4), (1.5, 5.0, 1.8), (-.5, 7.0, 3.0), (3.5, 8.5, 1.5)];
  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: LayoutBuilder(builder: (context, box) {
          final sx = box.maxWidth / 10, sz = (box.maxHeight - 48) / 10;
          final cam = Offset(box.maxWidth / 2 + camX * sx, box.maxHeight - 24);
          final half = math.tan(widget.fov * math.pi / 360);
          return GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanUpdate: (d) => setState(() => camX = (camX + d.delta.dx / sx).clamp(-4.0, 4.0)),
            child: Stack(fit: StackFit.expand, children: [
              ColoredBox(color: N.g10),
              CustomPaint(painter: _FrustumPainter(cam, sx, sz, half, widget.near, widget.far, [for (var i = 0; i < layers.length; i++) (layers[i], i == sel)], widget.look)),
              for (var i = 0; i < layers.length; i++)
                Positioned(
                  left: box.maxWidth / 2 + (layers[i].$1 - layers[i].$3 / 2) * sx,
                  top: box.maxHeight - 24 - layers[i].$2 * sz - 8,
                  width: layers[i].$3 * sx,
                  height: 16,
                  child: MouseRegion(cursor: SystemMouseCursors.click, child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => setState(() => sel = i))),
                ),
              Positioned(left: 12, top: 12, child: Text('TOP', style: T.micro(N.g56).copyWith(letterSpacing: .8))),
              Positioned(left: 12, bottom: 12, child: Readout([('FOV', '${widget.fov.round()}°', N.g56), ('NEAR', widget.near.toStringAsFixed(1), axisZ), ('FAR', widget.far.toStringAsFixed(1), axisZ)], look: widget.look)),
            ]),
          );
        }),
      );
}

class _FrustumPainter extends CustomPainter {
  _FrustumPainter(this.cam, this.sx, this.sz, this.half, this.near, this.far, this.layers, this.look);
  final Offset cam;
  final double sx, sz, half, near, far;
  final List<((double, double, double), bool)> layers;
  final Look look;
  @override
  void paint(Canvas c, Size s) {
    final base = s.height - 24;
    // depth ticks: one hairline kind, labels are values
    for (var z = 0; z <= 10; z++) {
      c.drawRect(Rect.fromLTWH(0, base - z * sz, s.width, 1), Paint()..color = N.g100.withValues(alpha: z % 5 == 0 ? .09 : .04));
      if (z % 2 == 0) paintText(c, '$z', Offset(s.width - 24, base - z * sz - 12), T.value(N.g63).copyWith(fontSize: 10));
    }
    Offset pt(double z, double sign) => Offset(cam.dx + sign * z * half * sx, base - z * sz);
    final poly = Path()..moveTo(pt(near, -1).dx, pt(near, -1).dy)..lineTo(pt(far, -1).dx, pt(far, -1).dy)..lineTo(pt(far, 1).dx, pt(far, 1).dy)..lineTo(pt(near, 1).dx, pt(near, 1).dy)..close();
    c.drawPath(poly, Paint()..color = look == Look.glow ? accent.withValues(alpha: .10) : N.g100.withValues(alpha: .05));
    final edge = look == Look.quiet ? N.g63.withValues(alpha: .6) : N.g76;
    c.drawPath(poly, Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = edge);
    c.drawLine(cam, pt(near, -1), Paint()..color = edge.withValues(alpha: .4));
    c.drawLine(cam, pt(near, 1), Paint()..color = edge.withValues(alpha: .4));
    // layers: inside the frustum bright, outside dim, the selected one accent
    for (final l in layers) {
      final (x, z, w) = l.$1;
      final cx = s.width / 2 + x * sx;
      final inside = (cx - cam.dx).abs() <= z * half * sx + w * sx / 2 && z >= near && z <= far;
      final col = l.$2 ? accent : (inside ? N.g91 : N.g38);
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(cx, base - z * sz), width: w * sx, height: 4), const Radius.circular(2)), Paint()..color = col);
    }
    // the camera: a body and a lens, pointing up
    final body = RRect.fromRectAndRadius(Rect.fromCenter(center: cam + const Offset(0, 3), width: 16, height: 10), const Radius.circular(2));
    final lens = Path()..moveTo(cam.dx - 4, cam.dy - 2)..lineTo(cam.dx - 6, cam.dy - 9)..lineTo(cam.dx + 6, cam.dy - 9)..lineTo(cam.dx + 4, cam.dy - 2);
    c.drawRRect(body, Paint()..color = N.g10);
    c.drawPath(lens, Paint()..color = N.g10);
    c.drawRRect(body, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.4..strokeJoin = StrokeJoin.round..color = N.g95);
    c.drawPath(lens, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.4..strokeJoin = StrokeJoin.round..color = N.g95);
    // axis key: X right, Z up
    const o = Offset(24, 0);
    final a0 = Offset(o.dx, s.height - 56 - 0);
    legibleLine(c, a0, a0 + const Offset(20, 0), axisX, w: 1.5);
    legibleLine(c, a0, a0 + const Offset(0, -20), axisZ, w: 1.5);
    paintText(c, 'X', a0 + const Offset(26, 0), T.micro(axisX), centerY: true);
    paintText(c, 'Z', a0 + const Offset(0, -28), T.micro(axisZ), centerX: true, centerY: true);
  }

  @override
  bool shouldRepaint(_FrustumPainter o) => true;
}

// ---------------------------------------------------------------- 9 zoom HUD and corner label

class _HudDemo extends StatelessWidget {
  const _HudDemo({required this.look, required this.zoom, required this.corner, required this.view});
  final Look look;
  final int zoom;
  final Alignment corner;
  final String view;
  @override
  Widget build(BuildContext context) => SpLive<int>(
        initial: zoom,
        builder: (context, z, set) => _stage([
          Positioned(left: 12, right: 12, top: 12, bottom: 12, child: Align(alignment: corner, child: _CornerLabel(key: ValueKey(view), initial: view, look: look, up: corner.y > 0, right: corner.x > 0))),
          Positioned(left: 0, right: 0, bottom: 12, child: Center(child: _ZoomPill(zoom: z, onChanged: set, look: look))),
        ]),
      );
}

const _zoomSteps = [12, 25, 50, 100, 200, 400, 800];
const _fitZoom = 64; // [open] the "fit" value depends on stage size; fixed here for the demo
