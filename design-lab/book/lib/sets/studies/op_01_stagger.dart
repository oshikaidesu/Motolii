part of 'op_01.dart';

// Stagger: offset = blue, direction = green, order = red (double-tap steps FWD / REV / CTR / RND).
// Every panel is caught mid-cascade at rest: half the items have gone, half are waiting, so a still reads.

const _staggerSpecs = <_S>[
  _S('dominoes', 'object|effect|drag|drawing|1', _G.drag, _t01, a: .35, b: .3, c: 0),
  _S('stadium wave', 'character|effect|drag|drawing|2', _G.drag, _t02, a: .3, b: .3, c: 0),
  _S('monkey drummers', 'character|instrument|drag|drawing|3', _G.drag, _t03, a: .45, b: .3, c: 0),
  _S('arpeggio keys', 'instrument|effect|drag|drawing|2', _G.drag, _t04, a: .3, b: .3, c: 0),
  _S('step lanes', 'device|mechanism|drag|diagram|1', _G.drag, _t05, a: .35, b: .3, c: 0),
  _S('train over hill', 'vehicle|effect|drag|drawing|2', _G.drag, _t06, a: .4, b: .3, c: 0),
  _S('falling letters', 'typographic|effect on sample|drag|drawing|2', _G.drag, _t07, a: .3, b: .3, c: 0),
  _S('birds off a wire', 'animal|reason|flick|drawing|2', _G.flick, _t08, a: .35, b: .3, c: 0),
  _S('rising blocks', 'isometric|effect|pinch|drawing|3', _G.pinch, _t09, a: .3, b: .3, c: 0),
  _S('clock row', 'machine|mechanism|spin|drawing|2', _G.spin, _t10, a: .3, b: .3, c: 0),
  _S('delay chain', 'diagram|mechanism|drag|diagram|1', _G.drag, _t11, a: .35, b: .3, c: 0),
  _S('envelope ridge', 'diagram|mechanism|drag|diagram|3', _G.drag, _t12, a: .3, b: .3, c: 0),
  _S('ant column', 'animal|effect|drag|drawing|2', _G.drag, _t13, a: .3, b: .3, c: 0),
  _S('pendulum wave', 'instrument|effect|spin|drawing|3', _G.spin, _t14, a: .4, b: .3, c: 0),
  _S('dealt cards', 'object|effect|flick|drawing|2', _G.flick, _t15, a: .35, b: .3, c: 0),
  _S('street lamps', 'vehicle|reason|drag|drawing|3', _G.drag, _t16, a: .35, b: .3, c: 0),
  _S('echo numerals', 'typographic|value|drag|numeral|2', _G.drag, _t17, a: .35, b: .3, c: 0),
  _S('rowing eight', 'vehicle|character|drag|drawing|3', _G.drag, _t18, a: .3, b: .3, c: 0),
  _S('draw the order', 'diagram|mechanism|draw|diagram|3', _G.draw, _t19, a: .35, b: .3, c: 0),
  _S('endless smear', 'abstract|pushed|rub|drawing|5', _G.rub, _t20, a: .8, b: .3, c: 0),
];

double _off(_P p) => .04 + p.a * .36;
bool _rev(_P p) => p.b > .5;

/// Order index of item i: direction flips, then the order mode.
int _k(_P p, int i, int n) => _ord(_rev(p) ? n - 1 - i : i, n, p.c);

void _dirTag(Canvas c, _P p, Offset o) {
  final r = _rev(p);
  _ln(c, o, o + const Offset(14, 0), _green, 1);
  _pl(c, r ? [o + const Offset(4, -3), o, o + const Offset(4, 3)] : [o + const Offset(10, -3), o + const Offset(14, 0), o + const Offset(10, 3)], _green, 1);
  _lb(c, _ordName(p.c), o + const Offset(18, -4), _red, 6.5);
}

void _offTag(Canvas c, _P p, Offset o) {
  _lb(c, 'OFFSET', o, _blue, 6.5);
  _num(c, '${(_off(p) * 1000).round()}', o + const Offset(0, 8), 18, _blue);
}

