// OP 05 kit: one sheet = 20 panels (5 x 4) in the OP-1 screen language, one Ticker per sheet.
// Black ground, monoline 1-1.5px, colour = which encoder you are touching (blue 1, green 2, white 3, red 4).
// A panel owns values v[0..2] (a, b, c in 0..1), a mode m (tap cycles it when the spec has modes),
// a phase ph advancing at rate(p) per second, and an optional drawn path pts.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/widgets.dart';
import 'package:flutter/scheduler.dart';

const kBk = Color(0xFF000000), kSheet = Color(0xFF0B0B0C), kFrame = Color(0xFF222222);
const kB = Color(0xFF4F6CFF), kG = Color(0xFF23E0A3), kW = Color(0xFFE8E8EA), kR = Color(0xFFFF3363);
const kPu = Color(0xFF9B7BFF), kDim = Color(0xFF3A3A40), kMid = Color(0xFF6E6E76);

/// Font family for labels; null = platform default. A test may load a font and set this.
String? opFont;

/// drag: x -> v[x], y -> v[y]. spin: angle round centre -> v[x]. pinch: radial distance -> v[x], angle unused.
/// flick: drag on v[x] that keeps its momentum. rub: any movement raises v[x], it relaxes back when let go.
/// draw: the finger path lands in pts (0..1 coords).
enum G { drag, spin, pinch, flick, rub, draw }

class OpP {
  OpP(List<double> init, this.m) : v = [...init];
  final List<double> v;
  double ph = 0, t = 0, vel = 0;
  int m;
  bool down = false;
  final List<Offset> pts = [];
  double get a => v[0];
  double get b => v[1];
  double get c => v[2];
}

class OpSpec {
  const OpSpec(this.name, this.tags, this.paint,
      {this.g = G.drag, this.x = 0, this.y = -1, this.init = const [.5, .5, .5], this.wrap = 0, this.modes = 0, this.m = 0});
  final String name, tags;
  final void Function(Canvas c, Size s, OpP p) paint;
  final G g;
  final int x, y;
  final List<double> init;

  /// bit i set = v[i] wraps round instead of clamping.
  final int wrap;
  final int modes, m;
}

class OpSheet extends StatefulWidget {
  const OpSheet({super.key, required this.concept, required this.panels, this.rate});
  final String concept;
  final List<OpSpec> panels;
  final double Function(OpP p)? rate;
  @override
  State<OpSheet> createState() => _OpSheetState();
}

class _OpSheetState extends State<OpSheet> with SingleTickerProviderStateMixin {
  final time = ValueNotifier<double>(0);
  late final Ticker _tk;
  late final List<OpP> ps = [for (final s in widget.panels) OpP(s.init, s.m)];
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tk = createTicker((e) {
      final dt = ((e - _last).inMicroseconds / 1e6).clamp(0.0, .1);
      _last = e;
      for (var i = 0; i < ps.length; i++) {
        final p = ps[i], s = widget.panels[i];
        p.t += dt;
        p.ph += dt * (widget.rate?.call(p) ?? 1);
        if (!p.down && s.g == G.flick && p.vel.abs() > 1e-4) {
          p.v[s.x] = fitV(s, s.x, p.v[s.x] + p.vel * dt);
          p.vel *= math.exp(-1.6 * dt);
        }
        if (!p.down && s.g == G.rub) {
          final home = s.init[s.x];
          p.v[s.x] += (home - p.v[s.x]) * (1 - math.exp(-.9 * dt));
        }
      }
      time.value = e.inMicroseconds / 1e6;
    })..start();
  }

  @override
  void dispose() {
    _tk.dispose();
    time.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: kSheet,
        child: Align(
          alignment: Alignment.topLeft,
          child: Padding(
            padding: const EdgeInsets.all(10),
            child: SizedBox(
              width: 5 * 156 + 4 * 8,
              child: Wrap(spacing: 8, runSpacing: 8, children: [
                for (var i = 0; i < ps.length; i++) _Panel(concept: widget.concept, n: i + 1, spec: widget.panels[i], p: ps[i], time: time),
              ]),
            ),
          ),
        ),
      );
}

double fitV(OpSpec s, int i, double v) => (s.wrap >> i) & 1 == 1 ? v - v.floorToDouble() : v.clamp(0.0, 1.0);

class _Panel extends StatefulWidget {
  const _Panel({required this.concept, required this.n, required this.spec, required this.p, required this.time});
  final String concept;
  final int n;
  final OpSpec spec;
  final OpP p;
  final ValueNotifier<double> time;
  @override
  State<_Panel> createState() => _PanelState();
}

class _PanelState extends State<_Panel> {
  static const art = Size(156, 120);
  double _ang = 0, _rad = 0;

  Offset get _c => Offset(art.width / 2, art.height / 2);

