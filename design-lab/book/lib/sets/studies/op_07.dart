// set: OP 07. Two SHEETS in the OP-1 / Max for Live screen language: black ground, monoline drawings,
// four encoder colours (blue, green, white, red), large thin numerals, tiny uppercase labels.
// Each sheet = one concept drawn as 20 different ideas (5 x 4 panels); the drawing itself is the control.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:widgetbook/widgetbook.dart';

part 'op_07_blend.dart';
part 'op_07_glass.dart';

WidgetbookComponent op07Set() => WidgetbookComponent(name: 'OP 07', useCases: [
      WidgetbookUseCase(name: 'Opacity / Blend x20', builder: (_) => _O7Sheet(concept: 'Opacity', specs: _blendSpecs)),
      WidgetbookUseCase(name: 'Glass / Refraction x20', builder: (_) => _O7Sheet(concept: 'Glass', specs: _glassSpecs)),
    ]);

const _bg = Color(0xFF0B0B0C), _fr = Color(0xFF222222);
const _bu = Color(0xFF4F6CFF), _gn = Color(0xFF23E0A3), _wt = Color(0xFFE8E8EA), _rd = Color(0xFFFF3363);
const _pu = Color(0xFF9B7BFF), _dg = Color(0xFF3A3A40);

Paint _s(Color c, [double w = 1.1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Paint _f(Color c) => Paint()..color = c;
Color _o(Color c, double a) => c.withValues(alpha: a.clamp(0.0, 1.0));
double _cl(double v) => v.clamp(0.0, 1.0);
double _h(int i) => (math.sin(i * 12.9898 + 4.1) * 43758.5453).abs() % 1;
double _ss(double a, double b, double x) {
  final t = _cl((x - a) / (b - a));
  return t * t * (3 - 2 * t);
}

/// Hides what is behind a line drawing by filling with the screen black (line-art occlusion, not a visible fill).
void _occ(Canvas c, Path p, double amount) => c.drawPath(p, _f(_o(_bg, amount)));
void _pl(Canvas c, List<Offset> pts, Paint p) => c.drawPoints(ui.PointMode.polygon, pts, p);
void _dot(Canvas c, Offset o, Color col, [double r = 1.6]) => c.drawCircle(o, r, _f(col));
void _dash(Canvas c, Offset a, Offset b, Paint p, {double on = 2, double off = 3}) {
  final d = (b - a).distance;
  if (d < .1) return;
  final u = (b - a) / d;
  for (var x = 0.0; x < d; x += on + off) {
    c.drawLine(a + u * x, a + u * math.min(d, x + on), p);
  }
}

/// A line from a to b split into n points and pushed through f (used for refraction warps).
List<Offset> _seg(Offset a, Offset b, int n, Offset Function(Offset) f) =>
    [for (var i = 0; i <= n; i++) f(Offset.lerp(a, b, i / n)!)];

/// Forward map of a thin circular lens: points inside are pushed outward (magnified) by k, continuous at the rim.
Offset _lens(Offset q, Offset c, double r, double k) {
  final d = (q - c).distance;
  if (d >= r) return q;
  final x = 1 - d / r;
  return c + (q - c) * (1 + k * x * x);
}

Offset _iso(Offset o, double x, double y, double z, [double k = 1]) =>
    Offset(o.dx + (x - y) * .866 * k, o.dy + (x + y) * .5 * k - z * k);

/// Only for offscreen capture in tests, where the engine default is a box font.
String? op07CaptureFont;

final _tpCache = <String, TextPainter>{};
TextPainter _tp(String s, Color col, double size, FontWeight w) {
  if (_tpCache.length > 600) _tpCache.clear();
  return _tpCache.putIfAbsent(
      '$s|${col.toARGB32()}|$size|${w.value}',
      () => TextPainter(
          text: TextSpan(
              text: s, style: TextStyle(fontFamily: op07CaptureFont, color: col, fontSize: size, fontWeight: w, letterSpacing: size * .1, height: 1)),
          textDirection: TextDirection.ltr)
        ..layout());
}

/// Tiny uppercase label. ax: 0 = left at o, .5 = centred, 1 = right-aligned at o.
void _lab(Canvas c, String s, Offset o, Color col, {double size = 7.5, double ax = 0, FontWeight w = FontWeight.w500}) {
  final tp = _tp(s, col, size, w);
  tp.paint(c, o - Offset(tp.width * ax, 0));
}

/// Large thin numerals drawn as monoline strokes (OP-1 face): digits, '.', '%', '+', '-'.
double _numW(String s, double h) {
  var w = 0.0;
  for (final ch in s.split('')) {
    w += switch (ch) { '1' => h * .32, '.' => h * .28, '-' || '+' => h * .5, '%' => h * .7, _ => h * .58 + h * .18 };
  }
  return w;
}

double _num(Canvas c, String s, Offset o, double h, Color col, {double w = 1.1, double ax = 0}) {
  final p = Path();
  var x = o.dx - _numW(s, h) * ax;
  final y = o.dy, gw = h * .58;
  for (final ch in s.split('')) {
    switch (ch) {
      case '0':
        p.addRRect(RRect.fromRectAndRadius(Rect.fromLTWH(x, y, gw, h), Radius.circular(gw / 2)));
      case '1':
        p
          ..moveTo(x + h * .06, y + h * .12)
          ..lineTo(x + h * .14, y)
          ..lineTo(x + h * .14, y + h);
      case '2':
        p
          ..addArc(Rect.fromLTWH(x, y, gw, gw), math.pi * 1.05, math.pi * 1.2)
          ..lineTo(x, y + h)
          ..lineTo(x + gw, y + h);
      case '3':
        p
          ..addArc(Rect.fromLTWH(x + gw * .04, y, gw * .92, h * .5), math.pi * 1.1, math.pi * 1.4)
          ..addArc(Rect.fromLTWH(x, y + h * .5, gw, h * .5), -math.pi * .5, math.pi * 1.4);
      case '4':
        p
          ..moveTo(x + gw * .78, y + h)
          ..lineTo(x + gw * .78, y)
          ..lineTo(x, y + h * .68)
          ..lineTo(x + gw, y + h * .68);
      case '5':
        p
          ..moveTo(x + gw, y)
          ..lineTo(x + gw * .08, y)
          ..lineTo(x + gw * .02, y + h * .44)
          ..arcTo(Rect.fromLTWH(x, y + h * .36, gw, h * .64), -math.pi * .82, math.pi * 1.6, false);
      case '6':
        p
          ..addOval(Rect.fromLTWH(x, y + h - gw * 1.05, gw, gw * 1.05))
          ..moveTo(x, y + h - gw * .5)
          ..quadraticBezierTo(x, y, x + gw * .85, y);
      case '7':
        p
          ..moveTo(x, y)
          ..lineTo(x + gw, y)
          ..lineTo(x + gw * .3, y + h);
      case '8':
        p
          ..addOval(Rect.fromLTWH(x + gw * .06, y, gw * .88, h * .46))
          ..addOval(Rect.fromLTWH(x, y + h * .46, gw, h * .54));
      case '9':
        p
          ..addOval(Rect.fromLTWH(x, y, gw, gw * 1.05))
          ..moveTo(x + gw, y + gw * .5)
          ..quadraticBezierTo(x + gw, y + h, x + gw * .15, y + h);
      case '.':
        p.addOval(Rect.fromCircle(center: Offset(x + h * .1, y + h - h * .04), radius: h * .035));
      case '-':
        p
          ..moveTo(x + h * .05, y + h * .55)
          ..lineTo(x + h * .4, y + h * .55);
      case '+':
        p
          ..moveTo(x + h * .05, y + h * .55)
          ..lineTo(x + h * .4, y + h * .55)
          ..moveTo(x + h * .225, y + h * .38)
          ..lineTo(x + h * .225, y + h * .72);
      case '%':
        p
          ..addOval(Rect.fromLTWH(x, y + h * .1, h * .18, h * .22))
          ..addOval(Rect.fromLTWH(x + h * .4, y + h * .68, h * .18, h * .22))
          ..moveTo(x + h * .5, y + h * .1)
          ..lineTo(x + h * .08, y + h * .9);
    }
    x += switch (ch) { '1' => h * .32, '.' => h * .28, '-' || '+' => h * .5, '%' => h * .7, _ => gw + h * .18 };
  }
  c.drawPath(p, _s(col, w));
  return x;
}

String _pct(double v) => (v * 99).round().toString().padLeft(2, '0');

typedef _O7Paint = void Function(Canvas c, Size s, O7 st, double t);

class _O7Spec {
  const _O7Spec(this.name, this.tags, this.paint, {this.a = .5, this.b = .5, this.p = const Offset(.5, .5)});
  final String name, tags;
  final _O7Paint paint;
  final double a, b;
  final Offset p;
}

/// Toy state per panel: a (drag right), b (drag up), p = last touch (0..1), rub = rubbed distance,
/// spin = flickable position with inertia, pinch = two-finger scale, marks = drawn trail (0..1).
class O7 {
  O7(this.a, this.b, this.p);
  double a, b, rub = 0, spin = 0, spinV = 0, pinch = 1, pinch0 = 1;
  Offset p;
  bool down = false;
  final List<Offset> marks = [];
}

class _O7Sheet extends StatefulWidget {
  const _O7Sheet({required this.concept, required this.specs});
  final String concept;
  final List<_O7Spec> specs;
  @override
  State<_O7Sheet> createState() => _O7SheetSt();
}

class _O7SheetSt extends State<_O7Sheet> with SingleTickerProviderStateMixin {
  static const _size = Size(156, 120);
  final _time = ValueNotifier<double>(0);
  late final List<O7> _st = [for (final s in widget.specs) O7(s.a, s.b, s.p)];
  late final Ticker _tk;
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tk = createTicker((d) {
      final dt = ((d - _last).inMicroseconds / 1e6).clamp(0.0, .05);
      _last = d;
      for (final s in _st) {
        if (!s.down && s.spinV.abs() > .01) {
          s.spin += s.spinV * dt;
          s.spinV *= math.pow(.2, dt).toDouble();
        }
      }
      _time.value = d.inMicroseconds / 1e6;
    })
      ..start();
  }

  @override
  void dispose() {
    _tk.dispose();
    _time.dispose();
    super.dispose();
  }

  Offset _n(Offset l) => Offset(_cl(l.dx / _size.width), _cl(l.dy / _size.height));

  Widget _cell(int i) {
    final sp = widget.specs[i], st = _st[i];
    return SizedBox(
      width: 156,
      height: 148,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        GestureDetector(
          onScaleStart: (d) {
            st
              ..down = true
              ..spinV = 0
              ..pinch0 = st.pinch
              ..p = _n(d.localFocalPoint);
          },
          onScaleUpdate: (d) {
            final dl = d.focalPointDelta;
            st
              ..a = _cl(st.a + dl.dx / 110)
              ..b = _cl(st.b - dl.dy / 90)
              ..p = _n(d.localFocalPoint)
              ..rub = (st.rub + dl.distance / 900).clamp(0.0, 1.0)
              ..spin += dl.dx / 40;
            if (d.pointerCount > 1 || d.scale != 1) st.pinch = (st.pinch0 * d.scale).clamp(.4, 2.5);
            if (st.marks.isEmpty || (st.marks.last - st.p).distance > .025) {
              st.marks.add(st.p);
              if (st.marks.length > 160) st.marks.removeAt(0);
            }
          },
          onScaleEnd: (d) {
            st
              ..down = false
              ..spinV = d.velocity.pixelsPerSecond.dx / 40;
          },
          onDoubleTap: () {
            st
              ..a = sp.a
              ..b = sp.b
              ..p = sp.p
              ..rub = 0
              ..spin = 0
              ..spinV = 0
              ..pinch = 1
              ..marks.clear();
          },
          child: RepaintBoundary(child: CustomPaint(size: _size, painter: _O7Painter(sp, st, _time))),
        ),
        const SizedBox(height: 4),
        Text('${widget.concept} #${i + 1}  ${sp.name}',
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(fontSize: 9.5, height: 1.2, color: _wt, fontWeight: FontWeight.w500)),
        Text(sp.tags,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(fontSize: 9, height: 1.2, color: Color(0xFF7C7C84))),
      ]),
    );
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: const Color(0xFF050506),
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(12),
            child: Column(mainAxisSize: MainAxisSize.min, children: [
              for (var r = 0; r < 4; r++)
                Padding(
                  padding: const EdgeInsets.only(bottom: 6),
                  child: Row(mainAxisSize: MainAxisSize.min, children: [
                    for (var c = 0; c < 5; c++) ...[if (c > 0) const SizedBox(width: 8), _cell(r * 5 + c)],
                  ]),
                ),
            ]),
          ),
        ),
      );
}

class _O7Painter extends CustomPainter {
  _O7Painter(this.sp, this.st, this.time) : super(repaint: time);
  final _O7Spec sp;
  final O7 st;
  final ValueNotifier<double> time;

  @override
  void paint(Canvas c, Size s) {
    final r = Offset.zero & s;
    c.save();
    c.clipRect(r);
    c.drawRect(r, _f(_bg));
    sp.paint(c, s, st, time.value);
    c.restore();
    c.drawRect(r.deflate(.5), _s(_fr, 1));
  }

  @override
  bool shouldRepaint(_O7Painter o) => true;
}
