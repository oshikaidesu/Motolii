part of 'op_01.dart';

const _ground = Color(0xFF0B0B0C), _frame = Color(0xFF222222), _tagC = Color(0xFF8A8A90);
const _blue = Color(0xFF4F6CFF), _green = Color(0xFF23E0A3), _white = Color(0xFFE8E8EA), _red = Color(0xFFFF3363);
const _purple = Color(0xFF9B7BFF), _dim = Color(0xFF3A3A40);
const _pw = 156.0, _ph = 120.0;

enum _G { drag, flick, rub, spin, pinch, draw }

typedef _Fn = void Function(Canvas c, _P p, double t);

class _S {
  const _S(this.name, this.tags, this.g, this.fn, {this.a = .45, this.b = .5, this.c = .25});
  final String name, tags;
  final _G g;
  final _Fn fn;
  final double a, b, c;
}

/// One panel's values: a / b from the gesture, c steps on double-tap. pts = a drawn path, v = flick momentum, ang = accumulated spin.
class _P extends ChangeNotifier {
  _P(this.a, this.b, this.c);
  double a, b, c, v = 0, ang = 0;
  Offset? touch;
  final pts = <Offset>[];
  void poke() => notifyListeners();
}

double _cl(double v) => v.clamp(0.0, 1.0).toDouble();

class _Sheet extends StatefulWidget {
  const _Sheet({required this.concept, required this.legend, required this.specs});
  final String concept;
  final List<(String, Color)> legend;
  final List<_S> specs;
  @override
  State<_Sheet> createState() => _SheetState();
}

class _SheetState extends State<_Sheet> with SingleTickerProviderStateMixin {
  final _t = ValueNotifier<double>(0);
  late final Ticker _tk;

  @override
  void initState() {
    super.initState();
    // 30 fps is enough for line drawings that breathe.
    _tk = createTicker((e) {
      final s = e.inMicroseconds / 1e6;
      if (s - _t.value >= 1 / 30) _t.value = s;
    })
      ..start();
  }

  @override
  void dispose() {
    _tk.dispose();
    _t.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final cells = <Widget>[];
    for (var i = 0; i < widget.specs.length; i++) {
      final s = widget.specs[i];
      final tags = s.tags.split('|');
      cells.add(SizedBox(
        width: _pw,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          _Panel(spec: s, t: _t),
          const SizedBox(height: 3),
          Text('${widget.concept.toLowerCase()} #${i + 1}  ${s.name}',
              maxLines: 1, overflow: TextOverflow.clip, style: _ts(9.5, _white, FontWeight.w500, .2)),
          Text(tags.join(' · '), maxLines: 1, overflow: TextOverflow.clip, style: _ts(8.2, _tagC, FontWeight.w400, .3)),
        ]),
      ));
    }
    return ColoredBox(
      color: const Color(0xFF000000),
      child: Align(
        alignment: Alignment.topLeft,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(12, 10, 12, 10),
          child: SizedBox(
            width: _pw * 5 + 8 * 4,
            child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
              Row(children: [
                Text('OP 01  ${widget.concept} x20', style: _ts(11, _white, FontWeight.w500, 1.2)),
                const Spacer(),
                for (final (n, c) in widget.legend) ...[
                  Container(width: 6, height: 6, decoration: BoxDecoration(color: c, shape: BoxShape.circle)),
                  const SizedBox(width: 4),
                  Text(n, style: _ts(8.5, c, FontWeight.w500, 1)),
                  const SizedBox(width: 12),
                ],
                Text('DOUBLE-TAP = 3RD', style: _ts(8.5, _tagC, FontWeight.w400, 1)),
              ]),
              const SizedBox(height: 8),
              Wrap(spacing: 8, runSpacing: 8, children: cells),
            ]),
          ),
        ),
      ),
    );
  }
}

TextStyle _ts(double size, Color c, FontWeight w, double ls) =>
    TextStyle(fontFamily: 'Roboto', fontFamilyFallback: const ['Helvetica Neue'], fontSize: size, color: c, fontWeight: w, letterSpacing: ls, height: 1.15);

