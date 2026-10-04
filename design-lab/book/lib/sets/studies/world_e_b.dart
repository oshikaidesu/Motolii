// World E, part B: 28 Time remap, 29 Loop, 30 Camera (research/op1-translation.md section 3.3).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_e_kit.dart';

// ---- 28. Time remap: a loom of threads between two strips -----------------------------------------------------------------------------
// The lower strip is the time you see, the upper strip the time of the footage; a thread joins each moment to the footage moment it shows.
// A blue = Speed (the field of threads: they fan out, or cross when it runs backwards), B green = Offset (the upper strip slides),
// C white = Hold (the pair of ticks on the lower strip: the threads between them knot into one point), D orange = Smooth (the beads at the middle of the threads).

const remapSpecs = [
  WSpec('speed', 'Speed', -400, 400, 100, unit: '%', digits: 0),
  WSpec('off', 'Offset', -10, 10, 0, unit: 's', digits: 1),
  WSpec('hold', 'Hold', 0, 100, 0, unit: '%', digits: 0),
  WSpec('smooth', 'Smooth', 0, 100, 0, unit: '%', digits: 0),
];

WorldDef remapDef() => WorldDef(
      name: 'Time remap',
      specs: remapSpecs,
      make: RemapWorld.new,
      silhouette: 'a loom of threads',
      rowH: 108,
      above: ('Opacity', '100 %'),
      below: ('Rotation', '0°'),
    );

class RemapWorld extends WeWorld {
  RemapWorld(super.doc);
  double _ph = .1;
  final _trail = <double>[];

  double _bh(Rect a) => weClamp(a.height * .13, 6, 12);
  Rect _top(Rect a) => Rect.fromLTWH(a.left, a.top + 1, a.width, _bh(a));
  Rect _bot(Rect a) => Rect.fromLTWH(a.left, a.bottom - 1 - _bh(a), a.width, _bh(a));

  double _raw(double x, double hw, double sp, double off) {
    final xp = x < .5 - hw ? x : (x > .5 + hw ? x - 2 * hw : .5 - hw);
    return .5 + off / 10 + sp * (xp - (1 - 2 * hw) / 2);
  }

  /// The footage moment (0..1 of the upper strip, may leave it) shown at output moment [x] (0..1).
  double src(double x) {
    final hw = v(2) / 100 * .22, sp = v(0) / 100, off = v(1), kw = v(3) / 100 * .25;
    if (kw < 1e-4) return _raw(x, hw, sp, off);
    var sum = 0.0;
    for (var i = 0; i < 9; i++) {
      sum += _raw(x + kw * (i / 4 - 1), hw, sp, off);
    }
    return sum / 9;
  }