// 1 dominoes: each tile tips after the one before; caught halfway down the row.
void _t01(Canvas c, _P p, double t) {
  const g = 96.0, n = 10;
  _ln(c, const Offset(4, g), const Offset(_pw - 4, g), _dim, 1);
  for (var i = 0; i < n; i++) {
    final k = _cas(_k(p, i, n), n, _off(p), t);
    final x = 12.0 + i * 14;
    final dir = _rev(p) ? -1.0 : 1.0;
    final pivot = Offset(x + (dir > 0 ? 5 : 0), g);
    c.save();
    c.translate(pivot.dx, pivot.dy);
    c.rotate(dir * _ease(k) * 1.15);
    final col = k >= 1 ? _a(_white, .45) : (k > 0 ? _blue : _white);
    c.drawRect(Rect.fromLTWH(dir > 0 ? -5 : 0, -26, 5, 26), _s(col, 1));
    _dot(c, Offset(dir > 0 ? -2.5 : 2.5, -19), .8, col);
    _dot(c, Offset(dir > 0 ? -2.5 : 2.5, -7), .8, col);
    c.restore();
  }
  _offTag(c, p, const Offset(8, 8));
  _dirTag(c, p, const Offset(100, 14));
}

// 2 stadium wave: a row of fans stand and raise their arms in turn.
void _t02(Canvas c, _P p, double t) {
  const g = 100.0, n = 9;
  _pl(c, [const Offset(4, g + 4), const Offset(_pw - 4, g + 4)], _dim, 1);
  _pl(c, [const Offset(4, g - 10), const Offset(_pw - 4, g - 10)], _dim, .7);
  for (var i = 0; i < n; i++) {
    final k = _cas(_k(p, i, n), n, _off(p), t, 1.2);
    final up = math.sin(k * math.pi);
    final x = 14.0 + i * 16;
    _stick(c, Offset(x, g - up * 10), 30 + up * 4, up > .3 ? _white : _a(_white, .55), arm: up);
    if (up > .6) {
      _ln(c, Offset(x - 6, g - 52 - up * 6), Offset(x - 9, g - 56 - up * 6), _green, .8);
      _ln(c, Offset(x + 6, g - 52 - up * 6), Offset(x + 9, g - 56 - up * 6), _green, .8);
    }
  }
  _offTag(c, p, const Offset(8, 8));
  _dirTag(c, p, const Offset(100, 14));
}

// 3 monkey drummers: four drummers hit in turn; the order numbers sit under each drum.
void _t03(Canvas c, _P p, double t) {
  const n = 4;
  for (var i = 0; i < n; i++) {
    final x = 22.0 + i * 37;
    final k = _cas(_k(p, i, n), n, _off(p) * 1.5, t, .6);
    final hit = k > 0 && k < 1 ? math.sin(k * math.pi) : 0.0;
    final hd = Offset(x, 44);
    _ring(c, hd, 8, _white, 1);
    _ring(c, hd + const Offset(-9, -2), 3.2, _white, 1);
    _ring(c, hd + const Offset(9, -2), 3.2, _white, 1);
    c.drawOval(Rect.fromCenter(center: hd + const Offset(0, 2.5), width: 9, height: 7), _s(_white, .8));
    _dot(c, hd + const Offset(-2.5, -2), .9, _white);
    _dot(c, hd + const Offset(2.5, -2), .9, _white);
    _ln(c, hd + const Offset(0, 8), hd + const Offset(0, 22), _white, 1);
    final drum = Offset(x, 82);
    c.drawOval(Rect.fromCenter(center: drum, width: 22, height: 7), _s(k >= 1 ? _dim : _white, 1));
    _ln(c, drum + const Offset(-11, 0), drum + const Offset(-11, 12), _white, 1);
    _ln(c, drum + const Offset(11, 0), drum + const Offset(11, 12), _white, 1);
    c.drawArc(Rect.fromCenter(center: drum + const Offset(0, 12), width: 22, height: 7), 0, math.pi, false, _s(_white, 1));
    final stickA = -1.9 + hit * 1.1;
    final sh = hd + const Offset(6, 14);
    _ln(c, sh, sh + Offset(math.cos(stickA), math.sin(stickA)) * -14, _blue, 1.1);
    if (hit > .7) {
      for (final a in [-2.4, -1.57, -.7]) {
        _ln(c, drum + Offset(math.cos(a), math.sin(a)) * 12, drum + Offset(math.cos(a), math.sin(a)) * 17, _green, .9);
      }
    }
    _lb(c, '${_k(p, i, n) + 1}', Offset(x, 102), _red, 8, .5);
  }
  _lb(c, 'OFFSET ${(_off(p) * 1500).round()}', const Offset(8, 8), _blue, 6.5);
  _dirTag(c, p, const Offset(100, 12));
}

