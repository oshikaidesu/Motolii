part of 'op_06.dart';

const _k = Color(0xFF0B0B0C), _fr = Color(0xFF222222), _dg = Color(0xFF3A3A40);
const _bl = Color(0xFF4F6CFF), _gr = Color(0xFF23E0A3), _wh = Color(0xFFE8E8EA), _rd = Color(0xFFFF3363), _pu = Color(0xFF9B7BFF);

enum OpG { drag, flick, rub, pinch, spin, draw }

/// One panel's own state: a and b in 0..1, the toggle, the pointer, a drawn trail, momentum and rub energy, the sheet clock.
class OpS {
  OpS(this.a, this.b);
  double a, b, t = 0, e = 0, vel = 0, dt = 0, a0 = 0, ang = 0;
  bool inv = false, down = false;
  Offset? p;
  final trail = <Offset>[];
  double _lt = -1;

  void tick(double now, bool wrap) {
    dt = _lt < 0 ? 0 : (now - _lt).clamp(0.0, .1);
    _lt = now;
    t = now;
    if (!down && vel.abs() > .002) {
      a += vel * dt;
      vel *= math.pow(.12, dt).toDouble();
      if (wrap) {
        a -= a.floorToDouble();
      } else if (a < 0 || a > 1) {
        a = a.clamp(0.0, 1.0);
        vel = -vel * .45;
      }
    }
    e *= math.pow(.2, dt).toDouble();
  }
}

typedef OpPaint = void Function(Canvas c, Size z, OpS s);

class OpP {
  const OpP(this.name, this.tags, this.g, this.paint, {this.a = .5, this.b = .5, this.wrap = false});
  final String name, tags;
  final OpG g;
  final OpPaint paint;
  final double a, b;
  final bool wrap;
}

class Op06Sheet extends StatefulWidget {
  const Op06Sheet({super.key, required this.concept, required this.panels});
  final String concept;
  final List<OpP> panels;
  @override
  State<Op06Sheet> createState() => _Op06SheetState();
}

class _Op06SheetState extends State<Op06Sheet> with SingleTickerProviderStateMixin {
  final _time = ValueNotifier<double>(0);
  late final Ticker _ticker;

  @override
  void initState() {
    super.initState();
    _ticker = createTicker((e) => _time.value = e.inMicroseconds / 1e6)..start();
  }

  @override
  void dispose() {
    _ticker.dispose();
    _time.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: const Color(0xFF000000),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: 5 * 156 + 4 * 8,
              child: Wrap(spacing: 8, runSpacing: 6, children: [
                for (var i = 0; i < widget.panels.length; i++) _OpCell(d: widget.panels[i], n: i + 1, concept: widget.concept, time: _time),
              ]),
            ),
          ),
        ),
      );
}

class _OpCell extends StatefulWidget {
  const _OpCell({required this.d, required this.n, required this.concept, required this.time});
  final OpP d;
  final int n;
  final String concept;
  final ValueNotifier<double> time;
  @override
  State<_OpCell> createState() => _OpCellState();
}

class _OpCellState extends State<_OpCell> {
  late final s = OpS(widget.d.a, widget.d.b);
  final _poke = ValueNotifier<int>(0);
  static const _sz = Size(156, 120);

  double _c(double v) => widget.d.wrap ? v - v.floorToDouble() : v.clamp(0.0, 1.0);

  void _start(ScaleStartDetails e) {
    s
      ..down = true
      ..vel = 0
      ..a0 = s.a
      ..p = e.localFocalPoint
      ..ang = _angle(e.localFocalPoint);
    if (widget.d.g == OpG.draw) s.trail.clear();
    _poke.value++;
  }

  double _angle(Offset p) => math.atan2(p.dy - _sz.height / 2, p.dx - _sz.width / 2);

  void _update(ScaleUpdateDetails e) {
    final d = e.focalPointDelta, p = e.localFocalPoint;
    s
      ..p = p
      ..e = (s.e + d.distance / 260).clamp(0.0, 1.6);
    switch (widget.d.g) {
      case OpG.drag:
      case OpG.flick:
        s.a = _c(s.a + d.dx / 110);
        s.b = (s.b - d.dy / 90).clamp(0.0, 1.0);
      case OpG.rub:
        s.a = _c(s.a + (d.dx.abs() - d.dy.abs() * .7) / 520);
      case OpG.pinch:
        if (e.pointerCount >= 2) {
          s.a = _c((s.a0 + .08) * e.scale - .08);
        } else {
          s.a = _c(s.a - d.dy / 90);
          s.b = (s.b + d.dx / 110).clamp(0.0, 1.0);
        }
      case OpG.spin:
        final a = _angle(p);
        var da = a - s.ang;
        if (da > math.pi) da -= 2 * math.pi;
        if (da < -math.pi) da += 2 * math.pi;
        s
          ..ang = a
          ..a = _c(s.a + da / (2 * math.pi));
        s.b = ((p - _sz.center(Offset.zero)).distance / 60).clamp(0.0, 1.0);
      case OpG.draw:
        s.trail.add(p);
        if (s.trail.length > 140) s.trail.removeAt(0);
        s.a = (p.dx / _sz.width).clamp(0.0, 1.0);
        s.b = (1 - p.dy / _sz.height).clamp(0.0, 1.0);
    }
    _poke.value++;
  }

