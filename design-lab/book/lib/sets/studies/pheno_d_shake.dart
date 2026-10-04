// Phenomenon D 3 and 6: Wiggle (a thread you shake) and Camera shake (a handheld viewfinder you jostle).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'pheno_d_kit.dart';

// ---- 3. Wiggle: frequency and amplitude are how a thread is being shaken --------------------------------------------------------------

const wiggleSpecs = [
  PhSpec('wiggle.freq', .2, 20, 3, unit: 'Hz', digits: 1),
  PhSpec('wiggle.amp', 0, 100, 20, unit: 'px', digits: 0),
];

class WiggleMini extends PhMini {
  WiggleMini(super.doc);
  double _clock = .7;
  final _sw = Stopwatch();
  final List<(double, double)> _samples = []; // (seconds, finger offset from the rest line)
  Offset? _finger; // while held: where the finger is

  @override
  String get cap => 'wiggle';

  @override
  List<PhR> readout() => [(doc.show('wiggle.freq'), doc.changed('wiggle.freq')), (doc.show('wiggle.amp'), doc.changed('wiggle.amp'))];

  double _px(Rect a, double amp) => a.height * .46 * math.sqrt(amp / 100);

  @override
  int? zoneAt(Offset p, Size s) => art(s).inflate(10).contains(p) ? 0 : null;

  @override
  void down(int z, Offset p, Size s) {
    _samples.clear();
    _sw..reset()..start();
    _finger = p;
    _sample(p, s);
  }

  void _sample(Offset p, Size s) {
    final a = art(s), t = _sw.elapsedMicroseconds / 1e6;
    _samples.add((t, p.dy - a.center.dy));
    while (_samples.isNotEmpty && t - _samples.first.$1 > 1.1) {
      _samples.removeAt(0);
    }
  }

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final a = art(s);
    _finger = p;
    _sample(p, s);
    final mix = fine ? .08 : .3;
    // peak excursion in the last moment: how far you shake
    var peak = 0.0;
    for (final q in _samples) {
      peak = math.max(peak, q.$2.abs());
    }
    final wantAmp = 100 * math.pow(phClamp(peak / (a.height * .46), 0, 1), 2).toDouble();
    if (wantAmp > doc['wiggle.amp'] || _reversals().length >= 3) doc.set('wiggle.amp', phLerp(doc['wiggle.amp'], wantAmp, mix));
    // how fast you shake: reversals of direction (with a small dead band so a trembling hand is not a shake)
    final rev = _reversals();
    if (rev.length >= 3) {
      final half = (rev.last - rev.first) / (rev.length - 1);
      if (half > .005) doc.set('wiggle.freq', math.exp(phLerp(math.log(doc['wiggle.freq']), math.log(1 / (2 * half)), mix)));
    }
  }

  List<double> _reversals() {
    final out = <double>[];
    if (_samples.length < 3) return out;
    var dir = 0, ext = _samples.first.$2;
    for (final q in _samples) {
      final y = q.$2;
      if (dir == 0) {
        if ((y - ext).abs() > 3) {
          dir = y > ext ? 1 : -1;
          ext = y;
        }
      } else if (dir > 0) {
        if (y > ext) {
          ext = y;
        } else if (ext - y > 3) {
          out.add(q.$1);
          dir = -1;
          ext = y;
        }
      } else {
        if (y < ext) {
          ext = y;
        } else if (y - ext > 3) {
          out.add(q.$1);
          dir = 1;
          ext = y;
        }
      }
    }
    return out;
  }

  @override
  void up(int z) => _finger = null;

  @override
  void cancelled(int z) => _finger = null;

  @override
  bool step(double dt, Size s) {
    if (view.hover || view.grab != null) _clock += dt;
    return false;
  }

  /// The thread's offset at time [tm] seconds: a smooth wobble with [freq] swings a second.
  double _wob(double tm) => phNoise(2 * math.pi * doc['wiggle.freq'] * tm, 1.3);

  @override
  void paint(Canvas c, Size s) {
    final a = art(s), cy = a.center.dy, hot = view.grab ?? view.hot;
    final amp = _px(a, doc['wiggle.amp']);
    const win = 2.0; // seconds across the thread
    phDash(c, Offset(a.left, cy), Offset(a.right, cy), phStroke(N.g20));
    // the thread (pinned at both ends); a held finger drags a soft bump out of it
    final path = Path();
    final fx = _finger?.dx, fy = (_finger?.dy ?? cy) - cy;
    for (double x = a.left; x <= a.right + .1; x += 2) {
      final xn = (x - a.left) / a.width, env = math.pow(math.sin(math.pi * xn), .6).toDouble();
      var y = cy + amp * env * _wob(_clock + xn * win);
      if (fx != null) y += (fy - (y - cy)) * math.exp(-math.pow((x - fx) / (a.width * .1), 2)) * env;
      x == a.left ? path.moveTo(x, y) : path.lineTo(x, y);
    }
    // two faint strands a moment behind and ahead: it is a fibre, not a trace
    for (final lag in [-.07, .07]) {
      final fib = Path();
      for (double x = a.left; x <= a.right + .1; x += 3) {
        final xn = (x - a.left) / a.width, env = math.pow(math.sin(math.pi * xn), .6).toDouble();
        final y = cy + amp * env * _wob(_clock + lag + xn * win);
        x == a.left ? fib.moveTo(x, y) : fib.lineTo(x, y);
      }
      c.drawPath(fib, phStroke(N.g44.withValues(alpha: .55)));
    }
    c.drawPath(path, phStroke(hot != null ? N.g95 : N.g76, hot != null ? 1.5 : 1.25));
    for (final px in [a.left + 1, a.right - 1]) {
      c.drawCircle(Offset(px, cy), 2.6, phStroke(N.g56));
    }
    // the bead rides the thread at "now" (a third of the way in); its stem shows the offset from rest
    final bx = a.left + a.width * .62, by = cy + amp * math.pow(math.sin(math.pi * .62), .6) * _wob(_clock + .62 * win);
    c.drawLine(Offset(bx, cy), Offset(bx, by), phStroke(N.g44));
    c.drawCircle(Offset(bx, by), 3.2, phFill(phAcc(Fam.scatter)));
    // grab hint: a ring on the thread under the pointer
    final ptr = view.ptr;
    if (ptr != null && hot != null && view.grab == null) {
      final xn = phClamp((ptr.dx - a.left) / a.width, 0, 1), env = math.pow(math.sin(math.pi * xn), .6).toDouble();
      c.drawCircle(Offset(ptr.dx, cy + amp * env * _wob(_clock + xn * win)), 5, phStroke(phHot));
    }
    if (fx != null) c.drawCircle(Offset(fx, _finger!.dy), 6, phStroke(phHot));
  }
}

