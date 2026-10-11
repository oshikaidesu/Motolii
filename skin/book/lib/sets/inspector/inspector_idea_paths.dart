part of 'inspector_parts.dart';

// ---- ideas: path operations -----------------------------------------------------------------------------------------
/// Three ways to edit path operations inside the Inspector, each drawing the real result of the ops in Dart.
class PathIdeas extends StatelessWidget {
  const PathIdeas({super.key});
  @override
  Widget build(BuildContext context) => const Column(
    mainAxisSize: MainAxisSize.min,
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      IdeaLabel('A', 'Path stage', 'The whole stack drawn live: drag Trim ends, a corner radius or the wiggle size right on the path.'),
      _PiA(),
      IdeaLabel('B', 'Before → after strip', 'Each op shows what it does to its input, so a folded stack still reads; open one to wipe-compare and edit.'),
      _PiB(),
      IdeaLabel('C', 'Draw-on timeline', 'Design the Trim animation here: keys on a mini timeline, hover to scrub, spacing dots show the ease.'),
      _PiCTrim(),
    ],
  );
}

// ---- geometry -------------------------------------------------------------------------------------------------------
/// contract: a contour is a run of segments sharing end points (a straight edge is 2 points, a curve many), so per-segment ops
/// (Zig Zag ridges, Rounded Corners, Pucker & Bloat) see the corners Lottie's shape modifiers see. A closed contour's flat ends on its start.
class _PiC {
  _PiC(this.segs, this.closed);
  final List<List<Offset>> segs;
  final bool closed;
  List<Offset> get flat => [for (final (i, s) in segs.indexed) ...(i == 0 ? s : s.skip(1))];
}

typedef _PiG = double Function(String);

double _piDot(Offset a, Offset b) => a.dx * b.dx + a.dy * b.dy;
Offset _piUnit(Offset o) => o.distance < 1e-9 ? Offset.zero : o / o.distance;
Offset _piN(Offset t) => Offset(t.dy, -t.dx);

double _piLen(List<Offset> p) {
  var l = 0.0;
  for (var i = 1; i < p.length; i++) {
    l += (p[i] - p[i - 1]).distance;
  }
  return l;
}

/// Position and unit tangent at arc length [d].
(Offset, Offset) _piAt(List<Offset> p, double d) {
  if (p.length < 2) return (p.isEmpty ? Offset.zero : p.first, const Offset(1, 0));
  var acc = 0.0;
  for (var i = 1; i < p.length; i++) {
    final l = (p[i] - p[i - 1]).distance;
    if (acc + l >= d || i == p.length - 1) {
      return (Offset.lerp(p[i - 1], p[i], l < 1e-9 ? 0 : ((d - acc) / l).clamp(0.0, 1.0))!, _piUnit(p[i] - p[i - 1]));
    }
    acc += l;
  }
  return (p.last, const Offset(1, 0));
}

List<Offset> _piCut(List<Offset> p, double a, double b) {
  final out = <Offset>[_piAt(p, a).$1];
  var acc = 0.0;
  for (var i = 1; i < p.length; i++) {
    acc += (p[i] - p[i - 1]).distance;
    if (acc > a && acc < b) out.add(p[i]);
  }
  return out..add(_piAt(p, math.max(a, b)).$1);
}

/// Arc length of the point on [p] nearest [q].
double _piNear(List<Offset> p, Offset q) {
  var best = double.infinity, at = 0.0, acc = 0.0;
  for (var i = 1; i < p.length; i++) {
    final a = p[i - 1], v = p[i] - a, l = v.distance;
    final u = l < 1e-9 ? 0.0 : (_piDot(q - a, v) / (l * l)).clamp(0.0, 1.0);
    final d = (a + v * u - q).distanceSquared;
    if (d < best) {
      best = d;
      at = acc + u * l;
    }
    acc += l;
  }
  return at;
}

List<Offset> _piDense(List<Offset> p, double step) {
  if (p.length < 2) return p;
  final l = _piLen(p), n = math.max(1, (l / step).ceil()), out = <Offset>[];
  var i = 1, acc = 0.0;
  for (var k = 0; k <= n; k++) {
    final d = l * k / n;
    while (i < p.length - 1 && acc + (p[i] - p[i - 1]).distance < d) {
      acc += (p[i] - p[i - 1]).distance;
      i++;
    }
    final sl = (p[i] - p[i - 1]).distance;
    out.add(Offset.lerp(p[i - 1], p[i], sl < 1e-9 ? 0 : ((d - acc) / sl).clamp(0.0, 1.0))!);
  }
  return out;
}

/// +1 when (t.dy, -t.dx) points out of the contour.
double _piSign(_PiC c) {
  if (!c.closed) return 1;
  final p = c.flat;
  var a = 0.0;
  for (var i = 0; i < p.length - 1; i++) {
    a += p[i].dx * p[i + 1].dy - p[i + 1].dx * p[i].dy;
  }
  return a >= 0 ? 1 : -1;
}

Offset _piCentre(_PiC c) => [for (final s in c.segs) s.first].fold(Offset.zero, (a, b) => a + b) / c.segs.length.toDouble();

List<Offset> _piQuad(Offset a, Offset c, Offset b, [int n = 8]) => [
  for (var i = 0; i <= n; i++) a * ((1 - i / n) * (1 - i / n)) + c * (2 * (1 - i / n) * (i / n)) + b * ((i / n) * (i / n)),
];

List<Offset> _piCubic(Offset a, Offset b, Offset c, Offset d, [int n = 16]) => [
  for (var i = 0; i <= n; i++)
    a * math.pow(1 - i / n, 3).toDouble() +
        b * (3 * math.pow(1 - i / n, 2) * (i / n)) +
        c * (3 * (1 - i / n) * math.pow(i / n, 2)) +
        d * math.pow(i / n, 3).toDouble(),
];

/// Points joined by quadratics through their midpoints: the Smooth point type.
List<Offset> _piSmooth(List<Offset> p) {
  if (p.length < 3) return p;
  final out = <Offset>[p.first];
  for (var i = 1; i < p.length - 1; i++) {
    out.addAll(_piQuad(i == 1 ? p.first : (p[i - 1] + p[i]) / 2, p[i], i == p.length - 2 ? p.last : (p[i] + p[i + 1]) / 2, 6).skip(1));
  }
  return out;
}

/// Lottie's Rounded Corners: each sharp joint is cut back [r] along both segments (never past half of either) and bridged by a curve.
_PiC _piRound(_PiC c, double r) {
  final s = c.segs, n = s.length;
  if (r <= 0 || n == 0) return c;
  final len = [for (final x in s) _piLen(x)], d = List.filled(n, 0.0);
  for (var i = c.closed ? 0 : 1; i < n; i++) {
    final a = s[(i - 1 + n) % n], b = s[i];
    if (a.length < 2 || b.length < 2 || _piDot(_piUnit(a.last - a[a.length - 2]), _piUnit(b[1] - b.first)) > .995) continue;
    d[i] = math.min(r, math.min(len[(i - 1 + n) % n], len[i]) / 2);
  }
  final core = [for (var i = 0; i < n; i++) _piCut(s[i], d[i], len[i] - (i + 1 < n ? d[i + 1] : (c.closed ? d[0] : 0)))];
  final out = <List<Offset>>[];
  for (var i = 0; i < n; i++) {
    out.add(core[i]);
    final j = i + 1 < n ? i + 1 : (c.closed ? 0 : -1);
    if (j >= 0 && d[j] > 0) out.add(_piQuad(core[i].last, s[j].first, core[j].first));
  }
  return _PiC(out, c.closed);
}

/// Lottie's Pucker & Bloat: vertices move toward the centre by [a], their handles away by [a].
_PiC _piPucker(_PiC c, double a) {
  if (a == 0 || c.segs.isEmpty) return c;
  final ctr = _piCentre(c);
  Offset mv(Offset v, double k) => v + (ctr - v) * k;
  List<Offset> seg(List<Offset> s) {
    final l = _piLen(s), v0 = s.first, v1 = s.last;
    final h0 = s.length > 2 ? v0 + _piUnit(s[1] - v0) * (l / 3) : v0, h1 = s.length > 2 ? v1 - _piUnit(v1 - s[s.length - 2]) * (l / 3) : v1;
    return _piCubic(mv(v0, a), mv(h0, -a), mv(h1, -a), mv(v1, a));
  }

  return _PiC([for (final s in c.segs) seg(s)], c.closed);
}

/// Zig Zag: [n] ridges per segment, alternating [amp] either side of the line. Corner ridges are real vertices (a later Round rounds each).
_PiC _piZig(_PiC c, double amp, int n, bool smooth) {
  if (n <= 0 || amp == 0) return c;
  final sg = _piSign(c);
  List<List<Offset>> seg(List<Offset> s) {
    final l = _piLen(s);
    if (l < 1) return [s];
    final p = [
      s.first,
      for (var j = 1; j <= n; j++) _piAt(s, l * j / (n + 1)).$1 + _piN(_piAt(s, l * j / (n + 1)).$2) * (sg * amp * (j.isOdd ? 1 : -1)),
      s.last,
    ];
    return smooth
        ? [_piSmooth(p)]
        : [
            for (var i = 1; i < p.length; i++) [p[i - 1], p[i]],
          ];
  }

  return _PiC([for (final s in c.segs) ...seg(s)], c.closed);
}

