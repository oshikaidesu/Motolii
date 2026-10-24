// Part of stage_set.dart (cut there by size, as it was).
part of 'stage_set.dart';

class _ZoomPill extends StatelessWidget {
  const _ZoomPill({required this.zoom, required this.onChanged, required this.look});
  final int zoom;
  final ValueChanged<int> onChanged;
  final Look look;
  Widget chip(String t, bool on, VoidCallback f) => Hover(
        onTap: f,
        builder: (context, h) => AnimatedContainer(duration: kFast, curve: Curves.easeOut, height: 24, padding: const EdgeInsets.symmetric(horizontal: 8), alignment: Alignment.center, decoration: BoxDecoration(color: on ? hair(.14) : (h ? hair(.08) : hair(0)), borderRadius: BorderRadius.circular(4)), child: Text(t, style: T.label(on ? N.g100 : N.g76))),
      );
  @override
  Widget build(BuildContext context) => Plate(
        look: look,
        padding: const EdgeInsets.all(2),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
          ChromeButton(glyph: Glyph.minus, size: 24, look: look, onTap: () => onChanged(_zoomSteps.lastWhere((s) => s < zoom, orElse: () => _zoomSteps.first))),
          Hover(
            onTap: () => onChanged(_zoomSteps.firstWhere((s) => s > zoom, orElse: () => _zoomSteps.first)),
            builder: (context, h) => SizedBox(width: 52, child: Center(child: Text('$zoom%', style: T.value(h ? N.g100 : N.g95)))),
          ),
          ChromeButton(glyph: Glyph.plus, size: 24, look: look, onTap: () => onChanged(_zoomSteps.firstWhere((s) => s > zoom, orElse: () => _zoomSteps.last))),
          const VDivider(),
          chip('Fit', zoom == _fitZoom, () => onChanged(_fitZoom)),
          chip('1:1', zoom == 100, () => onChanged(100)),
        ]),
      );
}

class _CornerLabel extends StatefulWidget {
  const _CornerLabel({super.key, required this.initial, required this.look, required this.up, required this.right});
  final String initial;
  final Look look;
  final bool up, right;
  @override
  State<_CornerLabel> createState() => _CornerLabelState();
}

class _CornerLabelState extends State<_CornerLabel> {
  late String v = widget.initial;
  bool open = false;
  @override
  Widget build(BuildContext context) {
    final head = Hover(
      onTap: () => setState(() => open = !open),
      builder: (context, h) => AnimatedContainer(
        duration: kFast,
        curve: Curves.easeOut,
        height: 24,
        padding: const EdgeInsets.only(left: 8, right: 4),
        decoration: BoxDecoration(color: N.g10.withValues(alpha: h || open ? .8 : .55), borderRadius: BorderRadius.circular(4), border: Border.all(color: hair(h || open ? .14 : .08))),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
          Text(v, style: T.name(h || open ? N.g100 : N.g91)),
          const SizedBox(width: 2),
          GlyphIcon(Glyph.chevron, color: N.g56, size: 14),
        ]),
      ),
    );
    final list = AnimatedSize(
      duration: kFast,
      curve: Curves.easeOut,
      alignment: widget.up ? Alignment.bottomCenter : Alignment.topCenter,
      child: !open
          ? const SizedBox(width: 0, height: 0)
          : Padding(
              padding: const EdgeInsets.symmetric(vertical: 4),
              child: Plate(look: widget.look, padding: const EdgeInsets.all(2), child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                for (final s in _views)
                  Hover(
                    onTap: () => setState(() { v = s; open = false; }),
                    builder: (context, h) => Container(height: 24, width: 120, padding: const EdgeInsets.symmetric(horizontal: 8), alignment: Alignment.centerLeft, decoration: BoxDecoration(color: s == v ? hair(.14) : (h ? hair(.08) : hair(0)), borderRadius: BorderRadius.circular(3)), child: Text(s, style: T.name(s == v ? N.g100 : N.g76))),
                  ),
              ])),
            ),
    );
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: widget.right ? CrossAxisAlignment.end : CrossAxisAlignment.start, children: widget.up ? [list, head] : [head, list]);
  }
}

// ---------------------------------------------------------------- 10 cursor tooltip

class _CursorDemo extends StatefulWidget {
  const _CursorDemo({required this.look, required this.crosshair, required this.centre, required this.percent});
  final Look look;
  final bool crosshair, centre, percent;
  @override
  State<_CursorDemo> createState() => _CursorDemoState();
}

