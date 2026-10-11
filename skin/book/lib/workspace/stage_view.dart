// The stage's viewport: the composition centred on a dark surround, rulers and guides, overlays, the selection, the zoom HUD and the navigator.
// contract: paints only when the scene, the selection, the hover or the view changes; the pointer readout repaints the rulers alone.
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'stage_art.dart';
import 'ws.dart';

/// Zoom steps of the header chip: fit, then fixed scales.
const stageZooms = ['Fit', '50%', '100%'];
const _fixed = [0.0, .5, 1.0];

/// Viewport sizes: the ruler band, the margin a fit leaves around the composition, the HUD pill and the navigator.
abstract final class _V {
  static const ruler = 16.0, fitMargin = 28.0, hud = 22.0, mapW = 136.0, pad = 8.0;
  static const handle = 7.0, stem = 18.0, tagH = 16.0;
  static Color get surround => Grey.g10;
  static Color get rulerGround => Grey.g13;
  static Color get rulerSpan => Grey.g15;
  static Color get tick => Grey.g38;
  static const guide = WsT.toneAudio, overlay = Color(0x40FFFFFF), safe = Color(0x99F6EFDD);
}

/// Document guides (left margin and the title's baseline), in composition px.
const _guidesX = [150.0, 1770.0], _guidesY = [812.0];

/// Composition px to screen px: the composition's top-left at [o], [s] screen px per composition px, inside [area].
@immutable
class _View {
  const _View(this.o, this.s, this.area);
  final Offset o;
  final double s;
  final Rect area;
  Rect get comp => o & WsArt.comp * s;
  Offset toScreen(Offset p) => o + p * s;
  Offset toComp(Offset p) => (p - o) / s;
  @override
  bool operator ==(Object x) => x is _View && x.o == o && x.s == s && x.area == area;
  @override
  int get hashCode => Object.hash(o, s, area);
}

class StageView extends StatefulWidget {
  const StageView({
    super.key,
    required this.scene,
    required this.selected,
    required this.zoom,
    required this.grid,
    required this.safe,
    required this.cursor,
    this.onPoke,
  });
  final ArtScene scene;
  final String? selected;
  final int zoom;
  final bool grid, safe;

  /// The pointer in composition px while it is over the viewport; the HUD and the rulers read it.
  final ValueNotifier<Offset?> cursor;
  final VoidCallback? onPoke;
  @override
  State<StageView> createState() => _StageViewState();
}

class _StageViewState extends State<StageView> {
  Offset _pan = Offset.zero;
  String? _hover;

  @override
  void didUpdateWidget(StageView o) {
    super.didUpdateWidget(o);
    if (o.zoom != widget.zoom) _pan = Offset.zero;
  }

  _View _view(Size size) {
    final area = Rect.fromLTRB(_V.ruler, _V.ruler, size.width, size.height);
    final fit = math.max(.02, math.min((area.width - _V.fitMargin * 2) / WsArt.comp.width, (area.height - _V.fitMargin * 2) / WsArt.comp.height));
    final s = widget.zoom == 0 ? fit : _fixed[widget.zoom];
    final half = Offset(WsArt.comp.width * s, WsArt.comp.height * s) / 2;
    return _View(area.center - half + _pan, s, area);
  }

  Offset _doc(_View v, Offset local) => WsArt.toDoc(widget.scene, v.toComp(local));

  String? _hit(_View v, Offset local) {
    if (!v.comp.contains(local) && widget.scene.camera) return null;
    final d = _doc(v, local), id = WsArt.hit(widget.scene, d);
    if (id != null || widget.scene.camera) return id;
    final cam = WsArt.cameraBox(widget.scene), tol = 8 / v.s;
    return cam.contains(d, tol) && !cam.contains(d, -tol) ? 'cam' : null;
  }

  void _scroll(PointerSignalEvent e, _View v) {
    if (e is! PointerScrollEvent || widget.zoom == 0) return;
    final lim = Offset(WsArt.comp.width * v.s / 2 + v.area.width / 2 - 60, WsArt.comp.height * v.s / 2 + v.area.height / 2 - 60);
    final p = _pan - e.scrollDelta;
    setState(() => _pan = Offset(p.dx.clamp(-lim.dx, lim.dx), p.dy.clamp(-lim.dy, lim.dy)));
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final v = _view(box.biggest), scene = widget.scene;
      return Listener(
        onPointerSignal: (e) => _scroll(e, v),
        child: MouseRegion(
          onHover: (e) {
            widget.cursor.value = v.toComp(e.localPosition);
            final h = _hit(v, e.localPosition);
            if (h != _hover) setState(() => _hover = h);
          },
          onExit: (_) {
            widget.cursor.value = null;
            if (_hover != null) setState(() => _hover = null);
          },
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTapUp: (d) {
              widget.onPoke?.call();
              WsScope.read(context).select(_hit(v, d.localPosition));
            },
            child: Stack(
              fit: StackFit.expand,
              children: [
                RepaintBoundary(
                  child: CustomPaint(
                    painter: _ViewPainter(scene, v, widget.selected, _hover == widget.selected ? null : _hover, grid: widget.grid, safe: widget.safe),
                  ),
                ),
                IgnorePointer(child: CustomPaint(painter: _RulerPainter(v, widget.cursor))),
                Positioned.fromRect(
                  rect: v.comp,
                  child: const IgnorePointer(child: SizedBox(key: ValueKey('ws-stage-comp'))),
                ),
                Positioned(
                  left: _V.ruler + _V.pad,
                  bottom: _V.pad,
                  child: _Hud(scale: v.s, cursor: widget.cursor),
                ),
                if (widget.zoom != 0 && box.maxWidth > 360 && box.maxHeight > 220)
                  Positioned(
                    right: _V.pad,
                    bottom: _V.pad,
                    child: _Navigator(scene: scene, view: v),
                  ),
              ],
            ),
          ),
        ),
      );
    },
  );
}

