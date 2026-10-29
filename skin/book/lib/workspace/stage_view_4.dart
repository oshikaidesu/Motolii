// Part of stage_view.dart (cut there by size, as it was).
part of 'stage_view.dart';

class _ViewPainter extends CustomPainter {
  _ViewPainter(this.scene, this.v, this.sel, this.hover, {required this.grid, required this.safe, this.part = _Part.all, this.pivots = false, this.met = (null, null), this.marked = false, this.trail, this.parent, this.grip});
  final String? grip;   // (the handle in hand: it alone shown, lit; the others gone till it is let go)

  /// The inside marks the picked thing and the one under the hand on its desk: not drawn here.
  final bool marked;
  final _Part part;

  /// Joined: every thing's point it turns about shown (Pivots), and the lines a carried thing has stuck to (Snap).
  final bool pivots;
  final (double?, double?) met;

  /// The picked thing's way over its time, in points of the picture (null: it stays put, or not read yet).
  final List<Offset>? trail;

  /// The picked thing's parent (what it goes with), its frame drawn grey: Alight Motion's parent outline.
  final String? parent;
  final ArtScene scene;
  final _View v;
  final String? sel, hover;
  final bool grid, safe;

  Offset _scr(Offset doc) => v.toScreen(WsArt.toView(scene, doc));
  Path _boxPath(ArtBox b) => Path()..addPolygon([for (final p in b.corners) _scr(p)], true);

  @override
  void paint(Canvas c, Size size) {
    final comp = v.comp;
    if (part != _Part.over) c.drawRect(Offset.zero & size, Paint()..color = _V.surround);
    c.save();
    c.clipRect(v.area);
    if (part != _Part.over) _captions(c, comp);
    if (part == _Part.under) {
      c.restore();
      return;
    }
    c.save();
    c.clipRect(comp);
    if (grid) _grid(c, comp);
    if (safe) _safe(c, comp);
    c.restore();
    _guides(c);
    if (trail case final w? when w.length > 1) {   // (thin, an arrow every so often the way it goes)
      final line = _line(WsT.accent.withValues(alpha: .75), 1.2), pts = [for (final q in w) _scr(q)];
      c.drawPoints(PointMode.polygon, pts, line);
      for (var i = 6; i + 1 < pts.length; i += 12) {
        final d = pts[i + 1] - pts[i];
        if (d.distance < .5) continue;
        final u = d / d.distance, n = Offset(-u.dy, u.dx), tip = pts[i];
        c.drawPath(Path()..moveTo((tip - u * 6 + n * 4).dx, (tip - u * 6 + n * 4).dy)..lineTo(tip.dx, tip.dy)..lineTo((tip - u * 6 - n * 4).dx, (tip - u * 6 - n * 4).dy), line);
      }
    }
    if (met.$1 case final x?) c.drawLine(Offset(v.toScreen(Offset(x, 0)).dx, v.area.top), Offset(v.toScreen(Offset(x, 0)).dx, v.area.bottom), _line(WsT.accent));
    if (met.$2 case final y?) c.drawLine(Offset(v.area.left, v.toScreen(Offset(0, y)).dy), Offset(v.area.right, v.toScreen(Offset(0, y)).dy), _line(WsT.accent));
    if (pivots) {
      for (final MapEntry(:key, :value) in scene.boxes.entries) {
        if (key == sel) continue;
        final a = _scr(value.anchor);
        c.drawCircle(a, 3.5, _line(Grey.g00, 3));
        c.drawCircle(a, 3.5, _line(Grey.g91, 1.2));
      }
    }
    if (WsArt.boxOf(scene, parent) case final b?) c.drawPath(_boxPath(b), _line(Grey.g63, 1));   // (what the picked thing goes with: its frame, grey)
    if (marked) return c.restore();
    if (WsArt.boxOf(scene, hover) case final b?) _legible(c, _boxPath(b), wsLabel(hover!), 1);   // (in its label colour, as its row)
    if (WsArt.boxOf(scene, sel) case final b?) _selection(c, b, scene.layer(sel)!);
    c.restore();
  }

  void _captions(Canvas c, Rect comp) {
    if (comp.top - v.area.top < 18) return;
    final l = _tp(scene.title, T.micro(Grey.g76).copyWith(fontWeight: FontWeight.w700)), d = _tp('${WsArt.comp.width.round()} × ${WsArt.comp.height.round()}  ·  ${Ws.fps} fps', T.micro(Grey.g56));
    final y = comp.top - 6 - l.height;
    l.paint(c, Offset(comp.left, y));
    d.paint(c, Offset(comp.left + l.width + 8, y));
    final f = scene.frame, tc = _tp('F ${f.toString().padLeft(3, '0')}', T.value(Grey.g76));
    tc.paint(c, Offset(comp.right - tc.width, comp.top - 6 - tc.height));
  }

