// Pop 10: two sheets of 20 pop miniatures each (Position / Transform, Macro / Variations). One sheet = one screenshot:
// 5 x 4 panels of 156 x 116 with a 2-line label under each. Every panel reads the same small state (_S) and paints
// its own world; a panel that needs physics brings a step function. One Ticker per sheet drives idle life and physics.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'pop_10_pos.dart';
part 'pop_10_macro.dart';

WidgetbookComponent pop10Set() => WidgetbookComponent(name: 'Pop 10', useCases: [
      WidgetbookUseCase(name: 'Position / Transform x20', builder: (c) => _P10Sheet(concept: 'Position', panels: _posPanels())),
      WidgetbookUseCase(name: 'Macro / Variations x20', builder: (c) => _P10Sheet(concept: 'Macro', panels: _macroPanels())),
    ]);

const _k0 = Color(0xFF1C1C1E), _k1 = Color(0xFF2A2A2E), _cr = Color(0xFFF4EBDD), _wh = Color(0xFFFFFFFF);
const _or = Color(0xFFFF7A3D), _ye = Color(0xFFFFD23F), _cy = Color(0xFF3DD6F5), _li = Color(0xFF9BE564);
const _pk = Color(0xFFFF5DA2), _vi = Color(0xFF9B7BFF), _rd = Color(0xFFFF4B4B), _bl = Color(0xFF2E5BFF);
const _pop = [_or, _ye, _cy, _li, _pk, _vi, _rd, _bl];

const double _cw = 156, _ch = 116;

/// Panel state. x/y: drag right/down (0..1). a: angle swept round the centre. dist: rubbed path length.
/// flings: fast releases. taps. p: pointer. vel: release velocity. b: panel-owned physics buffer.
class _S {
  _S(this.x, this.y);
  double x, y, a = 0, dist = 0, t = 0, rel = -9;
  int taps = 0, flings = 0, ends = 0;
  bool down = false;
  Offset? p;
  Offset vel = Offset.zero;
  final trail = <Offset>[];
  final b = <double>[];
}

typedef _Fn = void Function(Canvas c, Size s, _S st);
typedef _Step = void Function(_S st, double dt);

class _Pn {
  const _Pn(this.name, this.tags, this.paint, {this.x0 = .5, this.y0 = .5, this.step, this.light = false});
  final String name, tags;
  final _Fn paint;
  final double x0, y0;
  final _Step? step;
  final bool light;
}