/// Offset Paths: every vertex moves along its edges' normals; outer corners get the join, inner ones meet (clamped so they cannot explode).
_PiC _piOffset(_PiC c, double a, int join, double miter) {
  if (a == 0) return c;
  final sg = _piSign(c), q = <Offset>[];
  for (final x in c.flat) {
    if (q.isEmpty || (x - q.last).distance > 1e-6) q.add(x);
  }
  if (c.closed && q.length > 1 && (q.first - q.last).distance < 1e-6) q.removeLast();
  final n = q.length;
  if (n < 2) return c;
  final out = <Offset>[];
  for (var i = 0; i < n; i++) {
    final e1 = c.closed || i > 0 ? _piUnit(q[i] - q[(i - 1 + n) % n]) : null, e2 = c.closed || i < n - 1 ? _piUnit(q[(i + 1) % n] - q[i]) : null;
    if (e1 == null || e2 == null) {
      out.add(q[i] + _piN((e1 ?? e2)!) * (sg * a));
      continue;
    }
    final n1 = _piN(e1) * sg, n2 = _piN(e2) * sg, b = _piUnit(n1 + n2), cosH = _piDot(b, n1);
    if (_piDot(e1, e2) > .999 || cosH < 1e-3) {
      out.add(q[i] + n1 * a);
      continue;
    }
    final ml = 1 / cosH, outer = (e1.dx * e2.dy - e1.dy * e2.dx) * sg * a > 0;
    if (!outer || (join == 0 && ml <= miter)) {
      out.add(q[i] + b * (a * (outer ? ml : math.min(ml, 4))));
    } else if (join == 1) {
      final a0 = math.atan2(n1.dy, n1.dx);
      var da = math.atan2(n2.dy, n2.dx) - a0;
      da -= (da / (2 * math.pi)).roundToDouble() * 2 * math.pi;
      for (var k = 0; k <= 6; k++) {
        out.add(q[i] + Offset.fromDirection(a0 + da * k / 6) * a);
      }
    } else {
      out.addAll([q[i] + n1 * a, q[i] + n2 * a]);
    }
  }
  if (c.closed) out.add(out.first);
  return _PiC([out], c.closed);
}

/// Twist: each point turns about [ctr] by [deg] scaled by its distance over the farthest point's.
List<_PiC> _piTwist(List<_PiC> cs, double deg, Offset ctr) {
  if (deg == 0) return cs;
  var rMax = 1.0;
  for (final c in cs) {
    for (final p in c.flat) {
      rMax = math.max(rMax, (p - ctr).distance);
    }
  }
  Offset tw(Offset p) {
    final v = p - ctr, th = deg * math.pi / 180 * v.distance / rMax, co = math.cos(th), si = math.sin(th);
    return ctr + Offset(v.dx * co - v.dy * si, v.dx * si + v.dy * co);
  }

  return [
    for (final c in cs)
      _PiC([
        for (final s in c.segs) [for (final p in _piDense(s, 4)) tw(p)],
      ], c.closed),
  ];
}

double _piHash(int seed, int i, int j) {
  var h = (seed * 374761393 + i * 668265263 + j * 1274126177) & 0x7fffffff;
  h = ((h ^ (h >> 13)) * 1103515245) & 0x7fffffff;
  return ((h ^ (h >> 16)) & 0xffff) / 32767.5 - 1;
}

/// Smooth value noise in -1..1; [period] > 0 wraps along u so a closed contour's wiggle meets itself.
double _piNoise(double u, double v, int seed, int period) {
  final i = u.floor(), j = v.floor(), fu = u - i, fv = v - j, su = fu * fu * (3 - 2 * fu), sv = fv * fv * (3 - 2 * fv);
  double h(int x, int y) => _piHash(seed, period > 0 ? x % period : x, y);
  final a = h(i, j) + (h(i + 1, j) - h(i, j)) * su, b = h(i, j + 1) + (h(i + 1, j + 1) - h(i, j + 1)) * su;
  return a + (b - a) * sv;
}

/// Wiggle Paths: points every 160/[detail] px, each pushed by seeded noise up to [size]; [phase] moves through the noise.
_PiC _piWiggle(_PiC c, double size, double detail, double phase, int seed, bool smooth) {
  if (size == 0) return c;
  final step = math.max(4.0, 160 / detail.clamp(1, 100)), total = _piLen(c.flat);
  if (total < 1) return c;
  final period = c.closed ? math.max(1, (total / step).round()) : 0, k = c.closed ? period * step / total : 1.0;
  var g = 0.0;
  final out = <List<Offset>>[];
  for (final s in c.segs) {
    final l = _piLen(s), pts = _piDense(s, step), n = pts.length - 1;
    out.add([
      for (var i = 0; i <= n; i++)
        pts[i] +
            Offset(_piNoise((g + l * i / n) * k / step, phase / 120, seed, period), _piNoise((g + l * i / n) * k / step, phase / 120, seed + 101, period)) *
                size,
    ]);
    if (smooth) out.last = _piSmooth(out.last);
    g += l;
  }
  return _PiC(out, c.closed);
}

/// Trim Paths on one contour: Start/End in %, Offset in degrees (one turn = the whole length); wraps across a closed contour's seam.
List<_PiC> _piTrim(_PiC c, double s, double e, double off) {
  final p = c.flat, l = _piLen(p);
  var a = math.min(s, e) / 100, b = math.max(s, e) / 100;
  if (b - a <= 0 || l < 1e-6) return const <_PiC>[];
  if (b - a >= 1) return [c];
  a += off / 360;
  b += off / 360;
  final f = a.floorToDouble();
  a -= f;
  b -= f;
  if (b <= 1) {
    return [
      _PiC([_piCut(p, a * l, b * l)], false),
    ];
  }
  final tail = _piCut(p, a * l, l), head = _piCut(p, 0, (b - 1) * l);
  return c.closed
      ? [
          _PiC([
            [...tail, ...head.skip(1)],
          ], false),
        ]
      : [
          _PiC([tail], false),
          _PiC([head], false),
        ];
}

List<_PiC> _piApply(List<_PiC> cs, String k, _PiG g) => switch (k) {
  'round' => [for (final c in cs) _piRound(c, g('radius'))],
  'pucker' => [for (final c in cs) _piPucker(c, g('amount') / 100)],
  'zigzag' => [for (final c in cs) _piZig(c, g('amp'), g('ridges').round().clamp(0, 60), g('pt') == 1)],
  'offset' => [for (final c in cs) _piOffset(c, g('amount'), g('join').round(), g('miter'))],
  'twist' => _piTwist(cs, g('angle'), Offset(g('cx'), g('cy'))),
  'wiggle' => [for (final c in cs) _piWiggle(c, g('size'), g('detail'), g('phase'), g('seed').round(), g('pt') == 1)],
  'trim' => [for (final c in cs) ..._piTrim(c, g('start'), g('end'), g('offset'))],
  _ => cs,
};

/// Each op's input, then the result.
(List<List<_PiC>>, List<_PiC>) _piChain(List<_PiC> base, List<(String, _PiG, bool)> ops) {
  final ins = <List<_PiC>>[];
  var cs = base;
  for (final (k, g, on) in ops) {
    ins.add(cs);
    if (on) cs = _piApply(cs, k, g);
  }
  return (ins, cs);
}

_PiC _piPoly(List<Offset> v, bool closed) => _PiC([
  for (var i = 0; i < v.length - (closed ? 0 : 1); i++) [v[i], v[(i + 1) % v.length]],
], closed);

final _piStar = _piPoly([for (var i = 0; i < 10; i++) Offset.fromDirection(-math.pi / 2 + i * math.pi / 5, i.isEven ? 78 : 34)], true);
final _piRect = _piPoly(const [Offset(-80, -50), Offset(80, -50), Offset(80, 50), Offset(-80, 50)], true);
final _piLine = _piPoly(const [Offset(-112, 34), Offset(-46, -40), Offset(14, 24), Offset(108, -36)], false);
final _piScript = () {
  final m =
      (Path()
            ..moveTo(-118, 34)
            ..cubicTo(-80, -64, -16, -64, -28, 0)
            ..cubicTo(-40, 60, 42, 60, 30, -6)
            ..cubicTo(22, -56, 82, -52, 118, -26))
          .computeMetrics()
          .first;
  return _PiC([
    [for (var d = 0.0; d < m.length; d += 2) m.getTangentForOffset(d)!.position, m.getTangentForOffset(m.length)!.position],
  ], false);
}();

// ---- ops as cards ---------------------------------------------------------------------------------------------------
class _PiK {
  const _PiK(this.name, this.short, this.defs, this.sum);
  final String name, short;
  final Map<String, double> defs;
  final List<String> Function(_PiG g) sum;
}

