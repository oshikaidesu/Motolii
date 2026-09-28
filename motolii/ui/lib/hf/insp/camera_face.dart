// The Camera face: the view ray and nothing else. A target point, the eye on a sphere around it, the ray between them,
// and the twist about that ray. No frustum, no scene, no other layers: Depth answers where everything is;
// this answers how the camera stands to its target.
import 'dart:math' as math;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import '../desk/common.dart' show kMint, kBlue, kViolet, kPink;
import 'camera_model.dart';
import '../neutral.dart';

const targetColor = kMint, orbitColor = kViolet, distanceColor = kBlue, rollColor = kPink;

/// Where things are, from the store's numbers. Public so tests reach the same handles a pointer would.
class CameraGeom {
  CameraGeom(this.size, this.s);
  final Size size;
  final CameraStore s;
  static const orbitR = 54.0, ringR = 70.0;

  Offset get c => Offset(size.width / 2, size.height / 2);

  /// An oblique view of the unit sphere, so a camera straight in front of the target still stands apart from it.
  Offset get eye {
    final e = s.eyeDirection;
    // the one direction that lands on the target itself is behind it, where a camera rarely stands
    var v = Offset(e[0] - .62 * e[2], -e[1] + .46 * e[2]);
    // keep the camera clear of the target dot: a direction that would land near it is pushed out along the same line
    if (v.distance < .6) v = v.distance < 1e-3 ? const Offset(.6, -.0) : v / v.distance * .6;
    return c + v * (orbitR * .95);
  }

  bool get behind => s.eyeDirection[2] > 0; // the eye is on the far side of the target
  Offset get rayDir { final d = eye - c; return d.distance < 1e-3 ? const Offset(-.8, .6) : d / d.distance; }

  /// The distance handle rides the ray: nearer the target for a close camera, near the eye for a far one.
  Offset get distanceHandle {
    final t = (.5 + .28 * (math.log(s.distance) / math.ln10)).clamp(.22, .9);
    final d = eye - c;
    final base = d.distance < 1e-3 ? rayDir * orbitR * .9 : d;
    return c + base * t;
  }

  Offset get rollHandle {
    final a = (-90 + s.roll) * math.pi / 180;
    return c + Offset(math.cos(a), math.sin(a)) * ringR;
  }
}

enum _G { none, target, eye, distance, roll }

class CameraFace extends StatefulWidget {
  const CameraFace(this.store, {super.key, this.size = const Size(286, 176)});
  final CameraStore store;
  final Size size;
  @override
  State<CameraFace> createState() => _CameraFaceState();
}

class _CameraFaceState extends State<CameraFace> {
  _G grab = _G.none;
  Offset _down = Offset.zero;
  bool _moved = false;
  double _lastAngle = 0, _turned = 0, _roll0 = 0, _dist0 = 1;
  List<double> _orbit0 = const [0, 0], _center0 = const [0, 0];
  Offset _ray0 = const Offset(1, 0);
  CameraStore get s => widget.store;
  CameraGeom get g => CameraGeom(widget.size, s);
  bool get fine => HardwareKeyboard.instance.isShiftPressed;

  void _pick(Offset p) {
    grab = _G.none;
    final geo = g;
    if ((p - geo.c).distance < 9 && !s.targetLocked) { grab = _G.target; }
    else if ((p - geo.distanceHandle).distance < 10) { grab = _G.distance; }
    else if ((p - geo.eye).distance < 12) { grab = _G.eye; }
    else if (((p - geo.c).distance - CameraGeom.ringR).abs() < 9) { grab = _G.roll; }
    if (grab != _G.none) {
      _orbit0 = [s.pitch, s.yaw];
      _center0 = [s.cx, s.cy];
      _roll0 = s.roll;
      _dist0 = s.distance;
      _ray0 = geo.rayDir;
      _lastAngle = (p - geo.c).direction;
      _turned = 0;
    }
  }