class _CursorDemoState extends State<_CursorDemo> {
  Offset? p, down;
  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: LayoutBuilder(builder: (context, box) {
          final W = box.maxWidth, H = box.maxHeight, k = 1920 / W;
          String fmt(double v, double axisLen) => widget.percent ? '${(v / axisLen * 100).toStringAsFixed(1)}%' : (v * k).round().toString();
          const w = 112.0;
          final h = down != null ? 44.0 : 24.0;
          Offset corner() {
            final q = p ?? Offset(W * .45, H * .4);
            return Offset(q.dx + 14 + w > W ? q.dx - 14 - w : q.dx + 14, q.dy + 16 + h > H ? q.dy - 16 - h : q.dy + 16);
          }

          final q = p ?? Offset(W * .45, H * .4); // a fixed sample until the pointer arrives
          final ox = widget.centre ? W / 2 : 0.0, oy = widget.centre ? H / 2 : 0.0;
          return Listener(
            onPointerHover: (e) => setState(() => p = e.localPosition),
            onPointerMove: (e) => setState(() => p = e.localPosition),
            onPointerDown: (e) => setState(() => down = e.localPosition),
            onPointerUp: (_) => setState(() => down = null),
            child: MouseRegion(
              cursor: SystemMouseCursors.precise,
              onExit: (_) => setState(() => p = null),
              child: Stack(fit: StackFit.expand, children: [
                const SampleArt(),
                if (widget.crosshair) IgnorePointer(child: CustomPaint(painter: _CrossPainter(q))),
                if (true)
                  Positioned(
                    left: corner().dx,
                    top: corner().dy,
                    width: w,
                    child: IgnorePointer(
                      child: Plate(look: widget.look, radius: 4, padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6), child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                        Flex(direction: Axis.horizontal, children: [Text('X', style: T.micro(axisX)), const SizedBox(width: 4), Text(fmt(q.dx - ox, W), style: T.value(N.g95)), const Spacer(), Text('Y', style: T.micro(axisY)), const SizedBox(width: 4), Text(fmt(q.dy - oy, H), style: T.value(N.g95))]),
                        if (down != null) ...[
                          const SizedBox(height: 8),
                          Flex(direction: Axis.horizontal, children: [Text('Δ', style: T.micro(N.g56)), const SizedBox(width: 4), Text('${fmt(q.dx - down!.dx, W)}, ${fmt(q.dy - down!.dy, H)}', style: T.value(accent))]),
                        ],
                      ])),
                    ),
                  ),
              ]),
            ),
          );
        }),
      );
}

class _CrossPainter extends CustomPainter {
  _CrossPainter(this.p);
  final Offset p;
  @override
  void paint(Canvas c, Size s) {
    final paint = Paint()..color = N.g100.withValues(alpha: .22);
    c.drawRect(Rect.fromLTWH(0, p.dy.floorToDouble(), s.width, 1), paint);
    c.drawRect(Rect.fromLTWH(p.dx.floorToDouble(), 0, 1, s.height), paint);
  }

  @override
  bool shouldRepaint(_CrossPainter o) => o.p != p;
}

// ---------------------------------------------------------------- 11 snapping indicator

class _SnapDemo extends StatefulWidget {
  const _SnapDemo({required this.look, required this.on, required this.thr, required this.distance});
  final Look look;
  final bool on, distance;
  final double thr;
  @override
  State<_SnapDemo> createState() => _SnapDemoState();
}

class _SnapDemoState extends State<_SnapDemo> {
  static const others = [Rect.fromLTWH(70, 50, 130, 80), Rect.fromLTWH(330, 170, 110, 90)];
  Offset raw = const Offset(85, 150); // starts snapped, so the guides show statically
  static const msz = Size(100, 64);
  bool dragging = false;

  // returns the snapped rect plus the matched (vertical line x, other) and (horizontal line y, other)
  (Rect, (double, Rect)?, (double, Rect)?) solve() {
    var r = raw & msz;
    if (!widget.on) return (r, null, null);
    double bx = double.infinity, by = double.infinity;
    (double, Rect)? mx, my;
    for (final o in others) {
      for (final ox in [o.left, o.center.dx, o.right]) {
        for (final mxv in [r.left, r.center.dx, r.right]) {
          final d = ox - mxv;
          if (d.abs() < widget.thr && d.abs() < bx.abs()) { bx = d; mx = (ox, o); }
        }
      }
      for (final oy in [o.top, o.center.dy, o.bottom]) {
        for (final myv in [r.top, r.center.dy, r.bottom]) {
          final d = oy - myv;
          if (d.abs() < widget.thr && d.abs() < by.abs()) { by = d; my = (oy, o); }
        }
      }
    }
    r = r.shift(Offset(mx == null ? 0 : bx, my == null ? 0 : by));
    return (r, mx, my);
  }

