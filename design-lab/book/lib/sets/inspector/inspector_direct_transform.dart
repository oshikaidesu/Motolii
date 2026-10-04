import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'inspector_parts.dart';

/// What a point on the canvas grabs. Each zone edits one Transform row.
/// The colour is the zone's name: the canvas part and its row wear the same one.
enum TransformZone {
  move('pos', Color(0xFF3D6BFF)),
  scale('scale', Color(0xFFB3C66B)),
  rotate('rot', Color(0xFFFF6A2B)),
  pivot('anchor', Color(0xFFB57CFF)),
  lift('pos', Color(0xFF8FB0FF));

  const TransformZone(this.prop, this.color);
  final String prop;
  final Color color;
}

/// Rotation handles by index: 0 turns about Z (the knob), 1 tilts about X, 2 about Y.
const _tiltInks = [Color(0xFFFF6A2B), Color(0xFFE974AB), Color(0xFF7DD5B1)];

abstract final class DirectSurface {
  // surface: one canvas for the whole Transform block, sized to keep the rows below it on screen.
  static const height = 124.0;
  static const objectWidth = 64.0, objectHeight = 40.0, handle = 7.0;
  static const stem = 20.0, knob = 6.0, turnArc = 11.0;
  static const tiltStem = 16.0, rail = 12.0, zReach = 1000.0;
}

class DirectTransform extends StatelessWidget {
  const DirectTransform({super.key, this.titled = false, this.first = true});
  final bool titled, first;

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    final depth = doc.get('depth');
    return RowMark(
      marks: const {
        'pos': ValueGlyph(ValueShape.position),
        'scale': ValueGlyph(ValueShape.scale),
        'rot': ValueGlyph(ValueShape.rotation),
        'op': ValueGlyph(ValueShape.opacity),
        'depth': ValueGlyph(ValueShape.depth),
        'parent': ValueGlyph(ValueShape.parent),
      },
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisSize: MainAxisSize.min,
        children: [
          Sect(
            title: 'Transform',
            first: first,
            tone: Pop.toneTransform,
            keyIds: const ['pos', 'scale', 'rot'],
            mark: CardMark.transform,
            brief: ['${brief(doc.get('pos.x'))}, ${brief(doc.get('pos.y'))}', '${brief(doc.get('scale.x'))}%', '${brief(doc.get('rot.z'))}°'],
            children: const [
              SpaceRow(),
              TransformCanvas(),
              SizedBox(height: Pop.titleGap),
              AxisColumns(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  mainAxisSize: MainAxisSize.min,
                  children: [AxisHeader(), PosRow(), ScaleRow(), RotRow()],
                ),
              ),
            ],
          ),
          const AnchorCard(),
          Sect(
            title: 'Layer',
            tone: Pop.toneLayer,
            keyIds: const ['op', 'depth'],
            mark: CardMark.layer,
            brief: ['${brief(doc.get('op'), 0)}%', depth == 0 ? 'Flat' : '${brief(depth, 0)} px', if (doc.s2['parent'] case final p? when p != 'None') p],
            children: const [LayerTiles()],
          ),
        ],
      ),
    );
  }
}

enum ValueShape {
  position(TransformZone.move),
  scale(TransformZone.scale),
  rotation(TransformZone.rotate),
  opacity(null),
  depth(null),
  parent(null);

  const ValueShape(this.zone);
  final TransformZone? zone;
}

/// The row's current value as a picture: where it sits, how big, which way, how solid.
class ValueGlyph extends StatelessWidget {
  const ValueGlyph(this.shape, {super.key});
  final ValueShape shape;
  static const size = 20.0;

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    final ids = switch (shape) {
      ValueShape.position => ['pos.x', 'pos.y'],
      ValueShape.scale => ['scale.x', 'scale.y'],
      ValueShape.rotation => ['rot.z'],
      ValueShape.opacity => ['op'],
      ValueShape.depth => ['depth'],
      ValueShape.parent => ['parent'],
    };
    final values = shape == ValueShape.parent ? [(doc.s2['parent'] ?? 'None') == 'None' ? 0.0 : 1.0] : [for (final id in ids) doc.get(id)];
    final hot = shape.zone != null && doc.directHover == shape.zone!.prop;
    return CustomPaint(
      size: const Size(24, size),
      painter: _GlyphPaint(shape, values.contains(null) ? null : values.cast<double>(), hot ? shape.zone!.color : Grey.g76),
    );
  }
}