  @override
  void step(double dt, Size s) {
    _ph += dt / 5;
    if (_ph >= 1) {
      _ph -= 1;
      _trail.clear();
    }
    _trail.add(weClamp(src(_ph), 0, 1));
    if (_trail.length > 50) _trail.removeAt(0);
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final a = art(s), top = _top(a), bot = _bot(a), hwPx = v(2) / 100 * .22 * a.width;
    if (p.dy < top.bottom + 4 && p.dy > a.top - 6) return 1;
    if (p.dy > bot.top - 10 && (p.dx - a.center.dx).abs() <= hwPx + 16) return 2;
    if (p.dy > top.bottom && p.dy < bot.top) {
      final mid = (top.bottom + bot.top) / 2;
      return (p.dy - mid).abs() <= 9 ? 3 : 0;
    }
    return null;
  }

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0, a = art(s);
    switch (z) {
      case 0:
        put(0, base(0) + d.dx * k * 800 / a.width);
      case 1:
        put(1, base(1) + d.dx * k * 10 / a.width);
      case 2:
        put(2, base(2) + (p.dx > a.center.dx ? 1 : -1) * d.dx * k * 100 / (.22 * a.width));
      case 3:
        put(3, base(3) + d.dx * k * 100 / (a.width * .4));
    }
  }

  @override
  void paint(Canvas c, Size s) {
    final a = art(s), top = _top(a), bot = _bot(a), w = a.width, mid = (top.bottom + bot.top) / 2;
    final hw = v(2) / 100 * .22;
    // the two strips: film, a row of frames each
    const cells = 20;
    final cw = w / cells;
    for (var i = 0; i < cells; i++) {
      final x = a.left + i * cw + 1;
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(x, top.top, cw - 2, top.height), const Radius.circular(1.5)), weLine(slot(1, .5), on(1) ? 1.5 : 1));
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(x, bot.top, cw - 2, bot.height), const Radius.circular(1.5)), weLine(N.g26));
    }
    // the hold: two ticks on the lower strip
    for (final sd in [-1, 1]) {
      final x = a.center.dx + sd * hw * w;
      c.drawLine(Offset(x, bot.top - 3), Offset(x, bot.bottom + 3), weLine(slot(2, 1), on(2) ? 1.5 : 1));
    }
    // the threads: slack, so they hang and sway a little
    for (var i = 0; i < 9; i++) {
      final xo = i / 8, sr = src(xo), xs = a.left + xo * w, xe = a.left + weClamp(sr, 0, 1) * w;
      final inside = sr >= 0 && sr <= 1, sway = math.sin(doc.t * 1.1 + i * .7) * 1.8;
      final th = Path()..moveTo(xs, bot.top)..quadraticBezierTo((xs + xe) / 2 + sway, mid, xe, top.bottom);
      c.drawPath(th, weLine(slot(0, inside ? .6 : .2), on(0) ? 1.5 : 1));
      c.drawCircle(Offset((xs + xe) / 2 + sway / 2, mid), on(3) ? 2.2 : 1.4, weFill(slot(3, inside ? .9 : .4)));
    }
    if (on(3)) c.drawLine(Offset(a.left, mid), Offset(a.right, mid), weLine(slot(3).withValues(alpha: .35)));
    // the playing moment: a dot on the lower strip, and the footage moment it shows
    for (var i = 0; i < _trail.length; i++) {
      c.drawCircle(Offset(a.left + _trail[i] * w, top.center.dy), .8, weFill(N.g56.withValues(alpha: i / _trail.length * .6)));
    }
    final sr = weClamp(src(_ph), 0, 1), p0 = Offset(a.left + _ph * w, bot.center.dy), p1 = Offset(a.left + sr * w, top.center.dy);
    c.drawLine(p0, p1, weLine(N.g95.withValues(alpha: .6)));
    c.drawCircle(p0, 2.4, weFill(N.g95));
    c.drawCircle(p1, 2.4, weFill(N.g95));
  }
}

// ---- 29. Loop: a racetrack with start and finish lines ---------------------------------------------------------------------------------
// A dot laps a track between two gates. A blue = Count (ripple rings outside the track, one per lap, each lights as its lap ends; unlimited fades out),
// B green = Type (the track itself: slide along it to turn it from lap, to there-and-back, to a coil that drifts), C white = Start (the white gate), D orange = End (the orange gate).

const loopSpecs = [
  WSpec('count', 'Count', 1, 13, 13, digits: 0, integer: true, range: '1-∞', show: _countText),
  WSpec('type', 'Type', 0, 2, 0, digits: 0, drive: false, range: 'cycle | ping-pong | offset', show: _typeText),
  WSpec('start', 'Start', 0, 100, 0, unit: '%', digits: 0),
  WSpec('end', 'End', 0, 100, 100, unit: '%', digits: 0),
];

String _countText(double v) => v >= 13 ? '∞' : v.round().toString();
String _typeText(double v) => const ['↻', '⇄', '↗'][v.round().clamp(0, 2)];

WorldDef loopDef() => WorldDef(
      name: 'Loop',
      specs: loopSpecs,
      make: LoopWorld.new,
      silhouette: 'a racetrack with start and finish lines',
      rowH: 120,
      above: ('Position', '960, 540'),
      below: ('Scale', '100 %'),
    );