  void _end(ScaleEndDetails e) {
    s.down = false;
    if (widget.d.g == OpG.flick) s.vel = (e.velocity.pixelsPerSecond.dx / 110).clamp(-4.0, 4.0);
    _poke.value++;
  }

  @override
  void dispose() {
    _poke.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    const t1 = TextStyle(fontFamily: _ff, fontFamilyFallback: _fb, fontSize: 9.5, height: 1.3, color: _wh, letterSpacing: .2);
    const t2 = TextStyle(fontFamily: _ff, fontFamilyFallback: _fb, fontSize: 9, height: 1.3, color: Color(0xFF7A7A82), letterSpacing: .2);
    return SizedBox(
      width: 156,
      height: 148,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        GestureDetector(
          onScaleStart: _start,
          onScaleUpdate: _update,
          onScaleEnd: _end,
          onTap: () {
            s.inv = !s.inv;
            _poke.value++;
          },
          child: CustomPaint(size: _sz, painter: _OpPainter(widget.d, s, widget.time, Listenable.merge([widget.time, _poke]))),
        ),
        const SizedBox(height: 3),
        Text('${widget.concept} #${widget.n}  ${widget.d.name}', style: t1, maxLines: 1, overflow: TextOverflow.ellipsis),
        Text(widget.d.tags, style: t2, maxLines: 1, overflow: TextOverflow.ellipsis),
      ]),
    );
  }
}

class _OpPainter extends CustomPainter {
  _OpPainter(this.d, this.s, this.time, Listenable repaint) : super(repaint: repaint);
  final OpP d;
  final OpS s;
  final ValueNotifier<double> time;

  @override
  void paint(Canvas c, Size z) {
    s.tick(time.value, d.wrap);
    final r = Offset.zero & z;
    c.drawRect(r, Paint()..color = _k);
    c.save();
    c.clipRect(r);
    d.paint(c, z, s);
    c.restore();
    c.drawRect(r.deflate(.5), _st(s.down ? const Color(0xFF3A3A40) : _fr, 1));
  }

  @override
  bool shouldRepaint(covariant _OpPainter old) => true;
}

// ---------- drawing vocabulary ----------

const _ff = 'Inter';
const _fb = ['.AppleSystemUIFont'];

