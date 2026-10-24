// Part of op_04_spring.dart (cut there by size, as it was).
part of 'op_04_spring.dart';

// 19 Slingshot: pull the pouch, let go; the band rings, the stone flies.
void _sling(Canvas c, Size s, OpV v, double t) {
  const l = Offset(40, 46), r = Offset(64, 46), base = Offset(52, 70);
  pl(c, [l, const Offset(44, 62), base, const Offset(60, 62), r], _w, 1.4);
  ln(c, base, const Offset(52, 110), _w, 1.4);
  Offset pouch;
  if (v.press) {
    pouch = v.at;
  } else {
    final x = spv(_ls(v), v);
    pouch = const Offset(52, 46) + Offset(-x * 24 * (.4 + v[0]), x * 6);
  }
  pl(c, [l, pouch, r], pc(v, 0), 1);
  c.drawOval(Rect.fromCenter(center: pouch, width: 8, height: 5), st(_w, .9));
  if (!v.press) {
    final ft = _ls(v);
    final sx = 52 + ft * 160 * (.4 + v[0]), sy = 46 - ft * 50 + ft * ft * 60;
    if (sx < 160) dot(c, Offset(sx, sy), 2.4 + v[2] * 1.4, pc(v, 2));
  }
  rd(c, v, 0, 'BAND', const Offset(150, 6), h: 16, ax: 1);
  lb(c, 'DAMP ${d2(v[1])}', const Offset(96, 92), pc(v, 1), h: 5);
  lb(c, 'STONE ${d2(v[2])}', const Offset(96, 100), pc(v, 2), h: 5);
}

// 20 Jelly world: everything in the panel is on springs; rub and a wave runs through it.
void _jelly(Canvas c, Size s, OpV v, double t) {
  const nx = 13, ny = 10;
  final hit = v.at;
  final pts = List<Offset>.generate(nx * ny, (i) {
    final p = Offset(6 + (i % nx) * 12.0, 6 + (i ~/ nx) * 12.0);
    final d = p - hit;
    final dist = d.distance;
    final dir = dist == 0 ? Offset.zero : d / dist;
    if (v.press) return p + dir * 24 * math.exp(-dist / 30);
    final tau = _ls(v) - dist / 90;
    final x = tau < 0 ? 0.0 : spv(tau, v);
    return p + dir * x * 24 * math.exp(-dist / 60);
  });
  for (var j = 0; j < ny; j++) {
    pl(c, [for (var i = 0; i < nx; i++) pts[j * nx + i]], al(K.white, .55), .8);
  }
  for (var i = 0; i < nx; i++) {
    pl(c, [for (var j = 0; j < ny; j++) pts[j * nx + i]], al(K.white, .25), .7);
  }
  ci(c, hit, 4, pc(v, 2));
  final d = (1 - v[1]) * 99;
  nm(c, d.round().toString().padLeft(2, '0'), const Offset(150, 84), 28, pc(v, 1), ax: 1);
  lb(c, 'WOBBLE', const Offset(150, 74), pc(v, 1), ax: 1, h: 5);
}

final springPanels = <OpPanel>[
  OpPanel('OP-1 coil: turns / damp', 'diagram · mechanism · sensible', _turns),
  OpPanel('diving board', 'landscape+figure · effect · drawing', _board, g: const OpFlick(2, 0)),
  OpPanel('jack in the box', 'isometric · effect · playful', _jack, g: const OpPull(0, 1)),
  OpPanel('kangaroo hops', 'animal · effect · drawing', _roo),
  OpPanel('car suspension', 'vehicle · effect · rub the road', _car, g: const OpRub(0, 1)),
  OpPanel('ruler twang on a desk', 'instrument · effect · drawing', _ruler, g: const OpFlick(1, 0)),
  OpPanel('step response', 'diagram · mechanism · numeral', _resp),
  OpPanel('bobblehead', 'character · effect · drawing', _bobble, g: const OpDrag(0, 2)),
  OpPanel('bungee off a truss', 'landscape · effect · drawing', _bungee, g: const OpPull(2, 1)),
  OpPanel('gravity well', 'cosmic+isometric · effect', _well, g: const OpPinch(2, 0)),
  OpPanel('doorstop boing', 'object · effect · playful', _doorstop, g: const OpFlick(1, 0)),
  OpPanel('SPRING stretches', 'typographic · effect · type', _word),
  OpPanel('phase spiral', 'diagram · mechanism · pushed', _phase, g: const OpSpin(1, Offset(.5, .5), 0)),
  OpPanel('frog on a lily pad', 'animal · effect · drawing', _frog, g: const OpFlick(2, 0)),
  OpPanel('slinky on stairs', 'isometric · effect · drawing', _slinky),
  OpPanel('scale reading rings', 'machine · effect · numeral', _scale, g: const OpDrag(2, 0)),
  OpPanel('tuning fork', 'instrument · effect+sound', _fork, g: const OpRub(0, 1)),
  OpPanel('seismograph drum', 'machine · history · drawing', _seismo, g: const OpFlick(1, 0)),
  OpPanel('slingshot', 'object · effect · pull', _sling, g: const OpPull(0, 1)),
  OpPanel('jelly world', 'field · 振り切れた', _jelly, g: const OpRub(1, 0), init: const [.35, .15, .4]),
];
