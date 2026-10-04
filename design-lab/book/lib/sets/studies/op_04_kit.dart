// OP 04 shared kit, after the OP-1 screen: black ground, monoline drawings, the four encoder colours (blue, green, white, red),
// large thin numerals and tiny uppercase labels drawn as strokes (a small stroke font, so a numeral is a line drawing too).
// One sheet = 20 panels in a 5 x 4 grid under one Ticker. A panel only paints; its gesture decides how the pointer moves the
// three values (0..1). Param colours are fixed per sheet: value 0 = blue, 1 = green, 2 = red; the drawing itself is white.
import 'dart:math' as math;

import 'package:flutter/gestures.dart' show PointerScrollEvent;
import 'package:flutter/scheduler.dart' show Ticker;
import 'package:flutter/widgets.dart';

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

const _g = <String, String>{
  '0': 'M0 3 A3 3 3 180 180 L6 7 A3 7 3 0 180 L0 3',
  '1': 'M1.6 1.6 L3 0 L3 10',
  '2': 'M0 3 A3 3 3 180 220 L0 10 L6 10',
  '3': 'M0 0 L6 0 L3 4 A3 7 3 270 225',
  '4': 'M4.6 10 L4.6 0 L0 7 L6 7',
  '5': 'M6 0 L1 0 L0.6 4.9 A3 7 3 222 273',
  '6': 'M4.8 0 L0.9 4.9 O3 7 3',
  '7': 'M0 0 L6 0 L2 10',
  '8': 'O3 2.3 2.3 O3 7.3 2.7',
  '9': 'O3 3 3 M5.1 5.1 L1.2 10',
  'A': 'M0 10 L3 0 L6 10 M1.2 6 L4.8 6',
  'B': 'M0 10 L0 0 L3.5 0 A3.5 2.5 2.5 270 180 L0 5 M3.5 5 A3.5 7.5 2.5 270 180 L0 10',
  'C': 'M5.1 0.9 A3 3 3 315 -135 L0 7 A3 7 3 180 -135',
  'D': 'M0 0 L0 10 L3 10 A3 7 3 90 -90 L6 3 A3 3 3 0 -90 L0 0',
  'E': 'M6 0 L0 0 L0 10 L6 10 M0 5 L4.5 5',
  'F': 'M6 0 L0 0 L0 10 M0 5 L4.5 5',
  'G': 'M5.1 0.9 A3 3 3 315 -135 L0 7 A3 7 3 180 -180 L6 5.5 L3.5 5.5',
  'H': 'M0 0 L0 10 M6 0 L6 10 M0 5 L6 5',
  'I': 'M3 0 L3 10',
  'J': 'M6 0 L6 7 A3 7 3 0 180',
  'K': 'M0 0 L0 10 M6 0 L0 6 M2 4.2 L6 10',
  'L': 'M0 0 L0 10 L6 10',
  'M': 'M0 10 L0 0 L3 6 L6 0 L6 10',
  'N': 'M0 10 L0 0 L6 10 L6 0',
  'O': 'M0 3 A3 3 3 180 180 L6 7 A3 7 3 0 180 L0 3',
  'P': 'M0 10 L0 0 L3.2 0 A3.2 2.7 2.7 270 180 L0 5.4',
  'Q': 'M0 3 A3 3 3 180 180 L6 7 A3 7 3 0 180 L0 3 M3.6 7.4 L6.4 10.4',
  'R': 'M0 10 L0 0 L3.2 0 A3.2 2.7 2.7 270 180 L0 5.4 M3 5.4 L6 10',
  'S': 'M5.25 1.3 A3 2.6 2.6 330 -240 A3 7.6 2.4 270 240',
  'T': 'M0 0 L6 0 M3 0 L3 10',
  'U': 'M0 0 L0 7 A3 7 3 180 -180 L6 0',
  'V': 'M0 0 L3 10 L6 0',
  'W': 'M0 0 L1.5 10 L3 4 L4.5 10 L6 0',
  'X': 'M0 0 L6 10 M6 0 L0 10',
  'Y': 'M0 0 L3 5 L6 0 M3 5 L3 10',
  'Z': 'M0 0 L6 0 L0 10 L6 10',
  '.': 'O3 9.5 0.4',
  '%': 'O1.3 1.6 1.3 O4.7 8.4 1.3 M5.5 0 L0.5 10',
  '-': 'M1 5.5 L5 5.5',
  '+': 'M0.5 5.5 L5.5 5.5 M3 3 L3 8',
  ':': 'O3 3 0.4 O3 8 0.4',
  '/': 'M5 0 L1 10',
  'x': 'M1 4 L5 9 M5 4 L1 9',
  '>': 'M1 1 L5 5 L1 9',
  '<': 'M5 1 L1 5 L5 9',
  ' ': '',
};

final _gc = <String, Path>{};

Path _glyph(String ch) => _gc.putIfAbsent(ch, () {
      final p = Path();
      final tk = (_g[ch] ?? '').split(' ').where((s) => s.isNotEmpty).toList();
      var i = 0;
      double nx() => double.parse(tk[i].length > 1 && 'MLAO'.contains(tk[i][0]) ? tk[i++].substring(1) : tk[i++]);
      while (i < tk.length) {
        final cmd = tk[i][0];
        if (cmd == 'M') {
          final x = nx(), y = nx();
          p.moveTo(x, y);
        } else if (cmd == 'L') {
          final x = nx(), y = nx();
          p.lineTo(x, y);
        } else if (cmd == 'A') {
          final cx = nx(), cy = nx(), r = nx(), a0 = nx(), sw = nx();
          p.arcTo(Rect.fromCircle(center: Offset(cx, cy), radius: r), a0 * math.pi / 180, sw * math.pi / 180, false);
        } else if (cmd == 'O') {
          final cx = nx(), cy = nx(), r = nx();
          p.addOval(Rect.fromCircle(center: Offset(cx, cy), radius: r));
        } else {
          i++;
        }
      }
      return p;
    });

