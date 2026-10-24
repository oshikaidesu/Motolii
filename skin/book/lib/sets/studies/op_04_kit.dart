// OP 04 shared kit, after the OP-1 screen: black ground, monoline drawings, the four encoder colours (blue, green, white, red),
// large thin numerals and tiny uppercase labels drawn as strokes (a small stroke font, so a numeral is a line drawing too).
// One sheet = 20 panels in a 5 x 4 grid under one Ticker. A panel only paints; its gesture decides how the pointer moves the
// three values (0..1). Param colours are fixed per sheet: value 0 = blue, 1 = green, 2 = red; the drawing itself is white.
import 'dart:math' as math;

import 'package:flutter/gestures.dart' show PointerScrollEvent;
import 'package:flutter/scheduler.dart' show Ticker;
import 'package:flutter/widgets.dart';
part 'op_04_kit_2.dart';

abstract final class K {
  static const bg = Color(0xFF000000), page = Color(0xFF0B0B0C), frame = Color(0xFF222222);
  static const blue = Color(0xFF4F6CFF), green = Color(0xFF23E0A3), white = Color(0xFFE8E8EA), red = Color(0xFFFF3363);
  static const purple = Color(0xFF9B7BFF), dim = Color(0xFF3A3A40), grey = Color(0xFF8A8A92);
  static const enc = [blue, green, red];
}

double opNow = 0;

// ---- values and gestures ------------------------------------------------------------------------------------------------------------

class OpV extends ChangeNotifier {
  OpV(List<double> init)
      : p = [...init],
        _init = [...init];
  final List<double> p, _init;
  int hot = -1;
  double hotT = -9, kickT = 0, ang = 0, energy = 0;
  bool press = false;
  Offset? pos;
  Offset at = const Offset(78, 60);
  final List<Offset> trail = [];
  double operator [](int i) => p[i];
  void set(int i, double x) {
    if (i < 0 || i >= p.length) return;
    p[i] = x.clamp(0.0, 1.0);
    hot = i;
    hotT = opNow;
  }

  void bump() => notifyListeners();
  void reset() {
    for (var i = 0; i < p.length; i++) {
      p[i] = _init[i];
    }
    trail.clear();
    kickT = opNow;
    notifyListeners();
  }

  /// Seconds since the last release (or reset); one-shot drawings (springs, flicks) replay from here.
  double get since => opNow - kickT;
}

class OpE {
  const OpE(this.phase, this.pos, this.delta, this.size, this.vel);
  final int phase; // 0 start, 1 move, 2 end
  final Offset pos, delta, vel; // pos/delta/vel in px
  final Size size;
  Offset get n => Offset(pos.dx / size.width, pos.dy / size.height);
}

abstract class OpG {
  const OpG();
  String get word;
  void on(OpV v, OpE e);
}

/// Drag: horizontal moves value [x], vertical (up) moves value [y].
class OpDrag extends OpG {
  const OpDrag([this.x = 0, this.y = 1]);
  final int x, y;
  @override
  String get word => 'drag';
  @override
  void on(OpV v, OpE e) {
    if (e.phase != 1) return;
    final ax = e.delta.dx.abs(), ay = e.delta.dy.abs();
    if (ax >= ay) {
      v.set(x, v[x] + e.delta.dx / 130);
    } else {
      v.set(y, v[y] - e.delta.dy / 100);
    }
  }
}

/// Spin around [c] (fraction of the panel): turning moves value [i]; [j] follows the distance from the centre.
class OpSpin extends OpG {
  const OpSpin(this.i, [this.c = const Offset(.5, .5), this.j = -1]);
  final int i, j;
  final Offset c;
  @override
  String get word => 'spin';
  @override
  void on(OpV v, OpE e) {
    if (e.phase != 1) return;
    final o = Offset(c.dx * e.size.width, c.dy * e.size.height);
    final a0 = (e.pos - e.delta - o).direction, a1 = (e.pos - o).direction;
    var d = a1 - a0;
    if (d > math.pi) d -= 2 * math.pi;
    if (d < -math.pi) d += 2 * math.pi;
    v.ang += d;
    v.set(i, v[i] + d / (2 * math.pi) * .7);
    if (j >= 0) v.set(j, ((e.pos - o).distance / (e.size.shortestSide * .5)).clamp(0.0, 1.0));
    v.hot = i;
  }
}