// 4 arpeggio keys: keys go down one after another; notes rise from the pressed ones.
void _t04(Canvas c, _P p, double t) {
  const n = 12, x0 = 8.0, kw = 11.6, top = 60.0;
  for (var i = 0; i < n; i++) {
    final k = _cas(_k(p, i, n), n, _off(p) * .7, t, .8);
    final dn = k > 0 && k < 1 ? 3.0 : 0.0;
    final r = Rect.fromLTWH(x0 + i * kw, top + dn, kw, 46);
    c.drawRect(r, _s(k > 0 && k < 1 ? _green : (k >= 1 ? _dim : _white), 1));
    if (k > 0 && k < 1) {
      final ny = top - 8 - k * 22;
      _ring(c, Offset(r.center.dx, ny), 2.2, _a(_blue, 1 - k * .7), 1);
      _ln(c, Offset(r.center.dx + 2.2, ny), Offset(r.center.dx + 2.2, ny - 8), _a(_blue, 1 - k * .7), 1);
    }
  }
  for (final i in [0, 1, 3, 4, 5, 7, 8, 10]) {
    c.drawRect(Rect.fromLTWH(x0 + (i + 1) * kw - 3.5, top, 7, 27), _s(_white, .8));
  }
  _offTag(c, p, const Offset(8, 6));
  _dirTag(c, p, const Offset(100, 12));
}

// 5 step lanes (Max device): each lane starts later; the start dots form the slope you drag.
void _t05(Canvas c, _P p, double t) {
  const n = 8, x0 = 14.0, x1 = 146.0;
  final ph = 54 + math.sin(t * .8) * 30;
  for (var i = 0; i < n; i++) {
    final y = 20.0 + i * 11;
    _dotted(c, Offset(x0, y), Offset(x1, y), _dim, 4);
    final st = x0 + _k(p, i, n) * _off(p) * 30;
    final en = st + 46;
    final started = ph > st;
    _ln(c, Offset(st, y), Offset(math.min(en, x1), y), started ? _white : _a(_white, .35), 1.2);
    _dot(c, Offset(st, y), 1.8, started ? _blue : _dim);
    if (started && ph < en) _ring(c, Offset(ph, y), 2.4, _green, 1);
  }
  final s0 = Offset(x0 + _k(p, 0, n) * _off(p) * 30, 20), s1 = Offset(x0 + _k(p, n - 1, n) * _off(p) * 30, 20 + (n - 1) * 11.0);
  if ((p.c * 4).floor() % 4 < 2) _ln(c, s0, s1, _a(_blue, .6), .8);
  _ln(c, Offset(ph, 12), Offset(ph, 106), _red, 1);
  _lb(c, '${(_off(p) * 1000).round()}MS', const Offset(x1, 4), _blue, 6.5, 1);
  _lb(c, _ordName(p.c), const Offset(x0, 106), _red, 6.5);
}

// 6 train over a hill: the cars reach the hump one by one.
void _t06(Canvas c, _P p, double t) {
  double hill(double x) => 92 - 26 * math.exp(-math.pow((x - 78) / 26, 2));
  _pl(c, [for (var x = 0.0; x <= _pw; x += 3) Offset(x, hill(x) + 6)], _white, 1);
  for (var x = 2.0; x < _pw; x += 7) {
    _ln(c, Offset(x, hill(x) + 6), Offset(x, hill(x) + 9), _dim, .8);
  }
  const n = 6;
  for (var i = 0; i < n; i++) {
    final k = _cas(_k(p, i, n), n, _off(p) * .9 + .06, t, .9);
    final x = _rev(p) ? 150 - k * 140 : 6 + k * 140;
    final y = hill(x);
    final sl = math.atan2(hill(x + 2) - hill(x - 2), 4);
    c.save();
    c.translate(x, y);
    c.rotate(sl);
    c.drawRect(const Rect.fromLTWH(-9, -10, 18, 10), _s(i == 0 ? _blue : _white, 1));
    if (i == 0) _ln(c, const Offset(4, -10), const Offset(4, -15), _blue, 1);
    _ring(c, const Offset(-5, 2), 2.2, _white, .9);
    _ring(c, const Offset(5, 2), 2.2, _white, .9);
    c.restore();
  }
  _offTag(c, p, const Offset(8, 8));
  _dirTag(c, p, const Offset(100, 12));
}