/// Stroke text, [h] = cap height in px. [ax] 0 left, .5 centre, 1 right. Returns the drawn width.
double tx(Canvas c, String s, Offset at, double h, Color col, {double ax = 0, double w = 1, double track = .45}) {
  s = s.toUpperCase();
  final k = h / 10, adv = (6 + 10 * track) * k;
  final width = s.isEmpty ? 0.0 : s.length * adv - 10 * track * k;
  final p = st(col, w);
  c.save();
  c.translate(at.dx - width * ax, at.dy);
  c.scale(k);
  p.strokeWidth = w / k;
  for (var i = 0; i < s.length; i++) {
    final ch = s[i];
    if (ch == '1' && h > 12) {
      c.drawLine(const Offset(3, 0), const Offset(3, 10), p);
    } else {
      c.drawPath(_glyph(ch), p);
    }
    c.translate(6 + 10 * track, 0);
  }
  c.restore();
  return width;
}

/// Tiny label (OP-1 'FREQ', 'DAMP').
double lb(Canvas c, String s, Offset at, Color col, {double ax = 0, double h = 6}) => tx(c, s, at, h, col, ax: ax, w: .9, track: .5);

/// Large thin numeral.
double nm(Canvas c, String s, Offset at, double h, Color col, {double ax = 0}) => tx(c, s, at, h, col, ax: ax, w: 1.15, track: .35);

String d2(double x) => (x * 99).round().clamp(0, 99).toString().padLeft(2, '0');

/// Label over a two-digit numeral in the value's colour.
void rd(Canvas c, OpV v, int i, String label, Offset at, {double h = 20, double ax = 0}) {
  lb(c, label, at, pc(v, i), ax: ax);
  nm(c, d2(v[i]), at + const Offset(0, 10), h, pc(v, i), ax: ax);
}

// ---- motion maths -------------------------------------------------------------------------------------------------------------------

double _bz(double a, double b, double u) => 3 * a * u * (1 - u) * (1 - u) + 3 * b * u * u * (1 - u) + u * u * u;

/// Easing: [a] slow start, [b] slow arrival, [o] flying past the target before landing. [om] scales the overshoot.
double ez(double x, double a, double b, double o, [double om = 1.1]) {
  x = cl(x);
  final x1 = .02 + .9 * a, x2 = .98 - .9 * b, y2 = 1 + om * o;
  var lo = 0.0, hi = 1.0;
  for (var k = 0; k < 16; k++) {
    final u = (lo + hi) / 2;
    _bz(x1, x2, u) < x ? lo = u : hi = u;
  }
  return _bz(0, y2, (lo + hi) / 2);
}

double ezv(double x, OpV v, [double om = 1.1]) => ez(x, v[0], v[1], v[2], om);

/// Loop of [per] s: rest, travel, hold. Returns travel progress 0..1.
double trip(double t, [double per = 2.6, double shift = 0]) => cl(((t + shift) % per - .35) / (per * .6));

/// Spring displacement after release (1 -> 0). a stiffness, b damping, c mass (0..1 each).
double sp(double tau, double a, double b, double c) {
  if (tau <= 0) return 1;
  final k = 30 + 370 * a, m = .4 + 3.6 * c, d = .4 + 26 * b * b;
  final w = math.sqrt(k / m), z = d / (2 * math.sqrt(k * m));
  if (z >= 1) {
    final w2 = w * (z - math.sqrt(z * z - 1)) + .01;
    return math.exp(-w2 * tau) * (1 + w2 * tau);
  }
  final wd = w * math.sqrt(1 - z * z);
  return math.exp(-z * w * tau) * (math.cos(wd * tau) + z * w / wd * math.sin(wd * tau));
}

double spv(double tau, OpV v) => sp(tau, v[0], v[1], v[2]);

/// Spring re-fires on release and every [per] s on its own; held = 1 (stretched).
double ring(OpV v, double t, [double per = 3.2]) => v.press ? 1 : spv(v.since % per, v);

double _h(int i) {
  final x = math.sin(i * 127.1 + 311.7) * 43758.5453;
  return (x - x.floorToDouble()) * 2 - 1;
}

double _vn(double x, double smooth) {
  final i = x.floor(), f = x - i;
  final s = f * f * (3 - 2 * f);
  return lr(_h(i), _h(i + 1), lr(f, s, smooth));
}

/// Wiggle in about -1..1: [a] frequency (0.3..12 Hz), [cs] smoothness (1 = one soft wave, 0 = jagged layers).
double wg(double t, double a, double cs, [double seed = 0]) {
  final hz = .3 + 11.7 * a * a;
  var sum = 0.0, amp = 1.0, norm = 0.0, fr = hz;
  final rough = .1 + .65 * (1 - cs);
  for (var o = 0; o < 3; o++) {
    sum += amp * _vn(t * fr + seed * 17.3 + o * 41.1, cs);
    norm += amp;
    amp *= rough;
    fr *= 2.3;
  }
  return sum / norm;
}

/// Wiggle with all three values: frequency v[0], amplitude v[1], smoothness v[2].
double wv(double t, OpV v, [double seed = 0]) => wg(t, v[0], v[2], seed) * v[1];