  @override
  Widget build(BuildContext context) {
    final (m, mx, my) = solve();
    final lines = <Widget>[];
    Widget pill(Offset at, String t) => Positioned(left: at.dx - 18, top: at.dy - 9, width: 36, height: 18, child: Container(alignment: Alignment.center, decoration: BoxDecoration(color: N.g10.withValues(alpha: .88), borderRadius: BorderRadius.circular(3), border: Border.all(color: hair(.10))), child: Text(t, style: T.value(N.g95))));
    var vx = -1.0, hy = -1.0;
    var vr = Rect.zero, hr = Rect.zero;
    if (mx != null) {
      vx = mx.$1;
      vr = mx.$2;
      if (widget.distance) {
        final gap = m.top >= vr.bottom ? m.top - vr.bottom : (vr.top >= m.bottom ? vr.top - m.bottom : 0.0);
        if (gap > 0) lines.add(pill(Offset(vx, m.top >= vr.bottom ? (vr.bottom + m.top) / 2 : (m.bottom + vr.top) / 2), gap.round().toString()));
      }
    }
    if (my != null) {
      hy = my.$1;
      hr = my.$2;
      if (widget.distance) {
        final gap = m.left >= hr.right ? m.left - hr.right : (hr.left >= m.right ? hr.left - m.right : 0.0);
        if (gap > 0) lines.add(pill(Offset(m.left >= hr.right ? (hr.right + m.left) / 2 : (m.right + hr.left) / 2, hy), gap.round().toString()));
      }
    }
    return ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: Stack(fit: StackFit.expand, children: [
        const SampleArt(),
        for (var i = 0; i < others.length; i++)
          Positioned.fromRect(rect: others[i], child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: N.g63)))),
        Positioned.fromRect(
          rect: m,
          child: MouseRegion(
            cursor: SystemMouseCursors.move,
            child: GestureDetector(
              onPanStart: (_) => setState(() => dragging = true),
              onPanUpdate: (d) => setState(() => raw += d.delta),
              onPanEnd: (_) => setState(() { raw = m.topLeft; dragging = false; }),
              child: DecoratedBox(decoration: BoxDecoration(color: shade(.25), border: Border.all(color: N.g95))),
            ),
          ),
        ),
        IgnorePointer(child: CustomPaint(painter: _SnapPainter(m, mx == null ? null : (vx, vr), my == null ? null : (hy, hr), widget.look))),
        ...lines,
        Positioned(left: 12, bottom: 12, child: IgnorePointer(child: AnimatedOpacity(opacity: dragging || mx != null || my != null ? 0 : 1, duration: kFast, curve: Curves.easeOut, child: Text('Drag the box', style: T.label(N.g76))))),
      ]),
    );
  }
}

class _SnapPainter extends CustomPainter {
  _SnapPainter(this.m, this.v, this.h, this.look);
  final Rect m;
  final (double, Rect)? v, h;
  final Look look;
  @override
  void paint(Canvas c, Size s) {
    final col = look == Look.quiet ? magenta.withValues(alpha: .75) : magenta;
    void tick(Offset p, bool vertical) => legibleLine(c, p + (vertical ? const Offset(-3, 0) : const Offset(0, -3)), p + (vertical ? const Offset(3, 0) : const Offset(0, 3)), col);
    void line(Offset a, Offset b, bool vertical) {
      if (look == Look.glow) c.drawLine(a, b, Paint()..color = magenta.withValues(alpha: .5)..strokeWidth = 4..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3));
      legibleLine(c, a, b, col);
      if (look != Look.quiet) {
        tick(a, vertical);
        tick(b, vertical);
      }
    }

    if (v != null) line(Offset(v!.$1, math.min(m.top, v!.$2.top)), Offset(v!.$1, math.max(m.bottom, v!.$2.bottom)), true);
    if (h != null) line(Offset(math.min(m.left, h!.$2.left), h!.$1), Offset(math.max(m.right, h!.$2.right), h!.$1), false);
  }

  @override
  bool shouldRepaint(_SnapPainter o) => true;
}