// 7 falling letters: the word lands letter by letter; trails show where each came from.
void _t07(Canvas c, _P p, double t) {
  const word = 'STAGGER', base = 62.0;
  var x = 10.0;
  for (var i = 0; i < word.length; i++) {
    final tp = _tp(word[i], 30, _white, FontWeight.w200, 0, true);
    final k = _cas(_k(p, i, word.length), word.length, _off(p), t, .9);
    final e = 1 - math.pow(1 - k, 2) + (k > .7 ? math.sin((k - .7) / .3 * math.pi) * .06 : 0);
    final y = base - 70 + e * 70;
    if (k > 0 && k < 1) _dotted(c, Offset(x + tp.width / 2, y - 14), Offset(x + tp.width / 2, y - 2), _a(_blue, .7), 3);
    (k <= 0 ? _tp(word[i], 30, _dim, FontWeight.w200, 0, true) : (k < 1 ? _tp(word[i], 30, _green, FontWeight.w200, 0, true) : tp)).paint(c, Offset(x, y));
    x += tp.width + 2;
  }
  _ln(c, const Offset(8, base + 34), const Offset(148, base + 34), _dim, 1);
  _lb(c, 'OFFSET ${(_off(p) * 1000).round()}', const Offset(8, 104), _blue, 6.5);
  _dirTag(c, p, const Offset(100, 108));
}

// 8 birds off a wire: why you stagger -- a flock leaving at once looks fake; one by one looks alive.
void _t08(Canvas c, _P p, double t) {
  double wire(double x) => 70 + 10 * math.sin((x - 10) / 136 * math.pi);
  for (final x in [10.0, 146.0]) {
    _ln(c, Offset(x, 66), Offset(x, 118), _white, 1);
    _ln(c, Offset(x - 5, 70), Offset(x + 5, 70), _white, 1);
  }
  _pl(c, [for (var x = 10.0; x <= 146; x += 4) Offset(x, wire(x))], _white, .9);
  const n = 8;
  for (var i = 0; i < n; i++) {
    final x = 22.0 + i * 15;
    final k = _cas(_k(p, i, n), n, _off(p), t, .45);
    if (k <= 0) {
      final q = Offset(x, wire(x) - 4);
      c.drawOval(Rect.fromCenter(center: q, width: 7, height: 6), _s(_white, 1));
      _ring(c, q + const Offset(3.5, -4), 1.6, _white, .9);
      _ln(c, q + const Offset(5, -4), q + const Offset(7, -3.5), _green, .9);
      _ln(c, q + const Offset(-3, 1), q + const Offset(-6, 3), _white, .9);
    } else {
      final q = Offset(x + k * 30, wire(x) - 4 - k * 70);
      if (q.dy < -6) continue;
      final fl = math.sin(t * 12 + i * 2) * 4;
      _pl(c, [q + Offset(-6, -fl), q, q + Offset(6, -fl)], k < .5 ? _blue : _a(_white, 1 - k * .6), 1);
      _dotted(c, Offset(x, wire(x) - 4), q, _a(_dim, .9), 5);
    }
  }
  _offTag(c, p, const Offset(8, 8));
  _dirTag(c, p, const Offset(100, 12));
}

// 9 rising blocks: wire columns grow out of the floor along the order.
void _t09(Canvas c, _P p, double t) {
  const o = Offset(78, 50), n = 4, cs = 14.0;
  _isoGrid(c, o, n, cs, _dim);
  final items = <(int, int)>[];
  for (var s = 0; s < 2 * n - 1; s++) {
    for (var i = 0; i < n; i++) {
      final j = s - i;
      if (j >= 0 && j < n) items.add((i, j));
    }
  }
  final total = items.length;
  for (var idx = 0; idx < total; idx++) {
    final (i, j) = items[idx];
    final k = _cas(_k(p, i + j, 2 * n - 1), 2 * n - 1, _off(p), t, 1);
    final h = 2 + _ease(k) * 24;
    _isoBox(c, o, i * cs + 2.5, j * cs + 2.5, 0, cs - 5, cs - 5, h, k >= 1 ? _white : (k > 0 ? _blue : _dim), .8);
  }
  _offTag(c, p, const Offset(8, 8));
  _dirTag(c, p, const Offset(100, 108));
}