final _piKinds = <String, _PiK>{
  'pucker': _PiK('Pucker & Bloat', 'Pucker', const {'amount': 15}, (g) {
    final a = g('amount');
    return [a == 0 ? '0%' : '${a < 0 ? 'Pucker' : 'Bloat'} ${brief(a.abs(), 0)}%'];
  }),
  'round': _PiK('Rounded Corners', 'Round', const {'radius': 8}, (g) => [_px(g('radius'))]),
  'zigzag': _PiK('Zig Zag', 'Zig Zag', const {
    'amp': 6,
    'ridges': 5,
    'pt': 0,
  }, (g) => [_px(g('amp')), '${brief(g('ridges'), 0)} ridges', if (g('pt') == 1) 'Smooth']),
  'offset': _PiK('Offset Paths', 'Offset', const {
    'amount': 6,
    'join': 1,
    'miter': 4,
  }, (g) => ['${_sg(g('amount'), 1)} px', _joins[g('join').round().clamp(0, 2)]]),
  'twist': _PiK('Twist', 'Twist', const {'angle': 40, 'cx': 0, 'cy': 0}, (g) => ['${_sg(g('angle'))}°']),
  'wiggle': _PiK('Wiggle Paths', 'Wiggle', const {
    'size': 5,
    'detail': 6,
    'phase': 0,
    'seed': 1,
    'pt': 1,
  }, (g) => [_px(g('size')), 'Seed ${brief(g('seed'), 0)}']),
  'trim': _PiK('Trim Paths', 'Trim', const {
    'start': 0,
    'end': 72,
    'offset': 0,
  }, (g) => ['${brief(g('start'), 0)}–${brief(g('end'), 0)}%', if (g('offset') != 0) '${_sg(g('offset'))}°']),
};

// ---- stacks of instances --------------------------------------------------------------------------------------------
/// contract: a lane's stack is `pi_<lane>_order`, comma-separated instances (`round1`) in the order they apply; an instance is a kind
/// plus a never-reused number (`pi_<lane>_next`), so any kind repeats freely and a removed op's values never leak into a new one.
/// An instance's numbers are `pi_<lane>_<inst>_<param>`, its flags `pi_<lane>_<inst>.on` / `.open`.
String _piKind(String inst) => inst.replaceFirst(RegExp(r'\d+$'), '');

List<String> _piOrder(Doc d, String lane) => [
  for (final s in (d.s2['pi_${lane}_order'] ?? '').split(','))
    if (_piKinds.containsKey(_piKind(s))) s,
];

void _piSetOrder(Doc d, String lane, List<String> o) => d.str('pi_${lane}_order', o.join(','));

Map<String, double> _piDefs(String lane, String inst) => {for (final e in _piKinds[_piKind(inst)]!.defs.entries) 'pi_${lane}_${inst}_${e.key}': e.value};
_PiG _piG(Doc d, String lane, String inst) =>
    (p) => d.get('pi_${lane}_${inst}_$p') ?? 0;

/// A new instance of [kind] at [at] (the end by default); [copy] names an instance whose values and switch it starts from.
String _piAdd(Doc d, String lane, String kind, {int? at, String? copy}) {
  final n = int.tryParse(d.s2['pi_${lane}_next'] ?? '') ?? 1, inst = '$kind$n';
  d.s2['pi_${lane}_next'] = '${n + 1}';
  d.seed(_piDefs(lane, inst));
  if (copy != null) {
    for (final p in _piKinds[kind]!.defs.keys) {
      d.v['pi_${lane}_${inst}_$p'] = d.get('pi_${lane}_${copy}_$p') ?? 0;
    }
    d.b['pi_${lane}_$inst.on'] = d.b['pi_${lane}_$copy.on'] ?? true;
  }
  final o = _piOrder(d, lane);
  _piSetOrder(d, lane, o..insert((at ?? o.length).clamp(0, o.length), inst));
  return inst;
}

/// The name an instance wears: a second Rounded Corners in the stack reads "Rounded Corners 2".
String _piTitle(List<String> order, String inst, {bool short = false}) {
  final k = _piKind(inst), kd = _piKinds[k]!, same = order.takeWhile((s) => s != inst).where((s) => _piKind(s) == k).length;
  final name = short ? kd.short : kd.name;
  return same == 0 ? name : '$name ${same + 1}';
}

/// Each instance's input, then the result.
(List<List<_PiC>>, List<_PiC>) _piRun(Doc d, String lane, List<_PiC> base, List<String> order) {
  for (final s in order) {
    d.seed(_piDefs(lane, s));
  }
  return _piChain(base, [for (final s in order) (_piKind(s), _piG(d, lane, s), d.b['pi_${lane}_$s.on'] ?? true)]);
}

_PiC _piShapeA(String shape) => switch (shape) {
  'Rect' => _piRect,
  'Line' => _piLine,
  _ => _piStar,
};

/// The points a lane's stack strokes on its stage ('a' or 'b'), one list per contour.
List<List<Offset>> pathIdeaOutline(Doc doc, String lane) {
  final base = lane == 'a' ? _piShapeA(doc.s2['pi_a_shape'] ?? 'Star') : _piRect;
  return [
    for (final c in _piRun(doc, lane, [base], _piOrder(doc, lane)).$2) c.flat,
  ];
}

/// An instance's More: order, duplicate, remove.
class _PiMore extends StatelessWidget {
  const _PiMore(this.lane, this.inst);
  final String lane, inst;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, o = _piOrder(doc, lane), i = o.indexOf(inst);
    void move(int by) => _piSetOrder(
      doc,
      lane,
      o
        ..removeAt(i)
        ..insert(i + by, inst),
    );
    void picked(String s) => lane == 'a' ? doc.str('pi_a_sel', s) : doc.flag('pi_b_$s.open', true);
    final items = <(String, bool, VoidCallback, bool)>[
      ('Move earlier', i > 0, () => move(-1), false),
      ('Move later', i >= 0 && i < o.length - 1, () => move(1), false),
      ('Duplicate', i >= 0, () => picked(_piAdd(doc, lane, _piKind(inst), at: i + 1, copy: inst)), false),
      ('Remove', i >= 0, () => _piSetOrder(doc, lane, o..remove(inst)), true),
    ];
    return _Float(
      right: true,
      width: 160,
      need: items.length * 22 + 8,
      anchor: (open, toggle) => Hov(
        onTap: x.cfg.locked ? null : toggle,
        builder: (_, h) => Padding(
          padding: const EdgeInsets.symmetric(horizontal: 6),
          child: Text('More', key: ValueKey('pi_${lane}_$inst.more'), style: T.label(open || h ? Grey.g95 : Grey.g56)),
        ),
      ),
      menu: (close) => Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          for (final (s, ok, go, danger) in items)
            Hov(
              cursor: ok ? SystemMouseCursors.click : SystemMouseCursors.basic,
              onTap: ok
                  ? () {
                      close();
                      go();
                    }
                  : null,
              builder: (_, h) => Container(
                height: 22,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                decoration: BoxDecoration(color: h && ok ? Grey.g26 : null, borderRadius: BorderRadius.circular(5)),
                alignment: Alignment.centerLeft,
                child: Row(
                  children: [
                    if (danger) const ErrMark(size: 10, gap: 5),
                    Text(s, style: T.name(!ok ? Grey.g56 : (danger ? Role.error : Grey.g91))),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }
}

/// The stack's last row: every kind in a floating two-column list; a pick is a new instance at the bottom (a kind already there is fine).
class _PiAdd extends StatelessWidget {
  const _PiAdd(this.lane, {this.card = false});
  final String lane;

  /// Drawn as its own card row (B) rather than a row inside the stack's card (A).
  final bool card;
  static const _itemH = 22.0;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, kinds = _piKinds.keys.toList(), rows = (kinds.length / 2).ceil();
    return _Float(
      need: rows * _itemH + 8,
      anchor: (open, toggle) => Hov(
        key: ValueKey('pi_${lane}_add'),
        onTap: x.cfg.locked ? null : toggle,
        builder: (_, h) => Container(
          height: Pop.row,
          padding: EdgeInsets.symmetric(horizontal: card ? Pop.inset : 6),
          margin: EdgeInsets.only(bottom: card ? Pop.gap : 0),
          decoration: BoxDecoration(color: h || open ? Grey.g15 : (card ? Pop.card : null), borderRadius: card ? null : BorderRadius.circular(6)),
          child: Row(
            children: [
              SizedBox(
                width: 22,
                child: Text('+', style: T.name(h || open ? Grey.g95 : Grey.g63).copyWith(fontWeight: FontWeight.w700)),
              ),
              Text('Add path operation', style: T.name(x.cfg.locked ? Grey.g44 : (h || open ? Grey.g95 : Grey.g76))),
              const Spacer(),
              if (open) Text('Close', style: T.label(Grey.g56)),
            ],
          ),
        ),
      ),
      menu: (close) => Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (var r = 0; r < rows; r++)
            Row(
              children: [
                for (final k in kinds.skip(r * 2).take(2))
                  Expanded(
                    child: Hov(
                      key: ValueKey('pi_${lane}_add_$k'),
                      onTap: () {
                        close();
                        final s = _piAdd(doc, lane, k);
                        lane == 'a' ? doc.str('pi_a_sel', s) : doc.flag('pi_b_$s.open', true);
                      },
                      builder: (_, h) => Container(
                        height: _itemH,
                        padding: const EdgeInsets.symmetric(horizontal: 6),
                        decoration: BoxDecoration(color: h ? Grey.g26 : null, borderRadius: BorderRadius.circular(5)),
                        child: Row(
                          children: [
                            _OpGlyph(k, hot: h),
                            const SizedBox(width: 6),
                            Expanded(
                              child: Text(_piKinds[k]!.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(h ? Grey.g95 : Grey.g76)),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                if (r * 2 + 1 >= kinds.length) const Spacer(),
              ],
            ),
        ],
      ),
    );
  }
}

/// An op's numbers, small and exact, under whatever does it by hand.
class _PiFields extends StatelessWidget {
  const _PiFields(this.lane, this.inst);
  final String lane, inst;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, k = 'pi_${lane}_$inst', g = _piG(doc, lane, inst), op = _piKind(inst);
    NumField f(String p, String l, {String u = '', double? min, double? max, int dec = 0, double perPx = 1, bool bi = false, bool whole = false}) => NumField(
      id: '${k}_$p',
      label: l,
      unit: u,
      min: min,
      max: max,
      decimals: dec,
      perPx: perPx,
      bipolar: bi,
      onSet: whole ? (v) => doc.set('${k}_$p', v.roundToDouble()) : null,
    );
    Widget seg(String p, List<String> o) => Dis(
      child: Segmented(items: o, index: g(p).round().clamp(0, o.length - 1), expand: true, onChanged: (i) => doc.set('${k}_$p', i.toDouble())),
    );
    final (List<Widget> cells, Widget? choice) = switch (op) {
      'round' => ([f('radius', 'Radius', u: 'px', min: 0, dec: 1, perPx: .5)], null),
      'pucker' => ([f('amount', 'Amount', u: '%', min: -100, max: 100, bi: true)], null),
      'zigzag' => ([f('amp', 'Size', u: 'px', dec: 1, perPx: .25), f('ridges', 'Ridges', min: 0, max: 60, perPx: .1, whole: true)], seg('pt', _points)),
      'offset' => (
        [f('amount', 'Amount', u: 'px', dec: 1, perPx: .25), if (g('join') == 0) f('miter', 'Miter', min: 1, dec: 1, perPx: .05)],
        seg('join', _joins),
      ),
      'twist' => ([f('angle', 'Angle', u: '°'), f('cx', 'X', u: 'px'), f('cy', 'Y', u: 'px')], null),
      'wiggle' => (
        [f('size', 'Size', min: 0, dec: 1, perPx: .25), f('detail', 'Detail', min: 1, max: 100, perPx: .25, whole: true), f('phase', 'Phase', u: '°')],
        seg('pt', _points),
      ),
      'trim' => ([f('start', 'Start', u: '%', min: 0, max: 100), f('end', 'End', u: '%', min: 0, max: 100), f('offset', 'Offset', u: '°')], null),
      _ => (const <Widget>[], null),
    };
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            for (final (i, c) in cells.indexed) ...[if (i > 0) const SizedBox(width: 4), Expanded(child: c)],
          ],
        ),
        if (choice != null) ...[const SizedBox(height: 4), choice],
        if (op == 'wiggle') AdvancedFold('$k.adv', summary: 'Seed ${brief(g('seed'), 0)}', [f('seed', 'Seed', min: 0, max: 9999, perPx: .1, whole: true)]),
      ],
    );
  }
}

/// Brief chips that drop whole ones rather than clip.
class _PiChips extends StatelessWidget {
  const _PiChips(this.chips, {this.dim = false});
  final List<String> chips;
  final bool dim;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      var used = 0.0;
      final fit = <String>[];
      for (final t in chips) {
        final w = BriefChip.width(t) + (fit.isEmpty ? 0 : 4);
        if (used + w > box.maxWidth) break;
        used += w;
        fit.add(t);
      }
      return Row(
        children: [
          for (final (i, t) in fit.indexed) ...[if (i > 0) const SizedBox(width: 4), BriefChip(t, dim: dim)],
        ],
      );
    },
  );
}

