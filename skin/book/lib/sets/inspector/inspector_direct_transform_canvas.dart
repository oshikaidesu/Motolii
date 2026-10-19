// Rotation handles by index: 0 turns about Z (the knob), 1 tilts about X, 2 about Y.
import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform_layout.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

class TransformCanvas extends StatefulWidget {
  const TransformCanvas({super.key});
  @override
  State<TransformCanvas> createState() => _TransformCanvasState();
}

class _TransformCanvasState extends State<TransformCanvas> {
  final _focus = FocusNode(debugLabel: 'direct-transform');
  TransformLayout? _start;
  Doc? _doc;
  (TransformZone, int)? _grab, _hover;
  Offset _down = Offset.zero, _last = Offset.zero;

  bool _editable(Ctx x, TransformLayout g) => !x.cfg.locked && !RowOff.offOf(context) && g.ids.every((id) => x.doc.get(id) != null);

  void _setHover((TransformZone, int)? zone) {
    if (zone?.$1 == _hover?.$1 && zone?.$2 == _hover?.$2) return;
    setState(() => _hover = zone);
    Ctx.read(context).doc.setDirectHover((_grab ?? zone)?.$1.prop);
  }

  void _finish({bool cancel = false}) {
    if (_grab == null) return;
    cancel ? _doc?.cancelGesture() : _doc?.endGesture();
    setState(() {
      _grab = null;
      _start = null;
    });
  }

  @override
  void dispose() {
    if (_grab != null) _doc?.cancelGesture();
    _doc?.setDirectHover(null);
    _focus.dispose();
    super.dispose();
  }

  void _drag(Offset p) {
    final g = _start, grab = _grab, doc = _doc;
    if (g == null || grab == null || doc == null) return;
    final x = Ctx.read(context);
    if (!_editable(x, g) || doc.space != g.space) {
      _finish(cancel: true);
      return;
    }
    final next = g.drag(grab.$1, grab.$2, down: _down, last: _last, now: p, doc: doc, mult: x.cfg.mult());
    doc.setMany(next);
    _last = p;
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context);
    return Padding(
      padding: const EdgeInsets.only(top: 4),
      child: SizedBox(
        height: DirectSurface.height,
        child: LayoutBuilder(
          builder: (context, box) {
            final g = TransformLayout(box.biggest, Map.of(x.doc.v), space: x.doc.space);
            final mixed = g.ids.any((id) => x.doc.get(id) == null);
            final enabled = _editable(x, g);
            final own = _grab ?? _hover;
            final hot = own?.$1 ?? TransformZone.values.where((z) => z.prop == x.doc.directHover).firstOrNull;
            final canvas = Semantics(
              label: mixed
                  ? 'Transform canvas, mixed selection'
                  : 'Transform canvas: drag the object to move, handles to scale, the knob to rotate, the cross to move the pivot',
              enabled: enabled,
              child: Focus(
                focusNode: _focus,
                onKeyEvent: (_, e) {
                  if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _grab != null) {
                    _finish(cancel: true);
                    return KeyEventResult.handled;
                  }
                  return KeyEventResult.ignored;
                },
                child: MouseRegion(
                  cursor: !enabled || own == null ? SystemMouseCursors.basic : (_grab != null ? SystemMouseCursors.grabbing : SystemMouseCursors.grab),
                  onHover: (e) => _setHover(enabled ? g.pick(e.localPosition) : null),
                  onExit: (_) => _setHover(null),
                  child: Listener(
                    onPointerDown: (e) {
                      if (!enabled || e.buttons != kPrimaryButton) return;
                      final grab = g.pick(e.localPosition);
                      if (grab == null) return;
                      _focus.requestFocus();
                      _doc = x.doc..beginGesture();
                      _start = g;
                      _grab = grab;
                      _down = _last = e.localPosition;
                      x.doc.setDirectHover(grab.$1.prop);
                    },
                    onPointerMove: (e) => _drag(e.localPosition),
                    onPointerUp: (_) => _finish(),
                    onPointerCancel: (_) => _finish(cancel: true),
                    child: CustomPaint(
                      size: box.biggest,
                      painter: _CanvasPaint(g, enabled: enabled, mixed: mixed, hot: hot, hotIndex: own?.$2, link: x.doc.link),
                    ),
                  ),
                ),
              ),
            );
            final undo = !x.cfg.locked && x.doc.undoSteps > 0;
            final stack = Stack(
              children: [
                canvas,
                if (undo)
                  Positioned(
                    top: 4,
                    left: 4,
                    child: Semantics(
                      button: true,
                      label: 'Undo',
                      child: Hov(
                        onTap: x.doc.undoGesture,
                        builder: (_, h) => Container(
                          width: 22,
                          height: 22,
                          alignment: Alignment.center,
                          decoration: BoxDecoration(color: h ? Grey.g20 : Grey.g13, borderRadius: BorderRadius.all(Shape.r(5))),
                          child: CustomPaint(size: const Size.square(12), painter: _UndoPaint(h ? Grey.g100 : Grey.g76)),
                        ),
                      ),
                    ),
                  ),
              ],
            );
            return Pop.on(context) ? ClipRect(child: stack) : stack;
          },
        ),
      ),
    );
  }
}