Paint _st(Color c, [double w = 1.2]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Paint _fl(Color c) => Paint()..color = c;
Color _al(Color c, double a) => c.withValues(alpha: a.clamp(0.0, 1.0));
double _lp(double a, double b, double t) => a + (b - a) * t;
String _n(double v) => (v * 99).round().toString().padLeft(2, '0');
double _h(int i) {
  final x = math.sin(i * 12.9898 + 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

Offset _pol(Offset c, double r, double a) => c + Offset(math.cos(a), math.sin(a)) * r;

void _ln(Canvas c, double x1, double y1, double x2, double y2, Color col, [double w = 1.2]) => c.drawLine(Offset(x1, y1), Offset(x2, y2), _st(col, w));
void _dot(Canvas c, Offset o, Color col, [double r = 1.6]) => c.drawCircle(o, r, _fl(col));
void _ring(Canvas c, Offset o, double r, Color col, [double w = 1.2]) => c.drawCircle(o, math.max(.3, r), _st(col, w));
void _path(Canvas c, Path p, Color col, [double w = 1.2]) => c.drawPath(p, _st(col, w));

/// Dotted vertical guide, as on the OP-1 envelope drawings.
void _vdots(Canvas c, double x, double y1, double y2, Color col) {
  for (var y = y1; y <= y2; y += 3) {
    c.drawRect(Rect.fromLTWH(x - .5, y, 1, 1.2), _fl(col));
  }
}

void _hdots(Canvas c, double x1, double x2, double y, Color col, [double gap = 3]) {
  for (var x = x1; x <= x2; x += gap) {
    c.drawRect(Rect.fromLTWH(x, y - .5, 1.2, 1), _fl(col));
  }
}

Path _poly(List<Offset> pts, {bool close = false}) {
  final p = Path()..moveTo(pts.first.dx, pts.first.dy);
  for (final q in pts.skip(1)) {
    p.lineTo(q.dx, q.dy);
  }
  return close ? (p..close()) : p;
}

Path _star(Offset c, double ro, double ri, int n, [double rot = -math.pi / 2]) {
  final pts = [for (var i = 0; i < n * 2; i++) _pol(c, math.max(.4, i.isEven ? ro : ri), rot + i * math.pi / n)];
  return _poly(pts, close: true);
}

Path _blob(Offset c, double r, {double k = 1, int seed = 0, double sx = 1}) {
  final pts = <Offset>[];
  for (var i = 0; i < 48; i++) {
    final a = i / 48 * math.pi * 2;
    final rr = math.max(.5, r * (1 + k * (.16 * math.sin(3 * a + seed) + .09 * math.cos(5 * a + 1 + seed))));
    pts.add(c + Offset(math.cos(a) * rr * sx, math.sin(a) * rr));
  }
  return _poly(pts, close: true);
}

Path _scaled(Path p, Offset o, double k) => p.transform((Matrix4.identity()
      ..translateByDouble(o.dx, o.dy, 0, 1)
      ..scaleByDouble(k, k, 1, 1)
      ..translateByDouble(-o.dx, -o.dy, 0, 1))
    .storage);

Offset _iso(Offset o, double x, double y, double zz, double u) => o + Offset((x - y) * u * .866, (x + y) * u * .5 - zz * u);

final _tp = <String, TextPainter>{};

/// Text anchored at its top-left (or top-right with [right], top-centre with [mid]); painters are cached.
void _t(Canvas c, String s, double x, double y, Color col, {double size = 7.5, FontWeight w = FontWeight.w500, double ls = .9, bool right = false, bool mid = false, bool outline = false}) {
  final key = '$s|$size|${col.toARGB32()}|${w.value}|$ls|$outline';
  var tp = _tp[key];
  if (tp == null) {
    if (_tp.length > 900) _tp.clear();
    tp = TextPainter(
      text: TextSpan(
        text: s,
        style: TextStyle(
          fontFamily: _ff,
          fontFamilyFallback: _fb,
          fontSize: size,
          fontWeight: w,
          fontVariations: [FontVariation.weight(w.value.toDouble())],
          letterSpacing: ls,
          height: 1,
          color: outline ? null : col,
          foreground: outline ? _st(col, .9) : null,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    _tp[key] = tp;
  }
  tp.paint(c, Offset(right ? x - tp.width : (mid ? x - tp.width / 2 : x), y));
}

/// A large thin numeral: the OP-1 readout.
void _num(Canvas c, String s, double x, double y, Color col, [double size = 30, bool right = false]) => _t(c, s, x, y, col, size: size, w: FontWeight.w200, ls: -.5, right: right);

// ---------- path tools ----------

Path _trim(Path p, double a, double b) {
  final out = Path();
  for (final m in p.computeMetrics()) {
    final s = m.length * a.clamp(0.0, 1.0), e = m.length * b.clamp(0.0, 1.0);
    if (e > s) out.addPath(m.extractPath(s, e), Offset.zero);
  }
  return out;
}

ui.Tangent? _at(Path p, double f) {
  for (final m in p.computeMetrics()) {
    return m.getTangentForOffset(m.length * f.clamp(0.0, 1.0));
  }
  return null;
}

void _dash(Canvas c, Path p, Color col, double on, double off, {double w = 1.2, double phase = 0}) {
  final pt = _st(col, w);
  on = math.max(.6, on);
  off = math.max(.6, off);
  final per = on + off;
  for (final m in p.computeMetrics()) {
    var d = -((phase % per) + per) % per;
    while (d < m.length) {
      final s = math.max(0.0, d), e = math.min(m.length, d + on);
      if (e > s) c.drawPath(m.extractPath(s, e), pt);
      d += per;
    }
  }
}

/// The outline of a stroke of width [w] along [p], drawn as two monolines and round caps — width without a fill.
void _ribbon(Canvas c, Path p, double w, Color col, {double a = 0, double b = 1, double lw = 1.1, double Function(double u)? taper, bool caps = true}) {
  for (final m in p.computeMetrics()) {
    final s0 = m.length * a.clamp(0.0, 1.0), s1 = m.length * b.clamp(0.0, 1.0);
    if (s1 - s0 < 1) continue;
    final l = <Offset>[], r = <Offset>[];
    final n = math.max(2, ((s1 - s0) / 2.5).ceil());
    for (var i = 0; i <= n; i++) {
      final d = s0 + (s1 - s0) * i / n;
      final tg = m.getTangentForOffset(d)!;
      final nv = Offset(-tg.vector.dy, tg.vector.dx);
      final hw = w / 2 * (taper?.call(d / m.length) ?? 1);
      l.add(tg.position + nv * hw);
      r.add(tg.position - nv * hw);
    }
    final pt = _st(col, lw);
    c.drawPath(_poly(l), pt);
    c.drawPath(_poly(r), pt);
    if (caps) {
      for (final (q, k) in [(0, -1.0), (n, 1.0)]) {
        final mid = (l[q] + r[q]) / 2, rad = (l[q] - r[q]).distance / 2;
        if (rad < .6) continue;
        final ang = math.atan2(l[q].dy - mid.dy, l[q].dx - mid.dx);
        c.drawArc(Rect.fromCircle(center: mid, radius: rad), ang, k * math.pi, false, pt);
      }
    }
  }
}