// ---- the stage: a path view with handles ----------------------------------------------------------------------------
class _PiView {
  _PiView(Size s, Rect m) : k = math.min(s.width / m.width, s.height / m.height), c = s.center(Offset.zero), m = m.center;
  final double k;
  final Offset c, m;
  Offset on(Offset p) => c + (p - m) * k;
  Offset back(Offset p) => m + (p - c) / k;
}

Path _piPath(List<_PiC> cs, _PiView v) {
  final p = Path();
  for (final c in cs) {
    final f = c.flat;
    if (f.length > 1) p.addPolygon([for (final q in f) v.on(q)], c.closed);
  }
  return p;
}

/// A handle in model space: where it sits, what a drag to a model point writes, an optional guide line from [from].
class _PiH {
  const _PiH(this.id, this.at, this.drag, {this.tip, this.from, this.shape = 0, Color? color}) : _color = color;
  final Color? _color;
  Color get color => _color ?? Grey.g95;
  final String id;
  final Offset at;
  final void Function(Offset m) drag;
  final String Function()? tip;
  final Offset? from;

  /// 0 dot, 1 diamond, 2 ring, 3 tab.
  final int shape;
}

class _PiInk {
  const _PiInk(this.cs, this.color, this.w, {this.side = 0});
  final List<_PiC> cs;
  final Color color;
  final double w;

  /// -1 drawn left of the wipe only, 1 right of it only.
  final int side;
}

/// contract: repaints only when rebuilt (a value changed) or on pointer input; one drag = one undo step.
class _PiStage extends StatefulWidget {
  const _PiStage({
    super.key,
    required this.inks,
    required this.bounds,
    this.handles = const [],
    this.height = 150,
    this.marks = const [],
    this.rings = const [],
    this.wipe,
    this.overlay,
    this.note,
  });
  final List<_PiInk> inks;
  final List<_PiH> handles;
  final double height;
  final Rect bounds;
  final List<(Offset, Color, double)> marks;
  final List<(Offset, double)> rings;
  final double? wipe;
  final Widget? overlay;
  final String? note;
  @override
  State<_PiStage> createState() => _PiStageState();
}

class _PiStageState extends State<_PiStage> {
  String? _hot, _drag;
  _PiH? _find(String? id) => id == null ? null : widget.handles.where((h) => h.id == id).firstOrNull;
  String? _pick(_PiView v, Offset p) {
    String? best;
    var bd = 11.0;
    for (final h in widget.handles) {
      final d = (v.on(h.at) - p).distance;
      if (d < bd) {
        bd = d;
        best = h.id;
      }
    }
    return best;
  }