class _UndoPaint extends CustomPainter {
  const _UndoPaint(this.ink);
  final Color ink;

  @override
  void paint(Canvas c, Size s) {
    final st = Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.4
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    final w = s.width;
    c.drawPath(
      Path()
        ..moveTo(w * .2, w * .4)
        ..lineTo(w * .62, w * .4)
        ..arcToPoint(Offset(w * .62, w * .86), radius: Radius.circular(w * .23))
        ..lineTo(w * .35, w * .86),
      st,
    );
    c.drawPath(
      Path()
        ..moveTo(w * .4, w * .18)
        ..lineTo(w * .18, w * .4)
        ..lineTo(w * .4, w * .62),
      st,
    );
  }

  @override
  bool shouldRepaint(_UndoPaint old) => old.ink != ink;
}

class _CanvasPaint extends CustomPainter {
  const _CanvasPaint(this.g, {required this.enabled, required this.mixed, required this.hot, required this.hotIndex, required this.link});
  final TransformLayout g;
  final bool enabled, mixed, link;
  final TransformZone? hot;
  final int? hotIndex;

  Paint line(Color color, [double width = 1]) => Paint()
    ..color = color
    ..style = PaintingStyle.stroke
    ..strokeWidth = width
    ..strokeCap = StrokeCap.round;

  /// A zone's colour; the others fade while one is hot so the grabbed part stands alone.
  Color ink(TransformZone zone) {
    if (!enabled) return Grey.g44;
    if (hot == null || hot == zone) return zone.color;
    return zone.color.withValues(alpha: .3);
  }

  /// A rotation handle's colour: the Z knob is the Rotation colour, the tilts wear their axis colour.
  Color tiltInk(int axis) {
    final base = _tiltInks[axis];
    if (!enabled) return Grey.g44;
    if (hot == null) return base;
    final on = hot == TransformZone.rotate && (hotIndex == null || hotIndex == axis);
    return on ? base : base.withValues(alpha: .3);
  }

  Paint fill(Color color) => Paint()..color = color;

  void arrowHead(Canvas c, Offset tip, Offset from, Color color) {
    final back = (from - tip) / math.max(.001, (from - tip).distance);
    c.drawLine(tip, tip + rotate(back, .6) * 4, line(color));
    c.drawLine(tip, tip + rotate(back, -.6) * 4, line(color));
  }

  static const _outline = [Offset(-1, -1), Offset(.45, -1), Offset(1, -.45), Offset(1, 1), Offset(-1, 1)];

  List<Offset> face([double back = 0]) => [for (final p in _outline) g.world(Offset(p.dx * g.half.dx, p.dy * g.half.dy), back)];

  void object(Canvas c, {bool ghost = false}) {
    final front = Path()..addPolygon(face(), true);
    if (ghost) {
      c.drawPath(front, fill(Grey.g20));
      c.drawPath(front, line(Grey.g38));
      return;
    }
    final body = ink(TransformZone.move);
    if (g.solid) {
      final t = math.max(0.0, g.value('depth')) * g.compositionFactor;
      final a = face(), b = face(t);
      final side = Color.lerp(body, Grey.g07, .6)!;
      c.drawPath(Path()..addPolygon(b, true), fill(Color.lerp(body, Grey.g07, .75)!));
      for (var i = 0; i < a.length; i++) {
        final j = (i + 1) % a.length;
        c.drawPath(Path()..addPolygon([a[i], a[j], b[j], b[i]], true), fill(side));
      }
    }
    c.drawPath(
      front,
      Paint()
        ..shader = LinearGradient(
          colors: [body, Color.lerp(body, const Color(0xFF8FE3FF), .55)!],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ).createShader(front.getBounds()),
    );
    if (hot != TransformZone.move) return;
    final arrows = Grey.g100;
    final reach = math.min(g.half.dx.abs(), g.half.dy.abs()) * .7;
    for (final d in const [Offset(1, 0), Offset(-1, 0), Offset(0, 1), Offset(0, -1)]) {
      final from = g.center + d * (reach * .6), tip = g.center + d * reach;
      c.drawLine(from, tip, line(arrows, 1.5));
      arrowHead(c, tip, from, arrows);
    }
  }