// 10 clock row: five small clocks, each hand a step behind; spin to set the step.
void _t10(Canvas c, _P p, double t) {
  const n = 5;
  for (var i = 0; i < n; i++) {
    final o = Offset(18 + i * 30.0, 56);
    _ring(c, o, 12, _white, 1);
    for (var m = 0; m < 12; m++) {
      final a = m * math.pi / 6;
      _dot(c, o + Offset(math.cos(a), math.sin(a)) * 10, .45, _dim);
    }
    final k = _cas(_k(p, i, n), n, _off(p), t, 1);
    final a = -math.pi / 2 + _ease(k) * 2 * math.pi * .75;
    _ln(c, o, o + Offset(math.cos(a), math.sin(a)) * 9, k > 0 && k < 1 ? _blue : _white, 1.1);
    _ln(c, o, o + const Offset(0, -5), _dim, 1);
    _dot(c, o, 1.2, _white);
    _lb(c, '+${(_k(p, i, n) * _off(p) * 1000).round()}', o + const Offset(0, 16), _red, 6, .5);
  }
  final arc = 16 + p.a * 40;
  c.drawArc(Rect.fromCircle(center: const Offset(78, 56), radius: 46), -math.pi / 2 - arc / 80, arc / 40, false, _s(_a(_blue, .5), .8));
  _lb(c, 'SPIN', const Offset(8, 8), _blue, 6.5);
  _dirTag(c, p, const Offset(100, 12));
}

// 11 delay chain: one trigger runs through delay cells; each output fires one cell later.
void _t11(Canvas c, _P p, double t) {
  const y = 40.0, n = 5;
  c.drawRect(const Rect.fromLTWH(6, y - 9, 18, 18), _s(_white, 1));
  _pl(c, const [Offset(10, y + 4), Offset(15, y + 4), Offset(15, y - 4), Offset(20, y - 4)], _white, .9);
  for (var i = 0; i < n; i++) {
    final x = 32.0 + i * 24;
    _ln(c, Offset(x - 8, y), Offset(x, y), _dim, 1);
    _pl(c, [Offset(x, y - 8), Offset(x + 14, y), Offset(x, y + 8)], _blue, 1, true);
    _lb(c, 'Δ', Offset(x + 2, y - 4), _blue, 7);
    final k = _cas(_k(p, i, n), n, _off(p), t, .5);
    final fired = k > 0;
    _ln(c, Offset(x + 7, y + 4), Offset(x + 7, y + 30), _dim, .8);
    _ring(c, Offset(x + 7, y + 36), 5, fired ? _green : _dim, 1);
    if (fired && k < 1) _dot(c, Offset(x + 7, y + 36), 2.4, _green);
    _lb(c, '${_k(p, i, n) + 1}', Offset(x + 7, y + 46), _red, 6.5, .5);
  }
  for (var i = 0; i <= n; i++) {
    _dot(c, Offset(14 + i * 24.0, 106), 1, _white);
    _ln(c, Offset(14 + i * 24.0, 103), Offset(14 + i * 24.0, 109), _dim, .7);
  }
  _lb(c, '${(_off(p) * 1000).round()}MS', const Offset(118, 8), _blue, 6.5);
}

// 12 envelope ridge: envelopes stacked in depth, each starting later -- the OP-1 MAX/MIN ridge.
void _t12(Canvas c, _P p, double t) {
  const n = 9;
  const cols = [_blue, _green, _white, _red, _purple];
  _lb(c, 'MAX', const Offset(6, 10), _white, 6.5);
  _lb(c, 'MIN', const Offset(6, 98), _white, 6.5);
  _ln(c, const Offset(24, 12), const Offset(24, 100), _dim, .8);
  for (var i = n - 1; i >= 0; i--) {
    final o = Offset(28 + i * 5.0, 30 + i * 7.0);
    final k = _k(p, i, n);
    final st = k * _off(p) * 30;
    final phase = (t * 20) % 40;
    final pts = <Offset>[];
    for (var x = 0.0; x <= 90; x += 3) {
      final u = x - st;
      final env = u < 0 ? 0.0 : (u < 8 ? u / 8 : math.max(0.4, 1 - (u - 8) / 30) * (u > 60 ? math.max(0, 1 - (u - 60) / 14) : 1));
      pts.add(o + Offset(x, -env * 26));
    }
    _pl(c, pts, cols[i % 5], .9);
    _dotted(c, o + Offset(st, 0), o + Offset(st, -26), _a(_dim, .9), 3);
    if (i == 0) _ln(c, o + Offset(phase, 0), o + Offset(phase, -30), _a(_white, .4), .7);
  }
  _lb(c, '${(_off(p) * 1000).round()}', const Offset(150, 8), _blue, 7, 1);
  _dirTag(c, p, const Offset(100, 106));
}