// ---------------------------------------------------------------- 12 checkerboard backdrop

class _CheckerDemo extends StatelessWidget {
  const _CheckerDemo({required this.look, required this.cell, required this.alpha});
  final Look look;
  final double cell, alpha;
  @override
  Widget build(BuildContext context) {
    // concept: dark greys (quiet in the chrome); quiet: barely-there; glow: the light checker (reads alpha on dark art).
    final (a, b) = switch (look) { Look.concept => (N.g15, N.g20), Look.quiet => (N.g10, N.g13), Look.glow => (N.g63, N.g76) };
    return ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: Stack(fit: StackFit.expand, children: [
        CustomPaint(painter: _CheckerPainter(cell, a, b)),
        Center(child: Container(width: 220, height: 140, decoration: BoxDecoration(borderRadius: BorderRadius.circular(2), gradient: LinearGradient(colors: [Fam.scatter.c.withValues(alpha: alpha), Fam.attach.c.withValues(alpha: alpha)])))),
        Center(child: Container(width: 100, height: 100, decoration: BoxDecoration(shape: BoxShape.circle, color: Fam.along.c.withValues(alpha: alpha * .8)))),
      ]),
    );
  }
}

class _CheckerPainter extends CustomPainter {
  _CheckerPainter(this.cell, this.a, this.b);
  final double cell;
  final Color a, b;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = a);
    final p = Paint()..color = b;
    for (var y = 0; y * cell < s.height; y++) {
      for (var x = 0; x * cell < s.width; x++) {
        if ((x + y).isOdd) c.drawRect(Rect.fromLTWH(x * cell, y * cell, cell, cell), p);
      }
    }
  }

  @override
  bool shouldRepaint(_CheckerPainter o) => o.cell != cell || o.a != a || o.b != b;
}

// ---------------------------------------------------------------- 13 minimap

class _MiniMapDemo extends StatefulWidget {
  const _MiniMapDemo({required this.look, required this.zoom, required this.dim});
  final Look look;
  final double zoom;
  final bool dim;
  @override
  State<_MiniMapDemo> createState() => _MiniMapDemoState();
}

class _MiniMapDemoState extends State<_MiniMapDemo> {
  static const world = Size(1600, 900), mini = Size(160, 90), stageSz = Size(520, 300);
  Offset v = const Offset(300, 200); // view top-left in world px
  bool hot = false;
  Size get viewSz => Size(stageSz.width / widget.zoom, stageSz.height / widget.zoom);
  Offset clampV(Offset o) => Offset(o.dx.clamp(0.0, math.max(0.0, world.width - viewSz.width)), o.dy.clamp(0.0, math.max(0.0, world.height - viewSz.height)));

  @override
  Widget build(BuildContext context) {
    const m = 160 / 1600;
    final vv = clampV(v);
    final vr = Rect.fromLTWH(vv.dx * m, vv.dy * m, viewSz.width * m, viewSz.height * m);
    void center(Offset p) => setState(() => v = clampV(Offset(p.dx / m - viewSz.width / 2, p.dy / m - viewSz.height / 2)));
    return ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: Stack(fit: StackFit.expand, children: [
        ColoredBox(color: N.g07),
        OverflowBox(
          alignment: Alignment.topLeft,
          minWidth: 0,
          minHeight: 0,
          maxWidth: world.width * widget.zoom,
          maxHeight: world.height * widget.zoom,
          child: Transform.translate(offset: -vv * widget.zoom, child: SizedBox(width: world.width * widget.zoom, height: world.height * widget.zoom, child: const SampleArt())),
        ),
        Positioned(
          right: 12,
          bottom: 12,
          child: Container(
            width: mini.width + 8,
            height: mini.height + 8,
            padding: const EdgeInsets.all(4),
            decoration: plateDeco(widget.look),
            child: MouseRegion(
              onEnter: (_) => setState(() => hot = true),
              onExit: (_) => setState(() => hot = false),
              cursor: SystemMouseCursors.grab,
              child: GestureDetector(
                onPanDown: (d) => center(d.localPosition),
                onPanUpdate: (d) => center(d.localPosition),
                child: Stack(fit: StackFit.expand, children: [
                  const SampleArt(),
                  CustomPaint(painter: _MiniPainter(vr, widget.dim, hot, widget.look)),
                ]),
              ),
            ),
          ),
        ),
      ]),
    );
  }
}