// ---- 6. Camera shake: amount and roughness are a handheld frame -----------------------------------------------------------------------

const shakeSpecs = [
  PhSpec('shake.amount', 0, 40, 6, unit: 'px', digits: 1),
  PhSpec('shake.roughness', 0, 1, .5),
];

class ShakeMini extends PhMini {
  ShakeMini(super.doc);
  double _clock = 1.3, _trailAt = 0;
  Offset _finger = Offset.zero, _shown = Offset.zero; // offset of the held frame from the centre
  final List<Offset> _trail = [];
  final _sw = Stopwatch();
  double _speed = 0;

  @override
  String get cap => 'shake';

  @override
  List<PhR> readout() => [(doc.show('shake.amount'), doc.changed('shake.amount')), (doc.show('shake.roughness'), doc.changed('shake.roughness'))];

  double _span(Size s) => art(s).height * .28;

  /// The camera's offset (x, y in px) and roll (degrees) at time [t].
  (Offset, double) _cam(double t, Size s) {
    final k = doc['shake.amount'] / 40 * _span(s), r = doc['shake.roughness'];
    double ch(double seed) => (phNoise(t * 4.4, seed) + r * phNoise(t * 16, seed + 5) + r * r * phNoise(t * 47, seed + 9)) / (1 + r + r * r);
    return (Offset(ch(1) * k * 1.8, ch(2) * k * 1.8), ch(3) * k * .5);
  }

  Rect _frame(Size s) {
    final a = art(s), h = a.height * .78, w = math.min(h * 16 / 9, a.width * .6);
    return Rect.fromCenter(center: a.center, width: w, height: h);
  }

  @override
  int? zoneAt(Offset p, Size s) => _frame(s).inflate(12).shift(_shown + _cam(_clock, s).$1).contains(p) ? 0 : null;

