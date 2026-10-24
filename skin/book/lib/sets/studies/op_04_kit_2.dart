// Part of op_04_kit.dart (cut there by size, as it was).
part of 'op_04_kit.dart';

const _g = <String, String>{
  '0': 'M0 3 A3 3 3 180 180 L6 7 A3 7 3 0 180 L0 3',
  '1': 'M1.6 1.6 L3 0 L3 10',
  '2': 'M0 3 A3 3 3 180 220 L0 10 L6 10',
  '3': 'M0 0 L6 0 L3 4 A3 7 3 270 225',
  '4': 'M4.6 10 L4.6 0 L0 7 L6 7',
  '5': 'M6 0 L1 0 L0.6 4.9 A3 7 3 222 273',
  '6': 'M4.8 0 L0.9 4.9 O3 7 3',
  '7': 'M0 0 L6 0 L2 10',
  '8': 'O3 2.3 2.3 O3 7.3 2.7',
  '9': 'O3 3 3 M5.1 5.1 L1.2 10',
  'A': 'M0 10 L3 0 L6 10 M1.2 6 L4.8 6',
  'B': 'M0 10 L0 0 L3.5 0 A3.5 2.5 2.5 270 180 L0 5 M3.5 5 A3.5 7.5 2.5 270 180 L0 10',
  'C': 'M5.1 0.9 A3 3 3 315 -135 L0 7 A3 7 3 180 -135',
  'D': 'M0 0 L0 10 L3 10 A3 7 3 90 -90 L6 3 A3 3 3 0 -90 L0 0',
  'E': 'M6 0 L0 0 L0 10 L6 10 M0 5 L4.5 5',
  'F': 'M6 0 L0 0 L0 10 M0 5 L4.5 5',
  'G': 'M5.1 0.9 A3 3 3 315 -135 L0 7 A3 7 3 180 -180 L6 5.5 L3.5 5.5',
  'H': 'M0 0 L0 10 M6 0 L6 10 M0 5 L6 5',
  'I': 'M3 0 L3 10',
  'J': 'M6 0 L6 7 A3 7 3 0 180',
  'K': 'M0 0 L0 10 M6 0 L0 6 M2 4.2 L6 10',
  'L': 'M0 0 L0 10 L6 10',
  'M': 'M0 10 L0 0 L3 6 L6 0 L6 10',
  'N': 'M0 10 L0 0 L6 10 L6 0',
  'O': 'M0 3 A3 3 3 180 180 L6 7 A3 7 3 0 180 L0 3',
  'P': 'M0 10 L0 0 L3.2 0 A3.2 2.7 2.7 270 180 L0 5.4',
  'Q': 'M0 3 A3 3 3 180 180 L6 7 A3 7 3 0 180 L0 3 M3.6 7.4 L6.4 10.4',
  'R': 'M0 10 L0 0 L3.2 0 A3.2 2.7 2.7 270 180 L0 5.4 M3 5.4 L6 10',
  'S': 'M5.25 1.3 A3 2.6 2.6 330 -240 A3 7.6 2.4 270 240',
  'T': 'M0 0 L6 0 M3 0 L3 10',
  'U': 'M0 0 L0 7 A3 7 3 180 -180 L6 0',
  'V': 'M0 0 L3 10 L6 0',
  'W': 'M0 0 L1.5 10 L3 4 L4.5 10 L6 0',
  'X': 'M0 0 L6 10 M6 0 L0 10',
  'Y': 'M0 0 L3 5 L6 0 M3 5 L3 10',
  'Z': 'M0 0 L6 0 L0 10 L6 10',
  '.': 'O3 9.5 0.4',
  '%': 'O1.3 1.6 1.3 O4.7 8.4 1.3 M5.5 0 L0.5 10',
  '-': 'M1 5.5 L5 5.5',
  '+': 'M0.5 5.5 L5.5 5.5 M3 3 L3 8',
  ':': 'O3 3 0.4 O3 8 0.4',
  '/': 'M5 0 L1 10',
  'x': 'M1 4 L5 9 M5 4 L1 9',
  '>': 'M1 1 L5 5 L1 9',
  '<': 'M5 1 L1 5 L5 9',
  ' ': '',
};

final _gc = <String, Path>{};

Path _glyph(String ch) => _gc.putIfAbsent(ch, () {
      final p = Path();
      final tk = (_g[ch] ?? '').split(' ').where((s) => s.isNotEmpty).toList();
      var i = 0;
      double nx() => double.parse(tk[i].length > 1 && 'MLAO'.contains(tk[i][0]) ? tk[i++].substring(1) : tk[i++]);
      while (i < tk.length) {
        final cmd = tk[i][0];
        if (cmd == 'M') {
          final x = nx(), y = nx();
          p.moveTo(x, y);
        } else if (cmd == 'L') {
          final x = nx(), y = nx();
          p.lineTo(x, y);
        } else if (cmd == 'A') {
          final cx = nx(), cy = nx(), r = nx(), a0 = nx(), sw = nx();
          p.arcTo(Rect.fromCircle(center: Offset(cx, cy), radius: r), a0 * math.pi / 180, sw * math.pi / 180, false);
        } else if (cmd == 'O') {
          final cx = nx(), cy = nx(), r = nx();
          p.addOval(Rect.fromCircle(center: Offset(cx, cy), radius: r));
        } else {
          i++;
        }
      }
      return p;
    });