class LoopWorld extends WeWorld {
  LoopWorld(super.doc);
  double _tau = 0, _dwell = 0, _acc = 0;
  int _lap = 0;
  final _lit = List<double>.filled(12, 0);
  final _trail = <Offset>[];

  Offset _cen(Rect a) => a.center;
  double _rx(Rect a) => math.min(a.width * .37, a.height * 1.6);
  double _ry(Rect a) => a.height * .31;
  double _oT() => weClamp(v(1) - 1, 0, 1);
  double _pp() => weClamp(1 - (v(1) - 1).abs(), 0, 1);
  double _drift(Rect a) => _rx(a) * .3;
  double _shift(Rect a, int n) => (n - 2.0) * _drift(a) * _oT();
  int get _count => v(0).round();
  double get _total => math.min(_count >= 13 ? 1e9 : _count.toDouble(), _oT() > 0 ? 5.0 : 1e9);

  double _ang(double u) => -math.pi / 2 + 2 * math.pi * (v(2) / 100 + (v(3) / 100 - v(2) / 100) * u);

  Offset _on(Rect a, double ang, double sc, int lap) => Offset(_cen(a).dx + _shift(a, lap) + math.cos(ang) * _rx(a) * sc, _cen(a).dy + math.sin(ang) * _ry(a) * sc);

  Offset _dot(Rect a, double tau) {
    final n = tau.floor(), f = tau - n;
    final pp = n.isEven ? f : 1 - f, u = weLerp(f, pp, _pp());
    return _on(a, _ang(u), 1, n);
  }

  // the infield: a short row of lap frames, one per lap (A)
  static const _pip = 8.0;
  Rect _field(Rect a) => Rect.fromCenter(center: _cen(a), width: 12 * _pip, height: 10);

  @override
  void step(double dt, Size s) {
    final a = art(s);
    if (_tau >= _total) {
      _dwell += dt;
      if (_dwell > .9) {
        _tau = 0;
        _dwell = 0;
        _lap = 0;
        _trail.clear();
      }
    } else {
      _tau = math.min(_tau + dt / 1.9, _total);
      if (_tau.floor() > _lap) {
        _lit[(_tau.floor() - 1) % 12] = 1;
        _lap = _tau.floor();
      }
    }
    for (var i = 0; i < 12; i++) {
      _lit[i] = math.max(0, _lit[i] - dt * 1.6);
    }
    _trail.add(_dot(a, _tau));
    if (_trail.length > 16) _trail.removeAt(0);
  }

  Offset _gateA(Rect a) => _on(a, _ang(0), 1, 0);
  Offset _gateB(Rect a) => _on(a, _ang(1), 1, 0);

  double _pang(Rect a, Offset p) => math.atan2((p.dy - _cen(a).dy) / _ry(a), (p.dx - _cen(a).dx) / _rx(a));

  @override
  int? zoneAt(Offset p, Size s) {
    final a = art(s);
    double best = 1e9;
    int? z;
    void cand(int zz, double d, double reach, [double bias = 0]) {
      if (d <= reach && d - bias < best) {
        best = d - bias;
        z = zz;
      }
    }

    var dArc = 1e9;
    for (var i = 0; i <= 40; i++) {
      dArc = math.min(dArc, (p - _on(a, _ang(i / 40), 1, 0)).distance);
    }
    cand(2, (p - _gateA(a)).distance, 16, 6);
    cand(3, (p - _gateB(a)).distance, 16, 6);
    cand(1, dArc, 12);
    if (z == null && _field(a).inflate(12).contains(p)) z = 0;
    return z;
  }