double _wr(double x) => x - x.floorToDouble();
double _rn(int i) {
  final x = math.sin(i * 12.9898 + 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

double _cl(double v, [double lo = 0, double hi = 1]) => v.clamp(lo, hi).toDouble();
double _ss(double v) {
  final x = _cl(v);
  return x * x * (3 - 2 * x);
}

Paint _f(Color c) => Paint()..color = c;
Paint _s(Color c, [double w = 2]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
void _bg(Canvas c, Size s, Color col) => c.drawRect(Offset.zero & s, _f(col));
Color _al(Color c, double a) => c.withValues(alpha: _cl(a));
Color _mix(Color a, Color b, double t) => Color.lerp(a, b, _cl(t))!;
Color _h(double hue, [double sat = .9, double l = .6]) => HSLColor.fromAHSL(1, _wr(hue) * 360, _cl(sat), _cl(l)).toColor();
Offset _pol(double ang, double r) => Offset(math.cos(ang), math.sin(ang)) * r;
Path _poly(List<Offset> p) => Path()..addPolygon(p, true);

Path _star(Offset c, double r, double rot, {int k = 5, double inner = .45}) => _poly([
      for (var i = 0; i < k * 2; i++) c + _pol(rot - math.pi / 2 + i * math.pi / k, i.isEven ? r : r * inner),
    ]);

Path _ngon(Offset c, double r, int k, double rot) => _poly([for (var i = 0; i < k; i++) c + _pol(rot - math.pi / 2 + i * 2 * math.pi / k, r)]);

void _vgrad(Canvas c, Rect r, Color top, Color bot) => c.drawRect(
    r, Paint()..shader = ui.Gradient.linear(r.topCenter, r.bottomCenter, [top, bot]));

/// A small sample composition (the "work"): tinted ground, a sun, and n shapes; one mood = (hue, energy, seed).
void _comp(Canvas c, Rect r, double hue, double energy, int seed, {double round = 6}) {
  c.save();
  c.clipRRect(RRect.fromRectAndRadius(r, Radius.circular(round)));
  _vgrad(c, r, _h(hue, .55, .22), _h(hue + .08, .6, .34));
  final n = 3 + (energy * 7).round();
  for (var i = 0; i < n; i++) {
    final q = seed * 31 + i * 7;
    final at = Offset(r.left + r.width * (.1 + .8 * _rn(q)), r.top + r.height * (.15 + .7 * _rn(q + 1)));
    final sz = r.shortestSide * (.08 + .16 * _rn(q + 2)) * (1.2 - energy * .4);
    final col = _h(hue + (_rn(q + 3) - .5) * (.15 + energy * .5), .9, .6);
    switch ((_rn(q + 4) * 3).floor()) {
      case 0:
        c.drawCircle(at, sz, _f(col));
      case 1:
        c.drawPath(_star(at, sz * 1.3, _rn(q + 5) * 6, k: 4 + (energy * 4).round()), _f(col));
      default:
        c.drawPath(_ngon(at, sz * 1.2, 3, _rn(q + 5) * 6), _f(col));
    }
  }
  c.restore();
}

/// Total length of a polyline, the point and heading at distance d along it, and the part before d.
double _plen(List<Offset> p) {
  var l = 0.0;
  for (var i = 1; i < p.length; i++) {
    l += (p[i] - p[i - 1]).distance;
  }
  return l;
}

(Offset, double) _at(List<Offset> p, double d) {
  for (var i = 1; i < p.length; i++) {
    final seg = p[i] - p[i - 1], l = seg.distance;
    if (d <= l && l > 0) return (p[i - 1] + seg * (d / l), seg.direction);
    d -= l;
  }
  final n = p.length;
  return (p.last, n > 1 ? (p[n - 1] - p[n - 2]).direction : 0);
}

List<Offset> _sub(List<Offset> p, double d) {
  final out = <Offset>[p.first];
  for (var i = 1; i < p.length; i++) {
    final seg = p[i] - p[i - 1], l = seg.distance;
    if (d <= l) {
      if (l > 0) out.add(p[i - 1] + seg * (d / l));
      return out;
    }
    out.add(p[i]);
    d -= l;
  }
  return out;
}

void _line(Canvas c, List<Offset> p, Paint paint) {
  if (p.length < 2) return;
  final path = Path()..moveTo(p.first.dx, p.first.dy);
  for (final q in p.skip(1)) {
    path.lineTo(q.dx, q.dy);
  }
  c.drawPath(path, paint);
}

void _dash(Canvas c, List<Offset> p, Paint paint, [double on = 5, double off = 4]) {
  final l = _plen(p);
  for (var d = 0.0; d < l; d += on + off) {
    final a = _at(p, d).$1, b = _at(p, math.min(l, d + on)).$1;
    c.drawLine(a, b, paint);
  }
}

class _P10Sheet extends StatefulWidget {
  const _P10Sheet({required this.concept, required this.panels});
  final String concept;
  final List<_Pn> panels;
  @override
  State<_P10Sheet> createState() => _P10SheetState();
}

class _P10SheetState extends State<_P10Sheet> with SingleTickerProviderStateMixin {
  late final Ticker _tk;
  final _t = ValueNotifier<double>(0);
  late final List<_S> _st = [for (final p in widget.panels) _S(p.x0, p.y0)];
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tk = createTicker(_tick)..start();
  }

  void _tick(Duration d) {
    final dt = ((d - _last).inMicroseconds / 1e6).clamp(0.0, .05).toDouble();
    _last = d;
    final now = d.inMicroseconds / 1e6;
    for (var i = 0; i < _st.length; i++) {
      _st[i].t = now;
      widget.panels[i].step?.call(_st[i], dt);
    }
    _t.value = now;
  }

  @override
  void dispose() {
    _tk.dispose();
    _t.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ps = widget.panels;
    return ColoredBox(
      color: const Color(0xFF111113),
      child: Center(
        child: FittedBox(
          fit: BoxFit.scaleDown,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(mainAxisSize: MainAxisSize.min, children: [
              for (var r = 0; r < (ps.length + 4) ~/ 5; r++)
                Padding(
                  padding: EdgeInsets.only(top: r == 0 ? 0 : 8),
                  child: Row(mainAxisSize: MainAxisSize.min, children: [
                    for (var k = 0; k < 5 && r * 5 + k < ps.length; k++)
                      Padding(
                        padding: EdgeInsets.only(left: k == 0 ? 0 : 8),
                        child: _Tile(n: r * 5 + k + 1, concept: widget.concept, p: ps[r * 5 + k], st: _st[r * 5 + k], t: _t),
                      ),
                  ]),
                ),
            ]),
          ),
        ),
      ),
    );
  }
}

const _lab1 = TextStyle(fontSize: 10, height: 1.25, color: _cr, fontWeight: FontWeight.w600, decoration: TextDecoration.none);
const _lab2 = TextStyle(fontSize: 10, height: 1.25, color: Color(0x99F4EBDD), decoration: TextDecoration.none);

class _Tile extends StatelessWidget {
  const _Tile({required this.n, required this.concept, required this.p, required this.st, required this.t});
  final int n;
  final String concept;
  final _Pn p;
  final _S st;
  final ValueNotifier<double> t;

  static const _c = Offset(_cw / 2, _ch / 2);

  void _start(DragStartDetails d) {
    st
      ..down = true
      ..p = d.localPosition
      ..vel = Offset.zero;
    st.trail
      ..clear()
      ..add(d.localPosition);
  }

  void _move(DragUpdateDetails d) {
    final q = d.localPosition, o = q - d.delta;
    st
      ..p = q
      ..x = _cl(st.x + d.delta.dx / 110)
      ..y = _cl(st.y + d.delta.dy / 85)
      ..dist += d.delta.distance;
    var da = (q - _c).direction - (o - _c).direction;
    if (da > math.pi) da -= 2 * math.pi;
    if (da < -math.pi) da += 2 * math.pi;
    st.a += da;
    if ((q - st.trail.last).distance > 2.5) st.trail.add(q);
    if (st.trail.length > 200) st.trail.removeAt(0);
  }

  void _end(DragEndDetails d) {
    st
      ..down = false
      ..vel = d.velocity.pixelsPerSecond
      ..rel = st.t
      ..ends += 1;
    if (st.vel.distance > 250) st.flings++;
  }

  void _tap() {
    st.taps++;
    st.rel = st.t;
  }

  @override
  Widget build(BuildContext context) => SizedBox(
        width: _cw,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          GestureDetector(
            onPanStart: _start,
            onPanUpdate: _move,
            onPanEnd: _end,
            onTap: _tap,
            child: ClipRRect(
              borderRadius: BorderRadius.circular(8),
              child: CustomPaint(size: const Size(_cw, _ch), painter: _Painter(p, st, t)),
            ),
          ),
          const SizedBox(height: 5),
          Text('$concept #$n  ${p.name}', style: _lab1, maxLines: 1, overflow: TextOverflow.ellipsis, softWrap: false),
          Text(p.tags, style: _lab2, maxLines: 1, overflow: TextOverflow.ellipsis, softWrap: false),
        ]),
      );
}

class _Painter extends CustomPainter {
  _Painter(this.p, this.st, this.t) : super(repaint: t);
  final _Pn p;
  final _S st;
  final ValueNotifier<double> t;
  @override
  void paint(Canvas canvas, Size size) {
    _bg(canvas, size, p.light ? _cr : _k0);
    p.paint(canvas, size, st);
  }

  @override
  bool shouldRepaint(covariant _Painter old) => true;
}