  void _drag(Offset p) {
    final k = fine ? .1 : 1.0;
    final d = (p - _down) * k;
    switch (grab) {
      case _G.target:
        s.preview('camera.center', [_center0[0] + d.dx * 4, _center0[1] + d.dy * 4]);
      case _G.eye:
        // the camera follows the pointer around the target: pitch from the vertical, yaw from the horizontal
        s.preview('camera.orbit', [_orbit0[0] - d.dy * .8, _orbit0[1] - d.dx * .8]);
      case _G.distance:
        final along = d.dx * _ray0.dx + d.dy * _ray0.dy; // outward along the ray is farther
        s.preview('camera.distance', _dist0 * math.exp(along * .02));
      case _G.roll:
        final a = (p - g.c).direction;
        var da = a - _lastAngle;
        while (da > math.pi) { da -= 2 * math.pi; }
        while (da < -math.pi) { da += 2 * math.pi; }
        _lastAngle = a; // unwrapped, so turns are kept
        _turned += da;
        s.preview('camera.roll', _roll0 + _turned * 180 / math.pi * (fine ? .1 : 1));
      case _G.none:
        break;
    }
  }

  void _commit() {
    if (grab != _G.none && _moved) {
      s.commit(switch (grab) { _G.target => 'camera.center', _G.eye => 'camera.orbit', _G.distance => 'camera.distance', _ => 'camera.roll' });
    }
    grab = _G.none;
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: s,
        builder: (context, _) => Listener(
          key: const ValueKey('camera-face'),
          onPointerDown: (e) { _down = e.localPosition; _moved = false; if (!s.frozen) _pick(e.localPosition); },
          onPointerMove: (e) { if (grab == _G.none) return; if ((e.localPosition - _down).distance > 2) _moved = true; _drag(e.localPosition); },
          onPointerUp: (_) { _commit(); setState(() {}); },
          onPointerCancel: (_) => grab = _G.none,
          child: CustomPaint(size: widget.size, painter: _Painter(g, s, grab)),
        ),
      );
}

class _Painter extends CustomPainter {
  _Painter(this.g, this.s, this.grab);
  final CameraGeom g;
  final CameraStore s;
  final _G grab;