  void _end(Doc doc) {
    if (_drag == null) return;
    doc.endGesture();
    setState(() => _drag = null);
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked;
    return LayoutBuilder(
      builder: (context, box) {
        final v = _PiView(Size(box.maxWidth, widget.height), widget.bounds), tip = _find(_drag ?? _hot)?.tip?.call() ?? widget.note;
        return MouseRegion(
          cursor: !on
              ? SystemMouseCursors.basic
              : (_drag != null ? SystemMouseCursors.grabbing : (_hot != null ? SystemMouseCursors.grab : SystemMouseCursors.basic)),
          onHover: (e) {
            final h = on ? _pick(v, e.localPosition) : null;
            if (h != _hot) setState(() => _hot = h);
          },
          onExit: (_) {
            if (_hot != null) setState(() => _hot = null);
          },
          child: Listener(
            behavior: HitTestBehavior.opaque,
            onPointerDown: (e) {
              if (!on || e.buttons != kPrimaryButton) return;
              final h = _pick(v, e.localPosition);
              if (h == null) return;
              doc.beginGesture();
              setState(() => _drag = h);
            },
            onPointerMove: (e) => _find(_drag)?.drag(v.back(e.localPosition)),
            onPointerUp: (_) => _end(doc),
            onPointerCancel: (_) => _end(doc),
            child: SizedBox(
              height: widget.height,
              child: Stack(
                children: [
                  Positioned.fill(child: CustomPaint(painter: _PiStagePaint(widget, v, _drag ?? _hot))),
                  for (final h in widget.handles)
                    Positioned(
                      key: ValueKey('h:${h.id}'),
                      left: v.on(h.at).dx - 6,
                      top: v.on(h.at).dy - 6,
                      width: 12,
                      height: 12,
                      child: const IgnorePointer(child: SizedBox()),
                    ),
                  if (widget.overlay != null) Positioned(top: 6, right: 6, child: widget.overlay!),
                  if (tip != null)
                    Positioned(
                      left: 8,
                      bottom: 6,
                      child: IgnorePointer(child: Text(tip, style: T.label(_drag != null ? Grey.g95 : Grey.g63))),
                    ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}

class _PiStagePaint extends CustomPainter {
  _PiStagePaint(this.st, this.v, this.hot);
  final _PiStage st;
  final _PiView v;
  final String? hot;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = Grey.g10);
    final b = st.bounds, grid = Paint()..color = Grey.g20;
    for (var gx = (b.left / 20).ceil() * 20.0; gx <= b.right; gx += 20) {
      for (var gy = (b.top / 20).ceil() * 20.0; gy <= b.bottom; gy += 20) {
        c.drawCircle(v.on(Offset(gx, gy)), .8, grid);
      }
    }
    final w = st.wipe == null ? null : v.on(Offset(st.wipe!, 0)).dx;
    for (final k in st.inks) {
      c.save();
      if (w != null && k.side != 0) c.clipRect(k.side < 0 ? Rect.fromLTRB(0, 0, w, s.height) : Rect.fromLTRB(w, 0, s.width, s.height));
      c.drawPath(
        _piPath(k.cs, v),
        Paint()
          ..color = k.color
          ..style = PaintingStyle.stroke
          ..strokeWidth = k.w
          ..strokeCap = StrokeCap.round
          ..strokeJoin = StrokeJoin.round,
      );
      c.restore();
    }
    final line = Paint()
      ..color = Grey.g44
      ..strokeWidth = 1
      ..style = PaintingStyle.stroke;
    if (w != null) {
      c.drawLine(Offset(w, 0), Offset(w, s.height), line);
      for (final (t, right) in const [('BEFORE', false), ('AFTER', true)]) {
        final tp = TextPainter(
          text: TextSpan(text: t, style: T.micro(Grey.g44)),
          textDirection: TextDirection.ltr,
        )..layout();
        tp.paint(c, Offset(right ? s.width - tp.width - 8 : 8, 7));
      }
    }
    for (final (a, r) in st.rings) {
      c.drawCircle(v.on(a), r * v.k, line..color = Grey.g26);
    }
    for (final (a, col, r) in st.marks) {
      c.drawCircle(v.on(a), r, Paint()..color = col);
    }
    final edge = Paint()
      ..color = Grey.g07
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    for (final h in st.handles) {
      final q = v.on(h.at), big = h.id == hot, r = big ? 5.5 : 4.5, fill = Paint()..color = big ? Pop.accent : h.color;
      if (h.from != null) c.drawLine(v.on(h.from!), q, line..color = Grey.g56);
      switch (h.shape) {
        case 1:
          final d = Path()..addPolygon([q + Offset(0, -r - 1), q + Offset(r + 1, 0), q + Offset(0, r + 1), q + Offset(-r - 1, 0)], true);
          c.drawPath(d, fill);
          c.drawPath(d, edge);
        case 2:
          c.drawCircle(q, r, edge..strokeWidth = 4);
          c.drawCircle(
            q,
            r,
            Paint()
              ..color = fill.color
              ..style = PaintingStyle.stroke
              ..strokeWidth = 2,
          );
          edge.strokeWidth = 1.5;
        case 3:
          final rr = RRect.fromRectAndRadius(Rect.fromCenter(center: q, width: 10, height: 14), const Radius.circular(3));
          c.drawRRect(rr, fill);
          c.drawRRect(rr, edge);
        default:
          c.drawCircle(q, r, fill);
          c.drawCircle(q, r, edge);
      }
    }
  }

  @override
  bool shouldRepaint(_PiStagePaint o) => true;
}

class _PiThumb extends StatelessWidget {
  const _PiThumb(this.cs, this.ink, {this.w, this.h});
  final List<_PiC> cs;
  final Color ink;
  final double? w, h;
  @override
  Widget build(BuildContext context) =>
      CustomPaint(size: Size(w ?? 0, h ?? 0), painter: _PiThumbPaint(cs, ink), child: w == null ? const SizedBox.expand() : null);
}

class _PiThumbPaint extends CustomPainter {
  const _PiThumbPaint(this.cs, this.ink);
  final List<_PiC> cs;
  final Color ink;
  @override
  void paint(Canvas c, Size s) => c.drawPath(
    _piPath(cs, _PiView(s, _piBoundsB.inflate(6))),
    Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.1
      ..strokeJoin = StrokeJoin.round,
  );
  @override
  bool shouldRepaint(_PiThumbPaint o) => !identical(o.cs, cs) || o.ink != ink;
}

/// A handle on the joint before segment [i] that slides along it: the corner's radius.
_PiH? _piCornerH(String id, _PiC c, int i, double r, void Function(double) set, String Function() tip) {
  final n = c.segs.length;
  if (n <= i || (!c.closed && i == 0)) return null;
  final s = c.segs[i], lim = math.min(_piLen(s), _piLen(c.segs[(i - 1 + n) % n])) / 2;
  return _PiH(id, _piAt(s, r.clamp(0.0, lim)).$1, (q) => set((_piNear(s, q).clamp(0.0, lim) * 2).roundToDouble() / 2), from: s.first, tip: tip);
}

// ---- A: path stage --------------------------------------------------------------------------------------------------
const _piShapes = ['Star', 'Rect', 'Line'];
const _piBoundsA = Rect.fromLTRB(-125, -132, 125, 90);

/// One card, one stage: the result of the whole stack; the picked instance's handles sit on the path; the list below names, briefs,
/// switches and orders instances (any kind as often as wanted).
class _PiA extends StatelessWidget {
  const _PiA();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      {'pi_a_round1_radius': 6, 'pi_a_round3_radius': 5},
      strs: const {'pi_a_order': 'round1,pucker2,round3,wiggle4,trim5', 'pi_a_next': '6', 'pi_a_sel': 'trim5', 'pi_a_shape': 'Star'},
      flags: const {'pi_a_wiggle4.on': false},
    );
    final order = _piOrder(doc, 'a'), shape = doc.s2['pi_a_shape']!, open = doc.b['pi_a.open'] ?? true;
    final picked = doc.s2['pi_a_sel'], sel = order.contains(picked) ? picked : order.lastOrNull;
    bool on(String s) => doc.b['pi_a_$s.on'] ?? true;
    final base = [_piShapeA(shape)];
    final (ins, out) = _piRun(doc, 'a', base, order);
    final i = sel == null ? -1 : order.indexOf(sel), selIn = i < 0 ? null : ins[i], selOn = sel != null && on(sel);
    final start = sel != null && _piKind(sel) == 'trim' && selIn!.isNotEmpty && selIn.first.flat.isNotEmpty ? selIn.first.flat.first : null;
    return Sect(
      title: 'Path operations',
      tone: Pop.tonePath,
      mark: CardMark.path,
      open: open,
      onToggle: () => doc.flag('pi_a.open', !open),
      brief: [
        for (final s in order)
          if (on(s)) _piKinds[_piKind(s)]!.short,
      ],
      children: [
        _PiStage(
          key: const ValueKey('pi_a_stage'),
          height: 164,
          bounds: _piBoundsA,
          inks: [_PiInk(base, Grey.g26, 1), if (selOn && i > 0) _PiInk(selIn!, Grey.g38, 1.2), _PiInk(out, Pop.tonePath, 2.4)],
          marks: [if (start != null) (start, Grey.g63, 2.2)],
          handles: selOn ? _piHandlesA(doc, sel, selIn!) : const [],
          overlay: Dis(
            child: Segmented(items: _piShapes, index: _piShapes.indexOf(shape), onChanged: (i) => doc.str('pi_a_shape', _piShapes[i])),
          ),
          note: sel == null ? 'No path operations' : (selOn ? 'Drag on the path: ${_piTitle(order, sel)}' : '${_piTitle(order, sel)} is off'),
        ),
        const SizedBox(height: 4),
        if (order.isEmpty)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
            child: Text('No path operations. The path is drawn as built.', style: T.label(Grey.g56)),
          ),
        for (final s in order) ...[
          _PiARow(s, order, sel: s == sel, key: ValueKey('pi_a_row_$s')),
          if (s == sel) Padding(padding: const EdgeInsets.fromLTRB(0, 4, 0, 6), child: _PiFields('a', s)),
        ],
        const _PiAdd('a'),
      ],
    );
  }
}

List<_PiH> _piHandlesA(Doc doc, String inst, List<_PiC> inp) {
  if (inp.isEmpty || inp.first.segs.isEmpty) return const [];
  final c = inp.first, k = 'pi_a_$inst', g = _piG(doc, 'a', inst), op = _piKind(inst);
  void set(String p, double v) => doc.set('${k}_$p', v);
  switch (op) {
    case 'pucker':
      final ctr = _piCentre(c), s0 = c.segs.first, m = (s0.first + s0.last) / 2, r = (m - ctr).distance, u = _piUnit(m - ctr);
      if (r < 1) return const [];
      return [
        _PiH(
          '${k}_amount',
          m + u * (r * g('amount') / 200),
          (q) => set('amount', (_piDot(q - m, u) * 200 / r).clamp(-100.0, 100.0).roundToDouble()),
          from: m,
          tip: () => _piKinds[op]!.sum(g).first,
        ),
      ];
    case 'round':
      final h = _piCornerH('${k}_radius', c, c.closed ? 0 : 1, g('radius'), (v) => set('radius', v), () => 'Radius ${_px(g('radius'))}');
      return [?h];
    case 'wiggle':
      final p = c.flat;
      final (q0, t) = _piAt(p, _piLen(p) * .3);
      final n = _piN(t) * _piSign(c);
      return [
        _PiH(
          '${k}_size',
          q0 + n * g('size'),
          (q) => set('size', (math.max(0.0, _piDot(q - q0, n)) * 2).roundToDouble() / 2),
          from: q0,
          tip: () => 'Size ${_px(g('size'))}',
        ),
      ];
    case 'trim':
      final p = c.flat, l = _piLen(p), o = g('offset') / 360, s = g('start'), e = g('end');
      if (l < 1e-6) return const [];
      double w(double f) {
        final r = f - f.floorToDouble();
        return r == 0 && f > 0 ? 1 : r;
      }

      Offset at(double f) => _piAt(p, w(f) * l).$1;
      double frac(Offset q) => _piNear(p, q) / l;
      double pick(double f, double old) {
        var v = ((f - o) % 1) * 100;
        if ((v - old).abs() > 50) v = old > 50 ? 100 : 0;
        return v.roundToDouble();
      }

      final mid = (s + e) / 200 + o;
      return [
        _PiH('${k}_end', at(e / 100 + o), (q) => set('end', pick(frac(q), e)), tip: () => 'End ${brief(g('end'), 0)}%'),
        _PiH('${k}_start', at(s / 100 + o), (q) => set('start', pick(frac(q), s)), shape: 2, tip: () => 'Start ${brief(g('start'), 0)}%'),
        _PiH(
          '${k}_offset',
          at(mid),
          (q) {
            var d = frac(q) - w(mid) % 1;
            d -= d.roundToDouble();
            set('offset', g('offset') + d * 360);
          },
          shape: 1,
          color: Pop.tonePath,
          tip: () => 'Offset ${brief(g('offset'), 0)}°',
        ),
      ];
  }
  return const [];
}

class _PiARow extends StatelessWidget {
  const _PiARow(this.inst, this.order, {super.key, required this.sel});
  final String inst;
  final List<String> order;
  final bool sel;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, on = doc.b['pi_a_$inst.on'] ?? true, op = _piKind(inst), kd = _piKinds[op]!;
    return Hov(
      onTap: () => doc.str('pi_a_sel', inst),
      builder: (_, h) => Container(
        height: Pop.row,
        padding: const EdgeInsets.only(left: 6),
        decoration: BoxDecoration(color: sel ? Grey.g20 : (h ? Grey.g15 : null), borderRadius: BorderRadius.circular(6)),
        child: Row(
          children: [
            _OpGlyph(op, hot: sel),
            const SizedBox(width: 8),
            SizedBox(
              width: 104,
              child: Text(
                _piTitle(order, inst),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: T.name(!on ? Grey.g56 : (sel || h ? Grey.g100 : Grey.g76)).copyWith(decoration: on ? null : TextDecoration.lineThrough),
              ),
            ),
            Expanded(child: _PiChips(kd.sum(_piG(doc, 'a', inst)), dim: !on)),
            OnOff(on: on, onChanged: (v) => doc.flag('pi_a_$inst.on', v)),
            _PiMore('a', inst),
          ],
        ),
      ),
    );
  }
}

// ---- B: before → after strip ----------------------------------------------------------------------------------------
const _piBoundsB = Rect.fromLTRB(-118, -78, 118, 78);

/// Up to this many instances the strip's frames share its width; past it they keep a fixed width and the strip scrolls sideways.
const _piStripFit = 5;

/// The stack as a film strip (one frame per instance: the shape after it) over cards whose heads carry an in → out pair;
/// open, a card is a wipe-compare stage.
class _PiB extends StatelessWidget {
  const _PiB();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed({}, strs: const {'pi_b_order': 'zigzag1,round2,offset3,twist4', 'pi_b_next': '5'}, flags: const {'pi_b_zigzag1.open': true});
    final order = _piOrder(doc, 'b'), base = [_piRect], fit = order.length <= _piStripFit;
    final (ins, out) = _piRun(doc, 'b', base, order);
    List<_PiC> after(int i) => i + 1 < ins.length ? ins[i + 1] : out;
    final frames = Row(
      mainAxisSize: fit ? MainAxisSize.max : MainAxisSize.min,
      children: [
        _PiFrame('Path', base, null, width: fit ? null : 48),
        for (final (i, s) in order.indexed) ...[
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Text(' › ', style: T.name(Grey.g44)),
          ),
          _PiFrame(_piTitle(order, s, short: true), after(i), s, width: fit ? null : 48),
        ],
      ],
    );
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          color: Pop.card,
          margin: const EdgeInsets.only(bottom: Pop.gap),
          padding: const EdgeInsets.symmetric(horizontal: Pop.inset, vertical: Pop.insetY),
          child: fit ? frames : SingleChildScrollView(scrollDirection: Axis.horizontal, child: frames),
        ),
        for (final (i, s) in order.indexed) _PiBCard(s, _piTitle(order, s), ins[i], after(i), key: ValueKey('pi_b_$s')),
        const _PiAdd('b', card: true),
      ],
    );
  }
}