/// The pill at the bottom-left of the viewport: zoom and where the pointer is, in composition px.
class _Hud extends StatelessWidget {
  const _Hud({required this.scale, required this.cursor});
  final double scale;
  final ValueNotifier<Offset?> cursor;
  @override
  Widget build(BuildContext context) => Container(
    height: _V.hud,
    padding: const EdgeInsets.symmetric(horizontal: 8),
    decoration: BoxDecoration(color: WsT.card, borderRadius: BorderRadius.circular(WsT.radius)),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text('${(scale * 100).round()}%', style: T.value(Grey.g95)),
        Container(width: 1, height: 10, margin: const EdgeInsets.symmetric(horizontal: 8), color: WsT.line),
        ValueListenableBuilder<Offset?>(
          valueListenable: cursor,
          builder: (_, p, _) => Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('X ', style: T.micro(Grey.g56)),
              SizedBox(width: 34, child: Text(p == null ? '—' : '${p.dx.round()}', style: T.value(p == null ? Grey.g56 : Grey.g91))),
              Text('Y ', style: T.micro(Grey.g56)),
              SizedBox(width: 34, child: Text(p == null ? '—' : '${p.dy.round()}', style: T.value(p == null ? Grey.g56 : Grey.g91))),
            ],
          ),
        ),
      ],
    ),
  );
}

/// The navigator: the whole frame small, the part the viewport shows outlined in the accent. Shown only when zoomed past fit.
class _Navigator extends StatelessWidget {
  const _Navigator({required this.scene, required this.view});
  final ArtScene scene;
  final _View view;
  @override
  Widget build(BuildContext context) {
    final h = _V.mapW * WsArt.comp.height / WsArt.comp.width;
    return Container(
      width: _V.mapW + 8,
      height: h + 8,
      padding: const EdgeInsets.all(4),
      color: WsT.card,
      child: CustomPaint(painter: _MapPainter(scene, Rect.fromPoints(view.toComp(view.area.topLeft), view.toComp(view.area.bottomRight)))),
    );
  }
}

class _MapPainter extends CustomPainter {
  _MapPainter(this.scene, this.shown);
  final ArtScene scene;
  final Rect shown;
  @override
  void paint(Canvas c, Size s) {
    final k = s.width / WsArt.comp.width;
    c.save();
    c.clipRect(Offset.zero & s);
    c.scale(k);
    WsArt.paint(c, scene);
    c.restore();
    final r = Rect.fromLTRB(shown.left * k, shown.top * k, shown.right * k, shown.bottom * k).intersect(Offset.zero & s);
    c.drawRect(
      r.deflate(.75),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..color = WsT.accent,
    );
  }

  @override
  bool shouldRepaint(_MapPainter o) => o.scene != scene || o.shown != shown;
}

// ---- the picture and what sits on it ----------------------------------------------------------------------------------

Paint _line(Color c, [double w = 1]) => Paint()
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..color = c;

/// A line that reads on any artwork: a dark under-stroke, then the colour.
void _legible(Canvas c, Path p, Color col, [double w = 1.5]) {
  c.drawPath(p, _line(Grey.g00, w + 2));
  c.drawPath(p, _line(col, w));
}

TextPainter _tp(String t, TextStyle s) => TextPainter(
  text: TextSpan(text: t, style: s),
  textDirection: TextDirection.ltr,
  maxLines: 1,
)..layout();

class _ViewPainter extends CustomPainter {
  _ViewPainter(this.scene, this.v, this.sel, this.hover, {required this.grid, required this.safe});
  final ArtScene scene;
  final _View v;
  final String? sel, hover;
  final bool grid, safe;

  Offset _scr(Offset doc) => v.toScreen(WsArt.toView(scene, doc));
  Path _boxPath(ArtBox b) => Path()..addPolygon([for (final p in b.corners) _scr(p)], true);