class _GlyphPaint extends CustomPainter {
  const _GlyphPaint(this.shape, this.values, this.ink);
  final ValueShape shape;
  final List<double>? values;
  final Color ink;

  @override
  void paint(Canvas c, Size s) {
    final mid = s.center(Offset.zero);
    final rim = Paint()
      ..color = Grey.g44
      ..style = PaintingStyle.stroke;
    final solid = Paint()..color = ink;
    final v = values;
    if (v == null) {
      c.drawLine(mid - const Offset(4, 0), mid + const Offset(4, 0), rim);
      return;
    }
    switch (shape) {
      case ValueShape.position:
        final f = Rect.fromCenter(center: mid, width: 20, height: 11.25);
        c.drawRect(f, rim);
        final p = Offset(f.left + (v[0] / 1920).clamp(0, 1) * f.width, f.top + (v[1] / 1080).clamp(0, 1) * f.height);
        c.drawCircle(p, 2.5, solid);
      case ValueShape.scale:
        // only the proportion shows: a uniform scale keeps the square
        const side = 14.0;
        final sx = v[0].abs(), sy = v[1].abs(), big = math.max(sx, sy);
        c.drawRect(
          Rect.fromCenter(center: mid, width: big == 0 ? 1 : math.max(1.5, side * sx / big), height: big == 0 ? 1 : math.max(1.5, side * sy / big)),
          solid..color = ink.withValues(alpha: .85),
        );
      case ValueShape.rotation:
        c.drawCircle(mid, 8, rim);
        final a = v[0] * math.pi / 180 - math.pi / 2;
        c.drawLine(
          mid,
          mid + Offset(math.cos(a), math.sin(a)) * 8,
          Paint()
            ..color = ink
            ..strokeWidth = 2
            ..strokeCap = StrokeCap.round,
        );
        c.drawCircle(mid, 1.5, solid);
        final turns = (v[0].abs() / 360).floor().clamp(0, 3);
        for (var i = 0; i < turns; i++) {
          c.drawCircle(Offset(s.width - 1.5, 2 + i * 4.0), 1.2, solid);
        }
      case ValueShape.opacity:
        final r = Rect.fromCenter(center: mid, width: 14, height: 14);
        const cell = 3.5;
        for (var y = 0; y < 4; y++) {
          for (var x = 0; x < 4; x++) {
            c.drawRect(Rect.fromLTWH(r.left + x * cell, r.top + y * cell, cell, cell), Paint()..color = (x + y).isEven ? Grey.g26 : Grey.g15);
          }
        }
        c.drawRect(r, Paint()..color = ink.withValues(alpha: (v[0] / 100).clamp(0, 1)));
        c.drawRect(r, rim);
      case ValueShape.depth:
        final off = (v[0] / 25).clamp(0.0, 5.0);
        final front = Rect.fromCenter(center: mid + Offset(-off / 2, off / 2), width: 11, height: 11);
        if (off > 0) c.drawRect(front.shift(Offset(off, -off)), rim);
        c.drawRect(front, solid..color = ink.withValues(alpha: .85));
      case ValueShape.parent:
        final child = Rect.fromCenter(center: mid + const Offset(3, 3), width: 9, height: 9);
        if (v[0] > 0) {
          final top = mid + const Offset(-5, -5);
          c.drawLine(top, Offset(top.dx, child.center.dy), rim..color = Grey.g56);
          c.drawLine(Offset(top.dx, child.center.dy), child.centerLeft, rim);
          c.drawCircle(top, 2.5, solid);
        }
        c.drawRect(
          child,
          v[0] > 0
              ? (Paint()..color = ink.withValues(alpha: .85))
              : (Paint()
                  ..color = Grey.g44
                  ..style = PaintingStyle.stroke),
        );
    }
  }

  @override
  bool shouldRepaint(_GlyphPaint old) => old.shape != shape || old.ink != ink || !listEquals(old.values, values);
}

Offset _rotate(Offset p, double angle) => Offset(p.dx * math.cos(angle) - p.dy * math.sin(angle), p.dx * math.sin(angle) + p.dy * math.cos(angle));

double _wrapped(double a) => math.atan2(math.sin(a), math.cos(a));

