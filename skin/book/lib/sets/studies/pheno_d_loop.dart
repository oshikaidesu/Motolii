// Phenomenon D 4 and 5: Loop (a runner on a looped film-strip racetrack whose seam you pull) and Time remap (a film strip you push through a viewfinder).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'pheno_d_kit.dart';

// ---- 4. Loop: mode and length are the shape and size of a racetrack made of film frames ------------------------------------------------

const loopSpecs = [
  PhSpec('loop.mode', 0, 2, 1, digits: 0), // 0 once, 1 loop, 2 ping-pong
  PhSpec('loop.length', .5, 8, 2, unit: 's', digits: 1),
];

class LoopMini extends PhMini {
  LoopMini(super.doc);
  static const _modes = ['once', 'loop', 'ping'];
  static const _gapLen = 13.0, _pitch = 9.0;
  double _clock = .7, _gap = 0, _cap = 0, _pull = 0, _w = 0, _ripple = 9, _rippleAt = 0, _face = 1;
  bool _ready = false, _running = true;
  double _prevX = 0, _seamY0 = 0;

  int get _mode => doc['loop.mode'].round();

  @override
  String get cap => 'loop';

  @override
  List<PhR> readout() => [(_modes[_mode], doc.changed('loop.mode')), (doc.show('loop.length'), doc.changed('loop.length'))];

  Offset _ctr(Size s) => art(s).center;
  double _h(Size s) => math.min(art(s).height * .66, 54.0);
  double _wMax(Size s) => art(s).width - 14;
  double _wOf(Size s, double len) => phLerp(150, _wMax(s), (len - .5) / 7.5);
  double _perim(Size s) => 2 * (_w - _h(s)) + math.pi * _h(s);

  // the runner's position 0..1 along the track for the mode, at clock [t]
  double _run(double t) {
    final per = .8 + doc['loop.length'] * .5, u = t / per;
    return switch (_mode) {
      1 => u % 1,
      0 => math.min(1.0, (u % 1.35) / 1.0),
      _ => 1 - ((math.cos(math.pi * (u % 2 < 1 ? u % 1 : 2 - (u % 2))) + 1) / 2),
    };
  }

  double _sd(Size s, double q) {
    final g = _gap * _gapLen;
    return g + (_perim(s) - 2 * g) * q;
  }

  // a point and the unit tangent at distance [sd] along the stadium, starting at the top centre and going clockwise
  (Offset, Offset) _at(Size s, double sd) {
    final p = _perim(s), c = _ctr(s), r = _h(s) / 2, ls = _w - _h(s), h = ls / 2;
    sd = ((sd % p) + p) % p;
    if (sd < h) return (Offset(c.dx + sd, c.dy - r), const Offset(1, 0));
    sd -= h;
    if (sd < math.pi * r) {
      final a = -math.pi / 2 + sd / r;
      return (Offset(c.dx + h + math.cos(a) * r, c.dy + math.sin(a) * r), Offset(-math.sin(a), math.cos(a)));
    }
    sd -= math.pi * r;
    if (sd < ls) return (Offset(c.dx + h - sd, c.dy + r), const Offset(-1, 0));
    sd -= ls;
    if (sd < math.pi * r) {
      final a = math.pi / 2 + sd / r;
      return (Offset(c.dx - h + math.cos(a) * r, c.dy + math.sin(a) * r), Offset(-math.sin(a), math.cos(a)));
    }
    sd -= math.pi * r;
    return (Offset(c.dx - h + sd, c.dy - r), const Offset(1, 0));
  }

  @override
  int? zoneAt(Offset p, Size s) {
    if (!_ready) _snapNow(s);
    final c = _ctr(s), top = c.translate(0, -_h(s) / 2 + _pull);
    if ((p - top).distance <= 14) return 0;
    final h = (_w - _h(s)) / 2, x = phClamp(p.dx, c.dx - h, c.dx + h), d = (p - Offset(x, c.dy)).distance;
    if ((d - _h(s) / 2).abs() <= 13) return 1;
    return null;
  }

