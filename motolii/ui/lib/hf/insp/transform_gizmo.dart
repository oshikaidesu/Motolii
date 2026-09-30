// The Transform gizmo: one abstract body and one pivot, touched four ways. Not a miniature Stage: no picture,
// no composition, just the body, its pivot and the handles that belong to the thing being changed.
// Gizmo for gesture. Values (below it) for precision.
import 'dart:math' as math;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import '../desk/common.dart' show kMint, kBlue, kPink, kViolet;
import 'transform_model.dart';
import '../neutral.dart';
import '../metrics.dart' show Dn, Surface;

enum TMode { move, scale, rotate, anchor }

const modeColor = {TMode.move: kMint, TMode.scale: kBlue, TMode.rotate: kPink, TMode.anchor: kViolet};
const modeLabel = {TMode.move: 'Move', TMode.scale: 'Scale', TMode.rotate: 'Rotate', TMode.anchor: 'Anchor'};

class TransformGizmo extends StatefulWidget {
  const TransformGizmo(this.store, {super.key, required this.mode, required this.onMode, this.rotAxis = 0, this.size = const Size(170, 132)});
  final TransformStore store;
  final TMode mode;
  final ValueChanged<TMode> onMode;
  final int rotAxis; // 0 Z, 1 X, 2 Y
  final Size size;
  @override
  State<TransformGizmo> createState() => _TransformGizmoState();
}

enum _Grab { none, body, axisX, axisY, axisZ, corner, edgeX, edgeY, ring }

class _TransformGizmoState extends State<TransformGizmo> {
  final focus = FocusNode(debugLabel: 'gizmo');
  _Grab grab = _Grab.none;
  int grabIndex = 0; // which corner (0..3) or which side (-1/+1)
  Offset _down = Offset.zero;
  double _lastAngle = 0, _turned = 0;
  List<double> _pos0 = const [0, 0], _scale0 = const [1, 1];
  double _rot0 = 0, _z0 = 0;
  Offset _v0 = Offset.zero;
  bool _moved = false;
  int? hoverCell;

  TransformStore get s => widget.store;
  Size get size => widget.size;
  Offset get c => Offset(size.width / 2, size.height / 2);
  bool get fine => HardwareKeyboard.instance.isShiftPressed;

  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  // ---- geometry: the body at the layer's own scale and turn, kept inside the frame -----------------------------
  Offset get half {
    final l = s.active;
    return Offset(28 * l.scale[0].abs().clamp(.35, 2.2), 20 * l.scale[1].abs().clamp(.35, 2.2));
  }

  double get rad => s.active.rotation * math.pi / 180;
  Offset rot(Offset v, double a) => Offset(v.dx * math.cos(a) - v.dy * math.sin(a), v.dx * math.sin(a) + v.dy * math.cos(a));
  Offset toScreen(Offset local) => c + rot(local, rad);
  Offset toLocal(Offset p) => rot(p - c, -rad);
  double get ringR => (half.distance + 20).clamp(56.0, math.max(56.0, math.min(76.0, size.shortestSide / 2 - 4)));

  List<Offset> get corners => [Offset(-half.dx, -half.dy), Offset(half.dx, -half.dy), Offset(half.dx, half.dy), Offset(-half.dx, half.dy)];

  static const _cells = [0.0, .5, 1.0];
  Offset anchorPoint(double fx, double fy) => toScreen(Offset((fx - .5) * 2 * half.dx, (fy - .5) * 2 * half.dy));

  // ---- picking -----------------------------------------------------------------------------------------------
  bool _inBody(Offset p) {
    final l = toLocal(p);
    return l.dx.abs() <= half.dx && l.dy.abs() <= half.dy;
  }

