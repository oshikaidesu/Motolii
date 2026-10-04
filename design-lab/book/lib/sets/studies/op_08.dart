// OP 08: two sheets of 20 panels each (Repeat / Clone, Fill / Colour / Gradient) in the OP-1 screen language:
// black ground, monoline drawings, four encoder colours (blue, green, white, red), large thin numerals, tiny caps labels.
// The drawing is the control: every panel answers a drag through one shared gesture helper; one Ticker per sheet.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'op_08_figs.dart';
part 'op_08_repeat.dart';
part 'op_08_fill.dart';

WidgetbookComponent op08Set() => WidgetbookComponent(name: 'OP 08', useCases: [
      WidgetbookUseCase(name: 'Repeat / Clone x20', builder: (c) => _O8Sheet(concept: 'Repeat', panels: _repeatPanels())),
      WidgetbookUseCase(name: 'Fill / Colour / Gradient x20', builder: (c) => _O8Sheet(concept: 'Fill', panels: _fillPanels())),
    ]);

const _bk = Color(0xFF000000), _bl = Color(0xFF4F6CFF), _gr = Color(0xFF23E0A3), _wh = Color(0xFFE8E8EA);
const _rd = Color(0xFFFF3363), _pu = Color(0xFF9B7BFF), _dg = Color(0xFF3A3A40);

const double _cw = 156, _ch = 120;
const _ff = 'Avenir Next', _ffb = ['Roboto'];

/// What a one-finger drag does to (a, b). drag/flick: dx -> a, up -> b (flick keeps a's momentum).
/// spin: angle round the centre -> a, radius change -> b. pinch: two fingers or horizontal stretch -> b, up -> a.
/// rub: dx -> a, path length -> b. draw: a stroke (tap steps a).
enum _G { drag, flick, spin, pinch, rub, draw }

typedef _PaintO8 = void Function(Canvas c, Size s, _S st, double t);

class _Op {
  const _Op(this.name, this.tags, this.g, this.paint,
      {this.a = .5, this.b = .5, this.wrap = false, this.wrapB = false, this.inv = false, this.keep = false, this.tap = 0});
  final String name, tags;
  final _G g;
  final _PaintO8 paint;
  final double a, b, tap;
  final bool wrap, wrapB, inv, keep;
}

class _S {
  _S(this.a, this.b);
  double a, b, va = 0, b0 = 0;
  final List<Offset> pts = [];
  bool down = false;
}