// 13 ant column: ants set off one after another along a path, each with a crumb.
void _t13(Canvas c, _P p, double t) {
  Offset path(double u) => Offset(14 + u * 128, 70 + math.sin(u * 5) * 18);
  _pl(c, [for (var u = 0.0; u <= 1; u += .02) path(u)], _a(_dim, .9), .8);
  c.drawArc(Rect.fromCenter(center: const Offset(14, 76), width: 20, height: 14), math.pi, math.pi, false, _s(_white, 1));
  _dot(c, const Offset(146, 74), 2, _red);
  const n = 7;
  for (var i = 0; i < n; i++) {
    final sp = .08 + p.a * .1;
    final k = _cl(.5 + (n - 1) / 2 * sp - _k(p, i, n) * sp + math.sin(t * .5) * .05);
    final u = _rev(p) ? 1 - k : k;
    final q = path(u);
    final d = path(math.min(1, u + .01)) - path(math.max(0, u - .01));
    final a = math.atan2(d.dy, d.dx);
    c.save();
    c.translate(q.dx, q.dy - 3);
    c.rotate(a);
    for (final x in [-4.0, 0.0, 3.5]) {
      _ring(c, Offset(x, 0), x == 0 ? 1.2 : 1.6, k > 0 && k < 1 ? _white : _dim, .8);
    }
    final lg = math.sin(t * 14 + i) * 1.5;
    for (final x in [-1.5, 0.0, 1.5]) {
      _ln(c, Offset(x, 0), Offset(x + lg, 3), _white, .6);
      _ln(c, Offset(x, 0), Offset(x - lg, -3), _white, .6);
    }
    if (k > 0 && k < 1) _dot(c, const Offset(6.5, -1), 1.1, _green);
    c.restore();
  }
  _offTag(c, p, const Offset(8, 8));
  _dirTag(c, p, const Offset(100, 12));
}

// 14 pendulum wave: twelve bobs, each a phase behind, making a travelling snake.
void _t14(Canvas c, _P p, double t) {
  const n = 12, top = 12.0;
  _ln(c, const Offset(8, top), const Offset(148, top), _white, 1.2);
  final bobs = <Offset>[];
  for (var i = 0; i < n; i++) {
    final px = 14.0 + i * 11.6;
    final k = _k(p, i, n);
    final a = math.sin(t * 2.2 - k * _off(p) * 3) * .45;
    final l = 70.0 + i * 2;
    final b = Offset(px + math.sin(a) * l * .5, top + math.cos(a) * l);
    _ln(c, Offset(px, top), b, _a(_white, .6), .7);
    bobs.add(b);
  }
  _pl(c, bobs, _a(_green, .6), .8);
  for (var i = 0; i < n; i++) {
    _ring(c, bobs[i], 3, i == 0 ? _blue : _white, 1);
  }
  _lb(c, 'PHASE ${(_off(p) * 1000).round()}', const Offset(8, 106), _blue, 6.5);
  _lb(c, _ordName(p.c), const Offset(148, 106), _red, 6.5, 1);
}