  void _pick(Offset p) {
    grab = _Grab.none;
    switch (widget.mode) {
      case TMode.move:
        // the axis handles are the ring's own east and south points: pull one to constrain, or turn from anywhere else on the ring
        final tipX = c + Offset(ringR, 0), tipY = c + Offset(0, ringR);
        if ((p - tipX).distance < 13) { grab = _Grab.axisX; }
        else if ((p - tipY).distance < 13) { grab = _Grab.axisY; }
        else if (s.active.projection != '2D' && (p - (c + Offset(-ringR, -ringR * .5))).distance < 13) { grab = _Grab.axisZ; }
        else {
          // Move is the working mode: the corners and edges still scale, the ring still turns, the body moves.
          // The solo modes below are for when one thing needs the whole gizmo to itself.
          final cs = corners;
          for (var i = 0; i < 4; i++) { if ((p - toScreen(cs[i])).distance < 10) { grab = _Grab.corner; grabIndex = i; } }
          if (grab == _Grab.none) {
            for (final (sx, sy) in [(-1, 0), (1, 0), (0, -1), (0, 1)]) {
              if ((p - toScreen(Offset(sx * half.dx, sy * half.dy))).distance < 9) { grab = sx != 0 ? _Grab.edgeX : _Grab.edgeY; grabIndex = sx != 0 ? sx : sy; }
            }
          }
          if (grab == _Grab.none && ((p - c).distance - ringR).abs() < 8) { grab = _Grab.ring; }
          if (grab == _Grab.none && _inBody(p)) { grab = _Grab.body; }
        }
      case TMode.scale:
        final cs = corners;
        for (var i = 0; i < 4; i++) { if ((p - toScreen(cs[i])).distance < 12) { grab = _Grab.corner; grabIndex = i; } }
        if (grab == _Grab.none) {
          for (final (sx, sy) in [(-1, 0), (1, 0), (0, -1), (0, 1)]) {
            if ((p - toScreen(Offset(sx * half.dx, sy * half.dy))).distance < 12) { grab = sx != 0 ? _Grab.edgeX : _Grab.edgeY; grabIndex = sx != 0 ? sx : sy; }
          }
        }
      case TMode.rotate:
        if (((p - c).distance - ringR).abs() < 13) { grab = _Grab.ring; }
      case TMode.anchor:
        break; // a click, handled on release
    }
    if (grab != _Grab.none) {
      _pos0 = List.of(s.active.position);
      _scale0 = List.of(s.active.scale);
      _rot0 = _rotValue;
      _z0 = s.active.position[2];
      _v0 = toLocal(p);
      _lastAngle = (p - c).direction;
      _turned = 0;
    }
  }

  double get _rotValue => switch (widget.rotAxis) { 1 => s.active.rotX, 2 => s.active.rotY, _ => s.active.rotation };
  String get _rotId => switch (widget.rotAxis) { 1 => 'rotation.x', 2 => 'rotation.y', _ => 'rotation' };

  // ---- manipulating --------------------------------------------------------------------------------------------
  void _drag(Offset p) {
    final k = fine ? .1 : 1.0;
    final d = (p - _down) * k;
    switch (grab) {
      case _Grab.body:
        s.preview('position', [_pos0[0] + d.dx, _pos0[1] + d.dy]);
      case _Grab.axisX:
        s.preview('position', [_pos0[0] + d.dx, _pos0[1]]);
      case _Grab.axisY:
        s.preview('position', [_pos0[0], _pos0[1] + d.dy]);
      case _Grab.axisZ:
        s.preview('position.z', _z0 - d.dy);
      case _Grab.corner || _Grab.edgeX || _Grab.edgeY:
        final v = toLocal(p);
        final linked = s.linked.contains('scale');
        var fx = _v0.dx.abs() < 1 ? 1.0 : v.dx / _v0.dx, fy = _v0.dy.abs() < 1 ? 1.0 : v.dy / _v0.dy;
        if (grab == _Grab.edgeX) { fy = 1; }
        if (grab == _Grab.edgeY) { fx = 1; }
        if (linked) {
          final f = grab == _Grab.edgeX ? fx : (grab == _Grab.edgeY ? fy : (v.distance / math.max(1, _v0.distance)));
          fx = f;
          fy = f;
        }
        s.preview('scale', [_scale0[0] * fx, _scale0[1] * fy]);
      case _Grab.ring:
        final a = (p - c).direction;
        var da = a - _lastAngle;
        while (da > math.pi) { da -= 2 * math.pi; }
        while (da < -math.pi) { da += 2 * math.pi; }
        _lastAngle = a; // unwrapped: turns are kept, 0 and 720 are different values
        _turned += da;
        s.preview(_rotId, _rot0 + _turned * 180 / math.pi * (fine ? .1 : 1));
      case _Grab.none:
        break;
    }
  }