class _PiFrame extends StatelessWidget {
  const _PiFrame(this.label, this.cs, this.op, {this.width});
  final String label;
  final List<_PiC> cs;
  final String? op;
  final double? width;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, o = op, on = o == null || (doc.b['pi_b_$o.on'] ?? true), open = o != null && (doc.b['pi_b_$o.open'] ?? false);
    final w = width;
    Widget box(Widget child) => w == null ? Expanded(child: child) : SizedBox(width: w, child: child);
    return box(
      Hov(
        onTap: o == null ? null : () => doc.flag('pi_b_$o.open', !open),
        cursor: o == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
        builder: (_, h) => Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              height: 40,
              decoration: BoxDecoration(
                color: h ? Grey.g15 : Grey.g10,
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: open ? Pop.tonePath.withValues(alpha: .6) : const Color(0x00000000)),
              ),
              child: _PiThumb(cs, o == null ? Grey.g63 : (on ? Pop.tonePath : Grey.g44)),
            ),
            const SizedBox(height: 3),
            Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: T.micro(!on ? Grey.g44 : (open || h ? Grey.g95 : Grey.g63)).copyWith(decoration: on ? null : TextDecoration.lineThrough),
            ),
          ],
        ),
      ),
    );
  }
}

class _PiBCard extends StatefulWidget {
  const _PiBCard(this.inst, this.title, this.inp, this.out, {super.key});
  final String inst, title;
  final List<_PiC> inp, out;
  @override
  State<_PiBCard> createState() => _PiBCardState();
}

class _PiBCardState extends State<_PiBCard> {
  double _wipe = -30;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, inst = widget.inst, op = _piKind(inst), k = 'pi_b_$inst', kd = _piKinds[op]!, g = _piG(doc, 'b', inst);
    final on = doc.b['$k.on'] ?? true, open = doc.b['$k.open'] ?? false, b = _piBoundsB;
    return Sect(
      title: widget.title,
      tone: Pop.tonePath,
      mark: CardMark.path,
      dim: !on,
      open: open,
      onToggle: () => doc.flag('$k.open', !open),
      brief: kd.sum(g),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _PiThumb(widget.inp, Grey.g56, w: 24, h: 16),
          Text(' › ', style: T.label(Grey.g44)),
          _PiThumb(widget.out, on ? Pop.tonePath : Grey.g44, w: 24, h: 16),
          const SizedBox(width: 10),
          OnOff(on: on, onChanged: (v) => doc.flag('$k.on', v)),
          _PiMore('b', inst),
        ],
      ),
      children: [
        _PiStage(
          key: ValueKey('${k}_stage'),
          height: 128,
          bounds: b,
          wipe: _wipe,
          rings: [if (op == 'twist' && on) (Offset(g('cx'), g('cy')), 46.0)],
          inks: [_PiInk(widget.inp, Grey.g20, 1), _PiInk(widget.inp, Grey.g76, 1.4, side: -1), _PiInk(widget.out, Pop.tonePath, 2.2, side: 1)],
          handles: [
            if (on) ..._piHandlesB(doc, inst, widget.inp),
            _PiH(
              '${k}_wipe',
              Offset(_wipe, b.top + 9),
              (q) => setState(() => _wipe = q.dx.clamp(b.left + 6, b.right - 6)),
              shape: 3,
              color: Grey.g63,
              tip: () => 'Drag to compare before and after',
            ),
          ],
          note: on ? null : 'Off: its input passes through',
        ),
        const SizedBox(height: 6),
        _PiFields('b', inst),
      ],
    );
  }
}

List<_PiH> _piHandlesB(Doc doc, String inst, List<_PiC> inp) {
  if (inp.isEmpty || inp.first.segs.isEmpty) return const [];
  final c = inp.first, k = 'pi_b_$inst', g = _piG(doc, 'b', inst), sg = _piSign(c), op = _piKind(inst);
  void set(String p, double v) => doc.set('${k}_$p', v);
  switch (op) {
    case 'round':
      var at = 0, best = double.infinity;
      for (final (i, s) in c.segs.indexed) {
        final d = (s.first - const Offset(80, -50)).distance;
        if (d < best) (at, best) = (i, d);
      }
      return [?_piCornerH('${k}_radius', c, at, g('radius'), (v) => set('radius', v), () => 'Radius ${_px(g('radius'))}')];
    case 'zigzag':
      final s = c.segs.first, l = _piLen(s), n = g('ridges').round().clamp(1, 60);
      if (l < 4) return const [];
      final (b0, t) = _piAt(s, l / (n + 1));
      return [
        _PiH(
          '${k}_peak',
          b0 + _piN(t) * (sg * g('amp')),
          (q) {
            final d = _piNear(s, q).clamp(l / 61, l / 2);
            final (p, tt) = _piAt(s, d);
            doc.setMany({'${k}_ridges': (l / d - 1).roundToDouble().clamp(1, 60), '${k}_amp': (_piDot(q - p, _piN(tt) * sg) * 2).roundToDouble() / 2});
          },
          from: b0,
          tip: () => 'Size ${_px(g('amp'))} · ${brief(g('ridges'), 0)} ridges (drag along to space them)',
        ),
      ];
    case 'offset':
      final p = c.flat, d = _piNear(p, const Offset(80, 0)), q0 = _piAt(p, d).$1;
      final n = _piN(_piUnit(_piAt(p, d + 14).$1 - _piAt(p, d - 14).$1)) * sg;
      return [
        _PiH(
          '${k}_amount',
          q0 + n * g('amount'),
          (q) => set('amount', (_piDot(q - q0, n) * 2).roundToDouble() / 2),
          from: q0,
          tip: () => 'Amount ${_sg(g('amount'), 1)} px',
        ),
      ];
    case 'twist':
      final ctr = Offset(g('cx'), g('cy')), ang = g('angle');
      return [
        _PiH(
          '${k}_angle',
          ctr + Offset.fromDirection((ang - 90) * math.pi / 180, 46),
          (q) {
            var d = math.atan2(q.dy - ctr.dy, q.dx - ctr.dx) * 180 / math.pi + 90 - ang;
            d -= (d / 360).roundToDouble() * 360;
            set('angle', (ang + d).roundToDouble());
          },
          from: ctr,
          tip: () => 'Angle ${_sg(g('angle'))}° (keep turning past a full turn)',
        ),
        _PiH(
          '${k}_centre',
          ctr,
          (q) => doc.setMany({'${k}_cx': q.dx.roundToDouble(), '${k}_cy': q.dy.roundToDouble()}),
          shape: 2,
          tip: () => 'Centre ${brief(g('cx'), 0)}, ${brief(g('cy'), 0)}',
        ),
      ];
  }
  return const [];
}