  @override
  void down(int z, Offset p, Size s) => _acc = z == 0 ? base(0) : 0;

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0, a = art(s);
    switch (z) {
      case 0:
        _acc = weClamp(_acc + d.dx * k / _pip, 1, 13);
        put(0, _acc);
      case 1:
        put(1, base(1) + d.dx * k * 2 / (2 * _rx(a)));
      case 2 || 3:
        final dth = weWrap(_pang(a, p) - _pang(a, p - d));
        put(z, base(z) + dth / (2 * math.pi) * 100 * k);
    }
  }

  @override
  void up(int z) {
    if (z == 1) put(1, base(1).roundToDouble());
  }

  /// Film frames along the track between track-fractions [u0] and [u1] (of the looped part when [looped]), drawn as 1 px outlines.
  void _frames(Canvas c, Rect a, int lap, double u0, double u1, bool looped, Paint paint) {
    final rx = _rx(a), ry = _ry(a);
    final per = math.pi * (3 * (rx + ry) - math.sqrt((3 * rx + ry) * (rx + 3 * ry)));
    final span = looped ? ((v(3) - v(2)).abs() / 100) : 1.0;
    final n = math.max(2, (per * span * (u1 - u0) / 9).round());
    for (var i = 0; i <= n; i++) {
      final u = weLerp(u0, u1, i / n), ang = looped ? _ang(u) : -math.pi / 2 + 2 * math.pi * u;
      final o = _on(a, ang, 1, lap);
      c.save();
      c.translate(o.dx, o.dy);
      c.rotate(math.atan2(ry * math.cos(ang), -rx * math.sin(ang)));
      c.drawRect(const Rect.fromLTWH(-3, -2.5, 6, 5), paint);
      c.restore();
    }
  }

  @override
  void paint(Canvas c, Size s) {
    final a = art(s), cen = _cen(a);
    // the whole track in quiet film frames, and the part that is looped (B)
    _frames(c, a, 0, 0, 1, false, weLine(N.g20));
    final laps = _oT() > 0 ? math.min(5, math.max(1, _count)) : 1;
    for (var n = laps - 1; n >= 0; n--) {
      _frames(c, a, n, 0, 1, true, weLine(slot(1, n == 0 ? .6 : .22), on(1) && n == 0 ? 1.4 : 1));
    }
    // the infield: one frame per lap (A); unlimited fades out to the right
    final cnt = _count, inf = cnt >= 13, f = _field(a);
    for (var k = 0; k < (inf ? 12 : math.min(cnt, 12)); k++) {
      final r = Rect.fromLTWH(f.left + k * _pip + 1, cen.dy - 3.5, 5, 7), al = (inf ? .55 * (1 - k / 14) : .6) + _lit[k] * .4;
      c.drawRect(r, weLine(slot(0, weClamp(al, 0, 1))));
      if (_lit[k] > .05) c.drawRect(r.deflate(1.5), weFill(slot(0, weClamp(_lit[k] * .6, 0, 1))));
    }
    if (on(0)) c.drawRRect(RRect.fromRectAndRadius(f.inflate(5), const Radius.circular(3)), weLine(slot(0).withValues(alpha: .5)));
    // the gates: start inside the track (white), end outside it (orange)
    for (final g in [(2, 0.0, .78, 1.0), (3, 1.0, 1.0, 1.22)]) {
      final ang = _ang(g.$2), p1 = _on(a, ang, g.$3, 0), p2 = _on(a, ang, g.$4, 0);
      c.drawLine(p1, p2, weLine(slot(g.$1, 1), on(g.$1) ? 1.8 : 1.3));
      ring(c, (p1 + p2) / 2, 12, g.$1);
    }
    // the runner and its short trail
    for (var i = 1; i < _trail.length; i++) {
      if ((_trail[i] - _trail[i - 1]).distance < _rx(a)) c.drawLine(_trail[i - 1], _trail[i], weLine(N.g56.withValues(alpha: i / _trail.length * .6)));
    }
    c.drawCircle(_dot(a, _tau), 2.8, weFill(N.g95));
  }
}

