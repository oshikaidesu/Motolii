// Part of op_06_mask.dart (cut there by size, as it was).
part of 'op_06.dart';

// 16. OP-1 flow: subject → expand amp → blur stack → invert wheel; a pulse runs through it.
void _m16(Canvas c, Size z, OpS s) {
  const y = 52.0;
  c.drawRect(const Rect.fromLTWH(6, 38, 26, 28), _st(_wh, 1));
  _path(c, _star(const Offset(19, 52), 8, 3.6, 5), _wh, 1);
  _ln(c, 32, y, 42, y, _dg, 1);
  _path(c, _poly([const Offset(42, 36), const Offset(70, 52), const Offset(42, 68)], close: true), _gr, 1.2);
  _t(c, _n(s.b), 45, 46, _gr, size: 11, w: FontWeight.w300, ls: 0);
  _ln(c, 70, y, 80, y, _dg, 1);
  for (var i = 2; i >= 0; i--) {
    c.drawRect(Rect.fromLTWH(80 + i * 3.0, 38 - i * 3.0, 26, 28), Paint()..color = _k);
    c.drawRect(Rect.fromLTWH(80 + i * 3.0, 38 - i * 3.0, 26, 28), _st(_al(_bl, 1 - i * .3), 1));
  }
  final sp = 1 + s.a * 6;
  for (var r = 1; r <= 3; r++) {
    _ring(c, const Offset(93, 52), r * sp * .7, _al(_bl, 1 - r * .25), .8);
  }
  _ln(c, 106, y, 120, y, _dg, 1);
  const w = Offset(134, 52);
  _ring(c, w, 12, _rd, 1.1);
  for (var i = 0; i < 6; i++) {
    _dot(c, _pol(w, 7, i * math.pi / 3), i == (s.inv ? 3 : 0) ? _rd : _dg, 1.6);
  }
  final pu = (s.t * .5) % 1;
  _dot(c, Offset(_lp(32, 120, pu), y), _wh, 1.4);
  for (final (x, l, col) in [(19.0, 'SUBJ', _wh), (52.0, 'EXPAND', _gr), (93.0, 'FEATHER', _bl), (134.0, 'INVERT', _rd)]) {
    _t(c, l, x, 80, col, size: 6.5, mid: true);
  }
  _num(c, _n(s.a), 93, 92, _bl, 18);
}

// 17. A hedgehog: spine length is softness and can go absurdly long; the body size is the expand.
void _m17(Canvas c, Size z, OpS s) {
  final o = const Offset(82, 74);
  final r = 12 + s.b * 16, sp = 2 + s.a * 46;
  final br = math.sin(s.t * 2.2) * .6;
  for (var i = 0; i < 46; i++) {
    final a = math.pi + i / 45 * math.pi;
    final p0 = o + Offset(math.cos(a) * r * 1.35, math.sin(a) * r);
    final l = sp * (.6 + .4 * _h(i)) + br;
    final p1 = p0 + Offset(math.cos(a + (_h(i + 3) - .5) * .4), math.sin(a + (_h(i + 3) - .5) * .4)) * l;
    _ln(c, p0.dx, p0.dy, p1.dx, p1.dy, _al(_bl, .5 + .5 * _h(i + 7)), .8);
  }
  _path(c, Path()..addArc(Rect.fromCenter(center: o, width: r * 2.7, height: r * 2), math.pi, math.pi), s.inv ? _rd : _gr, 1.3);
  _ln(c, o.dx - r * 1.35, o.dy, o.dx + r * 1.35, o.dy, _gr, 1);
  final nose = o + Offset(r * 1.35, -2);
  _path(c, _poly([nose + const Offset(0, -6), nose + const Offset(12, 0), nose + const Offset(0, 2)]), _wh, 1.1);
  _dot(c, nose + const Offset(12, 0), _wh, 1.6);
  _dot(c, nose + const Offset(3, -3), _wh, 1);
  for (final fx in [-.7, .5]) {
    _ln(c, o.dx + fx * r, o.dy, o.dx + fx * r, o.dy + 6, _wh, 1);
  }
  _ln(c, 4, o.dy + 6, 152, o.dy + 6, _dg, 1);
  _t(c, 'SPINE', 6, 98, _bl);
  _num(c, _n(s.a), 6, 106, _bl, 12);
}