  @override
  void down(int z, Offset p, Size s) {
    _sw..reset()..start();
    _finger = p - art(s).center;
    _speed = 0;
  }

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final dt = math.max(_sw.elapsedMicroseconds / 1e6, .002);
    _sw..reset()..start();
    final sp = d.distance / dt;
    _finger = p - art(s).center;
    final k = fine ? .1 : 1.0, max = _span(s) * 1.8;
    doc.set('shake.amount', phLerp(doc['shake.amount'], 40 * phClamp(_finger.distance / max, 0, 1), .35 * k));
    if (sp > 30) {
      _speed = phLerp(_speed, sp, .25);
      doc.set('shake.roughness', phLerp(doc['shake.roughness'], phClamp(_speed / 1100, 0, 1), .22 * k));
    }
  }

  @override
  void up(int z) => _finger = Offset.zero;

  @override
  bool step(double dt, Size s) {
    final held = view.grab != null;
    if (view.hover || held) {
      _clock += dt;
      _trailAt += dt;
      _shown = Offset.lerp(_shown, held ? _finger * .6 : Offset.zero, 1 - math.exp(-dt * 12))!;
      if (_trailAt > .025) {
        _trailAt = 0;
        _trail.add(_shown + _cam(_clock, s).$1);
        if (_trail.length > 48) _trail.removeAt(0);
      }
    }
    return (_shown.distance > .1) && !held;
  }

  @override
  void paint(Canvas c, Size s) {
    final a = art(s), fr = _frame(s), hot = view.grab ?? view.hot;
    final cam = _cam(_clock, s), off = _shown + cam.$1, roll = cam.$2 * math.pi / 180;
    // the world: fixed hairlines (a horizon, two hills, a tree, dots to judge parallax)
    void world(Paint p) {
      final hz = a.center.dy + a.height * .16;
      c.drawLine(Offset(a.left, hz), Offset(a.right, hz), p);
      final hills = Path()
        ..moveTo(a.left, hz)
        ..lineTo(a.left + a.width * .22, hz - a.height * .2)
        ..lineTo(a.left + a.width * .36, hz - a.height * .08)
        ..lineTo(a.left + a.width * .55, hz - a.height * .26)
        ..lineTo(a.left + a.width * .78, hz);
      c.drawPath(hills, p);
      final t = Offset(a.left + a.width * .66, hz);
      c.drawLine(t, t.translate(0, -a.height * .14), p);
      c.drawCircle(t.translate(0, -a.height * .18), a.height * .05, p);
      for (var i = 0; i < 6; i++) {
        c.drawCircle(Offset(a.left + a.width * (.08 + i * .17), a.top + 3 + (i.isEven ? 3 : 10)), .8, p);
      }
    }

    world(phStroke(N.g26));
    // the frame sees the world brighter than the rest; the frame moves, the world does not
    c.save();
    c.translate(a.center.dx + off.dx, a.center.dy + off.dy);
    c.rotate(roll);
    c.translate(-a.center.dx, -a.center.dy);
    c.clipRect(fr);
    c.save();
    c.translate(a.center.dx, a.center.dy);
    c.rotate(-roll);
    c.translate(-a.center.dx - off.dx, -a.center.dy - off.dy);
    world(phStroke(N.g63));
    c.restore();
    // third lines inside
    final th = phStroke(N.g26.withValues(alpha: .4));
    for (var i = 1; i < 3; i++) {
      c.drawLine(Offset(fr.left + fr.width * i / 3, fr.top), Offset(fr.left + fr.width * i / 3, fr.bottom), th);
      c.drawLine(Offset(fr.left, fr.top + fr.height * i / 3), Offset(fr.right, fr.top + fr.height * i / 3), th);
    }
    c.restore();
    // the trail: where the frame's centre has been
    if (_trail.length > 2) {
      final tp = Path();
      for (var i = 0; i < _trail.length; i++) {
        final q = a.center + _trail[i];
        i == 0 ? tp.moveTo(q.dx, q.dy) : tp.lineTo(q.dx, q.dy);
      }
      c.drawPath(tp, phStroke(N.g38.withValues(alpha: .8)));
    }
    // the viewfinder: four corners and a small cross
    c.save();
    c.translate(a.center.dx + off.dx, a.center.dy + off.dy);
    c.rotate(roll);
    final f = fr.shift(-a.center), br = phStroke(hot != null ? phHot : N.g95, 1.25), l = 8.0;
    for (final sx in [-1, 1]) {
      for (final sy in [-1, 1]) {
        final p = Offset(sx < 0 ? f.left : f.right, sy < 0 ? f.top : f.bottom);
        c.drawLine(p, p.translate(-sx * l, 0), br);
        c.drawLine(p, p.translate(0, -sy * l), br);
      }
    }
    final x = phStroke(phAcc(Fam.face));
    c.drawLine(const Offset(-3, 0), const Offset(3, 0), x);
    c.drawLine(const Offset(0, -3), const Offset(0, 3), x);
    c.restore();
  }
}
