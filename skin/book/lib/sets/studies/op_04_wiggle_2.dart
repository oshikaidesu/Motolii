// Part of op_04_wiggle.dart (cut there by size, as it was).
part of 'op_04_wiggle.dart';

// 19 Maraca monkey: rub faster, it shakes faster.
void _maraca(Canvas c, Size s, OpV v, double t) {
  const hd = Offset(78, 52);
  ci(c, hd, 14, _w, 1.1);
  ci(c, hd + const Offset(-16, -2), 5, _w);
  ci(c, hd + const Offset(16, -2), 5, _w);
  c.drawOval(Rect.fromCenter(center: hd + const Offset(0, 6), width: 16, height: 10), st(_w, .9));
  dot(c, hd + const Offset(-5, -4), 1.3, _w);
  dot(c, hd + const Offset(5, -4), 1.3, _w);
  c.drawArc(Rect.fromCircle(center: hd + const Offset(0, 6), radius: 4), .3, math.pi - .6, false, st(_w, .8));
  pl(c, [hd + const Offset(-8, 12), const Offset(66, 96), const Offset(90, 96), hd + const Offset(8, 12)], _w);
  for (final side in [-1.0, 1.0]) {
    final sh = hd + Offset(side * 10, 20);
    final ang = -math.pi / 2 + side * .9 + wv(t, v, side + 2) * .9;
    final hand = pol(sh, 22, ang + side * .3);
    pl(c, [sh, hand], _w, 1);
    final top = pol(hand, 14, ang);
    ln(c, hand, ol(hand, top, .4), pc(v, 0), 1);
    final mc = ol(hand, top, .8);
    c.save();
    c.translate(mc.dx, mc.dy);
    c.rotate(ang);
    c.drawOval(const Rect.fromLTWH(-6, -5, 14, 10), st(pc(v, 0), 1));
    for (var k = 0; k < 3; k++) {
      dot(c, Offset(-1.0 + k * 3, (k - 1) * 2.0 + wv(t + k, v, 9) * 2), .9, pc(v, 2));
    }
    c.restore();
    if (v[1] > .15) lb(c, 'CHK', pol(top, 12, ang) - const Offset(6, 3), al(K.white, v[1]), h: 4);
  }
  _three(c, v, const Offset(6, 96));
}

// 20 Static: the extreme, a whole TV of jitter and a channel number that can't hold still.
void _static(Canvas c, Size s, OpV v, double t) {
  final r = RRect.fromRectAndRadius(const Rect.fromLTWH(10, 18, 136, 88), const Radius.circular(10));
  c.drawRRect(r, st(_w, 1.2));
  ln(c, const Offset(66, 20), const Offset(52, 4), _w, .9);
  ln(c, const Offset(90, 20), const Offset(106, 4), _w, .9);
  c.save();
  c.clipRRect(r.deflate(4));
  final amp = .3 + v[1] * 2.7;
  for (var row = 0; row < 25; row++) {
    final y = 24 + row * 3.4;
    final tear = wv(t + row * .02, v, 3) * 26 * amp;
    final fr = (t * (4 + 40 * v[0])).floor();
    for (var k = 0; k < 9; k++) {
      final h = math.sin((fr * 13 + row * 7 + k) * 12.9898) * 43758.5453;
      final f = h - h.floorToDouble();
      final x = 14 + ((k * 17 + f * 30 + tear) % 130);
      ln(c, Offset(x, y), Offset(x + 3 + f * 10, y), al(K.white, .15 + .45 * f), .8);
    }
  }
  final j = Offset(wv(t, v, 7) * 10 * amp, wv(t, v, 8) * 6 * amp);
  nm(c, d2(v[1]), const Offset(78, 44) + j, 40, pc(v, 1), ax: .5);
  nm(c, d2(v[1]), const Offset(80, 44) - j * .5, 40, al(K.red, .7), ax: .5);
  c.restore();
  lb(c, 'CH', const Offset(14, 10), K.grey, h: 5);
  lb(c, 'FREQ ${d2(v[0])}  SMTH ${d2(v[2])}', const Offset(150, 114), _w, ax: 1, h: 4);
}

final wigglePanels = <OpPanel>[
  OpPanel('hand-held viewfinder', 'machine · effect · sensible', _cam),
  OpPanel('shivering chihuahua', 'animal · effect · playful', _dog, g: const OpRub(1, 0)),
  OpPanel('polygraph pens', 'machine · history · diagram', _poly),
  OpPanel('firefly trail', 'cosmic+insect · effect 2D', _firefly, g: const OpPinch(1, 0)),
  OpPanel('heat haze mesa', 'landscape · effect · drawing', _haze, g: const OpRub(1, 2)),
  OpPanel('BOIL line boil', 'typographic · effect · type', _boil, g: const OpDrag(0, 1)),
  OpPanel('S&H lane: rate depth slew', 'Max device · mechanism', _sh),
  OpPanel('spectrum hump', 'diagram · mechanism · numeral', _spec, g: const OpPinch(0, 2)),
  OpPanel('fishing bobber', 'landscape · effect · drawing', _bobber, g: const OpFlick(1, 0)),
  OpPanel('drunk walker', 'isometric+figure · effect', _drunk),
  OpPanel('quake city', 'landscape · effect · rub', _quake, g: const OpRub(1, 0)),
  OpPanel('tumbling satellite', 'cosmic · effect · drawing', _sat, g: const OpSpin(0, Offset(.62, .42), 1)),
  OpPanel('hummingbird hover', 'animal · effect · drawing', _bird),
  OpPanel('coffee cup from above', 'object · effect · rub table', _cup, g: const OpRub(1, 0)),
  OpPanel('scope trace', 'instrument · signal · numeral', _scope, g: const OpDrag(0, 1)),
  OpPanel('scribble comes alive', 'your line · effect · draw', _scribble, g: const OpDraw(_fitScribble)),
  OpPanel('jitter cubes', 'isometric · effect · field', _cubes, g: const OpPinch(1, 0)),
  OpPanel('leaves in the wind', 'landscape · effect · rub', _leaf, g: const OpRub(0, 1)),
  OpPanel('maraca monkey', 'character · effect · playful', _maraca, g: const OpRub(0, 1)),
  OpPanel('static TV', 'type+machine · 振り切れた', _static, g: const OpRub(1, 0), init: const [.6, .55, .1]),
];