// ---- 30. Camera: a camera on a ring round its subject, and the picture it takes -----------------------------------------------------------
// Seen from above: a subject, a camera at some distance, the cone of what it sees, and a little picture frame in the corner showing the shot.
// A blue = Zoom (the cone: a long lens narrows it), B green = Distance (the ring round the subject that the camera sits on), C white = Yaw (the camera body turns),
// D orange = Roll (the picture frame in the corner: the horizon in it tilts).

const cameraSpecs = [
  WSpec('zoom', 'Zoom', 10, 300, 50, unit: 'mm', digits: 0),
  WSpec('dist', 'Distance', 200, 1800, 1000, unit: 'px', digits: 0),
  WSpec('yaw', 'Yaw', -180, 180, 0, unit: '°', digits: 0),
  WSpec('roll', 'Roll', -180, 180, 0, unit: '°', digits: 0),
];

WorldDef cameraDef() => WorldDef(
      name: 'Camera',
      specs: cameraSpecs,
      make: CameraWorld.new,
      silhouette: 'a cone seen from above',
      rowH: 120,
      above: ('Position', '960, 540'),
      below: ('Depth of field', '0 px'),
    );

class CameraWorld extends WeWorld {
  CameraWorld(super.doc);

  Offset _subj(Rect a) => Offset(a.left + a.width * .62, a.center.dy);
  double _dpx(Rect a) => weLerp(26, _subj(a).dx - a.left - 14, (v(1) - 200) / 1600);
  Offset _cam(Rect a) => _subj(a) - Offset(_dpx(a), 0);
  double _yaw() => v(2) * math.pi / 180 + math.sin(doc.t * 1.1) * .02;
  double _half() => math.atan(18 / v(0));
  Rect _frame(Rect a) {
    final iw = weClamp(a.width * .2, 36, 56);
    return Rect.fromLTWH(a.right - iw - 1, a.top + 1, iw, iw * 9 / 16);
  }

  double _len(Rect a) => _dpx(a) + a.width * .14;
  double _angTo(Offset from, Offset to) => math.atan2(to.dy - from.dy, to.dx - from.dx);

  @override
  int? zoneAt(Offset p, Size s) {
    final a = art(s), cam = _cam(a), sj = _subj(a), yaw = _yaw(), h = _half(), L = _len(a);
    if (_frame(a).inflate(8).contains(p)) return 3;
    double best = 1e9;
    int? z;
    void cand(int zz, double d, double reach, [double bias = 0]) {
      if (d <= reach && d - bias < best) {
        best = d - bias;
        z = zz;
      }
    }

    cand(2, (p - cam).distance, 15, 6);
    if ((p - cam).distance > 15) {
      for (final sd in [-1, 1]) {
        cand(0, weDistSeg(p, cam, cam + Offset(math.cos(yaw + sd * h), math.sin(yaw + sd * h)) * L), 10);
      }
    }
    cand(1, ((p - sj).distance - _dpx(a)).abs(), 12);
    cand(1, (p - sj).distance, 15, 8);
    return z;
  }

  bool _sub = false;