// 18. Max-style scanline field: each line lifts where the mask is; the shoulder of the bump is the softness.
void _m18(Canvas c, Size z, OpS s) {
  final cx = 78.0 + math.sin(s.t * .5) * 6, r = 6 + s.b * 34, soft = 2 + s.a * 30;
  for (var j = 0; j < 14; j++) {
    final y = 18 + j * 7.0;
    final pts = <Offset>[];
    for (var x = 6.0; x <= 150; x += 2) {
      final dy = (y - 64) * 1.2;
      final d = math.sqrt((x - cx) * (x - cx) + dy * dy);
      var al = ((r + soft / 2 - d) / soft).clamp(0.0, 1.0);
      al = al * al * (3 - 2 * al);
      if (s.inv) al = 1 - al;
      pts.add(Offset(x, y - al * 14 - math.sin(x * .3 + j + s.t * 2) * .4));
    }
    final col = j % 4 == 0 ? _bl : _wh;
    final ground = _poly([...pts, const Offset(150, 120), const Offset(6, 120)], close: true);
    c.drawPath(ground, _fl(_k));
    _path(c, _poly(pts), _al(col, .9), .9);
  }
  _t(c, 'RADIUS ${_n(s.b)}', 6, 4, _gr, size: 6.5);
  _t(c, 'SHOULDER ${_n(s.a)}', 150, 4, _bl, size: 6.5, right: true);
}

// 19. Dot matrix: draw to move the mask; dot size = coverage per cell.
void _m19(Canvas c, Size z, OpS s) {
  final o = s.trail.isEmpty ? const Offset(70, 54) : s.trail.last;
  final r = 14 + s.b * 0 + 16, soft = 4 + s.a * 30;
  for (var j = 0; j < 9; j++) {
    for (var i = 0; i < 15; i++) {
      final p = Offset(8 + i * 10.0, 10 + j * 10.0);
      var al = ((r + soft / 2 - (p - o).distance) / soft).clamp(0.0, 1.0);
      if (s.inv) al = 1 - al;
      if (al > .97) {
        c.drawRect(Rect.fromCenter(center: p, width: 7, height: 7), _st(_gr, .8));
      }
      if (al < .03) {
        _dot(c, p, _dg, .6);
      } else {
        _dot(c, p, al > .97 ? _wh : _bl, .6 + al * 2.2);
      }
    }
  }
  _ln(c, 4, 104, 152, 104, _dg, .8);
  _t(c, 'FTHR ${_n(s.a)}', 6, 108, _bl, size: 6.5);
  _t(c, 'INV ${s.inv ? 'ON' : 'OFF'}', 150, 108, s.inv ? _rd : _dg, size: 6.5, right: true);
}

// 20. A lighthouse: the beam is the reveal, its fanned rays are the softness — pushed to a storm of rays.
void _m20(Canvas c, Size z, OpS s) {
  const lamp = Offset(36, 40);
  final ang = s.a * math.pi * 2 + s.t * .3, half = .06 + s.b * .5;
  final rays = 2 + (s.b * 16).round();
  for (var i = -rays; i <= rays; i++) {
    final u = i / rays;
    final a = ang + u * (half + .02 * rays);
    final soft = u.abs() > .5;
    final e = _pol(lamp, 200, a);
    _ln(c, lamp.dx, lamp.dy, e.dx, e.dy, soft ? _al(_bl, (1 - u.abs()) * 1.6) : (s.inv ? _rd : _gr), soft ? .6 : .9);
  }
  _path(c, _poly([const Offset(28, 104), const Offset(31, 46), const Offset(41, 46), const Offset(44, 104)]), _wh, 1.1);
  for (final y in const <double>[62, 78, 94]) {
    _ln(c, 30.3 + (104 - y) * .05, y, 41.7 - (104 - y) * .05, y, _wh, .8);
  }
  c.drawRect(const Rect.fromLTWH(30, 34, 12, 12), Paint()..color = _k);
  c.drawRect(const Rect.fromLTWH(30, 34, 12, 12), _st(_wh, 1));
  _path(c, _poly([const Offset(28, 34), const Offset(36, 26), const Offset(44, 34)]), _wh, 1);
  for (var w = 0; w < 3; w++) {
    final pts = [for (var x = 0.0; x <= 156; x += 3) Offset(x, 106 + w * 5 + math.sin(x * .12 + s.t * 2 + w) * 1.4)];
    _path(c, _poly(pts), _al(_wh, .5 - w * .12), .8);
  }
  _t(c, 'BEAM', 150, 6, _gr, right: true);
  _num(c, _n(s.b), 150, 14, _gr, 22, true);
}