  void _snapNow(Size s) {
    _w = _wOf(s, doc['loop.length']);
    _gap = _mode == 1 ? 0 : 1;
    _cap = _mode == 2 ? 1 : 0;
    _ready = true;
  }

  @override
  void down(int z, Offset p, Size s) {
    _seamY0 = p.dy;
    _prevX = (p.dx - _ctr(s).dx).abs();
  }

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    if (z == 0) {
      final dy = p.dy - _seamY0;
      _pull = phClamp(dy * .4, -10, 10);
      doc.set('loop.mode', dy < -12 ? 0 : (dy > 12 ? 2 : 1));
    } else {
      // the track's end follows the finger; both ends move together, so the width changes by twice the finger's travel
      final x = (p.dx - _ctr(s).dx).abs(), dx = (x - _prevX) * (fine ? .1 : 1);
      _prevX = x;
      final nw = phClamp(_w + dx * 2, _wOf(s, .5), _wMax(s));
      doc.set('loop.length', .5 + 7.5 * (nw - _wOf(s, .5)) / (_wMax(s) - _wOf(s, .5)));
      _w = nw;
    }
  }

  @override
  void up(int z) {}

  @override
  void reset(int? z) => switch (z) {
        0 => doc.resetIds(['loop.mode']),
        1 => doc.resetIds(['loop.length']),
        _ => super.reset(z),
      };

  @override
  void synced() => _ready = false;

  @override
  bool step(double dt, Size s) {
    if (!_ready) _snapNow(s);
    final e = 1 - math.exp(-dt * 14);
    final tg = _wOf(s, doc['loop.length']);
    _gap += ((_mode == 1 ? 0 : 1) - _gap) * e;
    _cap += ((_mode == 2 ? 1 : 0) - _cap) * e;
    if (view.grab != 1) _w += (tg - _w) * e;
    _pull += (0 - _pull) * (view.grab == 0 ? 0 : 1 - math.exp(-dt * 18));
    _ripple += dt * 3;
    // the runner never stops lapping: that is the idle life
    final before = _run(_clock);
    _clock += dt;
    final now = _run(_clock);
    _running = !(_mode == 0 && now >= 1);
    // a small landing ripple where the runner turns round or stops
    if ((_mode == 0 && before < 1 && now >= 1) || (_mode == 2 && ((before < .02) != (now < .02) || (before > .98) != (now > .98)))) {
      _ripple = 0;
      _rippleAt = now;
    }
    return true;
  }

  @override
  void paint(Canvas c, Size s) {
    if (!_ready) _snapNow(s);
    final hot = view.grab ?? view.hot, q = _run(_clock), acc = phAcc(Fam.along);
    final span = _perim(s) - 2 * _gap * _gapLen, n = math.max(8, (span / _pitch).floor());
    final closed = _gap < .02;
    // the track is a strip of film frames bent into a racetrack: each frame is a little picture, the ones the runner just left glow
    for (var i = 0; i <= n; i++) {
      if (closed && i == n) break;
      final qi = i / n;
      final (p, t) = _at(s, _sd(s, qi));
      final d = closed ? (((q - qi) % 1) + 1) % 1 : (q - qi).abs();
      final lit = closed ? (d < .14 ? 1 - d / .14 : 0.0) : math.max(0.0, 1 - d / .1);
      c.save();
      c.translate(p.dx, p.dy);
      c.rotate(math.atan2(t.dy, t.dx));
      final cell = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 6.5, height: 9), const Radius.circular(1.3));
      c.drawRRect(cell, phFill(Color.lerp(N.g10, acc, .16 * lit)!));
      c.drawRRect(cell, phStroke(hot == 1 ? phHot.withValues(alpha: .55 + .3 * lit) : Color.lerp(N.g38, N.g76, lit)!));
      c.drawRect(Rect.fromCenter(center: Offset.zero, width: 2.6, height: 4.2), phFill(Color.lerp(N.g20, acc, .5 * lit)!));
      c.restore();
    }
    // ends: once = a hollow start and a stop bar; ping-pong = a stop bar at both ends; loop = no ends, only the seam diamond
    void bar(double sd) {
      final (p, t) = _at(s, sd);
      final nrm = Offset(t.dy, -t.dx);
      c.drawLine(p + nrm * 7, p - nrm * 7, phStroke(N.g95));
    }

    if (_gap > .5) {
      bar(_sd(s, 1));
      if (_cap > .5) {
        bar(_sd(s, 0));
      } else {
        c.drawCircle(_at(s, _sd(s, 0)).$1, 3, phStroke(N.g95));
      }
    }
    // the seam handle at the top, pulled by the finger like a bead on a thread
    final top = _at(s, 0).$1, seam = top.translate(0, _pull);
    if (_pull.abs() > .3) c.drawLine(top, seam, phStroke(N.g44));
    final dm = hot == 0 ? 4.2 : 3.2;
    final diamond = Path()
      ..moveTo(seam.dx, seam.dy - dm)
      ..lineTo(seam.dx + dm, seam.dy)
      ..lineTo(seam.dx, seam.dy + dm)
      ..lineTo(seam.dx - dm, seam.dy)
      ..close();
    c.drawPath(diamond, phFill(N.g10));
    c.drawPath(diamond, phStroke(hot == 0 ? phHot : N.g95));
    // the landing ripple
    if (_ripple < 1) c.drawCircle(_at(s, _sd(s, _rippleAt)).$1, 3 + 8 * _ripple, phStroke(acc.withValues(alpha: .5 * (1 - _ripple))));
    // the runner: a tiny stick figure that always stands up and faces the way it runs
    final (rp, rt) = _at(s, _sd(s, q));
    final dq = _run(_clock + .01) - q;
    if (rt.dx.abs() > .15) _face = (rt.dx >= 0 ? 1 : -1) * (_mode == 2 && dq < 0 ? -1 : 1);
    final ph = _running ? _clock * 14 : 0.0, sw = math.sin(ph) * 3, f = _face.toDouble();
    final fig = phStroke(acc, 1.2);
    final o = rp.translate(0, -2);
    c.drawLine(o.translate(0, -3), o.translate(0, 1.5), fig);
    c.drawLine(o.translate(0, 1.5), o.translate(sw, 5.5), fig);
    c.drawLine(o.translate(0, 1.5), o.translate(-sw, 5.5), fig);
    c.drawLine(o.translate(0, -2), o.translate(f * 1.5 - sw * .6, .5), fig);
    c.drawCircle(o.translate(f * .4, -5.2), 1.9, phFill(acc));
  }
}