  void _grid(Canvas c, Rect comp) {
    final p = _line(_V.overlay);
    for (var i = 1; i < 12; i++) {
      final x = comp.left + comp.width * i / 12;
      c.drawLine(Offset(x, comp.top), Offset(x, comp.bottom), p);
    }
    for (var j = 1; j < 7; j++) {
      final y = comp.top + comp.height * j / 7;
      c.drawLine(Offset(comp.left, y), Offset(comp.right, y), p);
    }
  }

  void _safe(Canvas c, Rect comp) {
    final p = _line(_V.safe);
    c.drawRect(Rect.fromCenter(center: comp.center, width: comp.width * .9, height: comp.height * .9), _line(_V.overlay));
    final t = Rect.fromCenter(center: comp.center, width: comp.width * .8, height: comp.height * .8), k = comp.width * .03;
    for (final (o, dx, dy) in [(t.topLeft, 1.0, 1.0), (t.topRight, -1.0, 1.0), (t.bottomRight, -1.0, -1.0), (t.bottomLeft, 1.0, -1.0)]) {
      c.drawPath(
        Path()
          ..moveTo(o.dx + dx * k, o.dy)
          ..lineTo(o.dx, o.dy)
          ..lineTo(o.dx, o.dy + dy * k),
        p,
      );
    }
    final m = comp.center;
    c.drawLine(m - const Offset(8, 0), m + const Offset(8, 0), p);
    c.drawLine(m - const Offset(0, 8), m + const Offset(0, 8), p);
  }

  void _guides(Canvas c) {
    final p = _line(_V.guide);
    for (final gx in scene.guidesX) {
      final x = v.toScreen(Offset(gx, 0)).dx.roundToDouble() + .5;
      c.drawLine(Offset(x, v.area.top), Offset(x, v.area.bottom), p);
    }
    for (final gy in scene.guidesY) {
      final y = v.toScreen(Offset(0, gy)).dy.roundToDouble() + .5;
      c.drawLine(Offset(v.area.left, y), Offset(v.area.right, y), p);
    }
  }

  void _selection(Canvas c, ArtBox b, ArtLayer l) {
    final h = _handles(_scr, b, knob: l.kind != WsKind.camera && l.id != 'bg'), pts = [for (var i = 0; i < 4; i++) h['corner $i']!];
    _legible(c, Path()..addPolygon(pts, true), WsT.accent);
    final top = _scr(b.top);
    final lit = WsT.accent.withValues(alpha: 1), seen = (String k) => grip == null || grip == k;
    if (h['turn'] case final knob? when seen('turn')) {
      _legible(
        c,
        Path()
          ..moveTo(top.dx, top.dy)
          ..lineTo(knob.dx, knob.dy),
        WsT.accent,
        1,
      );
      final big = grip == 'turn' ? 2.0 : 1.0;
      c.drawCircle(knob, 4.5 * big, Paint()..color = Grey.g00);
      c.drawCircle(knob, 3.5 * big, Paint()..color = lit);
    }
    for (final (i, p) in pts.indexed) {
      if (!seen('corner $i')) continue;
      final r = Rect.fromCenter(center: p, width: _V.handle, height: _V.handle).inflate(grip == 'corner $i' ? _V.handle / 2 : 0);
      c.drawRect(r.inflate(1), Paint()..color = Grey.g00);
      c.drawRect(r, Paint()..color = lit);
    }
    final a = h['anchor']!;
    if (seen('anchor')) {
      final big = grip == 'anchor' ? 1.6 : 1.0;
      c.drawCircle(a, 6 * big, _line(Grey.g00, 3.5));
      c.drawCircle(a, 6 * big, _line(lit, 1.5));
      for (final d in const [Offset(1, 0), Offset(-1, 0), Offset(0, 1), Offset(0, -1)]) {
        c.drawLine(a + d * 3, a + d * 10 * big, _line(Grey.g00, 3));
        c.drawLine(a + d * 3, a + d * 10 * big, _line(lit, 1.2));
      }
    }
    final bounds = pts.skip(1).fold(Rect.fromPoints(pts[0], pts[0]), (r, p) => r.expandToInclude(Rect.fromPoints(p, p)));
    _tag(c, bounds.topLeft, l.name, WsT.accent, WsT.onAccent, wsTone(l.kind));
  }

  /// A small rounded name tag sitting on top of [at] (moved inside the viewport when it would leave it).
  void _tag(Canvas c, Offset at, String text, Color fill, Color ink, Color? pip) {
    final tp = _tp(text, T.micro(ink).copyWith(fontWeight: FontWeight.w700));
    final w = tp.width + 12 + (pip == null ? 0 : 10);
    var o = Offset(at.dx, at.dy - _V.tagH - 4);
    o = Offset(
      o.dx.clamp(v.area.left + 4, math.max(v.area.left + 4, v.area.right - w - 4)),
      o.dy.clamp(v.area.top + 4, math.max(v.area.top + 4, v.area.bottom - _V.tagH - 4)),
    );
    final r = RRect.fromRectAndRadius(Rect.fromLTWH(o.dx, o.dy, w, _V.tagH), Shape.r(WsT.chipRadius));
    c.drawRRect(r.inflate(1), Paint()..color = Grey.g00);
    c.drawRRect(r, Paint()..color = fill);
    var x = o.dx + 6;
    if (pip != null) {
      c.drawRect(Rect.fromLTWH(x, o.dy + 5, 6, 6), Paint()..color = Grey.g00);
      c.drawRect(Rect.fromLTWH(x + 1, o.dy + 6, 4, 4), Paint()..color = pip);
      x += 10;
    }
    tp.paint(c, Offset(x, o.dy + (_V.tagH - tp.height) / 2));
  }

