// Pop 04 shared kit: the pop palette, one sheet (5 x 4 panels, one Ticker), one panel (three values 0..1 + a gesture), and the motion maths
// the three concepts share (cubic-bezier easing, a damped spring, a layered wiggle). Each panel only paints; the gesture kind decides
// how the pointer moves its three values, so every panel answers at least a drag.
import 'dart:math' as math;

import 'package:flutter/scheduler.dart' show Ticker;
import 'package:flutter/widgets.dart';

// ---- palette ------------------------------------------------------------------------------------------------------------------------

abstract final class Pop {
  static const panel = Color(0xFF1C1C1E), panel2 = Color(0xFF2A2A2E), sheet = Color(0xFF121214);
  static const cream = Color(0xFFF4EBDD), white = Color(0xFFFFFFFF), ink = Color(0xFF1C1C1E);
  static const orange = Color(0xFFFF7A3D), yellow = Color(0xFFFFD23F), cyan = Color(0xFF3DD6F5), lime = Color(0xFF9BE564);
  static const pink = Color(0xFFFF5DA2), violet = Color(0xFF9B7BFF), red = Color(0xFFFF4B4B), blue = Color(0xFF2E5BFF);
}

Color pa(Color c, double a) => c.withValues(alpha: (c.a * a).clamp(0.0, 1.0));
Paint pf(Color c) => Paint()..color = c;
Paint ps(Color c, [double w = 2]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
double cl(double v, [double a = 0, double b = 1]) => v < a ? a : (v > b ? b : v);
double lr(double a, double b, double t) => a + (b - a) * t;
Offset ol(Offset a, Offset b, double t) => Offset(lr(a.dx, b.dx, t), lr(a.dy, b.dy, t));
Offset polar(Offset c, double r, double a) => c + Offset(math.cos(a), math.sin(a)) * r;

Path poly(List<Offset> pts, {bool close = true}) {
  final p = Path()..moveTo(pts.first.dx, pts.first.dy);
  for (final q in pts.skip(1)) {
    p.lineTo(q.dx, q.dy);
  }
  if (close) p.close();
  return p;
}

void rr(Canvas c, Rect r, double rad, Paint p) => c.drawRRect(RRect.fromRectAndRadius(r, Radius.circular(rad)), p);

// ---- motion maths -------------------------------------------------------------------------------------------------------------------

double _bz(double a, double b, double u) => 3 * a * u * (1 - u) * (1 - u) + 3 * b * u * u * (1 - u) + u * u * u;

/// Easing from the three values: [i] slow start, [o] slow arrival, [ov] how far it flies past the target before settling.
double ease(double t, double i, double o, double ov) {
  t = cl(t);
  final x1 = .02 + .9 * i, x2 = .98 - .9 * o, y1 = 0.0, y2 = 1 + 1.1 * ov;
  var lo = 0.0, hi = 1.0;
  for (var k = 0; k < 18; k++) {
    final u = (lo + hi) / 2;
    _bz(x1, x2, u) < t ? lo = u : hi = u;
  }
  return _bz(y1, y2, (lo + hi) / 2);
}

/// One trip in a 2.4 s loop: 0.3 s rest, 1.5 s travel, 0.6 s hold. Returns time fraction 0..1 of the travel.
double tripT(double time, [double shift = 0]) => cl(((time + shift) % 2.4 - .3) / 1.5);

/// Spring displacement from rest after a release at time 0 (starts at 1, rings to 0). k stiffness, d damping, m mass, all 0..1.
double spring(double tau, double k, double d, double m) {
  if (tau < 0) return 1;
  final w = (3 + 16 * k) / math.sqrt(.3 + 1.6 * m), z = .02 + .98 * d * d * d;
  if (z >= 1) return math.exp(-w * tau) * (1 + w * tau);
  final wd = w * math.sqrt(1 - z * z);
  return math.exp(-z * w * tau) * (math.cos(wd * tau) + z * w / wd * math.sin(wd * tau));
}

/// Time since the last release, looping every [period] so the spring rings again on its own.
double ring(PopV p, [double period = 3]) => p.touch != null ? -1 : (p.t - p.kick) % period;

/// A layered wiggle in about -1..1: [f] frequency 0..1 (0.4..9 Hz), [smooth] 0..1 (1 = one soft wave, 0 = jagged layers).
double wig(double time, double f, double smooth, [double seed = 0]) {
  final hz = .4 + 8.6 * f * f;
  var sum = 0.0, amp = 1.0, norm = 0.0, fr = hz;
  final rough = .15 + .7 * (1 - smooth);
  for (var o = 0; o < 4; o++) {
    sum += amp * _vn(time * fr + seed * 13.7 + o * 31.1, smooth);
    norm += amp;
    amp *= rough;
    fr *= 2.13;
  }
  return sum / norm;
}

double _h(int n) {
  final x = math.sin(n * 127.1 + 311.7) * 43758.5453;
  return (x - x.floorToDouble()) * 2 - 1;
}

double _vn(double x, double smooth) {
  final i = x.floorToDouble(), f = x - i;
  final u = smooth > .5 ? f * f * f * (f * (f * 6 - 15) + 10) : lr(f, f * f * (3 - 2 * f), smooth * 2);
  return lr(_h(i.toInt()), _h(i.toInt() + 1), u);
}

// ---- panel model --------------------------------------------------------------------------------------------------------------------

enum G { drag, flick, rub, spin, pinch, paint }

/// What a panel paints from: three values 0..1, the sheet clock, the last release time, and the pointer while it is down.
class PopV extends ChangeNotifier {
  PopV(List<double> init) : v = List.of(init);
  final List<double> v;
  double t = 0, kick = 0, energy = 0;
  Offset? touch;
  final List<Offset> trail = [];
  double operator [](int i) => v[i];
  void set(int i, double x) => v[i] = cl(x);
  void poke() => notifyListeners();
}

typedef PaintFn = void Function(Canvas c, Size s, PopV p);

class PopPanel {
  const PopPanel(this.name, this.tags, this.g, this.paint, {this.init = const [.5, .5, .3], this.light = false});
  final String name, tags;
  final G g;
  final PaintFn paint;
  final List<double> init;
  final bool light;
}

// ---- the sheet: 5 x 4 panels, one clock ---------------------------------------------------------------------------------------------

const double kArtW = 156, kArtH = 116;

class PopSheet extends StatefulWidget {
  const PopSheet({super.key, required this.concept, required this.panels});
  final String concept;
  final List<PopPanel> panels;
  @override
  State<PopSheet> createState() => _PopSheetState();
}

class _PopSheetState extends State<PopSheet> with SingleTickerProviderStateMixin {
  final clock = ValueNotifier<double>(0);
  late final Ticker _tk;

  @override
  void initState() {
    super.initState();
    _tk = createTicker((d) => clock.value = d.inMicroseconds / 1e6)..start();
  }

  @override
  void dispose() {
    _tk.dispose();
    clock.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: Pop.sheet,
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(12),
            child: Wrap(
              spacing: 8,
              runSpacing: 10,
              children: [
                for (var i = 0; i < widget.panels.length; i++)
                  SizedBox(width: kArtW, child: _Cell(n: i + 1, concept: widget.concept, spec: widget.panels[i], clock: clock)),
              ],
            ),
          ),
        ),
      );
}