// ---- 5. Time remap: speed is how fast a strip of film frames is pushed through a viewfinder ------------------------------------------------

const tapeSpecs = [PhSpec('remap.speed', -4, 4, 1)];

class TapeMini extends PhMini {
  TapeMini(super.doc);
  double _pos = 0, _t = 0, _raw = 1;
  static const _pitch = 34.0;

  @override
  String get cap => 'remap';

  @override
  List<PhR> readout() => [('${doc.show('remap.speed')}x', doc.changed('remap.speed'))];

  @override
  int? zoneAt(Offset p, Size s) => art(s).inflate(4).contains(p) ? 0 : null;

  @override
  void down(int z, Offset p, Size s) => _raw = doc['remap.speed'];

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0;
    // the strip follows the finger; the speed is how far it has been pushed, with soft detents at -1, 0 and 1
    _pos -= d.dx * k / _pitch;
    _raw = phClamp(_raw + d.dx * k / 60, -4, 4);
    var v = _raw;
    if (!fine) {
      for (final m in [-1.0, 0.0, 1.0]) {
        if ((v - m).abs() < .07) v = m;
      }
    }
    doc.set('remap.speed', v);
  }

  @override
  bool step(double dt, Size s) {
    _t += dt;
    if (view.grab == null) _pos -= doc['remap.speed'] * dt; // idle life: the strip keeps running at the set speed
    return true;
  }

  @override
  void paint(Canvas c, Size s) {
    final a = art(s), hot = view.grab ?? view.hot, v = doc['remap.speed'];
    final cx = a.center.dx, cy = a.center.dy, halfW = a.width / 2 - 2;
    final fh = math.min(34.0, a.height * .66), fw = fh * .78;
    final blur = math.min(v.abs() * 3.2, 11.0), dir = v >= 0 ? 1.0 : -1.0;
    final sway = v == 0 ? math.sin(_t * 1.6) * .03 : 0.0, pos = _pos + sway;
    final first = (pos - halfW / _pitch).floor() - 1, last = (pos + halfW / _pitch).ceil() + 1;
    // the strip drapes in a shallow arch and shrinks and fades towards both ends, so it never reads as a straight ruler
    for (var i = first; i <= last; i++) {
      final x = (i - pos) * _pitch, r = (x.abs() / halfW);
      if (r > 1) continue;
      final sc = 1 - .38 * math.pow(r, 1.5), al = 1 - .78 * r * r;
      final y = cy + 7 * r * r, rot = .22 * (x / halfW);
      c.save();
      c.translate(cx + x, y);
      c.rotate(rot);
      c.scale(sc);
      final w = fw, h = fh, box = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: w, height: h), const Radius.circular(2.5));
      // speed as blur: faint ghost frames trail behind the way it runs
      if (blur > .6) {
        for (var j = 3; j >= 1; j--) {
          c.drawRRect(box.shift(Offset(-dir * blur * j / 3, 0)), phStroke(N.g63.withValues(alpha: .16 * al * (1 - j / 4.5))));
        }
      }
      final centre = r < .5 * _pitch / halfW;
      c.drawRRect(box, phFill(N.g10));
      c.drawRRect(box, phStroke(centre && hot != null ? phHot : N.g38.withValues(alpha: al)));
      // sprocket holes
      for (final sx in [-w * .3, 0.0, w * .3]) {
        c.drawCircle(Offset(sx, -h / 2 + 2.6), .7, phFill(N.g44.withValues(alpha: al)));
        c.drawCircle(Offset(sx, h / 2 - 2.6), .7, phFill(N.g44.withValues(alpha: al)));
      }
      // the picture in the frame: a ball at one moment of its hop (every frame a little further on)
      final u = (((i % 8) + 8) % 8) / 8, bx = (u - .5) * w * .62, floor = h * .24, by = floor - 3 - math.sin(math.pi * u) * h * .34;
      c.drawLine(Offset(-w * .36, floor), Offset(w * .36, floor), phStroke(N.g26.withValues(alpha: al)));
      c.drawCircle(Offset(bx, by), 2.2, phFill(centre ? phAcc(Fam.stagger) : N.g56.withValues(alpha: al)));
      if (blur > 1) c.drawLine(Offset(bx - dir * blur * .7, by), Offset(bx, by), phStroke(N.g56.withValues(alpha: .35 * al), 2));
      c.restore();
    }
    // the viewfinder brackets on the frame in the window
    final win = Rect.fromCenter(center: Offset(cx, cy), width: fw + 8, height: fh + 8), k = 5.0;
    final br = phStroke(hot != null ? phHot : N.g95.withValues(alpha: .55 + .2 * math.sin(_t * 2)));
    for (final (px, py) in [(win.left, win.top), (win.right, win.top), (win.left, win.bottom), (win.right, win.bottom)]) {
      final sx = px == win.left ? 1.0 : -1.0, sy = py == win.top ? 1.0 : -1.0;
      c.drawLine(Offset(px, py), Offset(px + sx * k, py), br);
      c.drawLine(Offset(px, py), Offset(px, py + sy * k), br);
    }
  }
}