  @override
  bool shouldRepaint(_ViewPainter o) => o.scene != scene || o.v != v || o.sel != sel || o.hover != hover || o.grid != grid || o.safe != safe || o.part != part || o.pivots != pivots || o.met != met || o.marked != marked || !identical(o.trail, trail) || o.parent != parent || o.grip != grip;
}

// ---- rulers -----------------------------------------------------------------------------------------------------------

class _RulerPainter extends CustomPainter {
  _RulerPainter(this.v, this.cursor, this.scene) : super(repaint: cursor);
  final _View v;
  final ArtScene scene;
  final ValueNotifier<Offset?> cursor;

  static double _step(double s) {
    for (final st in [25.0, 50.0, 100.0, 200.0, 250.0, 500.0, 1000.0]) {
      if (st * s >= 56) return st;
    }
    return 2000;
  }

  @override
  void paint(Canvas c, Size size) {
    final r = _V.ruler;
    final comp = v.comp, bg = Paint()..color = _V.rulerGround, span = Paint()..color = _V.rulerSpan;
    c.drawRect(Rect.fromLTWH(0, 0, size.width, r), bg);
    c.drawRect(Rect.fromLTWH(0, 0, r, size.height), bg);
    c.drawRect(Rect.fromLTRB(math.max(comp.left, r), 0, math.min(comp.right, size.width), r), span);
    c.drawRect(Rect.fromLTRB(0, math.max(comp.top, r), r, math.min(comp.bottom, size.height)), span);
    final edge = _line(WsT.line);
    c.drawLine(Offset(0, r - .5), Offset(size.width, r - .5), edge);
    c.drawLine(Offset(r - .5, 0), Offset(r - .5, size.height), edge);
    final step = _step(v.s), minor = step / 5, tick = Paint()..color = _V.tick;
    final label = T.micro(Grey.g56);

    c.save();
    c.clipRect(Rect.fromLTWH(r, 0, size.width - r, r));
    final x0 = (v.toComp(Offset(r, 0)).dx / minor).floor() * minor, x1 = v.toComp(Offset(size.width, 0)).dx;
    for (var x = x0; x <= x1; x += minor) {
      final sx = v.toScreen(Offset(x, 0)).dx.roundToDouble() + .5, major = (x / step).round() * step == x;
      c.drawLine(Offset(sx, major ? 2 : r - 4), Offset(sx, r), tick);
      if (major) _tp('${x.round()}', label).paint(c, Offset(sx + 3, 2));
    }
    c.restore();

    c.save();
    c.clipRect(Rect.fromLTWH(0, r, r, size.height - r));
    final y0 = (v.toComp(Offset(0, r)).dy / minor).floor() * minor, y1 = v.toComp(Offset(0, size.height)).dy;
    for (var y = y0; y <= y1; y += minor) {
      final sy = v.toScreen(Offset(0, y)).dy.roundToDouble() + .5, major = (y / step).round() * step == y;
      c.drawLine(Offset(major ? 2 : r - 4, sy), Offset(r, sy), tick);
      if (major) {
        c.save();
        c.translate(2, sy - 3);
        c.rotate(-math.pi / 2);
        _tp('${y.round()}', label).paint(c, Offset.zero);
        c.restore();
      }
    }
    c.restore();

    final g = Paint()..color = _V.guide;
    for (final gx in scene.guidesX) {
      final x = v.toScreen(Offset(gx, 0)).dx;
      if (x > r) c.drawPath(Path()..addPolygon([Offset(x - 4, r - 5), Offset(x + 4, r - 5), Offset(x, r)], true), g);
    }
    for (final gy in scene.guidesY) {
      final y = v.toScreen(Offset(0, gy)).dy;
      if (y > r) c.drawPath(Path()..addPolygon([Offset(r - 5, y - 4), Offset(r - 5, y + 4), Offset(r, y)], true), g);
    }

    if (cursor.value case final p?) {
      final s = v.toScreen(p),
          hot = Paint()
            ..color = WsT.accent
            ..strokeWidth = 1;
      if (s.dx > r) c.drawLine(Offset(s.dx, 0), Offset(s.dx, r), hot);
      if (s.dy > r) c.drawLine(Offset(0, s.dy), Offset(r, s.dy), hot);
    }
    c.drawRect(Rect.fromLTWH(0, 0, r, r), bg);
  }

  @override
  bool shouldRepaint(_RulerPainter o) => o.v != v || o.cursor != cursor || o.scene != scene;
}