// ---- C: draw-on timeline --------------------------------------------------------------------------------------------
const _piCKeys = ['s0', 's1', 'e0', 'e1'], _piEases = ['Linear', 'Ease', 'Ease out'];
const _piBoundsC = Rect.fromLTRB(-128, -60, 128, 60);
const _piPresets = <String, Map<String, double>>{
  'Draw on': {'s0t': 0, 's0v': 0, 's1t': 2, 's1v': 0, 'e0t': 0, 'e0v': 0, 'e1t': 1.2, 'e1v': 100},
  'Draw off': {'s0t': .4, 's0v': 0, 's1t': 1.6, 's1v': 100, 'e0t': 0, 'e0v': 100, 'e1t': 2, 'e1v': 100},
  'Write & erase': {'s0t': .7, 's0v': 0, 's1t': 1.7, 's1v': 100, 'e0t': 0, 'e0v': 0, 'e1t': 1, 'e1v': 100},
  'From middle': {'s0t': 0, 's0v': 50, 's1t': 1.2, 's1v': 0, 'e0t': 0, 'e0v': 50, 'e1t': 1.2, 'e1v': 100},
};

double _piEase(int k, double u) => switch (k) {
  1 => u * u * (3 - 2 * u),
  2 => 1 - math.pow(1 - u, 3).toDouble(),
  _ => u,
};

/// The ease's curve, small: what the Ease link says is set.
class _PiEaseGlyph extends CustomPainter {
  const _PiEaseGlyph(this.ease, this.ink);
  final int ease;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final r = Rect.fromLTWH(1, 1, s.width - 2, s.height - 2), p = Path()..moveTo(r.left, r.bottom);
    for (var i = 1; i <= 16; i++) {
      p.lineTo(r.left + r.width * i / 16, r.bottom - r.height * _piEase(ease, i / 16));
    }
    c.drawPath(
      p,
      Paint()
        ..color = ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.3
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round,
    );
  }

  @override
  bool shouldRepaint(_PiEaseGlyph o) => o.ease != ease || o.ink != ink;
}

/// A Trim lane's value at [t] between its two keys.
double _piLane(Map<String, double> v, String lane, double t, int ease) {
  var (t0, v0, t1, v1) = (v['${lane}0t']!, v['${lane}0v']!, v['${lane}1t']!, v['${lane}1v']!);
  if (t1 < t0) (t0, v0, t1, v1) = (t1, v1, t0, v0);
  if (t <= t0) return v0;
  if (t >= t1) return v1;
  return v0 + (v1 - v0) * _piEase(ease, (t - t0) / (t1 - t0));
}

Map<String, double> _piCVals(Doc d) => {
  for (final k in _piCKeys)
    for (final p in const ['t', 'v']) '$k$p': d.get('pi_c_$k$p') ?? 0,
};

/// Trim Paths as an animation designed in place: the stage shows the stroke at the hovered (or current) time, with the heads' frame-by-frame spacing.
class _PiCTrim extends StatefulWidget {
  const _PiCTrim();
  @override
  State<_PiCTrim> createState() => _PiCTrimState();
}

class _PiCTrimState extends State<_PiCTrim> {
  double? _peek;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      {for (final e in _piPresets['Write & erase']!.entries) 'pi_c_${e.key}': e.value, 'pi_c_off': 0, 'pi_c_ease': 1, 'pi_c_t': .8},
      strs: const {'pi_c_sel': 'e1'},
    );
    final vals = _piCVals(doc), ease = (doc.get('pi_c_ease') ?? 0).round().clamp(0, 2), off = doc.get('pi_c_off') ?? 0;
    final t = _peek ?? doc.get('pi_c_t') ?? 0, s = _piLane(vals, 's', t, ease), e = _piLane(vals, 'e', t, ease);
    final open = doc.b['pi_c.open'] ?? true, sel = doc.s2['pi_c_sel']!;
    final p = _piScript.flat, l = _piLen(p);
    Offset at(double v) {
      final f = v / 100 + off / 360, r = f - f.floorToDouble();
      return _piAt(p, (r == 0 && f > 0 ? 1 : r) * l).$1;
    }

    final marks = <(Offset, Color, double)>[
      for (final lane in const ['s', 'e'])
        for (var f = 0; f <= 60; f += 2)
          if (f / 30 >= math.min(vals['${lane}0t']!, vals['${lane}1t']!) && f / 30 <= math.max(vals['${lane}0t']!, vals['${lane}1t']!))
            (at(_piLane(vals, lane, f / 30, ease)), lane == 'e' ? Grey.g56 : Grey.g38, 1.4),
      (at(s), Grey.g76, 3),
      (at(e), Pop.accent, 3.5),
    ];
    final preset = _piPresets.entries.where((m) => m.value.entries.every((x) => (vals[x.key]! - x.value).abs() < 1e-6)).firstOrNull?.key;
    final keyName = '${sel[0] == 's' ? 'Start' : 'End'} ${sel[1] == '0' ? 1 : 2}';
    return Sect(
      title: 'Trim Paths',
      tone: Pop.tonePath,
      mark: CardMark.path,
      open: open,
      onToggle: () => doc.flag('pi_c.open', !open),
      keyIds: const ['pi_c'],
      brief: [preset ?? 'Custom', _piEases[ease], '${brief(math.max(vals['s1t']!, vals['e1t']!), 1)}s'],
      children: [
        _PiStage(
          key: const ValueKey('pi_c_stage'),
          height: 120,
          bounds: _piBoundsC,
          inks: [
            _PiInk([_piScript], Grey.g26, 1.2),
            _PiInk(_piTrim(_piScript, s, e, off), Pop.tonePath, 2.6),
          ],
          marks: marks,
          note: '${t.toStringAsFixed(2)}s${_peek != null ? ' (hover)' : ''} · Start ${brief(s, 0)}% · End ${brief(e, 0)}%',
        ),
        const SizedBox(height: 6),
        _PiTimeline(
          onPeek: (v) {
            if (v != _peek) setState(() => _peek = v);
          },
        ),
        const SizedBox(height: 6),
        PanelLink(
          to: LinkTo.ease,
          label: 'Ease',
          value: _piEases[ease],
          lead: CustomPaint(size: const Size(16, 12), painter: _PiEaseGlyph(ease, Pop.tonePath)),
        ),
        const SizedBox(height: 6),
        Row(
          children: [
            for (final (i, name) in _piPresets.keys.indexed) ...[
              if (i > 0) const SizedBox(width: 4),
              Expanded(
                child: Hov(
                  onTap: () {
                    doc.beginGesture();
                    doc.setMany({for (final x in _piPresets[name]!.entries) 'pi_c_${x.key}': x.value});
                    doc.endGesture();
                  },
                  builder: (_, h) => Container(
                    height: 20,
                    padding: const EdgeInsets.symmetric(horizontal: 6),
                    alignment: Alignment.center,
                    decoration: BoxDecoration(
                      color: preset == name ? Grey.g26 : (h ? Grey.g20 : Pop.well),
                      borderRadius: BorderRadius.circular(10),
                      border: Border.all(color: preset == name ? Pop.accentInk.withValues(alpha: .7) : const Color(0x00000000)),
                    ),
                    child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(preset == name || h ? Grey.g95 : Grey.g76)),
                  ),
                ),
              ),
            ],
          ],
        ),
        const SizedBox(height: 6),
        Row(
          children: [
            SizedBox(
              width: _PiTl.lab + 12,
              child: Text(keyName, style: T.label(Grey.g76).copyWith(fontWeight: FontWeight.w600)),
            ),
            Expanded(
              child: NumField(id: 'pi_c_${sel}t', label: 'Time', unit: 's', decimals: 2, min: 0, max: _PiTl.dur, perPx: .01, step: 1 / 30),
            ),
            const SizedBox(width: 4),
            Expanded(
              child: NumField(id: 'pi_c_${sel}v', label: 'Value', unit: '%', min: 0, max: 100),
            ),
            const SizedBox(width: 4),
            const Expanded(
              child: NumField(id: 'pi_c_off', label: 'Offset', unit: '°'),
            ),
          ],
        ),
      ],
    );
  }
}