  String? get _grabId => switch (grab) {
        _Grab.body || _Grab.axisX || _Grab.axisY => 'position',
        _Grab.axisZ => 'position.z',
        _Grab.corner || _Grab.edgeX || _Grab.edgeY => 'scale',
        _Grab.ring => _rotId,
        _Grab.none => null,
      };

  void _commit() {
    final id = _grabId;
    if (id != null && _moved) s.commit(id);
    grab = _Grab.none;
  }

  /// Esc during a drag: the layer is where the gesture found it, and the rest of the drag does nothing.
  void _abort() {
    final id = _grabId;
    if (id == null) return;
    s.cancel(id);
    setState(() => grab = _Grab.none);
  }

  int? _cellAt(Offset p) {
    int? best;
    var bd = 12.0;
    for (var iy = 0; iy < 3; iy++) {
      for (var ix = 0; ix < 3; ix++) {
        final d = (p - anchorPoint(_cells[ix], _cells[iy])).distance;
        if (d < bd) { bd = d; best = iy * 3 + ix; }
      }
    }
    return best;
  }

  void _hover(Offset p) {
    if (widget.mode != TMode.anchor) return;
    final cell = _cellAt(p);
    if (cell != hoverCell) {
      setState(() => hoverCell = cell);
      s.anchorPreview.value = cell == null ? null : [_cells[cell % 3], _cells[cell ~/ 3]];
    }
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: Listenable.merge([s, s.anchorPreview]),
        builder: (context, _) {
          final enabled = s.canEdit;
          return Focus(
            focusNode: focus,
            onKeyEvent: (_, e) {
              if (e is! KeyDownEvent) return KeyEventResult.ignored;
              if (e.logicalKey == LogicalKeyboardKey.escape && grab != _Grab.none) { _abort(); return KeyEventResult.handled; }
              final i = {LogicalKeyboardKey.digit1: 0, LogicalKeyboardKey.digit2: 1, LogicalKeyboardKey.digit3: 2, LogicalKeyboardKey.digit4: 3}[e.logicalKey];
              if (i != null) { widget.onMode(TMode.values[i]); return KeyEventResult.handled; }
              if (e.logicalKey == LogicalKeyboardKey.tab) { widget.onMode(TMode.values[(widget.mode.index + 1) % 4]); return KeyEventResult.handled; }
              return KeyEventResult.ignored;
            },
            child: MouseRegion(
              onHover: (e) => _hover(e.localPosition),
              onExit: (_) { if (hoverCell != null) { setState(() => hoverCell = null); } s.anchorPreview.value = null; },
              child: Listener(
                key: const ValueKey('gizmo'),
                onPointerDown: (e) {
                  focus.requestFocus();
                  _down = e.localPosition;
                  _moved = false;
                  if (enabled) _pick(e.localPosition);
                },
                onPointerMove: (e) {
                  if (grab == _Grab.none) return;
                  if ((e.localPosition - _down).distance > 2) _moved = true;
                  _drag(e.localPosition);
                },
                onPointerUp: (e) {
                  if (widget.mode == TMode.anchor && enabled && !_moved) {
                    final cell = _cellAt(e.localPosition);
                    if (cell != null) s.setAnchor(_cells[cell % 3], _cells[cell ~/ 3]);
                  }
                  _commit();
                  setState(() {});
                },
                onPointerCancel: (_) => _abort(), // an interrupted drag is let go, whatever it was holding
                child: CustomPaint(size: size, painter: _GizmoPainter(this, enabled)),
              ),
            ),
          );
        },
      );
}