// 15 dealt cards: the deck deals into a fan, card by card.
void _t15(Canvas c, _P p, double t) {
  const deck = Offset(20, 96), n = 7;
  for (var k = 0; k < 4; k++) {
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: deck + Offset(k * .8, -k * 1.6), width: 18, height: 26), const Radius.circular(2.5)), _s(_dim, .9));
  }
  for (var i = 0; i < n; i++) {
    final k = _cas(_k(p, i, n), n, _off(p), t, .8);
    final dest = Offset(48 + i * 15.0, 52 - math.sin(i / (n - 1) * math.pi) * 10);
    final rotD = (i - (n - 1) / 2) * .14;
    final e = _ease(k);
    final pos = Offset.lerp(deck, dest, e)! + Offset(0, -math.sin(e * math.pi) * 22);
    if (k <= 0) continue;
    c.save();
    c.translate(pos.dx, pos.dy);
    c.rotate(rotD * e + (1 - e) * 1.2);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 18, height: 26), const Radius.circular(2.5)), _s(k < 1 ? _blue : _white, 1));
    _lb(c, '${_k(p, i, n) + 1}', const Offset(-7, -11), _red, 6.5);
    _dot(c, Offset.zero, 1, k < 1 ? _blue : _white);
    c.restore();
  }
  _offTag(c, p, const Offset(8, 8));
  _dirTag(c, p, const Offset(100, 12));
}

// 16 street lamps: driving at night, the lamps come on ahead of you one by one toward the skyline.
void _t16(Canvas c, _P p, double t) {
  const vp = Offset(78, 44);
  _pl(c, const [Offset(52, 44), Offset(56, 38), Offset(60, 44), Offset(62, 34), Offset(66, 34), Offset(68, 44), Offset(74, 40), Offset(80, 44), Offset(84, 30), Offset(88, 44), Offset(96, 38), Offset(100, 44)], _a(_white, .6), .8);
  _ln(c, const Offset(0, 44), const Offset(_pw, 44), _dim, .8);
  _ln(c, const Offset(20, 120), vp, _white, 1);
  _ln(c, const Offset(136, 120), vp, _white, 1);
  final sh = (t * 1.2) % 1;
  for (var k = 0; k < 6; k++) {
    final u = math.pow((k + sh) / 6, 2).toDouble();
    final a = Offset.lerp(vp, const Offset(78, 120), u)!, b = Offset.lerp(vp, const Offset(78, 120), math.min(1, u + .04))!;
    _ln(c, a, b, _dim, 1);
  }
  const n = 6;
  for (var i = 0; i < n; i++) {
    final u = 1 - i / n * .85;
    final k = _cas(_k(p, i, n), n, _off(p), t, .4);
    for (final side in [-1.0, 1.0]) {
      final base = Offset.lerp(vp, Offset(78 + side * 70, 120), u)!;
      final h = 34 * u;
      final head = base + Offset(-side * h * .25, -h);
      _ln(c, base, base + Offset(0, -h), _white, .9);
      _ln(c, base + Offset(0, -h), head, _white, .9);
      if (k > 0) {
        _dot(c, head, 1.3 * u + .4, _green);
        if (k < 1 || u > .6) {
          for (final a in [.6, 1.57, 2.5]) {
            _ln(c, head + Offset(math.cos(a), math.sin(a)) * 3 * u, head + Offset(math.cos(a), math.sin(a)) * 8 * u, _a(_green, .8), .7);
          }
        }
      }
    }
  }
  _pl(c, const [Offset(0, 108), Offset(30, 104), Offset(126, 104), Offset(_pw, 108)], _blue, 1);
  _lb(c, 'OFFSET ${(_off(p) * 1000).round()}', const Offset(8, 8), _blue, 6.5);
  _lb(c, _ordName(p.c), const Offset(148, 8), _red, 6.5, 1);
}

// 17 echo numerals: the same counter printed five times, each a beat behind.
void _t17(Canvas c, _P p, double t) {
  const n = 4;
  for (var i = n - 1; i >= 0; i--) {
    final lag = _k(p, i, n) * _off(p);
    final v = ((t - lag) * 4).floor() % 100;
    final o = Offset(8 + i * 34.0, 12 + i * 16.0);
    _num(c, v.toString().padLeft(2, '0'), o, 40 - i * 5, i == 0 ? _white : _a([_blue, _green, _purple][(i - 1) % 3], 1 - i * .12));
  }
  _lb(c, 'LAG ${(_off(p) * 1000).round()}', const Offset(112, 104), _blue, 6.5);
  _dirTag(c, p, const Offset(8, 108));
}

