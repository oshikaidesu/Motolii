part of 'op_10.dart';

// Shared monoline figures for the OP 10 sheets. Angles are radians from straight down.

Offset _polar(Offset o, double len, double ang) => o + Offset(math.sin(ang) * len, math.cos(ang) * len);

/// A line person standing on [foot]. Returns the hand positions (left, right).
(Offset, Offset) _stick(Canvas c, Offset foot, double h, Color col,
    {double armL = .5, double armR = -.5, double legL = .25, double legR = -.25, double lean = 0, double w = 1.2}) {
  final leg = h * .42, body = h * .36, head = h * .1, arm = h * .3;
  final hip = foot - Offset(0, leg * math.cos(legL.abs() * .8));
  final neck = _polar(hip, body, math.pi + lean);
  _ln(c, hip, _polar(hip, leg, legL), col, w);
  _ln(c, hip, _polar(hip, leg, legR), col, w);
  _ln(c, hip, neck, col, w);
  final hl = _polar(neck, arm, armL), hr = _polar(neck, arm, armR);
  _ln(c, neck, hl, col, w);
  _ln(c, neck, hr, col, w);
  _ring(c, _polar(neck, head + 1, math.pi + lean), head, col, w);
  return (hl, hr);
}

/// A line fish facing +x (or -x when [flip]).
void _fish(Canvas c, Offset at, double len, Color col, {bool flip = false, double wag = 0, double w = 1}) {
  final d = flip ? -1.0 : 1.0;
  c.drawOval(Rect.fromCenter(center: at, width: len, height: len * .45), _s(col, w));
  final tb = at - Offset(d * len * .5, 0);
  final tip = tb - Offset(d * len * .3, wag * len * .2);
  _pl(c, [tb, tip + Offset(0, -len * .22), tip + Offset(0, len * .22), tb], col, w: w);
  _dot(c, at + Offset(d * len * .26, -len * .05), .9, col);
}

/// A cloud outline made of arcs sitting on [base], width [wd].
void _cloud(Canvas c, Offset base, double wd, Color col, [double w = 1]) {
  final p = Path()..moveTo(base.dx - wd / 2, base.dy);
  p.arcToPoint(base + Offset(-wd * .2, -wd * .22), radius: Radius.circular(wd * .18));
  p.arcToPoint(base + Offset(wd * .22, -wd * .2), radius: Radius.circular(wd * .24));
  p.arcToPoint(base + Offset(wd / 2, 0), radius: Radius.circular(wd * .16));
  p.close();
  c.drawPath(p, _s(col, w));
}

/// A sail boat outline floating at [at] (waterline centre).
void _boat(Canvas c, Offset at, double k, Color col, {double tilt = 0}) {
  c.save();
  c.translate(at.dx, at.dy);
  c.rotate(tilt);
  _pl(c, [Offset(-14 * k, -4 * k), Offset(14 * k, -4 * k), Offset(9 * k, 3 * k), Offset(-9 * k, 3 * k)], col, close: true);
  _ln(c, Offset(0, -4 * k), Offset(0, -26 * k), col);
  _pl(c, [Offset(1.5 * k, -25 * k), Offset(12 * k, -7 * k), Offset(1.5 * k, -7 * k)], col, close: true);
  _pl(c, [Offset(-1.5 * k, -22 * k), Offset(-9 * k, -7 * k), Offset(-1.5 * k, -7 * k)], col, close: true);
  c.restore();
}

/// A ship's anchor glyph hanging from its ring at [ring].
void _anchor(Canvas c, Offset ring, double k, Color col) {
  _ring(c, ring, 2.2 * k, col);
  final bot = ring + Offset(0, 16 * k);
  _ln(c, ring + Offset(0, 2.2 * k), bot, col);
  _ln(c, ring + Offset(-5 * k, 6 * k), ring + Offset(5 * k, 6 * k), col);
  c.drawArc(Rect.fromCircle(center: bot - Offset(0, 6 * k), radius: 7 * k), .25, math.pi - .5, false, _s(col));
  _ln(c, bot + Offset(-6.6 * k, -3.6 * k), bot + Offset(-8.2 * k, -6 * k), col);
  _ln(c, bot + Offset(6.6 * k, -3.6 * k), bot + Offset(8.2 * k, -6 * k), col);
}

/// A four-point star sparkle.
void _spark(Canvas c, Offset p, double r, Color col, [double w = 1]) {
  _ln(c, p - Offset(r, 0), p + Offset(r, 0), col, w);
  _ln(c, p - Offset(0, r), p + Offset(0, r), col, w);
}

/// Small tick-marked arc readout (an instrument dial seen, not a knob to turn).
void _gauge(Canvas c, Offset o, double r, double v, Color col) {
  c.drawArc(Rect.fromCircle(center: o, radius: r), math.pi, math.pi, false, _s(_dg, 1));
  for (var i = 0; i <= 8; i++) {
    final an = math.pi + i / 8 * math.pi;
    _ln(c, o + Offset(math.cos(an), math.sin(an)) * (r - 2), o + Offset(math.cos(an), math.sin(an)) * r, _mg, .8);
  }
  final an = math.pi + v.clamp(0.0, 1.0) * math.pi;
  _ln(c, o, o + Offset(math.cos(an), math.sin(an)) * (r - 1), col);
  _dot(c, o, 1.4, col);
}
