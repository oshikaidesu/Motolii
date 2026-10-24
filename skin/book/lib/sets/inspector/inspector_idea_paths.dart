import 'dart:math' as math;
import 'inspector_studies.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
part 'inspector_idea_paths_2.dart';
part 'inspector_idea_paths_3.dart';
part 'inspector_idea_paths_4.dart';

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
  'round': _PiK('Rounded Corners', 'Round', const {'radius': 8}, (g) => [px(g('radius'))]),
  'zigzag': _PiK('Zig Zag', 'Zig Zag', const {
    'amp': 6,
    'ridges': 5,
    'pt': 0,
  }, (g) => [px(g('amp')), '${brief(g('ridges'), 0)} ridges', if (g('pt') == 1) 'Smooth']),
  'offset': _PiK('Offset Paths', 'Offset', const {
    'amount': 6,
    'join': 1,
    'miter': 4,
  }, (g) => ['${sgPaths(g('amount'), 1)} px', joins[g('join').round().clamp(0, 2)]]),
  'twist': _PiK('Twist', 'Twist', const {'angle': 40, 'cx': 0, 'cy': 0}, (g) => ['${sgPaths(g('angle'))}°']),
  'wiggle': _PiK('Wiggle Paths', 'Wiggle', const {
    'size': 5,
    'detail': 6,
    'phase': 0,
    'seed': 1,
    'pt': 1,
  }, (g) => [px(g('size')), 'Seed ${brief(g('seed'), 0)}']),
  'trim': _PiK('Trim Paths', 'Trim', const {
    'start': 0,
    'end': 72,
    'offset': 0,
  }, (g) => ['${brief(g('start'), 0)}–${brief(g('end'), 0)}%', if (g('offset') != 0) '${sgPaths(g('offset'))}°']),
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