class _Cell extends StatefulWidget {
  const _Cell({required this.n, required this.concept, required this.spec, required this.clock});
  final int n;
  final String concept;
  final PopPanel spec;
  final ValueNotifier<double> clock;
  @override
  State<_Cell> createState() => _CellState();
}

class _CellState extends State<_Cell> {
  late final PopV p = PopV(widget.spec.init);
  int _zone = 0;
  double _v1at = 0, _lastA = 0;

  @override
  void dispose() {
    p.dispose();
    super.dispose();
  }

  void _start(ScaleStartDetails d) {
    final x = d.localFocalPoint;
    _zone = x.dy < kArtH * .24 ? 2 : (x.dx < kArtW / 2 ? 0 : 1);
    _v1at = p[1];
    _lastA = (x - const Offset(kArtW / 2, kArtH / 2)).direction;
    p.touch = x;
    p.trail
      ..clear()
      ..add(x);
    if (widget.spec.g == G.paint) _paintAt(x);
    p.poke();
  }

  void _paintAt(Offset x) {
    p.set(0, x.dx / kArtW);
    p.set(1, 1 - x.dy / kArtH);
  }

  void _update(ScaleUpdateDetails d) {
    final x = d.localFocalPoint, dl = d.focalPointDelta;
    p.touch = x;
    p.trail.add(x);
    if (p.trail.length > 40) p.trail.removeAt(0);
    switch (widget.spec.g) {
      case G.drag:
        if (_zone == 2) {
          p.set(2, p[2] + (dl.dx - dl.dy) / 120);
        } else {
          p.set(_zone, p[_zone] + (_zone == 0 ? dl.dx : -dl.dx) / 110 - dl.dy / 110);
        }
      case G.flick:
        p.set(0, p[0] + dl.dx / 160);
        p.set(1, p[1] - dl.dy / 120);
      case G.rub:
        p.energy = cl(p.energy + dl.distance / 300);
        p.set(_zone == 2 ? 2 : 1, p[_zone == 2 ? 2 : 1] + dl.distance / 500 * (dl.dx >= 0 ? 1 : -1));
        p.set(0, p[0] - dl.dy / 160);
      case G.spin:
        final a = (x - const Offset(kArtW / 2, kArtH / 2)).direction;
        var da = a - _lastA;
        if (da > math.pi) da -= 2 * math.pi;
        if (da < -math.pi) da += 2 * math.pi;
        _lastA = a;
        p.set(0, p[0] + da / (2 * math.pi));
        p.set(1, p[1] - dl.dy / 200);
      case G.pinch:
        if (d.scale != 1) {
          p.set(1, _v1at + (d.scale - 1) * .6);
        } else {
          p.set(0, p[0] + dl.dx / 120);
          p.set(1, p[1] - dl.dy / 110);
        }
      case G.paint:
        _paintAt(x);
        p.set(2, p[2] + dl.distance / 900);
    }
    p.poke();
  }