  @override
  void paint(Canvas c, Size size) {
    final comp = v.comp;
    c.drawRect(Offset.zero & size, Paint()..color = _V.surround);
    c.save();
    c.clipRect(v.area);
    _captions(c, comp);
    c.save();
    c.clipRect(comp);
    c.save();
    c.translate(v.o.dx, v.o.dy);
    c.scale(v.s);
    WsArt.paint(c, scene);
    c.restore();
    if (grid) _grid(c, comp);
    if (safe) _safe(c, comp);
    c.restore();
    _guides(c);
    if (!scene.camera && scene.layer('cam') != null) _camera(c);
    if (WsArt.boxOf(scene, hover) case final b?) _legible(c, _boxPath(b), wsTone(scene.layer(hover)!.kind), 1);
    if (WsArt.boxOf(scene, sel) case final b?) _selection(c, b, scene.layer(sel)!);
    c.restore();
  }

  void _captions(Canvas c, Rect comp) {
    if (comp.top - v.area.top < 18) return;
    final l = _tp('CODA_main', T.micro(Grey.g76).copyWith(fontWeight: FontWeight.w700)), d = _tp('1920 × 1080  ·  30 fps', T.micro(Grey.g56));
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
    for (final gx in _guidesX) {
      final x = v.toScreen(Offset(gx, 0)).dx.roundToDouble() + .5;
      c.drawLine(Offset(x, v.area.top), Offset(x, v.area.bottom), p);
    }
    for (final gy in _guidesY) {
      final y = v.toScreen(Offset(0, gy)).dy.roundToDouble() + .5;
      c.drawLine(Offset(v.area.left, y), Offset(v.area.right, y), p);
    }
  }

  void _camera(Canvas c) {
    final b = WsArt.cameraBox(scene), pts = [for (final p in b.corners) v.toScreen(p)];
    final dash = _line(WsT.toneCamera, 1.5);
    for (var i = 0; i < 4; i++) {
      final a = pts[i], z = pts[(i + 1) % 4], len = (z - a).distance, dir = (z - a) / len;
      for (var t = 0.0; t < len; t += 10) {
        c.drawLine(a + dir * t, a + dir * math.min(t + 6, len), dash);
      }
    }
    if (sel != 'cam') _tag(c, pts[0], 'Camera', WsT.toneCamera, WsT.onAccent, null);
  }

  void _selection(Canvas c, ArtBox b, ArtLayer l) {
    final pts = [for (final p in b.corners) _scr(p)];
    _legible(c, Path()..addPolygon(pts, true), WsT.accent);
    final top = _scr(b.top), dir = top - _scr(b.c), n = dir.distance < 1 ? const Offset(0, -1) : dir / dir.distance, knob = top + n * _V.stem;
    if (l.kind != WsKind.camera && l.id != 'bg') {
      _legible(
        c,
        Path()
          ..moveTo(top.dx, top.dy)
          ..lineTo(knob.dx, knob.dy),
        WsT.accent,
        1,
      );
      c.drawCircle(knob, 4.5, Paint()..color = Grey.g00);
      c.drawCircle(knob, 3.5, Paint()..color = WsT.accent);
    }
    for (final p in pts) {
      final r = Rect.fromCenter(center: p, width: _V.handle, height: _V.handle);
      c.drawRect(r.inflate(1), Paint()..color = Grey.g00);
      c.drawRect(r, Paint()..color = WsT.accent);
    }
    final a = _scr(b.anchor);
    c.drawCircle(a, 6, _line(Grey.g00, 3.5));
    c.drawCircle(a, 6, _line(WsT.accent, 1.5));
    for (final d in const [Offset(1, 0), Offset(-1, 0), Offset(0, 1), Offset(0, -1)]) {
      c.drawLine(a + d * 3, a + d * 10, _line(Grey.g00, 3));
      c.drawLine(a + d * 3, a + d * 10, _line(WsT.accent, 1.2));
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
    final r = RRect.fromRectAndRadius(Rect.fromLTWH(o.dx, o.dy, w, _V.tagH), const Radius.circular(WsT.chipRadius));
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
  bool shouldRepaint(_ViewPainter o) => o.scene != scene || o.v != v || o.sel != sel || o.hover != hover || o.grid != grid || o.safe != safe;
}

// ---- rulers -----------------------------------------------------------------------------------------------------------

class _RulerPainter extends CustomPainter {
  _RulerPainter(this.v, this.cursor) : super(repaint: cursor);
  final _View v;
  final ValueNotifier<Offset?> cursor;

  static double _step(double s) {
    for (final st in const [25.0, 50.0, 100.0, 200.0, 250.0, 500.0, 1000.0]) {
      if (st * s >= 56) return st;
    }
    return 2000;
  }

  @override
  void paint(Canvas c, Size size) {
    const r = _V.ruler;
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
    for (final gx in _guidesX) {
      final x = v.toScreen(Offset(gx, 0)).dx;
      if (x > r) c.drawPath(Path()..addPolygon([Offset(x - 4, r - 5), Offset(x + 4, r - 5), Offset(x, r)], true), g);
    }
    for (final gy in _guidesY) {
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
    c.drawRect(const Rect.fromLTWH(0, 0, r, r), bg);
  }

  @override
  bool shouldRepaint(_RulerPainter o) => o.v != v || o.cursor != cursor;
}