/// Pinch (with a mouse: drag toward / away from the centre): the distance moves value [i]; going around moves [j].
class OpPinch extends OpG {
  const OpPinch(this.i, [this.j = -1, this.c = const Offset(.5, .5)]);
  final int i, j;
  final Offset c;
  @override
  String get word => 'pinch';
  @override
  void on(OpV v, OpE e) {
    if (e.phase != 1) return;
    final o = Offset(c.dx * e.size.width, c.dy * e.size.height);
    final d0 = (e.pos - e.delta - o).distance, d1 = (e.pos - o).distance;
    if (j >= 0 && (d1 - d0).abs() < e.delta.distance * .5) {
      final a0 = (e.pos - e.delta - o).direction, a1 = (e.pos - o).direction;
      var d = a1 - a0;
      if (d.abs() > math.pi) d -= d.sign * 2 * math.pi;
      v.set(j, v[j] + d / 4);
    } else {
      v.set(i, v[i] + (d1 - d0) / 60);
    }
  }
}

/// Rub: how hard you scrub sets value [i] (gentle = low, frantic = high); the height you rub at sets [j].
class OpRub extends OpG {
  const OpRub(this.i, [this.j = -1]);
  final int i, j;
  @override
  String get word => 'rub';
  @override
  void on(OpV v, OpE e) {
    if (e.phase != 1) return;
    final s = (e.delta.distance / 14).clamp(0.0, 1.0);
    v.energy = math.min(1.0, v.energy + e.delta.distance / 300);
    v.set(i, v[i] + (s - v[i]) * .12);
    if (j >= 0) v.set(j, 1 - e.n.dy);
    v.hot = i;
  }
}

/// Flick: while held, horizontal aims value [j]; the release speed sets value [i] and replays the drawing.
class OpFlick extends OpG {
  const OpFlick(this.i, [this.j = -1]);
  final int i, j;
  @override
  String get word => 'flick';
  @override
  void on(OpV v, OpE e) {
    if (e.phase == 1 && j >= 0) v.set(j, v[j] + e.delta.dx / 130);
    if (e.phase == 2) v.set(i, (e.vel.distance / 2400).clamp(.04, 1.0));
  }
}

/// Pull (slingshot): drag back from where you pressed; distance sets [i], vertical angle sets [j]; release fires.
class OpPull extends OpG {
  const OpPull(this.i, [this.j = -1]);
  final int i, j;
  @override
  String get word => 'pull';
  @override
  void on(OpV v, OpE e) {
    if (e.phase == 0) {
      v.trail
        ..clear()
        ..add(e.n);
    }
    if (e.phase == 1 && v.trail.isNotEmpty) {
      final d = e.n - v.trail.first;
      v.set(i, d.distance * 1.6);
      if (j >= 0) v.set(j, .5 - d.dy * 1.4);
      v.hot = i;
    }
  }
}

/// Draw: the stroke is kept in [OpV.trail] (normalised); [fit] reads the values off the drawing on release.
class OpDraw extends OpG {
  const OpDraw(this.fit);
  final void Function(OpV v) fit;
  @override
  String get word => 'draw';
  @override
  void on(OpV v, OpE e) {
    if (e.phase == 0) v.trail.clear();
    v.trail.add(e.n);
    if (e.phase == 2 && v.trail.length > 3) fit(v);
  }
}

// ---- panel and sheet ----------------------------------------------------------------------------------------------------------------

typedef OpPaint = void Function(Canvas c, Size s, OpV v, double t);

class OpPanel {
  const OpPanel(this.name, this.tags, this.paint, {this.g = const OpDrag(), this.init = const [.45, .55, .3]});
  final String name, tags;
  final OpPaint paint;
  final OpG g;
  final List<double> init;
}

const opW = 156.0, opH = 120.0;

class OpSheet extends StatefulWidget {
  const OpSheet({super.key, required this.concept, required this.params, required this.panels});
  final String concept;
  final List<String> params;
  final List<OpPanel> panels;
  @override
  State<OpSheet> createState() => _OpSheetState();
}

class _OpSheetState extends State<OpSheet> with SingleTickerProviderStateMixin {
  final time = ValueNotifier<double>(0);
  late final vs = [for (final p in widget.panels) OpV(p.init)];
  late final Ticker _tk;

  @override
  void initState() {
    super.initState();
    _tk = createTicker((d) {
      final t = d.inMicroseconds / 1e6;
      final dt = t - time.value;
      opNow = t;
      for (final v in vs) {
        if (v.energy > 0) v.energy = math.max(0, v.energy - dt * .6);
      }
      time.value = t;
    })
      ..start();
  }

