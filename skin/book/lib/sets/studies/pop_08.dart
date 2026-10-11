// Pop 08: two sheets of 20 pop miniatures each (Repeat / Clone, Fill / Colour / Gradient). One sheet = one screenshot:
// 5 x 4 panels of 156 x 116 with a 2-line label under each. Every panel paints its own picture from two values (a, b)
// plus an optional drawn path; the gesture kind decides how a drag moves them. One Ticker per sheet drives idle life and flick momentum.
import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'pop_08_repeat.dart';
part 'pop_08_fill.dart';

WidgetbookComponent pop08Set() => WidgetbookComponent(name: 'Pop 08', useCases: [
      WidgetbookUseCase(name: 'Repeat / Clone x20', builder: (c) => _P8Sheet(concept: 'Repeat', panels: _repeatPanels())),
      WidgetbookUseCase(name: 'Fill / Colour / Gradient x20', builder: (c) => _P8Sheet(concept: 'Fill', panels: _fillPanels())),
    ]);

const _k0 = Color(0xFF1C1C1E), _k1 = Color(0xFF2A2A2E), _cr = Color(0xFFF4EBDD), _wh = Color(0xFFFFFFFF);
const _or = Color(0xFFFF7A3D), _ye = Color(0xFFFFD23F), _cy = Color(0xFF3DD6F5), _li = Color(0xFF9BE564);
const _pk = Color(0xFFFF5DA2), _vi = Color(0xFF9B7BFF), _rd = Color(0xFFFF4B4B), _bl = Color(0xFF2E5BFF);

const double _cw = 156, _ch = 116;

/// How a drag moves the panel's values. xy: dx -> a, up -> b. spin: angle round the centre -> a (wraps), radius -> b.
/// flick: like xy, and a keeps the throw's momentum. rub: path length -> b, dx -> a. paint: a fresh stroke. spray: strokes pile up.
enum _G { xy, spin, flick, rub, paint, spray }

typedef _Paint8 = void Function(Canvas c, Size s, _P8 st, double t);

class _Pan8 {
  const _Pan8(this.name, this.tags, this.g, this.paint, {this.a = .5, this.b = .5, this.wrap = false, this.tap = false});
  final String name, tags;
  final _G g;
  final _Paint8 paint;
  final double a, b;
  final bool wrap, tap;
}

class _P8 {
  _P8(this.a, this.b);
  double a, b, va = 0, rel = 0;
  final List<Offset> pts = [];
  Offset? at;
  bool down = false;
}