class _MiniPainter extends CustomPainter {
  _MiniPainter(this.vr, this.dim, this.hot, this.look);
  final Rect vr;
  final bool dim, hot;
  final Look look;
  @override
  void paint(Canvas c, Size s) {
    final all = Path()..addRect(Offset.zero & s);
    final view = Path()..addRect(vr.intersect(Offset.zero & s));
    if (dim) c.drawPath(Path.combine(PathOperation.difference, all, view), Paint()..color = shade(.55));
    c.drawRect(vr, Paint()..color = accent.withValues(alpha: hot ? .22 : .12));
    legible(c, Path()..addRect(vr.deflate(.5)), look == Look.quiet ? N.g100 : accent);
  }

  @override
  bool shouldRepaint(_MiniPainter o) => true;
}

// ---------------------------------------------------------------- 14 composition frame

class _CompFrameDemo extends StatefulWidget {
  const _CompFrameDemo({required this.look, required this.dim, required this.aspect, required this.hide, required this.label});
  final Look look;
  final double dim, aspect;
  final bool hide, label;
  @override
  State<_CompFrameDemo> createState() => _CompFrameDemoState();
}

class _CompFrameDemoState extends State<_CompFrameDemo> {
  Offset shift = Offset.zero;
  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: LayoutBuilder(builder: (context, box) {
          final s = box.biggest;
          final maxW = s.width - 96, maxH = s.height - 72;
          final w = math.min(maxW, maxH * widget.aspect), h = w / widget.aspect;
          final f = Rect.fromCenter(center: Offset(s.width / 2, s.height / 2 + 8), width: w, height: h);
          final pxW = widget.aspect == 1 ? 1080 : (widget.aspect > 1 ? 1920 : 1080), pxH = widget.aspect == 1 ? 1080 : (widget.aspect > 1 ? 1080 : 1920);
          final art = Transform.translate(offset: shift, child: OverflowBox(maxWidth: s.width * 1.6, maxHeight: s.height * 1.6, child: SizedBox(width: s.width * 1.6, height: s.height * 1.6, child: const SampleArt())));
          return GestureDetector(
            onPanUpdate: (d) => setState(() => shift += d.delta),
            child: Stack(fit: StackFit.expand, children: [
              ColoredBox(color: N.g07),
              if (widget.hide) Positioned.fromRect(rect: f, child: ClipRect(child: Transform.translate(offset: Offset(-f.left, -f.top), child: OverflowBox(alignment: Alignment.topLeft, minWidth: 0, minHeight: 0, maxWidth: s.width, maxHeight: s.height, child: SizedBox(width: s.width, height: s.height, child: art))))) else Positioned.fill(child: art),
              if (!widget.hide) IgnorePointer(child: CustomPaint(painter: _DimPainter(f, widget.dim))),
              IgnorePointer(child: CustomPaint(painter: _FramePainter(f, widget.look))),
              if (widget.label) Positioned(left: f.left, top: f.top - 18, child: IgnorePointer(child: Text('Composition 1  ·  $pxW × $pxH', style: T.label(N.g63)))),
            ]),
          );
        }),
      );
}

class _DimPainter extends CustomPainter {
  _DimPainter(this.f, this.dim);
  final Rect f;
  final double dim;
  @override
  void paint(Canvas c, Size s) => c.drawPath(Path.combine(PathOperation.difference, Path()..addRect(Offset.zero & s), Path()..addRect(f)), Paint()..color = N.g07.withValues(alpha: dim));
  @override
  bool shouldRepaint(_DimPainter o) => o.f != f || o.dim != dim;
}

class _FramePainter extends CustomPainter {
  _FramePainter(this.f, this.look);
  final Rect f;
  final Look look;
  @override
  void paint(Canvas c, Size s) {
    final r = f.inflate(.5);
    switch (look) {
      case Look.concept:
        c.drawRect(r, Paint()..style = PaintingStyle.stroke..color = hair(.5));
      case Look.quiet:
        break;
      case Look.glow:
        c.drawRect(r, Paint()..style = PaintingStyle.stroke..strokeWidth = 4..color = accent.withValues(alpha: .35)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
        c.drawRect(r, Paint()..style = PaintingStyle.stroke..color = accent);
    }
  }

  @override
  bool shouldRepaint(_FramePainter o) => o.f != f || o.look != look;
}