class _GizmoPainter extends CustomPainter {
  _GizmoPainter(this.g, this.enabled) : super(repaint: null);
  final _TransformGizmoState g;
  final bool enabled;

  @override
  void paint(Canvas cv, Size sz) {
    final s = g.s, l = s.active, mode = g.widget.mode;
    final role = enabled ? modeColor[mode]! : N.g33;
    final c = g.c;
    cv.drawRRect(RRect.fromRectAndRadius(Offset.zero & sz, const Radius.circular(4.5)), Paint()..color = N.g07);
    // a quiet field that slides under the body as Position changes: the world moves, the body stays
    final dot = Paint()..color = N.g15;
    const step = 16.0;
    final ox = (-l.position[0]) % step, oy = (-l.position[1]) % step;
    for (var x = ox; x < sz.width; x += step) { for (var y = oy; y < sz.height; y += step) { cv.drawCircle(Offset(x, y), 1, dot); } }
    if (mode == TMode.move) {
      final p = Paint()..color = kMint.withValues(alpha: .35)..strokeWidth = 1;
      cv.drawLine(Offset(0, c.dy), Offset(sz.width, c.dy), p);
      cv.drawLine(Offset(c.dx, 0), Offset(c.dx, sz.height), p);
      // the other gestures, quietly present: scale at the corners and edges, turn on the ring
      cv.drawCircle(c, g.ringR, Paint()..color = kPink.withValues(alpha: .22)..style = PaintingStyle.stroke..strokeWidth = 1.2);
      for (final o in g.corners.map(g.toScreen)) { cv.drawCircle(o, 3.6, Paint()..color = kBlue.withValues(alpha: .9)..style = PaintingStyle.stroke..strokeWidth = 1.5); }
      for (final (sx, sy) in [(-1, 0), (1, 0), (0, -1), (0, 1)]) { cv.drawCircle(g.toScreen(Offset(sx * g.half.dx, sy * g.half.dy)), 2.6, Paint()..color = kBlue.withValues(alpha: .7)); }
    }
    // body: flat, at the layer's own scale and turn
    final cs = g.corners.map(g.toScreen).toList();
    final body = Path()..addPolygon(cs, true);
    cv.drawPath(body, Paint()..color = enabled ? N.g20 : N.g15);
    cv.drawPath(body, Paint()..color = role.withValues(alpha: .9)..style = PaintingStyle.stroke..strokeWidth = 1.6..strokeJoin = StrokeJoin.round);
    // a notch on the top edge so a turn can be read
    final notch = Path()..moveTo(g.toScreen(Offset(-6, -g.half.dy)).dx, g.toScreen(Offset(-6, -g.half.dy)).dy)..lineTo(g.toScreen(Offset(0, -g.half.dy - 6)).dx, g.toScreen(Offset(0, -g.half.dy - 6)).dy)..lineTo(g.toScreen(Offset(6, -g.half.dy)).dx, g.toScreen(Offset(6, -g.half.dy)).dy);
    cv.drawPath(notch, Paint()..color = role..style = PaintingStyle.stroke..strokeWidth = 1.6..strokeCap = StrokeCap.round);
    // the pivot, always
    final pv = g.anchorPoint(l.anchor[0], l.anchor[1]);
    cv.drawCircle(pv, mode == TMode.anchor ? 5.5 : 4, Paint()..color = enabled ? kViolet : N.g33);
    cv.drawCircle(pv, mode == TMode.anchor ? 5.5 : 4, Paint()..color = N.g07..style = PaintingStyle.stroke..strokeWidth = 1.4);

    void handle(Offset o, {double r = 5.5, Color? col}) {
      cv.drawCircle(o, r + 3, Paint()..color = (col ?? role).withValues(alpha: .22));
      cv.drawCircle(o, r, Paint()..color = col ?? role);
      cv.drawCircle(o, r, Paint()..color = N.g07..style = PaintingStyle.stroke..strokeWidth = 1.4);
    }

    switch (mode) {
      case TMode.move:
        void arrow(Offset to, String t) {
          cv.drawLine(c, to, Paint()..color = role..strokeWidth = 2);
          handle(to, r: 5);
          final tp = TextPainter(text: TextSpan(text: t, style: sans(Dn.microSize, c: N.g07, w: FontWeight.w800)), textDirection: TextDirection.ltr)..layout();
          tp.paint(cv, to - Offset(tp.width / 2, tp.height / 2));
        }
        arrow(c + Offset(g.ringR, 0), 'X');
        arrow(c + Offset(0, g.ringR), 'Y');
        if (l.projection != '2D') arrow(c + Offset(-g.ringR, -g.ringR * .5), 'Z');
      case TMode.scale:
        for (final o in cs) { handle(o); }
        for (final (sx, sy) in [(-1, 0), (1, 0), (0, -1), (0, 1)]) { handle(g.toScreen(Offset(sx * g.half.dx, sy * g.half.dy)), r: 4, col: Surface.ink); }
        if (s.linked.contains('scale')) {
          final tp = TextPainter(text: TextSpan(text: 'LINKED', style: sans(Dn.microSize, c: role, w: FontWeight.w700, ls: 1)), textDirection: TextDirection.ltr)..layout();
          tp.paint(cv, Offset(sz.width - tp.width - 8, sz.height - tp.height - 6));
        }
      case TMode.rotate:
        final r = g.ringR;
        cv.drawCircle(c, r, Paint()..color = role.withValues(alpha: .35)..style = PaintingStyle.stroke..strokeWidth = 1.4);
        final a = g.rad - math.pi / 2;
        final tip = c + Offset(math.cos(a), math.sin(a)) * r;
        cv.drawLine(c, tip, Paint()..color = role.withValues(alpha: .6)..strokeWidth = 1.2);
        handle(tip, r: 6);
        if (g.widget.rotAxis != 0) {
          final tp = TextPainter(text: TextSpan(text: g.widget.rotAxis == 1 ? 'X AXIS' : 'Y AXIS', style: sans(Dn.microSize, c: role, w: FontWeight.w700, ls: 1)), textDirection: TextDirection.ltr)..layout();
          tp.paint(cv, const Offset(8, 8));
        }
      case TMode.anchor:
        for (var iy = 0; iy < 3; iy++) {
          for (var ix = 0; ix < 3; ix++) {
            final fx = _TransformGizmoState._cells[ix], fy = _TransformGizmoState._cells[iy];
            final o = g.anchorPoint(fx, fy);
            final cur = (l.anchor[0] - fx).abs() < .01 && (l.anchor[1] - fy).abs() < .01;
            final hov = g.hoverCell == iy * 3 + ix;
            cv.drawCircle(o, cur ? 6.5 : (hov ? 6 : 4), Paint()..color = cur ? kViolet : (hov ? kViolet.withValues(alpha: .7) : N.g51));
            if (cur) cv.drawCircle(o, 6.5, Paint()..color = Surface.ink..style = PaintingStyle.stroke..strokeWidth = 1.5);
          }
        }
    }
    // the value that is changing, quietly, in the corner
    final t = switch (mode) {
      TMode.move => 'X ${l.position[0].toStringAsFixed(0)}  Y ${l.position[1].toStringAsFixed(0)}',
      TMode.scale => '${(l.scale[0] * 100).round()}% × ${(l.scale[1] * 100).round()}%',
      TMode.rotate => '${(g.widget.rotAxis == 1 ? l.rotX : (g.widget.rotAxis == 2 ? l.rotY : l.rotation)).toStringAsFixed(1)}°',
      TMode.anchor => '${l.anchor[0]}, ${l.anchor[1]}',
    };
    final tp = TextPainter(text: TextSpan(text: mode == TMode.anchor ? '' : t, style: mono(Dn.microSize, c: N.g63)), textDirection: TextDirection.ltr)..layout();
    tp.paint(cv, const Offset(8, 8));
  }

  @override
  bool shouldRepaint(_GizmoPainter o) => true;
}