/// Where everything sits on a canvas of [size] for the layer values [values], seen from the front as in 2D.
/// Every space edits the same values with the same handles; outside 2D nearer layers draw larger, and 3D shows Depth as thickness.
class TransformLayout {
  TransformLayout(this.size, this.values, {this.space = Space.d2});
  final Size size;
  final Map<String, double> values;
  final Space space;

  bool get deep => space != Space.d2;
  bool get solid => space == Space.d3;

  /// Every value the canvas edits; any of them mixed makes it inert.
  List<String> get ids => ['pos.x', 'pos.y', 'scale.x', 'scale.y', 'rot.z', 'anchor.x', 'anchor.y', 'pos.z', 'rot.x', 'rot.y', 'depth'];

  double value(String id) => values[id] ?? 0;
  double get angle => value('rot.z') * math.pi / 180;
  double get tiltX => value('rot.x') * math.pi / 180;
  double get tiltY => value('rot.y') * math.pi / 180;
  double get z => value('pos.z');
  Rect get frame {
    final w = math.min(size.width - 24, (size.height - 24) * 1920 / 1080);
    return Rect.fromCenter(center: size.center(Offset.zero), width: w, height: w * 1080 / 1920);
  }

  double get compositionFactor => frame.width / 1920;

  // camera: a composition-wide distance, so Z -1000 reads clearly bigger without leaving the frame
  static const _camera = 2400.0;

  /// Held to the camera (2D), Z orders the layer but never changes its size.
  double depthScale(double zc) => deep ? _camera / math.max(300, _camera + zc) : 1;
  Offset _project(Offset flat, double zc) => frame.center + (flat - frame.center) * depthScale(zc);

  Offset get half => Offset(DirectSurface.objectWidth / 2 * value('scale.x') / 100, DirectSurface.objectHeight / 2 * value('scale.y') / 100);
  Offset get anchor => Offset((value('anchor.x') / 50 - 1) * half.dx, (value('anchor.y') / 50 - 1) * half.dy);
  Offset get flatPivot => frame.topLeft + Offset(value('pos.x'), value('pos.y')) * compositionFactor;
  Offset get pivot => _project(flatPivot, z);

  /// A point of the layer, in canvas units at scale around the layer centre, pushed [back] units away from the viewer.
  Offset world(Offset p, [double back = 0]) {
    var (x, y, d) = (p.dx - anchor.dx, p.dy - anchor.dy, back);
    (y, d) = (y * math.cos(tiltX) - d * math.sin(tiltX), y * math.sin(tiltX) + d * math.cos(tiltX));
    (x, d) = (x * math.cos(tiltY) + d * math.sin(tiltY), -x * math.sin(tiltY) + d * math.cos(tiltY));
    return _project(flatPivot + _rotate(Offset(x, y), angle), z + d / compositionFactor);
  }

  Offset get center => world(Offset.zero);
  Offset local(Offset p) => _rotate((p - pivot) / depthScale(z), -angle) + anchor;

  static const _handlePoints = [Offset(-1, -1), Offset(1, -1), Offset(1, 1), Offset(-1, 1), Offset(-1, 0), Offset(1, 0), Offset(0, -1), Offset(0, 1)];
  List<Offset> get handles => [for (final p in _handlePoints) world(Offset(p.dx * half.dx, p.dy * half.dy))];
  List<Offset> get corners => handles.take(4).toList();
  Offset get stemBase => world(Offset(0, -half.dy.abs()));
  Offset get knob => world(Offset(0, -half.dy.abs() - DirectSurface.stem));

  /// Tilt knobs: below the layer tilts it about X, left of it about Y.
  Offset tiltBase(int axis) => axis == 1 ? world(Offset(0, half.dy.abs())) : world(Offset(-half.dx.abs(), 0));
  Offset tiltKnob(int axis) => axis == 1 ? world(Offset(0, half.dy.abs() + DirectSurface.tiltStem)) : world(Offset(-half.dx.abs() - DirectSurface.tiltStem, 0));

  /// The Z rail: near (negative Z) at the top, far at the bottom.
  double get railX => size.width - DirectSurface.rail;
  double get railHalf => size.height / 2 - 16;
  Offset get liftKnob => Offset(railX, size.height / 2 + (z / DirectSurface.zReach).clamp(-1, 1) * railHalf);

  bool body(Offset p) => (Path()..addPolygon(corners, true)).contains(p);