class _PiTl {
  _PiTl(this.w);
  final double w;
  static const dur = 2.0, lab = 36.0, ruler = 14.0, laneH = 24.0, gap = 4.0, h = ruler + laneH * 2 + gap;
  double tx(double t) => lab + 5 + t / dur * (w - lab - 10);
  double xt(double x) => ((x - lab - 5) / (w - lab - 10) * dur).clamp(0.0, dur);
  double top(int i) => ruler + i * (laneH + gap);
  double vy(int i, double v) => top(i) + 3 + (1 - v / 100) * (laneH - 6);
  double yv(int i, double y) => ((1 - (y - top(i) - 3) / (laneH - 6)) * 100).clamp(0.0, 100.0);
  Offset key(String k, Map<String, double> v) => Offset(tx(v['${k}t']!), vy(k[0] == 's' ? 0 : 1, v['${k}v']!));
}

/// Start and End as two lanes of value over time: drag a key (time sideways, value up and down), press elsewhere to move the playhead.
/// contract: hovering reports the time under the pointer (the stage previews it); nothing runs between pointer events.
class _PiTimeline extends StatefulWidget {
  const _PiTimeline({required this.onPeek});
  final ValueChanged<double?> onPeek;
  @override
  State<_PiTimeline> createState() => _PiTimelineState();
}

class _PiTimelineState extends State<_PiTimeline> {
  String? _drag, _hot;
  bool _scrub = false;
  double? _hover;

  String? _pick(_PiTl g, Map<String, double> v, Offset p) {
    String? best;
    var bd = 9.0;
    for (final k in _piCKeys) {
      final d = (g.key(k, v) - p).distance;
      if (d < bd) {
        bd = d;
        best = k;
      }
    }
    return best;
  }

  void _end(Doc doc) {
    if (_drag == null && !_scrub) return;
    doc.endGesture();
    setState(() {
      _drag = null;
      _scrub = false;
    });
    widget.onPeek(_hover);
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked;
    return LayoutBuilder(
      builder: (context, box) {
        final g = _PiTl(box.maxWidth), v = _piCVals(doc), t = doc.get('pi_c_t') ?? 0;
        double snap(double s) => (s * 30).roundToDouble() / 30;
        void move(Offset p) {
          final k = _drag;
          if (k != null) {
            final other = v['${k[0]}${k[1] == '0' ? 1 : 0}t']!;
            final tt = (k[1] == '0' ? math.min(snap(g.xt(p.dx)), other - 1 / 30) : math.max(snap(g.xt(p.dx)), other + 1 / 30)).clamp(0.0, _PiTl.dur);
            doc.setMany({'pi_c_${k}t': tt, 'pi_c_${k}v': g.yv(k[0] == 's' ? 0 : 1, p.dy).roundToDouble()});
            widget.onPeek(tt);
          } else if (_scrub) {
            doc.set('pi_c_t', snap(g.xt(p.dx)));
          }
        }

        return MouseRegion(
          cursor: !on ? SystemMouseCursors.basic : (_drag != null || _hot != null ? SystemMouseCursors.move : SystemMouseCursors.precise),
          onHover: (e) {
            final ht = e.localPosition.dx >= _PiTl.lab ? g.xt(e.localPosition.dx) : null;
            setState(() {
              _hot = on ? _pick(g, v, e.localPosition) : null;
              _hover = ht;
            });
            widget.onPeek(ht);
          },
          onExit: (_) {
            setState(() {
              _hot = null;
              _hover = null;
            });
            if (_drag == null && !_scrub) widget.onPeek(null);
          },
          child: Listener(
            behavior: HitTestBehavior.opaque,
            onPointerDown: (e) {
              if (!on || e.buttons != kPrimaryButton) return;
              final k = _pick(g, v, e.localPosition);
              if (k == null && e.localPosition.dx < _PiTl.lab) return;
              if (k != null) doc.str('pi_c_sel', k);
              doc.beginGesture();
              setState(() {
                _drag = k;
                _scrub = k == null;
              });
              if (k == null) move(e.localPosition);
            },
            onPointerMove: (e) => move(e.localPosition),
            onPointerUp: (_) => _end(doc),
            onPointerCancel: (_) => _end(doc),
            child: SizedBox(
              height: _PiTl.h,
              child: Stack(
                children: [
                  Positioned.fill(
                    child: CustomPaint(painter: _PiTlPaint(g, v, (doc.get('pi_c_ease') ?? 0).round(), doc.s2['pi_c_sel'] ?? '', t, _hover, _drag ?? _hot)),
                  ),
                  for (final k in _piCKeys)
                    Positioned(
                      key: ValueKey('h:pi_c_$k'),
                      left: g.key(k, v).dx - 6,
                      top: g.key(k, v).dy - 6,
                      width: 12,
                      height: 12,
                      child: const IgnorePointer(child: SizedBox()),
                    ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}

class _PiTlPaint extends CustomPainter {
  _PiTlPaint(this.g, this.v, this.ease, this.sel, this.t, this.hover, this.hot);
  final _PiTl g;
  final Map<String, double> v;
  final int ease;
  final String sel;
  final double t;
  final double? hover;
  final String? hot;

  void _text(Canvas c, String s, Offset at, Color col, {bool centre = false}) {
    final tp = TextPainter(
      text: TextSpan(text: s, style: T.micro(col)),
      textDirection: TextDirection.ltr,
    )..layout();
    tp.paint(c, at - Offset(centre ? tp.width / 2 : 0, tp.height / 2));
  }

  @override
  void paint(Canvas c, Size s) {
    final tick = Paint()
      ..color = Grey.g38
      ..strokeWidth = 1;
    for (var q = 0; q <= 8; q++) {
      final x = g.tx(q * .25);
      c.drawLine(Offset(x, _PiTl.ruler - (q % 4 == 0 ? 6 : 3)), Offset(x, _PiTl.ruler - 1), tick);
      if (q % 4 == 0) _text(c, '${q ~/ 4}s', Offset((x + (q == 0 ? 8 : (q == 8 ? -8 : 0))).toDouble(), 4), Grey.g56, centre: true);
    }
    for (final (i, lane, name) in const [(0, 's', 'Start'), (1, 'e', 'End')]) {
      final top = g.top(i);
      c.drawRRect(
        RRect.fromRectAndRadius(Rect.fromLTWH(_PiTl.lab, top, s.width - _PiTl.lab, _PiTl.laneH), const Radius.circular(4)),
        Paint()..color = Grey.g10,
      );
      _text(c, name, Offset(0, top + _PiTl.laneH / 2), Grey.g63);
      final curve = Path(), fillP = Path()..moveTo(g.tx(0), top + _PiTl.laneH);
      for (var k = 0; k <= 60; k++) {
        final q = Offset(g.tx(k / 30), g.vy(i, _piLane(v, lane, k / 30, ease)));
        k == 0 ? curve.moveTo(q.dx, q.dy) : curve.lineTo(q.dx, q.dy);
        fillP.lineTo(q.dx, q.dy);
      }
      fillP
        ..lineTo(g.tx(_PiTl.dur), top + _PiTl.laneH)
        ..close();
      c.drawPath(fillP, Paint()..color = Pop.tonePath.withValues(alpha: .12));
      c.drawPath(
        curve,
        Paint()
          ..color = Pop.tonePath.withValues(alpha: .9)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.4,
      );
    }
    final ph = g.tx(t);
    c.drawLine(Offset(ph, 0), Offset(ph, s.height), Paint()..color = Pop.accentInk);
    c.drawPath(Path()..addPolygon([Offset(ph - 4, 0), Offset(ph + 4, 0), Offset(ph, 5)], true), Paint()..color = Pop.accentInk);
    final hv = hover;
    if (hv != null && (g.tx(hv) - ph).abs() > 1) {
      final hx = g.tx(hv);
      c.drawLine(Offset(hx, _PiTl.ruler), Offset(hx, s.height), Paint()..color = Grey.g63);
      final tp = TextPainter(
        text: TextSpan(text: '${hv.toStringAsFixed(2)}s', style: T.micro(Grey.g95)),
        textDirection: TextDirection.ltr,
      )..layout();
      final r = Rect.fromLTWH((hx - tp.width / 2 - 3).clamp(_PiTl.lab, s.width - tp.width - 6), 0, tp.width + 6, 12);
      c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(3)), Paint()..color = Grey.g26);
      tp.paint(c, r.topLeft + Offset(3, (12 - tp.height) / 2));
    }
    for (final k in _piCKeys) {
      final q = g.key(k, v), r = k == hot ? 6.0 : 4.5;
      final d = Path()..addPolygon([q + Offset(0, -r), q + Offset(r, 0), q + Offset(0, r), q + Offset(-r, 0)], true);
      c.drawPath(d, Paint()..color = Pop.keyDot);
      c.drawPath(
        d,
        Paint()
          ..color = k == sel ? Grey.g100 : Grey.g07
          ..style = PaintingStyle.stroke
          ..strokeWidth = k == sel ? 1.6 : 1.2,
      );
    }
  }

  @override
  bool shouldRepaint(_PiTlPaint o) => true;
}