/// Stroke text, [h] = cap height in px. [ax] 0 left, .5 centre, 1 right. Returns the drawn width.
double tx(Canvas c, String s, Offset at, double h, Color col, {double ax = 0, double w = 1, double track = .45}) {
  s = s.toUpperCase();
  final k = h / 10, adv = (6 + 10 * track) * k;
  final width = s.isEmpty ? 0.0 : s.length * adv - 10 * track * k;
  final p = st(col, w);
  c.save();
  c.translate(at.dx - width * ax, at.dy);
  c.scale(k);
  p.strokeWidth = w / k;
  for (var i = 0; i < s.length; i++) {
    final ch = s[i];
    if (ch == '1' && h > 12) {
      c.drawLine(const Offset(3, 0), const Offset(3, 10), p);
    } else {
      c.drawPath(_glyph(ch), p);
    }
    c.translate(6 + 10 * track, 0);
  }
  c.restore();
  return width;
}

/// Tiny label (OP-1 'FREQ', 'DAMP').
double lb(Canvas c, String s, Offset at, Color col, {double ax = 0, double h = 6}) => tx(c, s, at, h, col, ax: ax, w: .9, track: .5);

/// Large thin numeral.
double nm(Canvas c, String s, Offset at, double h, Color col, {double ax = 0}) => tx(c, s, at, h, col, ax: ax, w: 1.15, track: .35);

String d2(double x) => (x * 99).round().clamp(0, 99).toString().padLeft(2, '0');

/// Label over a two-digit numeral in the value's colour.
void rd(Canvas c, OpV v, int i, String label, Offset at, {double h = 20, double ax = 0}) {
  lb(c, label, at, pc(v, i), ax: ax);
  nm(c, d2(v[i]), at + const Offset(0, 10), h, pc(v, i), ax: ax);
}

// ---- motion maths -------------------------------------------------------------------------------------------------------------------

double _bz(double a, double b, double u) => 3 * a * u * (1 - u) * (1 - u) + 3 * b * u * u * (1 - u) + u * u * u;

/// Easing: [a] slow start, [b] slow arrival, [o] flying past the target before landing. [om] scales the overshoot.
double ez(double x, double a, double b, double o, [double om = 1.1]) {
  x = cl(x);
  final x1 = .02 + .9 * a, x2 = .98 - .9 * b, y2 = 1 + om * o;
  var lo = 0.0, hi = 1.0;
  for (var k = 0; k < 16; k++) {
    final u = (lo + hi) / 2;
    _bz(x1, x2, u) < x ? lo = u : hi = u;
  }
  return _bz(0, y2, (lo + hi) / 2);
}

double ezv(double x, OpV v, [double om = 1.1]) => ez(x, v[0], v[1], v[2], om);

/// Loop of [per] s: rest, travel, hold. Returns travel progress 0..1.
double trip(double t, [double per = 2.6, double shift = 0]) => cl(((t + shift) % per - .35) / (per * .6));

/// Spring displacement after release (1 -> 0). a stiffness, b damping, c mass (0..1 each).
double sp(double tau, double a, double b, double c) {
  if (tau <= 0) return 1;
  final k = 30 + 370 * a, m = .4 + 3.6 * c, d = .4 + 26 * b * b;
  final w = math.sqrt(k / m), z = d / (2 * math.sqrt(k * m));
  if (z >= 1) {
    final w2 = w * (z - math.sqrt(z * z - 1)) + .01;
    return math.exp(-w2 * tau) * (1 + w2 * tau);
  }
  final wd = w * math.sqrt(1 - z * z);
  return math.exp(-z * w * tau) * (math.cos(wd * tau) + z * w / wd * math.sin(wd * tau));
}

double spv(double tau, OpV v) => sp(tau, v[0], v[1], v[2]);

/// Spring re-fires on release and every [per] s on its own; held = 1 (stretched).
double ring(OpV v, double t, [double per = 3.2]) => v.press ? 1 : spv(v.since % per, v);

double _h(int i) {
  final x = math.sin(i * 127.1 + 311.7) * 43758.5453;
  return (x - x.floorToDouble()) * 2 - 1;
}

double _vn(double x, double smooth) {
  final i = x.floor(), f = x - i;
  final s = f * f * (3 - 2 * f);
  return lr(_h(i), _h(i + 1), lr(f, s, smooth));
}

/// Wiggle in about -1..1: [a] frequency (0.3..12 Hz), [cs] smoothness (1 = one soft wave, 0 = jagged layers).
double wg(double t, double a, double cs, [double seed = 0]) {
  final hz = .3 + 11.7 * a * a;
  var sum = 0.0, amp = 1.0, norm = 0.0, fr = hz;
  final rough = .1 + .65 * (1 - cs);
  for (var o = 0; o < 3; o++) {
    sum += amp * _vn(t * fr + seed * 17.3 + o * 41.1, cs);
    norm += amp;
    amp *= rough;
    fr *= 2.3;
  }
  return sum / norm;
}

/// Wiggle with all three values: frequency v[0], amplitude v[1], smoothness v[2].
double wv(double t, OpV v, [double seed = 0]) => wg(t, v[0], v[2], seed) * v[1];