double _wr(double x) => x - x.floorToDouble();
double _rn(int i) {
  final x = math.sin(i * 12.9898 + 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

String _d2(num v) => v.round().toString().padLeft(2, '0');
String _d3(num v) => v.round().toString().padLeft(3, '0');

Paint _f(Color c) => Paint()..color = c;
Paint _s(Color c, [double w = 1.2]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Color _al(Color c, double a) => c.withValues(alpha: a.clamp(0.0, 1.0).toDouble());
Color _hc(double hue, [double sat = .9, double v = 1]) => sat < .04
    ? Color.lerp(const Color(0xFF8A8A90), _wh, v.clamp(0.0, 1.0).toDouble())!
    : HSVColor.fromAHSV(1, _wr(hue) * 360, sat.clamp(0.0, 1.0).toDouble(), v.clamp(0.0, 1.0).toDouble()).toColor();

void _ln(Canvas c, Offset a, Offset b, Color col, [double w = 1.2]) => c.drawLine(a, b, _s(col, w));
void _dot(Canvas c, Offset p, double r, Color col) => c.drawCircle(p, r, _f(col));
void _ring(Canvas c, Offset p, double r, Color col, [double w = 1.2]) => c.drawCircle(p, r, _s(col, w));
void _pl(Canvas c, List<Offset> p, Color col, {double w = 1.2, bool close = false}) {
  if (p.length < 2) return;
  final path = Path()..addPolygon(p, close);
  c.drawPath(path, _s(col, w));
}

void _dots(Canvas c, Offset a, Offset b, Color col, {double gap = 3, double w = 1}) {
  final d = (b - a).distance;
  final n = (d / gap).floor();
  if (n < 1) return;
  final pts = [for (var i = 0; i <= n; i++) a + (b - a) * (i / n)];
  c.drawPoints(ui.PointMode.points, pts, _s(col, w));
}

Offset _iso(Offset o, double x, double y, double z, double k) => o + Offset((x - y) * .866 * k, (x + y) * .5 * k - z * k);

void _isoBox(Canvas c, Offset o, double x, double y, double z, double w, double d, double h, double k, Color col,
    [double sw = 1]) {
  Offset p(double i, double j, double l) => _iso(o, x + i * w, y + j * d, z + l * h, k);
  final bot = [p(0, 0, 0), p(1, 0, 0), p(1, 1, 0), p(0, 1, 0)];
  final top = [p(0, 0, 1), p(1, 0, 1), p(1, 1, 1), p(0, 1, 1)];
  _pl(c, bot, col, w: sw, close: true);
  _pl(c, top, col, w: sw, close: true);
  for (var i = 0; i < 4; i++) {
    _ln(c, bot[i], top[i], col, sw);
  }
}

final _tc = <String, TextPainter>{};
TextPainter _tp(String s, double size, Color col, FontWeight w, double ls) {
  final k = '$s|$size|${col.toARGB32()}|$w|$ls';
  var tp = _tc[k];
  if (tp == null) {
    if (_tc.length > 900) {
      for (final v in _tc.values) {
        v.dispose();
      }
      _tc.clear();
    }
    tp = TextPainter(
      text: TextSpan(
          text: s,
          style: TextStyle(
              fontFamily: _ff,
              fontFamilyFallback: _ffb,
              fontSize: size,
              color: col,
              fontWeight: w,
              letterSpacing: ls,
              height: 1,
              decoration: TextDecoration.none)),
      textDirection: TextDirection.ltr,
    )..layout();
    _tc[k] = tp;
  }
  return tp;
}

/// Large thin numeral. [ax] 0 = left-aligned at [at], .5 = centred, 1 = right-aligned.
void _num(Canvas c, String s, Offset at, double size, Color col, {double ax = 0}) {
  final tp = _tp(s, size, col, FontWeight.w200, -size * .02);
  tp.paint(c, at - Offset(tp.width * ax, 0));
}

/// Tiny upper-case label.
void _lab(Canvas c, String s, Offset at, Color col, {double ax = 0, double size = 8}) {
  final tp = _tp(s.toUpperCase(), size, col, FontWeight.w500, .7);
  tp.paint(c, at - Offset(tp.width * ax, 0));
}

class _O8Sheet extends StatefulWidget {
  const _O8Sheet({required this.concept, required this.panels});
  final String concept;
  final List<_Op> panels;
  @override
  State<_O8Sheet> createState() => _O8SheetState();
}

class _O8SheetState extends State<_O8Sheet> with SingleTickerProviderStateMixin {
  late final Ticker _tk;
  final _t = ValueNotifier<double>(0);
  late final List<_S> _st = [for (final p in widget.panels) _S(p.a, p.b)];
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
      s.va *= math.pow(.15, dt).toDouble();
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
    Widget tile(int i) => _O8Tile(n: i + 1, concept: widget.concept, p: ps[i], st: _st[i], t: _t);
    return ColoredBox(
      color: const Color(0xFF0B0B0C),
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

const _lab1 = TextStyle(fontFamily: _ff, fontFamilyFallback: _ffb, fontSize: 10, height: 1.2, color: _wh, fontWeight: FontWeight.w500, decoration: TextDecoration.none);
const _lab2 = TextStyle(fontFamily: _ff, fontFamilyFallback: _ffb, fontSize: 9, height: 1.25, color: Color(0xFF8A8A92), decoration: TextDecoration.none);

class _O8Tile extends StatelessWidget {
  const _O8Tile({required this.n, required this.concept, required this.p, required this.st, required this.t});
  final int n;
  final String concept;
  final _Op p;
  final _S st;
  final ValueNotifier<double> t;

  static const _c = Offset(_cw / 2, _ch / 2);

  double _a(double v) => p.wrap ? _wr(v) : v.clamp(0.0, 1.0);
  double _b(double v) => p.wrapB ? _wr(v) : v.clamp(0.0, 1.0);

  void _start(ScaleStartDetails d) {
    st
      ..down = true
      ..va = 0
      ..b0 = st.b;
    if (p.g == _G.draw) {
      if (!p.keep) st.pts.clear();
      st.pts.add(d.localFocalPoint);
    }
  }

  void _move(ScaleUpdateDetails d) {
    final q = d.localFocalPoint, dl = d.focalPointDelta, dx = dl.dx / _cw, dy = -dl.dy / _ch * (p.inv ? -1 : 1);
    if (d.pointerCount > 1) {
      st.b = _b(st.b0 * d.scale);
      return;
    }
    switch (p.g) {
      case _G.drag || _G.flick:
        st.a = _a(st.a + dx);
        st.b = _b(st.b + dy);
      case _G.spin:
        final o = q - dl - _c, nw = q - _c;
        var da = nw.direction - o.direction;
        if (da > math.pi) da -= 2 * math.pi;
        if (da < -math.pi) da += 2 * math.pi;
        st.a = _wr(st.a + da / (2 * math.pi));
        st.b = _b(st.b + (nw.distance - o.distance) / (_ch * .5));
      case _G.pinch:
        st.b = _b(st.b + dx);
        st.a = _a(st.a + dy);
      case _G.rub:
        st.a = _a(st.a + dx);
        st.b = _b(st.b + dl.distance / (_cw * 3));
      case _G.draw:
        if (st.pts.isEmpty || (q - st.pts.last).distance > 2.5) st.pts.add(q);
        if (st.pts.length > 260) st.pts.removeAt(0);
    }
  }

  void _end(ScaleEndDetails d) {
    st.down = false;
    if (p.g == _G.flick) st.va = d.velocity.pixelsPerSecond.dx / _cw * .5;
  }

  void _tapped() {
    if (p.tap > 0) st.a = st.a + p.tap > 1.001 ? 0 : st.a + p.tap;
    if (p.g == _G.draw && p.keep) st.pts.clear();
  }

  @override
  Widget build(BuildContext context) => SizedBox(
        width: _cw,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          GestureDetector(
            onScaleStart: _start,
            onScaleUpdate: _move,
            onScaleEnd: _end,
            onTap: _tapped,
            child: DecoratedBox(
              position: DecorationPosition.foreground,
              decoration: BoxDecoration(border: Border.all(color: const Color(0xFF222222))),
              child: ClipRect(
                child: CustomPaint(size: const Size(_cw, _ch), painter: _O8Painter(p, st, t)),
              ),
            ),
          ),
          const SizedBox(height: 4),
          Text('$concept #$n  ${p.name}', style: _lab1, maxLines: 1, overflow: TextOverflow.ellipsis, softWrap: false),
          Text(p.tags, style: _lab2, maxLines: 1, overflow: TextOverflow.ellipsis, softWrap: false),
        ]),
      );
}

class _O8Painter extends CustomPainter {
  _O8Painter(this.p, this.st, this.t) : super(repaint: t);
  final _Op p;
  final _S st;
  final ValueNotifier<double> t;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, _f(_bk));
    p.paint(canvas, size, st, t.value);
  }

  @override
  bool shouldRepaint(covariant _O8Painter old) => true;
}