double _wr(double x) => x - x.floorToDouble();
double _rn(int i) {
  final x = math.sin(i * 12.9898 + 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

Paint _f(Color c) => Paint()..color = c;
Paint _s(Color c, [double w = 2]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
void _bg(Canvas c, Size s, Color col) => c.drawRect(Offset.zero & s, _f(col));
Color _h(double hue, [double sat = .9, double l = .6]) =>
    HSLColor.fromAHSL(1, _wr(hue) * 360, sat.clamp(0.0, 1.0).toDouble(), l.clamp(0.0, 1.0).toDouble()).toColor();
Color _al(Color c, double a) => c.withValues(alpha: a.clamp(0.0, 1.0).toDouble());
Path _poly(List<Offset> p) => Path()..addPolygon(p, true);

Path _star(Offset c, double r, double rot, {int k = 5, double inner = .45}) {
  final p = <Offset>[];
  for (var i = 0; i < k * 2; i++) {
    final a = rot - math.pi / 2 + i * math.pi / k, rr = i.isEven ? r : r * inner;
    p.add(c + Offset(math.cos(a), math.sin(a)) * rr);
  }
  return _poly(p);
}

/// Marks every [gap] px along a polyline: f(index, point, heading).
void _walk(List<Offset> p, double gap, void Function(int i, Offset at, double ang) f) {
  var rem = gap * .5, i = 0;
  for (var k = 1; k < p.length; k++) {
    final a = p[k - 1], b = p[k], d = (b - a).distance;
    if (d == 0) continue;
    final ang = math.atan2(b.dy - a.dy, b.dx - a.dx);
    var pos = 0.0;
    while (pos + rem <= d) {
      pos += rem;
      f(i++, a + (b - a) * (pos / d), ang);
      rem = gap;
    }
    rem -= d - pos;
  }
}

List<Offset> _defPath(Size s, double ph) => [
      for (var i = 0; i <= 24; i++)
        Offset(12 + (s.width - 24) * i / 24, s.height * (.56 - .26 * math.sin(i / 24 * math.pi * 1.6 + ph))),
    ];

class _P8Sheet extends StatefulWidget {
  const _P8Sheet({required this.concept, required this.panels});
  final String concept;
  final List<_Pan8> panels;
  @override
  State<_P8Sheet> createState() => _P8SheetState();
}

class _P8SheetState extends State<_P8Sheet> with SingleTickerProviderStateMixin {
  late final Ticker _tk;
  final _t = ValueNotifier<double>(0);
  late final List<_P8> _st = [for (final p in widget.panels) _P8(p.a, p.b)];
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tk = createTicker(_tick)..start();
  }

  void _tick(Duration d) {
    final dt = ((d - _last).inMicroseconds / 1e6).clamp(0.0, .1);
    _last = d;
    for (var i = 0; i < _st.length; i++) {
      final p = widget.panels[i], s = _st[i];
      if (p.g != _G.flick || s.down || s.va.abs() < 1e-4) continue;
      s.a += s.va * dt;
      s.va *= math.pow(.2, dt).toDouble();
      if (p.wrap) {
        s.a = _wr(s.a);
      } else if (s.a < 0 || s.a > 1) {
        s.a = s.a.clamp(0.0, 1.0);
        s.va = -s.va * .5;
      }
    }
    _t.value = d.inMicroseconds / 1e6;
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
    Widget tile(int i) => _P8Tile(n: i + 1, concept: widget.concept, p: ps[i], st: _st[i], t: _t);
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
                      Padding(padding: EdgeInsets.only(left: k == 0 ? 0 : 8), child: tile(r * 5 + k)),
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

class _P8Tile extends StatelessWidget {
  const _P8Tile({required this.n, required this.concept, required this.p, required this.st, required this.t});
  final int n;
  final String concept;
  final _Pan8 p;
  final _P8 st;
  final ValueNotifier<double> t;

  static const _c = Offset(_cw / 2, _ch / 2);

  void _start(DragStartDetails d) {
    st
      ..down = true
      ..at = d.localPosition
      ..va = 0;
    if (p.g == _G.paint) st.pts.clear();
    if (p.g == _G.paint || p.g == _G.spray) st.pts.add(d.localPosition);
  }

  void _move(DragUpdateDetails d) {
    final q = d.localPosition, dx = d.delta.dx / _cw, dy = -d.delta.dy / _ch;
    st.at = q;
    switch (p.g) {
      case _G.xy || _G.flick:
        st.a = p.wrap ? _wr(st.a + dx) : (st.a + dx).clamp(0.0, 1.0);
        st.b = (st.b + dy).clamp(0.0, 1.0);
      case _G.spin:
        final o = q - d.delta - _c, n = q - _c;
        var da = n.direction - o.direction;
        if (da > math.pi) da -= 2 * math.pi;
        if (da < -math.pi) da += 2 * math.pi;
        st.a = _wr(st.a + da / (2 * math.pi));
        st.b = (st.b + (n.distance - o.distance) / (_ch * .5)).clamp(0.0, 1.0);
      case _G.rub:
        st.a = (st.a + dx).clamp(0.0, 1.0);
        st.b = (st.b + d.delta.distance / (_cw * 4)).clamp(0.0, 1.0);
      case _G.paint || _G.spray:
        if ((q - st.pts.last).distance > 2.5) st.pts.add(q);
        if (st.pts.length > 260) st.pts.removeAt(0);
    }
  }

  void _end(DragEndDetails d) {
    st
      ..down = false
      ..rel = t.value;
    if (p.g == _G.flick) st.va = d.velocity.pixelsPerSecond.dx / _cw * .5;
  }

  void _tapped() {
    st.rel = t.value;
    if (p.tap) st.a = st.a >= .99 ? 0 : (st.a + .08).clamp(0.0, 1.0);
  }

  @override
  Widget build(BuildContext context) => SizedBox(
        width: _cw,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          GestureDetector(
            onPanStart: _start,
            onPanUpdate: _move,
            onPanEnd: _end,
            onTap: _tapped,
            child: ClipRRect(
              borderRadius: BorderRadius.circular(8),
              child: CustomPaint(size: const Size(_cw, _ch), painter: _P8Painter(p, st, t)),
            ),
          ),
          const SizedBox(height: 5),
          Text('$concept #$n  ${p.name}', style: _lab1, maxLines: 1, overflow: TextOverflow.ellipsis, softWrap: false),
          Text(p.tags, style: _lab2, maxLines: 1, overflow: TextOverflow.ellipsis, softWrap: false),
        ]),
      );
}

class _P8Painter extends CustomPainter {
  _P8Painter(this.p, this.st, this.t) : super(repaint: t);
  final _Pan8 p;
  final _P8 st;
  final ValueNotifier<double> t;
  @override
  void paint(Canvas canvas, Size size) => p.paint(canvas, size, st, t.value);
  @override
  bool shouldRepaint(covariant _P8Painter old) => true;
}
