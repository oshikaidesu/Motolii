// OP 10: two sheets of 20 panels each (Position / Transform as a world, Macro / Variations) in the OP-1 screen
// language: black ground, monoline drawings, four encoder colours (blue, green, white, red), large thin numerals,
// tiny caps labels. The drawing is the control: every panel answers a drag through one shared gesture helper;
// one Ticker per sheet.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'op_10_figs.dart';
part 'op_10_pos.dart';
part 'op_10_macro.dart';

WidgetbookComponent op10Set() => WidgetbookComponent(name: 'OP 10', useCases: [
      WidgetbookUseCase(name: 'Position / Transform x20', builder: (c) => _O10Sheet(concept: 'Position', panels: _posPanels())),
      WidgetbookUseCase(name: 'Macro / Variations x20', builder: (c) => _O10Sheet(concept: 'Macro', panels: _macroPanels())),
    ]);

const _bk = Color(0xFF000000), _bl = Color(0xFF4F6CFF), _gr = Color(0xFF23E0A3), _wh = Color(0xFFE8E8EA);
const _rd = Color(0xFFFF3363), _pu = Color(0xFF9B7BFF), _dg = Color(0xFF3A3A40), _mg = Color(0xFF6A6A72);

const double _cw = 156, _ch = 120;
const _ff = 'Avenir Next', _ffb = ['Roboto'];

/// What a one-finger drag does to (a, b). drag: dx -> a, up -> b. flick: like drag, then both keep momentum.
/// spin: angle round the centre -> a (wraps), radius change -> b. pinch: two fingers or horizontal stretch -> b,
/// up -> a. rub: dx -> a, path length -> b. draw: a stroke (tap clears).
enum _G { drag, flick, spin, pinch, rub, draw }

typedef _P10 = void Function(Canvas c, _S st, double t);

class _Op {
  const _Op(this.name, this.tags, this.g, this.paint, {this.a = .5, this.b = .5, this.fine = 1});
  final String name, tags;
  final _G g;
  final _P10 paint;
  final double a, b, fine;
}

class _S {
  _S(this.a, this.b);
  double a, b, va = 0, vb = 0, b0 = 0, rel = 0, up = 0;
  int n = 0;
  final List<Offset> pts = [], tr = [];
  bool down = false, released = false;
}