  @override
  void down(int z, Offset p, Size s) {
    final a = art(s), sj = _subj(a), dd = (p - sj).distance;
    _sub = z == 1 && dd <= 15 && (dd - _dpx(a)).abs() > 12;
  }

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0, a = art(s), cam = _cam(a), sj = _subj(a);
    final q = p - d;
    if (z == 1 && _sub) {
      // the subject itself: pull it away from the camera to lengthen the distance
      put(1, base(1) + d.dx * k * 1600 / (sj.dx - a.left - 40));
      return;
    }
    switch (z) {
      case 0:
        final side = weWrap(_angTo(cam, p) - _yaw()) >= 0 ? 1 : -1, dth = weWrap(_angTo(cam, p) - _angTo(cam, q));
        final th = weClamp(math.atan(18 / base(0)) + side * dth * k, .035, 1.1);
        put(0, 18 / math.tan(th));
      case 1:
        put(1, base(1) + ((p - sj).distance - (q - sj).distance) * k * 1600 / (sj.dx - a.left - 40));
      case 2:
        put(2, base(2) + weWrap(_angTo(cam, p) - _angTo(cam, q)) * 180 / math.pi * k);
      case 3:
        final fc = _frame(a).center;
        put(3, base(3) + weWrap(_angTo(fc, p) - _angTo(fc, q)) * 180 / math.pi * k);
    }
  }

  @override
  void paint(Canvas c, Size s) {
    final a = art(s), sj = _subj(a), cam = _cam(a), dpx = _dpx(a), yaw = _yaw(), h = _half(), L = _len(a);
    c.save();
    c.clipRect(a.inflate(6));
    // the ring the camera sits on (B): faint all round, lit near the camera
    c.drawArc(Rect.fromCircle(center: sj, radius: dpx), math.pi - 1.25, 2.5, false, weLine(N.g20));
    c.drawArc(Rect.fromCircle(center: sj, radius: dpx), math.pi - .7, 1.4, false, weLine(slot(1, .9), on(1) ? 1.5 : 1));
    // the cone (A) and its axis
    for (final sd in [-1, 1]) {
      c.drawLine(cam, cam + Offset(math.cos(yaw + sd * h), math.sin(yaw + sd * h)) * L, weLine(slot(0, .9), on(0) ? 1.5 : 1));
    }
    weDash(c, cam, cam + Offset(math.cos(yaw), math.sin(yaw)) * L, weLine(N.g26), on: 2, off: 4);
    // the subject: lit while the cone holds it
    final diff = weWrap(_angTo(cam, sj) - yaw), seen = diff.abs() <= h;
    final dm = Path()..moveTo(sj.dx, sj.dy - 5.5)..lineTo(sj.dx + 4.5, sj.dy)..lineTo(sj.dx, sj.dy + 5.5)..lineTo(sj.dx - 4.5, sj.dy)..close();
    c.drawPath(dm, weFill(seen ? N.g95 : N.g38));
    ring(c, sj, 12, 1);
    // the camera body (C)
    c.save();
    c.translate(cam.dx, cam.dy);
    c.rotate(yaw);
    final body = Path()
      ..addRect(const Rect.fromLTWH(-7, -4, 9, 8))
      ..moveTo(2, -2.5)
      ..lineTo(6, -4.5)
      ..lineTo(6, 4.5)
      ..lineTo(2, 2.5);
    c.drawPath(body, weLine(slot(2, 1), on(2) ? 1.5 : 1));
    c.restore();
    ring(c, cam, 11, 2);
    c.restore();
    // the picture frame (D): the shot; the horizon and the subject tilt with the roll
    final fr = _frame(a), roll = v(3) * math.pi / 180;
    c.drawRect(fr, weFill(N.g07));
    c.save();
    c.clipRect(fr);
    c.translate(fr.center.dx, fr.center.dy);
    c.rotate(-roll);
    c.drawLine(Offset(-fr.width, 0), Offset(fr.width, 0), weLine(N.g38));
    final x = weClamp(diff / h, -2, 2) * fr.width / 2, sz = weClamp(1.6 + 2.6 * (v(0) / 50) * (1000 / v(1)), 1.5, fr.height * .4);
    final sd = Path()..moveTo(x, -sz)..lineTo(x + sz * .8, 0)..lineTo(x, sz)..lineTo(x - sz * .8, 0)..close();
    c.drawPath(sd, weFill(N.g95));
    c.restore();
    c.drawRect(fr, weLine(slot(3, 1), on(3) ? 1.5 : 1));
    c.drawLine(Offset(fr.center.dx - 2, fr.center.dy), Offset(fr.center.dx + 2, fr.center.dy), weLine(N.g44));
    c.drawLine(Offset(fr.center.dx, fr.center.dy - 2), Offset(fr.center.dx, fr.center.dy + 2), weLine(N.g44));
  }
}