class _Panel extends StatefulWidget {
  const _Panel({required this.spec, required this.t});
  final _S spec;
  final ValueNotifier<double> t;
  @override
  State<_Panel> createState() => _PanelState();
}

class _PanelState extends State<_Panel> {
  late final _P p = _P(widget.spec.a, widget.spec.b, widget.spec.c);
  double _last = 0, _sc = 1;
  Offset? _prev;

  @override
  void initState() {
    super.initState();
    widget.t.addListener(_tick);
  }

  void _tick() {
    final now = widget.t.value, dt = (now - _last).clamp(0.0, .1);
    _last = now;
    if (p.v.abs() > .002 && p.touch == null) {
      p.a += p.v * dt;
      if (p.a < 0 || p.a > 1) p.v = -p.v * .6;
      p.a = _cl(p.a);
      p.v *= .93;
      p.poke();
    }
  }

  @override
  void dispose() {
    widget.t.removeListener(_tick);
    p.dispose();
    super.dispose();
  }

  void _start(ScaleStartDetails d) {
    p.touch = d.localFocalPoint;
    _prev = d.localFocalPoint;
    _sc = 1;
    p.v = 0;
    if (widget.spec.g == _G.draw) p.pts.clear();
    p.poke();
  }

  void _update(ScaleUpdateDetails d) {
    final pos = d.localFocalPoint, dl = d.focalPointDelta;
    p.touch = pos;
    switch (widget.spec.g) {
      case _G.drag:
        p.a = _cl(p.a + dl.dx / _pw);
        p.b = _cl(p.b - dl.dy / _ph);
      case _G.flick:
        p.a = _cl(p.a + dl.dx / _pw);
        p.b = _cl(p.b - dl.dy / _ph);
      case _G.rub:
        p.a = _cl(p.a + dl.distance / _pw * .18);
        p.b = _cl(1 - pos.dy / _ph);
      case _G.spin:
        const o = Offset(_pw / 2, _ph / 2);
        final a0 = ((_prev ?? pos) - o).direction, a1 = (pos - o).direction;
        var da = a1 - a0;
        if (da > math.pi) da -= 2 * math.pi;
        if (da < -math.pi) da += 2 * math.pi;
        p.ang += da;
        p.a = _cl(p.a + da / (2 * math.pi));
        p.b = _cl((pos - o).distance / 60);
      case _G.pinch:
        if (d.pointerCount >= 2) {
          p.a = _cl(p.a * (d.scale / _sc));
          _sc = d.scale;
        } else {
          p.a = _cl(p.a - dl.dy / _ph);
          p.b = _cl(p.b + dl.dx / _pw);
        }
      case _G.draw:
        if (p.pts.length < 80 && (p.pts.isEmpty || (p.pts.last - pos).distance > 3)) p.pts.add(pos);
        p.a = _cl(pos.dx / _pw);
        p.b = _cl(1 - pos.dy / _ph);
    }
    _prev = pos;
    p.poke();
  }

  void _end(ScaleEndDetails d) {
    if (widget.spec.g == _G.flick) p.v = d.velocity.pixelsPerSecond.dx / _pw * .9;
    p.touch = null;
    p.poke();
  }

  @override
  Widget build(BuildContext context) => GestureDetector(
        onScaleStart: _start,
        onScaleUpdate: _update,
        onScaleEnd: _end,
        onDoubleTap: () {
          p.c = (p.c + .25) % 1.0;
          p.poke();
        },
        child: RepaintBoundary(
          child: CustomPaint(size: const Size(_pw, _ph), painter: _Pp(widget.spec.fn, p, widget.t)),
        ),
      );
}