  void _start(DragStartDetails d) {
    final p = widget.p;
    setState(() {
      p.down = true;
      p.vel = 0;
      final r = d.localPosition - _c;
      _ang = r.direction;
      _rad = r.distance;
      if (widget.spec.g == G.draw) {
        p.pts
          ..clear()
          ..add(Offset(d.localPosition.dx / art.width, d.localPosition.dy / art.height));
      }
    });
  }

  void _update(DragUpdateDetails d) {
    final p = widget.p, s = widget.spec;
    void add(int i, double dv) {
      if (i >= 0) p.v[i] = fitV(s, i, p.v[i] + dv);
    }

    setState(() {
      switch (s.g) {
        case G.drag || G.flick:
          add(s.x, d.delta.dx / 130);
          add(s.y, -d.delta.dy / 100);
        case G.spin:
          final r = d.localPosition - _c;
          var da = r.direction - _ang;
          if (da > math.pi) da -= 2 * math.pi;
          if (da < -math.pi) da += 2 * math.pi;
          _ang = r.direction;
          add(s.x, da / (2 * math.pi));
          add(s.y, -d.delta.dy / 160);
        case G.pinch:
          final r = (d.localPosition - _c).distance;
          add(s.x, (r - _rad) / 70);
          _rad = r;
        case G.rub:
          add(s.x, d.delta.distance / 420);
        case G.draw:
          if (p.pts.length < 240) p.pts.add(Offset((d.localPosition.dx / art.width).clamp(0.0, 1.0), (d.localPosition.dy / art.height).clamp(0.0, 1.0)));
      }
    });
  }

  void _end(DragEndDetails d) => setState(() {
        final p = widget.p;
        p.down = false;
        if (widget.spec.g == G.flick) p.vel = d.velocity.pixelsPerSecond.dx / 130;
      });

  @override
  Widget build(BuildContext context) {
    final s = widget.spec, p = widget.p;
    return SizedBox(
      width: 156,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        GestureDetector(
          onPanStart: _start,
          onPanUpdate: _update,
          onPanEnd: _end,
          onTap: s.modes > 0 ? () => setState(() => p.m = (p.m + 1) % s.modes) : null,
          child: DecoratedBox(
            decoration: BoxDecoration(color: kBk, border: Border.all(color: kFrame, width: 1)),
            child: ClipRect(child: CustomPaint(size: art, painter: _Painter(s, p, widget.time))),
          ),
        ),
        const SizedBox(height: 3),
        Text('${widget.concept} #${widget.n}  ${s.name}',
            maxLines: 1, overflow: TextOverflow.ellipsis, style: TextStyle(fontFamily: opFont, fontSize: 10, height: 1.2, color: kW)),
        Text(s.tags, maxLines: 1, overflow: TextOverflow.ellipsis, style: TextStyle(fontFamily: opFont, fontSize: 9, height: 1.25, color: kMid)),
      ]),
    );
  }
}

class _Painter extends CustomPainter {
  _Painter(this.s, this.p, Listenable t) : super(repaint: t);
  final OpSpec s;
  final OpP p;
  @override
  void paint(Canvas c, Size size) => s.paint(c, size, p);
  @override
  bool shouldRepaint(_Painter o) => true;
}