  @override
  void paint(Canvas c, Size sz) {
    final off = s.frozen;
    final a = off ? .45 : 1.0;
    final ctr = g.c;
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & sz, const Radius.circular(6)), Paint()..color = N.g07);
    // the roll ring: a twist about the ray, turns kept
    c.drawCircle(ctr, CameraGeom.ringR, Paint()..color = rollColor.withValues(alpha: .28 * a)..style = PaintingStyle.stroke..strokeWidth = 1.3);
    for (var i = 0; i < 12; i++) {
      final t = i * math.pi / 6;
      c.drawLine(ctr + Offset(math.cos(t), math.sin(t)) * (CameraGeom.ringR - 3), ctr + Offset(math.cos(t), math.sin(t)) * (CameraGeom.ringR + 3), Paint()..color = rollColor.withValues(alpha: .25 * a)..strokeWidth = 1);
    }
    // the sphere the eye rides on
    c.drawCircle(ctr, CameraGeom.orbitR, Paint()..color = orbitColor.withValues(alpha: .10 * a));
    c.drawCircle(ctr, CameraGeom.orbitR, Paint()..color = orbitColor.withValues(alpha: .45 * a)..style = PaintingStyle.stroke..strokeWidth = 1.2);
    c.drawOval(Rect.fromCenter(center: ctr, width: CameraGeom.orbitR * 2, height: CameraGeom.orbitR * .7), Paint()..color = orbitColor.withValues(alpha: .28 * a)..style = PaintingStyle.stroke..strokeWidth = 1);
    final eye = g.eye;
    // the view ray, from the eye to the target
    c.drawLine(eye, ctr, Paint()..color = distanceColor.withValues(alpha: .9 * a)..strokeWidth = 2..strokeCap = StrokeCap.round);
    // distance handle
    final dh = g.distanceHandle;
    c.save();
    c.translate(dh.dx, dh.dy);
    c.rotate(g.rayDir.direction + math.pi / 2);
    final bar = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 16, height: 6), const Radius.circular(3));
    c.drawRRect(bar, Paint()..color = distanceColor.withValues(alpha: a));
    c.drawRRect(bar, Paint()..color = N.g07..style = PaintingStyle.stroke..strokeWidth = 1.2);
    c.restore();
    // the camera: a body with its lens toward the target; hollow when it is on the far side
    c.save();
    c.translate(eye.dx, eye.dy);
    c.rotate((ctr - eye).direction);
    final body = RRect.fromRectAndRadius(const Rect.fromLTRB(-9, -6, 5, 6), const Radius.circular(2.5));
    final lens = Path()..moveTo(5, -3.5)..lineTo(11, -6.5)..lineTo(11, 6.5)..lineTo(5, 3.5)..close();
    final fill = g.behind ? N.g07 : orbitColor.withValues(alpha: a);
    c.drawRRect(body, Paint()..color = fill);
    c.drawPath(lens, Paint()..color = fill);
    c.drawRRect(body, Paint()..color = orbitColor.withValues(alpha: a)..style = PaintingStyle.stroke..strokeWidth = 1.5);
    c.drawPath(lens, Paint()..color = orbitColor.withValues(alpha: a)..style = PaintingStyle.stroke..strokeWidth = 1.5);
    c.restore();
    // the target: a point; hollow with a link mark when a layer decides it
    final locked = s.targetLocked;
    c.drawCircle(ctr, 8, Paint()..color = targetColor.withValues(alpha: .18 * a));
    c.drawCircle(ctr, 5, Paint()..color = locked ? N.g07 : targetColor.withValues(alpha: a));
    c.drawCircle(ctr, 5, Paint()..color = targetColor.withValues(alpha: a)..style = PaintingStyle.stroke..strokeWidth = 1.6);
    final cross = Paint()..color = targetColor.withValues(alpha: .8 * a)..strokeWidth = 1.2;
    c.drawLine(ctr + const Offset(-11, 0), ctr + const Offset(-7, 0), cross);
    c.drawLine(ctr + const Offset(7, 0), ctr + const Offset(11, 0), cross);
    c.drawLine(ctr + const Offset(0, -11), ctr + const Offset(0, -7), cross);
    c.drawLine(ctr + const Offset(0, 7), ctr + const Offset(0, 11), cross);
    // roll handle at the ring
    final rh = g.rollHandle;
    c.drawCircle(rh, 7, Paint()..color = rollColor.withValues(alpha: .2 * a));
    c.drawCircle(rh, 4.6, Paint()..color = rollColor.withValues(alpha: a));
    c.drawCircle(rh, 4.6, Paint()..color = N.g07..style = PaintingStyle.stroke..strokeWidth = 1.3);
    final t = 'pitch ${s.pitch.toStringAsFixed(0)}°  yaw ${s.yaw.toStringAsFixed(0)}°   ×${s.distance.toStringAsFixed(2)}   roll ${s.roll.toStringAsFixed(0)}°';
    final tp = TextPainter(text: TextSpan(text: t, style: mono(8.5, c: N.g51)), textDirection: TextDirection.ltr, maxLines: 1, ellipsis: '…')..layout(maxWidth: sz.width - 16);
    tp.paint(c, const Offset(8, 7));
    if (locked) {
      final lp = TextPainter(text: TextSpan(text: 'TARGET LAYER', style: sans(8, c: targetColor, w: FontWeight.w700, ls: 1)), textDirection: TextDirection.ltr)..layout();
      lp.paint(c, Offset(sz.width - lp.width - 8, sz.height - lp.height - 6));
    }
  }

  @override
  bool shouldRepaint(_Painter o) => true;
}