  @override
  void dispose() {
    _tk.dispose();
    time.dispose();
    for (final v in vs) {
      v.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    const lab = TextStyle(fontFamily: 'Avenir Next', fontFamilyFallback: ['.AppleSystemUIFont'], fontSize: 9.5, height: 1.15, color: K.white, decoration: TextDecoration.none);
    final legend = <Widget>[
      Text('${widget.concept.toUpperCase()}  x20', style: lab.copyWith(fontSize: 11, fontWeight: FontWeight.w600, letterSpacing: 1.2)),
      const SizedBox(width: 16),
      for (var i = 0; i < widget.params.length; i++) ...[
        Container(width: 7, height: 7, decoration: BoxDecoration(shape: BoxShape.circle, color: K.enc[i])),
        const SizedBox(width: 4),
        Text(widget.params[i].toUpperCase(), style: lab.copyWith(color: K.enc[i], letterSpacing: 1)),
        const SizedBox(width: 12),
      ],
      Text('drag any drawing  ·  double-tap resets', style: lab.copyWith(color: K.grey)),
    ];
    final cells = <Widget>[];
    for (var k = 0; k < widget.panels.length; k++) {
      final p = widget.panels[k];
      cells.add(SizedBox(
        width: opW,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          _OpCell(panel: p, v: vs[k], time: time),
          const SizedBox(height: 3),
          Text('${widget.concept.toLowerCase()} #${k + 1}  ${p.name}', maxLines: 1, overflow: TextOverflow.ellipsis, style: lab),
          Text('${p.tags} · ${p.g.word}', maxLines: 1, overflow: TextOverflow.ellipsis, style: lab.copyWith(fontSize: 8.5, color: K.grey)),
        ]),
      ));
    }
    return ColoredBox(
      color: K.page,
      child: Align(
        alignment: Alignment.topLeft,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(12),
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
            Row(mainAxisSize: MainAxisSize.min, children: legend),
            const SizedBox(height: 10),
            SizedBox(width: opW * 5 + 8 * 4, child: Wrap(spacing: 8, runSpacing: 8, children: cells)),
          ]),
        ),
      ),
    );
  }
}

class _OpCell extends StatelessWidget {
  const _OpCell({required this.panel, required this.v, required this.time});
  final OpPanel panel;
  final OpV v;
  final ValueNotifier<double> time;

  void _send(int ph, Offset pos, Offset d, [Offset vel = Offset.zero]) {
    panel.g.on(v, OpE(ph, pos, d, const Size(opW, opH), vel));
    v.pos = ph == 2 ? null : pos;
    v.at = pos;
    v.bump();
  }

  @override
  Widget build(BuildContext context) {
    Offset last = Offset.zero;
    return Listener(
      onPointerSignal: (e) {
        if (e is PointerScrollEvent) {
          final g = panel.g;
          final i = g is OpDrag ? g.y : 0;
          v.set(i, v[i] - e.scrollDelta.dy / 600);
          v.bump();
        }
      },
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onDoubleTap: v.reset,
        onPanStart: (d) {
          last = d.localPosition;
          v.press = true;
          _send(0, d.localPosition, Offset.zero);
        },
        onPanUpdate: (d) {
          last = d.localPosition;
          _send(1, d.localPosition, d.delta);
        },
        onPanEnd: (d) {
          v.press = false;
          v.kickT = opNow;
          _send(2, last, Offset.zero, d.velocity.pixelsPerSecond);
        },
        child: Container(
          width: opW,
          height: opH,
          decoration: BoxDecoration(color: K.bg, border: Border.all(color: K.frame, width: 1)),
          child: ClipRect(child: CustomPaint(painter: _OpPainter(panel, v, time), size: const Size(opW, opH))),
        ),
      ),
    );
  }
}

class _OpPainter extends CustomPainter {
  _OpPainter(this.panel, this.v, this.time) : super(repaint: Listenable.merge([v, time]));
  final OpPanel panel;
  final OpV v;
  final ValueNotifier<double> time;
  @override
  void paint(Canvas c, Size s) => panel.paint(c, s, v, time.value);
  @override
  bool shouldRepaint(_OpPainter o) => o.panel != panel || o.v != v;
}

// ---- drawing ------------------------------------------------------------------------------------------------------------------------

final _pp = Paint()
  ..style = PaintingStyle.stroke
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round
  ..isAntiAlias = true;
final _pf = Paint()..isAntiAlias = true;