double _wr(double x) => x - x.floorToDouble();
double _rn(int i) {
  final x = math.sin(i * 12.9898 + 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

double _lp(double a, double b, double t) => a + (b - a) * t;
double _sm(double x) {
  final v = x.clamp(0.0, 1.0);
  return v * v * (3 - 2 * v);
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

void _ln(Canvas c, Offset a, Offset b, Color col, [double w = 1.2]) => c.drawLine(a, b, _s(col, w));
void _dot(Canvas c, Offset p, double r, Color col) => c.drawCircle(p, r, _f(col));
void _ring(Canvas c, Offset p, double r, Color col, [double w = 1.2]) => c.drawCircle(p, r, _s(col, w));
void _pl(Canvas c, List<Offset> p, Color col, {double w = 1.2, bool close = false}) {
  if (p.length < 2) return;
  c.drawPath(Path()..addPolygon(p, close), _s(col, w));
}

void _dots(Canvas c, Offset a, Offset b, Color col, {double gap = 3, double w = 1}) {
  final n = ((b - a).distance / gap).floor();
  if (n < 1) return;
  c.drawPoints(ui.PointMode.points, [for (var i = 0; i <= n; i++) a + (b - a) * (i / n)], _s(col, w));
}

void _rect(Canvas c, Rect r, Color col, [double w = 1.2]) => c.drawRect(r, _s(col, w));

Offset _iso(Offset o, double x, double y, double z, double k) => o + Offset((x - y) * .866 * k, (x + y) * .5 * k - z * k);

void _isoBox(Canvas c, Offset o, double x, double y, double z, double w, double d, double h, double k, Color col,
    [double sw = 1]) {
  Offset p(double i, double j, double l) => _iso(o, x + i * w, y + j * d, z + l * h, k);
  _pl(c, [p(0, 0, 0), p(1, 0, 0), p(1, 1, 0), p(0, 1, 0)], col, w: sw, close: true);
  _pl(c, [p(0, 0, 1), p(1, 0, 1), p(1, 1, 1), p(0, 1, 1)], col, w: sw, close: true);
  for (final q in const [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]) {
    _ln(c, p(q[0], q[1], 0), p(q[0], q[1], 1), col, sw);
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
void _lab(Canvas c, String s, Offset at, Color col, {double ax = 0, double size = 7.5}) {
  final tp = _tp(s.toUpperCase(), size, col, FontWeight.w500, .7);
  tp.paint(c, at - Offset(tp.width * ax, 0));
}

class _O10Sheet extends StatefulWidget {
  const _O10Sheet({required this.concept, required this.panels});
  final String concept;
  final List<_Op> panels;
  @override
  State<_O10Sheet> createState() => _O10SheetState();
}

class _O10SheetState extends State<_O10Sheet> with SingleTickerProviderStateMixin {
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
      if (p.g != _G.flick || s.down || (s.va.abs() < 1e-4 && s.vb.abs() < 1e-4)) continue;
      s.a += s.va * dt;
      s.b += s.vb * dt;
      final k = math.pow(.2, dt).toDouble();
      s.va *= k;
      s.vb *= k;
      if (s.a < 0 || s.a > 1) {
        s.a = s.a.clamp(0.0, 1.0);
        s.va = -s.va * .6;
      }
      if (s.b < 0 || s.b > 1) {
        s.b = s.b.clamp(0.0, 1.0);
        s.vb = -s.vb * .6;
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
    Widget tile(int i) => _O10Tile(n: i + 1, concept: widget.concept, p: ps[i], st: _st[i], t: _t);
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

class _O10Tile extends StatelessWidget {
  const _O10Tile({required this.n, required this.concept, required this.p, required this.st, required this.t});
  final int n;
  final String concept;
  final _Op p;
  final _S st;
  final ValueNotifier<double> t;

  static const _c = Offset(_cw / 2, _ch / 2);

  double _a(double v) => v.clamp(0.0, 1.0);
  double _b(double v) => v.clamp(0.0, 1.0);

  void _start(ScaleStartDetails d) {
    st
      ..down = true
      ..released = false
      ..va = 0
      ..vb = 0
      ..b0 = st.b;
    if (p.g == _G.draw) {
      st.pts
        ..clear()
        ..add(d.localFocalPoint);
    }
  }

  void _move(ScaleUpdateDetails d) {
    final q = d.localFocalPoint, dl = d.focalPointDelta;
    final dx = dl.dx / _cw * p.fine, dy = -dl.dy / _ch * p.fine;
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
        st.b = _wr(st.b + dl.distance / (_cw * 4));
      case _G.draw:
        if (st.pts.isEmpty || (q - st.pts.last).distance > 2.5) st.pts.add(q);
        if (st.pts.length > 220) st.pts.removeAt(0);
    }
  }

  void _end(ScaleEndDetails d) {
    st
      ..down = false
      ..released = true
      ..n += 1;
    if (p.g == _G.flick) {
      st.va = d.velocity.pixelsPerSecond.dx / _cw * .5;
      st.vb = -d.velocity.pixelsPerSecond.dy / _ch * .5;
    }
  }

  @override
  Widget build(BuildContext context) => SizedBox(
        width: _cw,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          GestureDetector(
            onScaleStart: _start,
            onScaleUpdate: _move,
            onScaleEnd: _end,
            onTap: () => st.pts.clear(),
            child: DecoratedBox(
              position: DecorationPosition.foreground,
              decoration: BoxDecoration(border: Border.all(color: const Color(0xFF222222))),
              child: ClipRect(child: CustomPaint(size: const Size(_cw, _ch), painter: _O10Painter(p, st, t))),
            ),
          ),
          const SizedBox(height: 4),
          Text('$concept #$n  ${p.name}', style: _lab1, maxLines: 1, overflow: TextOverflow.ellipsis, softWrap: false),
          Text(p.tags, style: _lab2, maxLines: 1, overflow: TextOverflow.ellipsis, softWrap: false),
        ]),
      );
}

class _O10Painter extends CustomPainter {
  _O10Painter(this.p, this.st, this.t) : super(repaint: t);
  final _Op p;
  final _S st;
  final ValueNotifier<double> t;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, _f(_bk));
    if (st.released) {
      st
        ..released = false
        ..rel = t.value;
    }
    p.paint(canvas, st, t.value);
  }

  @override
  bool shouldRepaint(covariant _O10Painter old) => true;
}