  void _end(ScaleEndDetails d) {
    if (widget.spec.g == G.flick) p.set(2, d.velocity.pixelsPerSecond.distance / 2600);
    p.kick = p.t;
    p.touch = null;
    p.poke();
  }

  @override
  Widget build(BuildContext context) {
    final s = widget.spec;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        GestureDetector(
          onScaleStart: _start,
          onScaleUpdate: _update,
          onScaleEnd: _end,
          onDoubleTap: () {
            for (var i = 0; i < 3; i++) {
              p.v[i] = s.init[i];
            }
            p.kick = p.t;
            p.poke();
          },
          child: ClipRRect(
            borderRadius: BorderRadius.circular(9),
            child: CustomPaint(size: const Size(kArtW, kArtH), painter: _Painter(s, p, widget.clock)),
          ),
        ),
        const SizedBox(height: 4),
        Text('${widget.concept} #${widget.n}  ${s.name}',
            maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 10, height: 1.25, color: Pop.cream, fontWeight: FontWeight.w600)),
        Text(s.tags, maxLines: 1, overflow: TextOverflow.ellipsis, style: TextStyle(fontSize: 10, height: 1.25, color: pa(Pop.cream, .5))),
      ],
    );
  }
}

class _Painter extends CustomPainter {
  _Painter(this.spec, this.p, this.clock) : super(repaint: Listenable.merge([p, clock]));
  final PopPanel spec;
  final PopV p;
  final ValueNotifier<double> clock;

  @override
  void paint(Canvas c, Size s) {
    p.t = clock.value;
    c.drawRect(Offset.zero & s, pf(spec.light ? Pop.cream : Pop.panel));
    spec.paint(c, s, p);
  }

  @override
  bool shouldRepaint(_Painter o) => true;
}