  /// The Z rail: a big square at the near end, a small one at the far end, the knob between.
  void rail(Canvas c, Size size) {
    final x = g.railX, top = size.height / 2 - g.railHalf;
    final bottom = size.height / 2 + g.railHalf;
    final lift = ink(TransformZone.lift);
    final on = hot == TransformZone.lift;
    c.drawLine(Offset(x, top + 6), Offset(x, bottom - 4), line(Grey.g26, 2));
    c.drawLine(Offset(x - 3, size.height / 2), Offset(x + 3, size.height / 2), line(Grey.g38));
    c.drawRect(Rect.fromCenter(center: Offset(x, top), width: 8, height: 8), line(g.deep ? Grey.g56 : Grey.g26));
    c.drawRect(Rect.fromCenter(center: Offset(x, bottom), width: 3, height: 3), line(g.deep ? Grey.g56 : Grey.g26));
    if (!g.deep) return; // (2D: no depth to pull, the rail only shows where it would be)
    c.drawLine(Offset(x, size.height / 2), g.liftKnob, line(lift, 2));
    c.drawCircle(g.liftKnob, on ? 6 : 5, fill(lift));
  }

  void tilt(Canvas c, int axis) {
    final color = tiltInk(axis);
    final k = g.tiltKnob(axis);
    c.drawLine(g.tiltBase(axis), k, line(color, 1.5));
    final on = hot == TransformZone.rotate && hotIndex == axis;
    final r = Rect.fromCenter(center: k, width: axis == 1 ? 7 : (on ? 13 : 11), height: axis == 1 ? (on ? 13 : 11) : 7);
    c.drawRRect(RRect.fromRectAndRadius(r, Shape.r(3.5)), fill(color));
  }

  @override
  void paint(Canvas c, Size size) {
    c.save();
    c.clipRect(Offset.zero & size);
    c.drawRect(Offset.zero & size, Paint()..color = Grey.g07);
    final f = g.frame;
    c.drawRect(f, line(Grey.g38));
    c.drawLine(f.topCenter, f.bottomCenter, line(Grey.g15));
    c.drawLine(f.centerLeft, f.centerRight, line(Grey.g15));
    if (mixed) {
      object(c, ghost: true);
      c.restore();
      return;
    }
    final guide = ink(TransformZone.move).withValues(alpha: hot == TransformZone.move ? .8 : .28);
    c.drawLine(Offset(g.pivot.dx, f.top), g.pivot, line(guide));
    c.drawLine(Offset(f.left, g.pivot.dy), g.pivot, line(guide));
    rail(c, size);

    object(c);

    final scale = ink(TransformZone.scale);
    final scaleHot = hot == TransformZone.scale;
    c.drawPath(Path()..addPolygon(g.corners, true), line(scale.withValues(alpha: scaleHot ? 1 : .45)));
    if (link && scaleHot) {
      c.drawLine(g.corners[0], g.corners[2], line(scale.withValues(alpha: .5)));
    }
    for (final (i, p) in g.handles.indexed) {
      final side = (i < 4 ? DirectSurface.handle : DirectSurface.handle - 2) + (scaleHot ? 2 : 0);
      final r = Rect.fromCenter(center: p, width: side, height: side);
      c.drawRRect(RRect.fromRectAndRadius(r, Shape.r(1.5)), fill(scale));
    }
    tilt(c, 1);
    tilt(c, 2);

    final rotate = tiltInk(0);
    c.drawLine(g.stemBase, g.knob, line(rotate, 1.5));
    c.drawCircle(g.knob, DirectSurface.knob + (hot == TransformZone.rotate && hotIndex == 0 ? 1.5 : 0), fill(rotate));
    const sweep = math.pi * 1.2;
    final start = g.angle - math.pi / 2 + .5;
    c.drawArc(Rect.fromCircle(center: g.knob, radius: DirectSurface.turnArc), start, sweep, false, line(rotate, 1.5));
    final endA = start + sweep;
    final tip = g.knob + Offset(math.cos(endA), math.sin(endA)) * DirectSurface.turnArc;
    arrowHead(c, tip, tip - Offset(-math.sin(endA), math.cos(endA)) * 4, rotate);

    final pivot = ink(TransformZone.pivot);
    final p = g.pivot;
    final r = hot == TransformZone.pivot ? 6.0 : 4.5;
    c.drawCircle(p, r + 1.5, fill(Grey.g07));
    c.drawCircle(p, r, line(pivot, 2));
    c.drawCircle(p, 1.5, fill(pivot));
    if (hot == TransformZone.rotate && (hotIndex ?? 0) == 0) {
      c.drawCircle(p, (g.knob - p).distance, line(rotate.withValues(alpha: .35)));
    }
    c.restore();
  }

  @override
  bool shouldRepaint(_CanvasPaint old) =>
      !mapEquals(old.g.values, g.values) ||
      old.g.size != g.size ||
      old.g.space != g.space ||
      old.enabled != enabled ||
      old.mixed != mixed ||
      old.hot != hot ||
      old.hotIndex != hotIndex ||
      old.link != link;
}

/// Rotation handles by index: 0 turns about Z (the knob), 1 tilts about X, 2 about Y.
const _tiltInks = [Color(0xFFFF6A2B), Color(0xFFE974AB), Color(0xFF7DD5B1)];