class _Pp extends CustomPainter {
  _Pp(this.fn, this.p, this.t) : super(repaint: Listenable.merge([p, t]));
  final _Fn fn;
  final _P p;
  final ValueNotifier<double> t;
  @override
  void paint(Canvas c, Size s) {
    final r = Offset.zero & s;
    c.drawRect(r, Paint()..color = _ground);
    c.save();
    c.clipRect(r);
    fn(c, p, t.value);
    c.restore();
    c.drawRect(r.deflate(.5), _s(_frame, 1));
  }

  @override
  bool shouldRepaint(_Pp o) => o.fn != fn || o.p != p;
}

// ---- drawing helpers ------------------------------------------------------------------------------

Paint _s(Color c, [double w = 1.2]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;

Paint _f(Color c) => Paint()..color = c;

void _ln(Canvas c, Offset a, Offset b, Color col, [double w = 1.2]) => c.drawLine(a, b, _s(col, w));

void _dot(Canvas c, Offset o, [double r = 1.6, Color col = _white]) => c.drawCircle(o, r, _f(col));

void _ring(Canvas c, Offset o, double r, Color col, [double w = 1.1]) => c.drawCircle(o, r, _s(col, w));

void _pl(Canvas c, List<Offset> pts, Color col, [double w = 1.2, bool close = false]) {
  if (pts.length < 2) return;
  final path = Path()..moveTo(pts.first.dx, pts.first.dy);
  for (final q in pts.skip(1)) {
    path.lineTo(q.dx, q.dy);
  }
  if (close) path.close();
  c.drawPath(path, _s(col, w));
}

void _dash(Canvas c, Offset a, Offset b, Color col, {double on = 2, double off = 3, double w = 1}) {
  final d = (b - a).distance;
  if (d < .1) return;
  final u = (b - a) / d;
  for (var x = 0.0; x < d; x += on + off) {
    c.drawLine(a + u * x, a + u * math.min(x + on, d), _s(col, w));
  }
}

void _dotted(Canvas c, Offset a, Offset b, Color col, [double gap = 3]) {
  final d = (b - a).distance;
  final n = math.max(1, (d / gap).floor());
  for (var i = 0; i <= n; i++) {
    c.drawCircle(Offset.lerp(a, b, i / n)!, .6, _f(col));
  }
}

Color _a(Color c, double o) => c.withValues(alpha: o.clamp(0.0, 1.0).toDouble());

/// Deterministic hash in 0..1.
double _h(int i, [int s = 0]) {
  var x = (i * 374761393 + s * 668265263) & 0x7fffffff;
  x = ((x ^ (x >> 13)) * 1274126177) & 0x7fffffff;
  return (x & 0xffff) / 0xffff;
}

/// Isometric projection: x right-down, y left-down, z up.
Offset _iso(Offset o, double x, double y, double z) => o + Offset((x - y) * .866, (x + y) * .5 - z);

void _isoBox(Canvas c, Offset o, double x, double y, double z, double w, double d, double h, Color col, [double lw = 1]) {
  Offset q(double a, double b, double e) => _iso(o, x + a, y + b, z + e);
  final pts = [
    [q(0, 0, 0), q(w, 0, 0)], [q(w, 0, 0), q(w, d, 0)], [q(w, d, 0), q(0, d, 0)], [q(0, d, 0), q(0, 0, 0)],
    [q(0, 0, h), q(w, 0, h)], [q(w, 0, h), q(w, d, h)], [q(w, d, h), q(0, d, h)], [q(0, d, h), q(0, 0, h)],
    [q(0, 0, 0), q(0, 0, h)], [q(w, 0, 0), q(w, 0, h)], [q(w, d, 0), q(w, d, h)], [q(0, d, 0), q(0, d, h)],
  ];
  for (final e in pts) {
    _ln(c, e[0], e[1], col, lw);
  }
}

void _isoGrid(Canvas c, Offset o, int n, double cell, Color col) {
  for (var i = 0; i <= n; i++) {
    _ln(c, _iso(o, i * cell, 0, 0), _iso(o, i * cell, n * cell, 0), col, .8);
    _ln(c, _iso(o, 0, i * cell, 0), _iso(o, n * cell, i * cell, 0), col, .8);
  }
}

/// Stagger helper: progress 0..1 of item at order k of n, with the cascade always caught mid-way (half done) plus a slow sway.
double _cas(int k, int n, double off, double t, [double dur = 1]) {
  final head = (n - 1) * off * .5 + dur * .5 + math.sin(t * .9) * (n * off * .12 + .05);
  return ((head - k * off) / dur).clamp(0.0, 1.0).toDouble();
}

int _ord(int i, int n, double order) {
  // order steps: 0 linear, .25 centre-out, .5 edges-in, .75 shuffled
  final m = (order * 4).floor() % 4;
  final mid = (n - 1) / 2;
  switch (m) {
    case 1:
      return ((i - mid).abs() * 2).round();
    case 2:
      return ((mid - (i - mid).abs()) * 2).round();
    case 3:
      return (_h(i, 9) * n).floor();
    default:
      return i;
  }
}

String _ordName(double order) => const ['LIN', 'CTR', 'EDGE', 'RND'][(order * 4).floor() % 4];

String _nn(double v) => (v * 99).round().toString().padLeft(2, '0');

double _ease(double x) => 1 - math.pow(1 - x, 3).toDouble();

final _tpCache = <String, TextPainter>{};

TextPainter _tp(String s, double size, Color col, FontWeight w, double ls, bool outline) {
  final k = '$s|$size|${col.toARGB32()}|${w.value}|$ls|$outline';
  final hit = _tpCache[k];
  if (hit != null) return hit;
  if (_tpCache.length > 1500) _tpCache.clear();
  final st = TextStyle(
    fontFamily: 'Roboto',
    fontFamilyFallback: const ['Helvetica Neue'],
    fontSize: size,
    fontWeight: w,
    letterSpacing: ls,
    height: 1,
    color: outline ? null : col,
    foreground: outline ? (_s(col, .9)) : null,
  );
  final tp = TextPainter(text: TextSpan(text: s, style: st), textDirection: TextDirection.ltr)..layout();
  return _tpCache[k] = tp;
}

/// Tiny uppercase label. ax: 0 left, .5 centre, 1 right.
void _lb(Canvas c, String s, Offset o, [Color col = _white, double size = 7.5, double ax = 0]) {
  final tp = _tp(s, size, col, FontWeight.w500, .7, false);
  tp.paint(c, o - Offset(tp.width * ax, 0));
}

/// Large thin numeral.
void _num(Canvas c, String s, Offset o, double size, Color col, [double ax = 0]) {
  final tp = _tp(s, size, col, FontWeight.w100, -1, false);
  tp.paint(c, o - Offset(tp.width * ax, 0));
}

/// Outlined text (neon tubes).
Size _outline(Canvas c, String s, Offset o, double size, Color col, [double ax = 0]) {
  final tp = _tp(s, size, col, FontWeight.w300, 1, true);
  tp.paint(c, o - Offset(tp.width * ax, 0));
  return tp.size;
}

/// A stick figure standing at foot point f, height h. arm: 0 down .. 1 straight up.
void _stick(Canvas c, Offset f, double h, Color col, {double arm = 0, double lean = 0, double step = 0}) {
  final hip = f + Offset(lean * h * .2, -h * .45), neck = hip + Offset(lean * h * .25, -h * .35), head = neck + Offset(0, -h * .1);
  _ring(c, head, h * .1, col, 1);
  _ln(c, hip, neck, col, 1);
  _ln(c, hip, f + Offset(-h * .12 + step * h * .1, 0), col, 1);
  _ln(c, hip, f + Offset(h * .12 - step * h * .1, 0), col, 1);
  for (final sgn in [-1.0, 1.0]) {
    final ang = math.pi / 2 + sgn * (.25 + arm * 2.6);
    _ln(c, neck + Offset(0, h * .04), neck + Offset(0, h * .04) + Offset(math.cos(ang), math.sin(ang)) * h * .32, col, 1);
  }
}