  /// The zone under [p] (with the handle index for scale and rotation), front to back.
  (TransformZone, int)? pick(Offset p) {
    if ((p - knob).distance <= 8) return (TransformZone.rotate, 0);
    if ((p - liftKnob).distance <= 8) return (TransformZone.lift, 0);
    for (final axis in const [1, 2]) {
      if ((p - tiltKnob(axis)).distance <= 7) {
        return (TransformZone.rotate, axis);
      }
    }
    if ((p - pivot).distance <= 7) return (TransformZone.pivot, 0);
    for (final (i, h) in handles.indexed) {
      if ((p - h).distance <= 7) return (TransformZone.scale, i);
    }
    if (body(p)) return (TransformZone.move, 0);
    return null;
  }

  /// New values for a drag of [zone] from [last] to [now]; [down] is where it started on this layout.
  Map<String, double> drag(
    TransformZone zone,
    int index, {
    required Offset down,
    required Offset last,
    required Offset now,
    required Doc doc,
    required double mult,
  }) {
    double cur(String id) => doc.get(id)!;
    final step = now - last;
    switch (zone) {
      case TransformZone.move:
        final delta = step / (compositionFactor * depthScale(z)) * mult;
        return {'pos.x': cur('pos.x') + delta.dx, 'pos.y': cur('pos.y') + delta.dy};
      case TransformZone.lift:
        return {'pos.z': cur('pos.z') + step.dy / railHalf * DirectSurface.zReach * mult};
      case TransformZone.rotate when index == 1:
        return {'rot.x': cur('rot.x') - step.dy * .8 * mult};
      case TransformZone.rotate when index == 2:
        return {'rot.y': cur('rot.y') + step.dx * .8 * mult};
      case TransformZone.rotate:
        final turn = _wrapped((now - pivot).direction - (last - pivot).direction);
        return {'rot.z': cur('rot.z') + turn * 180 / math.pi * mult};
      case TransformZone.scale:
        final from = _rotate(down - pivot, -angle), to = _rotate(now - pivot, -angle);
        double ratio(double a, double b) => b.abs() < 1 ? 1 : math.max(.01, a / b);
        var fx = index < 6 ? ratio(to.dx, from.dx) : 1.0;
        var fy = index < 4 || index >= 6 ? ratio(to.dy, from.dy) : 1.0;
        if (doc.link) {
          fx = fy = index < 4 ? math.max(.01, to.distance / math.max(1, from.distance)) : (index < 6 ? fx : fy);
        }
        fx = math.pow(fx, mult).toDouble();
        fy = math.pow(fy, mult).toDouble();
        return {'scale.x': value('scale.x') * fx, 'scale.y': value('scale.y') * fy, if (doc.link) 'scale.z': value('scale.z') * fx};
      case TransformZone.pivot:
        final delta = local(now) - local(last);
        double anchor(String id, double change, double half) => (cur(id) + change / (2 * half) * 100 * mult).clamp(0, 100).toDouble();
        return {'anchor.x': anchor('anchor.x', delta.dx, half.dx), 'anchor.y': anchor('anchor.y', delta.dy, half.dy)};
    }
  }
}

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
                          decoration: BoxDecoration(color: h ? Grey.g20 : Grey.g13, borderRadius: BorderRadius.circular(5)),
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
    c.drawLine(tip, tip + _rotate(back, .6) * 4, line(color));
    c.drawLine(tip, tip + _rotate(back, -.6) * 4, line(color));
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
    c.drawRect(Rect.fromCenter(center: Offset(x, top), width: 8, height: 8), line(Grey.g56));
    c.drawRect(Rect.fromCenter(center: Offset(x, bottom), width: 3, height: 3), line(Grey.g56));
    c.drawLine(Offset(x, size.height / 2), g.liftKnob, line(lift, 2));
    c.drawCircle(g.liftKnob, on ? 6 : 5, fill(lift));
  }

  void tilt(Canvas c, int axis) {
    final color = tiltInk(axis);
    final k = g.tiltKnob(axis);
    c.drawLine(g.tiltBase(axis), k, line(color, 1.5));
    final on = hot == TransformZone.rotate && hotIndex == axis;
    final r = Rect.fromCenter(center: k, width: axis == 1 ? 7 : (on ? 13 : 11), height: axis == 1 ? (on ? 13 : 11) : 7);
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(3.5)), fill(color));
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
      c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(1.5)), fill(scale));
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