Paint st(Color col, [double w = 1.1]) => _pp
  ..color = col
  ..strokeWidth = w;
Paint fl(Color col) => _pf..color = col;

/// Encoder colour for value [i]; the one being touched lights, the others step back while it is touched.
Color pc(OpV v, int i, [double a = 1]) {
  final recent = v.press || opNow - v.hotT < .9;
  final k = !recent || v.hot < 0 ? .95 : (v.hot == i ? 1.0 : .38);
  return K.enc[i].withValues(alpha: k * a);
}

Color al(Color c, double a) => c.withValues(alpha: (c.a * a).clamp(0.0, 1.0));
double cl(double x, [double a = 0, double b = 1]) => x < a ? a : (x > b ? b : x);
double lr(double a, double b, double t) => a + (b - a) * t;
Offset ol(Offset a, Offset b, double t) => Offset(lr(a.dx, b.dx, t), lr(a.dy, b.dy, t));
Offset pol(Offset c, double r, double a) => c + Offset(math.cos(a), math.sin(a)) * r;

void ln(Canvas c, Offset a, Offset b, Color col, [double w = 1.1]) => c.drawLine(a, b, st(col, w));
void ci(Canvas c, Offset o, double r, Color col, [double w = 1.1]) => c.drawCircle(o, r, st(col, w));
void dot(Canvas c, Offset o, double r, Color col) => c.drawCircle(o, r, fl(col));
void rc(Canvas c, Rect r, Color col, [double w = 1.1]) => c.drawRect(r, st(col, w));
void arc(Canvas c, Offset o, double r, double a0, double sw, Color col, [double w = 1.1]) =>
    c.drawArc(Rect.fromCircle(center: o, radius: r), a0, sw, false, st(col, w));

void pl(Canvas c, List<Offset> pts, Color col, [double w = 1.1, bool close = false]) {
  if (pts.length < 2) return;
  final p = Path()..moveTo(pts[0].dx, pts[0].dy);
  for (var i = 1; i < pts.length; i++) {
    p.lineTo(pts[i].dx, pts[i].dy);
  }
  if (close) p.close();
  c.drawPath(p, st(col, w));
}

/// Polyline from a generator f(u), u in 0..1, [n] segments.
void fn(Canvas c, int n, Offset Function(double u) f, Color col, [double w = 1.1]) {
  final a = f(0);
  final p = Path()..moveTo(a.dx, a.dy);
  for (var i = 1; i <= n; i++) {
    final q = f(i / n);
    p.lineTo(q.dx, q.dy);
  }
  c.drawPath(p, st(col, w));
}

/// Dotted line (OP-1 guides).
void dl(Canvas c, Offset a, Offset b, Color col, [double gap = 3.2, double r = .55]) {
  final d = (b - a).distance;
  final n = math.max(1, (d / gap).floor());
  for (var i = 0; i <= n; i++) {
    c.drawCircle(ol(a, b, i / n), r, fl(col));
  }
}

/// Isometric point: x right-down, y left-down, z up.
Offset iso(Offset o, double x, double y, double z, double s) => o + Offset((x - y) * .866 * s, ((x + y) * .5 - z) * s);

void isoBox(Canvas c, Offset o, double x, double y, double z, double w, double d, double h, double s, Color col, [double lw = 1]) {
  Offset q(double a, double b, double e) => iso(o, x + a, y + b, z + e, s);
  final b1 = q(w, 0, 0), b2 = q(w, d, 0), b3 = q(0, d, 0);
  final t0 = q(0, 0, h), t1 = q(w, 0, h), t2 = q(w, d, h), t3 = q(0, d, h);
  pl(c, [b1, b2, b3], col, lw);
  pl(c, [t0, t1, t2, t3], col, lw, true);
  ln(c, b1, t1, col, lw);
  ln(c, b2, t2, col, lw);
  ln(c, b3, t3, col, lw);
}

void isoGrid(Canvas c, Offset o, int n, int m, double s, Color col, [double z = 0]) {
  for (var i = 0; i <= n; i++) {
    ln(c, iso(o, i.toDouble(), 0, z, s), iso(o, i.toDouble(), m.toDouble(), z, s), col, .8);
  }
  for (var j = 0; j <= m; j++) {
    ln(c, iso(o, 0, j.toDouble(), z, s), iso(o, n.toDouble(), j.toDouble(), z, s), col, .8);
  }
}

// ---- stroke font: glyph box 6 x 10 ---------------------------------------------------------------------------------------------------