// 18 rowing eight: oars dip in turn; with no offset the boat rows in perfect sync.
void _t18(Canvas c, _P p, double t) {
  const y = 64.0;
  final hull = Path()
    ..moveTo(8, y)
    ..quadraticBezierTo(78, y + 16, 148, y - 2)
    ..lineTo(8, y);
  c.drawPath(hull, _s(_white, 1.1));
  for (var k = 0; k < 3; k++) {
    final yy = y + 24 + k * 9.0;
    _pl(c, [for (var x = 0.0; x <= _pw; x += 6) Offset(x, yy + math.sin(x * .15 - t * 2 + k) * 1.2)], _a(_blue, .45 - k * .1), .8);
  }
  const n = 6;
  for (var i = 0; i < n; i++) {
    final x = 26.0 + i * 20;
    final ph = t * 2.4 - _k(p, i, n) * _off(p) * 3;
    final a = math.sin(ph) * .5;
    final dip = math.cos(ph) > 0;
    _ring(c, Offset(x, y - 12), 3, _white, 1);
    _ln(c, Offset(x, y - 9), Offset(x + a * 4, y - 1), _white, 1);
    final oarEnd = Offset(x - 2 + math.sin(a) * 26, y + 16 + (dip ? 4 : -2));
    _ln(c, Offset(x + a * 4, y - 4), oarEnd, dip ? _green : _white, 1);
    if (dip) c.drawOval(Rect.fromCenter(center: oarEnd + const Offset(0, 2), width: 8, height: 3), _s(_a(_white, .6), .7));
  }
  _offTag(c, p, const Offset(8, 8));
  _lb(c, _ordName(p.c), const Offset(148, 8), _red, 6.5, 1);
}

// 19 draw the order: draw a path through the dots; the dots light in the order your path visits them.
void _t19(Canvas c, _P p, double t) {
  const cols = 6, rows = 4;
  final dots = [for (var j = 0; j < rows; j++) for (var i = 0; i < cols; i++) Offset(22 + i * 22.0, 26 + j * 22.0)];
  final n = dots.length;
  List<Offset> path = p.pts.length > 2
      ? p.pts
      : [for (var a = 0.0; a < 4 * math.pi; a += .2) Offset(78 + math.cos(a) * (4 + a * 5), 59 + math.sin(a) * (4 + a * 3))];
  _pl(c, path, _a(_purple, .7), .9);
  final rank = <int>[];
  for (var d = 0; d < n; d++) {
    var best = 0;
    var bd = 1e9;
    for (var k = 0; k < path.length; k++) {
      final dd = (path[k] - dots[d]).distance;
      if (dd < bd) {
        bd = dd;
        best = k;
      }
    }
    rank.add(best);
  }
  final sorted = List<int>.generate(n, (i) => i)..sort((a, b) => rank[a].compareTo(rank[b]));
  for (var o = 0; o < n; o++) {
    final d = sorted[o];
    final k = _cas(o, n, _off(p) * .3, t, .6);
    _ring(c, dots[d], 4, k > 0 ? (k < 1 ? _green : _white) : _dim, 1);
    if (k > 0 && k < 1) _dot(c, dots[d], 2.2, _green);
    _lb(c, '${o + 1}', dots[d] + const Offset(5, -10), _red, 5.5);
  }
  if (p.touch != null) _ring(c, p.touch!, 6, _white, .8);
  _lb(c, 'DRAW', const Offset(8, 108), _purple, 6.5);
  _lb(c, '${(_off(p) * 300).round()}MS', const Offset(148, 108), _blue, 6.5, 1);
}

// 20 endless smear (pushed): 160 lines, offsets so long the cascade wraps into a spiral and runs off the frame.
void _t20(Canvas c, _P p, double t) {
  const o = Offset(78, 60), n = 160;
  for (var i = 0; i < n; i++) {
    final k = _k(p, i, n);
    final lag = k * (.005 + p.a * .06);
    final ph = (t * .6 - lag);
    final a = i * .19 + ph;
    final r = 4 + i * .55 * (.4 + p.a);
    final q = o + Offset(math.cos(a) * r, math.sin(a) * r * .8);
    final u = Offset(math.cos(a + 1.3), math.sin(a + 1.3)) * (3 + math.sin(ph * 3) * 2);
    _ln(c, q - u, q + u, _a([_blue, _green, _white, _red][i % 4], .85), .8);
  }
  _num(c, '${(.005 + p.a * .06) * n * 1000 ~/ 1}', Offset(_pw - 6, 4), 26, _white, 1);
  _lb(c, 'MS TOTAL', Offset(_pw - 6, 32), _blue, 6, 1);
}