// ---- drawing ----
const lw = 1.2;
Paint st(Color c, [double w = lw]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Paint fl(Color c) => Paint()..color = c;
Color al(Color c, double o) => c.withValues(alpha: o.clamp(0.0, 1.0));

void ln(Canvas c, Offset a, Offset b, Color col, [double w = lw]) => c.drawLine(a, b, st(col, w));
void ring(Canvas c, Offset o, double r, Color col, [double w = lw]) => c.drawCircle(o, r, st(col, w));
void dot(Canvas c, Offset o, Color col, [double r = 2]) => c.drawCircle(o, r, fl(col));
void pl(Canvas c, List<Offset> pts, Color col, {bool close = false, double w = lw}) {
  if (pts.length < 2) return;
  c.drawPath(poly(pts, close: close), st(col, w));
}

Path poly(List<Offset> pts, {bool close = false}) {
  final path = Path()..moveTo(pts.first.dx, pts.first.dy);
  for (final q in pts.skip(1)) {
    path.lineTo(q.dx, q.dy);
  }
  if (close) path.close();
  return path;
}

/// Dotted line (OP-1 guide verticals).
void dots(Canvas c, Offset a, Offset b, Color col, [double gap = 3]) {
  final d = (b - a).distance;
  final n = (d / gap).floor();
  final pts = <Offset>[for (var i = 0; i <= n; i++) Offset.lerp(a, b, n == 0 ? 0 : i / n)!];
  c.drawPoints(ui.PointMode.points, pts, st(col, 1.2));
}

void arc(Canvas c, Offset o, double r, double a0, double sweep, Color col, [double w = lw]) =>
    c.drawArc(Rect.fromCircle(center: o, radius: r), a0, sweep, false, st(col, w));

// ---- type ----
final _tp = <String, TextPainter>{};

/// Tiny uppercase label. ax: 0 left, .5 centre, 1 right.
void lab(Canvas c, String s, Offset o, Color col, {double size = 8, double ax = 0}) {
  final k = '$s|$size|${col.toARGB32()}';
  if (_tp.length > 600) _tp.clear();
  final tp = _tp.putIfAbsent(
      k,
      () => TextPainter(
            text: TextSpan(text: s, style: TextStyle(fontFamily: opFont, fontSize: size, color: col, letterSpacing: .7, height: 1)),
            textDirection: TextDirection.ltr,
          )..layout());
  tp.paint(c, o - Offset(tp.width * ax, 0));
}

/// Boxed tag like OP-1's [HOLD].
void tag(Canvas c, String s, Offset o, Color col, {double ax = 0}) {
  final w = s.length * 5.6 + 6;
  final r = Rect.fromLTWH(o.dx - w * ax, o.dy, w, 11);
  c.drawRect(r, st(col, 1));
  lab(c, s, Offset(r.left + 3.5, r.top + 2), col, size: 7.5);
}

/// Thin monoline numerals drawn as strokes (digits, . - + x : / %), height h. Returns width. ax: 0 left, 1 right.
double num(Canvas c, String s, Offset o, double h, Color col, {double w = 1.2, double ax = 0}) {
  final gw = h * .56, gap = h * .2;
  double adv(String ch) => switch (ch) { '.' || ':' => h * .18, ' ' => h * .3, _ => gw };
  var tw = 0.0;
  for (final ch in s.split('')) {
    tw += adv(ch) + gap;
  }
  tw -= gap;
  var x = o.dx - tw * ax;
  final path = Path(), dotsP = <Offset>[];
  for (final ch in s.split('')) {
    _glyph(path, dotsP, ch, x, o.dy, gw, h);
    x += adv(ch) + gap;
  }
  c.drawPath(path, st(col, w));
  for (final d in dotsP) {
    c.drawCircle(d, math.max(w * .9, h * .035), fl(col));
  }
  return tw;
}

void _glyph(Path p, List<Offset> dts, String ch, double x0, double y0, double w, double h) {
  final r = w / 2, cx = x0 + r;
  Rect circ(double cy, double rr) => Rect.fromCircle(center: Offset(cx, cy), radius: rr);
  switch (ch) {
    case '0':
      p.addRRect(RRect.fromRectXY(Rect.fromLTWH(x0, y0, w, h), r, r));
    case '1':
      p
        ..moveTo(cx - r * .35, y0 + h * .12)
        ..lineTo(cx + r * .1, y0)
        ..lineTo(cx + r * .1, y0 + h);
    case '2':
      p
        ..moveTo(x0, y0 + r)
        ..arcTo(circ(y0 + r, r), math.pi, math.pi * 1.2, false)
        ..lineTo(x0, y0 + h)
        ..lineTo(x0 + w, y0 + h);
    case '3':
      p
        ..moveTo(x0 + w * .05, y0)
        ..lineTo(x0 + w, y0)
        ..arcTo(circ(y0 + h - r, r), -math.pi * .62, math.pi * 1.48, false);
    case '4':
      p
        ..moveTo(x0 + w * .78, y0 + h)
        ..lineTo(x0 + w * .78, y0)
        ..lineTo(x0, y0 + h * .7)
        ..lineTo(x0 + w, y0 + h * .7);
    case '5':
      p
        ..moveTo(x0 + w, y0)
        ..lineTo(x0 + w * .12, y0)
        ..lineTo(x0 + w * .06, y0 + h * .44)
        ..arcTo(circ(y0 + h - r, r), -math.pi * .78, math.pi * 1.64, false);
    case '6':
      p
        ..moveTo(x0 + w * .8, y0)
        ..lineTo(x0 + w * .06, y0 + h - r * 1.2)
        ..addOval(circ(y0 + h - r, r));
    case '7':
      p
        ..moveTo(x0, y0)
        ..lineTo(x0 + w, y0)
        ..lineTo(x0 + w * .28, y0 + h);
    case '8':
      p
        ..addOval(circ(y0 + r * .86, r * .86))
        ..addOval(circ(y0 + h - r, r));
    case '9':
      p
        ..addOval(circ(y0 + r, r))
        ..moveTo(x0 + w * .98, y0 + r * 1.2)
        ..lineTo(x0 + w * .25, y0 + h);
    case '.':
      dts.add(Offset(x0 + h * .09, y0 + h - h * .03));
    case ':':
      dts
        ..add(Offset(x0 + h * .09, y0 + h * .35))
        ..add(Offset(x0 + h * .09, y0 + h * .8));
    case '-':
      p
        ..moveTo(x0 + w * .1, y0 + h * .55)
        ..lineTo(x0 + w * .9, y0 + h * .55);
    case '+':
      p
        ..moveTo(x0 + w * .05, y0 + h * .55)
        ..lineTo(x0 + w * .95, y0 + h * .55)
        ..moveTo(cx, y0 + h * .55 - r * .9)
        ..lineTo(cx, y0 + h * .55 + r * .9);
    case 'x':
      p
        ..moveTo(x0 + w * .1, y0 + h * .35)
        ..lineTo(x0 + w * .9, y0 + h)
        ..moveTo(x0 + w * .9, y0 + h * .35)
        ..lineTo(x0 + w * .1, y0 + h);
    case '/':
      p
        ..moveTo(x0 + w, y0)
        ..lineTo(x0, y0 + h);
    case '%':
      p
        ..addOval(Rect.fromCircle(center: Offset(x0 + w * .22, y0 + h * .16), radius: w * .17))
        ..addOval(Rect.fromCircle(center: Offset(x0 + w * .78, y0 + h * .84), radius: w * .17))
        ..moveTo(x0 + w, y0)
        ..lineTo(x0, y0 + h);
  }
}

// ---- math ----
double lerp(double a, double b, double t) => a + (b - a) * t;
double frac(double x) => x - x.floorToDouble();
double tri(double x) => 1 - (2 * frac(x) - 1).abs();
Offset pol(Offset c, double r, double a, [double sy = 1]) => c + Offset(math.cos(a) * r, math.sin(a) * r * sy);
String f1(double v) => v.toStringAsFixed(1);
String d2(int v) => v.abs().toString().padLeft(2, '0');

/// Isometric projection: x right-down, y left-down, z up.
Offset iso(Offset o, double x, double y, double z, [double k = 1]) => o + Offset((x - y) * .866 * k, ((x + y) * .5 - z) * k);

/// Isometric wire box from (x,y,z) with size (w,d,h).
void isoBox(Canvas c, Offset o, double x, double y, double z, double w, double d, double h, Color col, [double k = 1]) {
  Offset q(double a, double b, double e) => iso(o, x + a, y + b, z + e, k);
  final path = Path();
  void seg(Offset a, Offset b) => path
    ..moveTo(a.dx, a.dy)
    ..lineTo(b.dx, b.dy);
  final b0 = [q(0, 0, 0), q(w, 0, 0), q(w, d, 0), q(0, d, 0)], t0 = [q(0, 0, h), q(w, 0, h), q(w, d, h), q(0, d, h)];
  for (var i = 0; i < 4; i++) {
    seg(b0[i], b0[(i + 1) % 4]);
    seg(t0[i], t0[(i + 1) % 4]);
    seg(b0[i], t0[i]);
  }
  c.drawPath(path, st(col));
}

/// Tiny 3D point and a pinhole camera orbiting the origin.
class V3 {
  const V3(this.x, this.y, this.z);
  final double x, y, z;
}

class Cam {
  Cam({required this.yaw, this.pitch = .35, required this.dist, required this.f, required this.o});
  final double yaw, pitch, dist, f;
  final Offset o;

  /// Returns null behind the camera.
  Offset? p(V3 v) {
    final cy = math.cos(yaw), sy = math.sin(yaw), cp = math.cos(pitch), sp = math.sin(pitch);
    final x1 = v.x * cy - v.z * sy, z1 = v.x * sy + v.z * cy;
    final y2 = v.y * cp - z1 * sp, z2 = v.y * sp + z1 * cp;
    final d = dist + z2;
    if (d < .05) return null;
    return o + Offset(x1 * f / d, -y2 * f / d);
  }

  void seg(Canvas c, V3 a, V3 b, Color col, [double w = lw]) {
    final pa = p(a), pb = p(b);
    if (pa != null && pb != null) c.drawLine(pa, pb, st(col, w));
  }

  void cube(Canvas c, double s, Color col, {double y0 = 0}) {
    final v = [for (var i = 0; i < 8; i++) V3((i & 1) == 0 ? -s : s, y0 + ((i & 2) == 0 ? -s : s), (i & 4) == 0 ? -s : s)];
    for (final e in const [[0, 1], [2, 3], [4, 5], [6, 7], [0, 2], [1, 3], [4, 6], [5, 7], [0, 4], [1, 5], [2, 6], [3, 7]]) {
      seg(c, v[e[0]], v[e[1]], col);
    }
  }

  void floor(Canvas c, double s, int n, Color col, {double y = -1}) {
    for (var i = 0; i <= n; i++) {
      final u = -s + 2 * s * i / n;
      seg(c, V3(u, y, -s), V3(u, y, s), col, 1);
      seg(c, V3(-s, y, u), V3(s, y, u), col, 1);
    }
  }
}
