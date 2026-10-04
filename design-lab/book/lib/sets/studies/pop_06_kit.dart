part of 'pop_06.dart';

const _p0 = Color(0xFF1C1C1E), _p1 = Color(0xFF2A2A2E), _cr = Color(0xFFF4EBDD), _wh = Color(0xFFFFFFFF);
const _or = Color(0xFFFF7A3D), _ye = Color(0xFFFFD23F), _cy = Color(0xFF3DD6F5), _li = Color(0xFF9BE564);
const _pk = Color(0xFFFF5DA2), _vi = Color(0xFF9B7BFF), _rd = Color(0xFFFF4B4B), _bl = Color(0xFF2E5BFF);

/// Local state of one panel: v (drag right), w (drag up), inv (tap), the pointer and its trail, t (sheet clock).
class PopS {
  PopS(this.v, this.w);
  double v, w, t = 0;
  bool inv = false;
  Offset? p;
  final trail = <Offset>[];
}

typedef PopPaint = void Function(Canvas c, Size s, PopS st);

class PopPanel {
  const PopPanel(this.name, this.tags, this.paint, {this.v0 = .5, this.w0 = .5, this.light = false});
  final String name, tags;
  final PopPaint paint;
  final double v0, w0;
  final bool light;
}

class Pop06Sheet extends StatefulWidget {
  const Pop06Sheet({super.key, required this.concept, required this.panels});
  final String concept;
  final List<PopPanel> panels;
  @override
  State<Pop06Sheet> createState() => _Pop06SheetState();
}

class _Pop06SheetState extends State<Pop06Sheet> with SingleTickerProviderStateMixin {
  final _time = ValueNotifier<double>(0);
  late final Ticker _ticker = createTicker((e) => _time.value = e.inMicroseconds / 1e6)..start();

  @override
  void initState() {
    super.initState();
    _ticker;
  }

  @override
  void dispose() {
    _ticker.dispose();
    _time.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: _p0,
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: 5 * 156 + 4 * 8,
              child: Wrap(spacing: 8, runSpacing: 8, children: [
                for (var i = 0; i < widget.panels.length; i++) _Cell(d: widget.panels[i], n: i + 1, concept: widget.concept, time: _time),
              ]),
            ),
          ),
        ),
      );
}

class _Cell extends StatefulWidget {
  const _Cell({required this.d, required this.n, required this.concept, required this.time});
  final PopPanel d;
  final int n;
  final String concept;
  final ValueNotifier<double> time;
  @override
  State<_Cell> createState() => _CellState();
}

class _CellState extends State<_Cell> {
  late final s = PopS(widget.d.v0, widget.d.w0);
  final _poke = ValueNotifier<int>(0);

  void _move(Offset p, Offset d) {
    s.v = (s.v + d.dx / 110).clamp(0.0, 1.0);
    s.w = (s.w - d.dy / 90).clamp(0.0, 1.0);
    s.p = p;
    s.trail.add(p);
    if (s.trail.length > 60) s.trail.removeAt(0);
    _poke.value++;
  }

  @override
  void dispose() {
    _poke.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    const t1 = TextStyle(fontSize: 10, height: 1.25, color: _cr, fontWeight: FontWeight.w600);
    final t2 = TextStyle(fontSize: 10, height: 1.25, color: _cr.withValues(alpha: .5));
    return SizedBox(
      width: 156,
      height: 150,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        GestureDetector(
          onPanStart: (e) {
            s.trail.clear();
            s.p = e.localPosition;
          },
          onPanUpdate: (e) => _move(e.localPosition, e.delta),
          onTap: () {
            s.inv = !s.inv;
            _poke.value++;
          },
          child: ClipRRect(
            borderRadius: BorderRadius.circular(10),
            child: CustomPaint(size: const Size(156, 120), painter: _CellPainter(widget.d, s, widget.time, Listenable.merge([widget.time, _poke]))),
          ),
        ),
        const SizedBox(height: 3),
        Text('${widget.concept} #${widget.n}  ${widget.d.name}', style: t1, maxLines: 1, overflow: TextOverflow.ellipsis),
        Text(widget.d.tags, style: t2, maxLines: 1, overflow: TextOverflow.ellipsis),
      ]),
    );
  }
}

class _CellPainter extends CustomPainter {
  _CellPainter(this.d, this.s, this.time, Listenable repaint) : super(repaint: repaint);
  final PopPanel d;
  final PopS s;
  final ValueNotifier<double> time;

  @override
  void paint(Canvas c, Size size) {
    s.t = time.value;
    c.drawRect(Offset.zero & size, _f(d.light ? _cr : _p1));
    d.paint(c, size, s);
  }

  @override
  bool shouldRepaint(covariant _CellPainter old) => true;
}

// ---------- drawing helpers ----------

Paint _f(Color c) => Paint()..color = c;
Paint _s(Color c, double w, {StrokeCap cap = StrokeCap.round}) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = cap
  ..strokeJoin = StrokeJoin.round;
Color _a(Color c, double a) => c.withValues(alpha: a.clamp(0.0, 1.0));
double _l(double a, double b, double t) => a + (b - a) * t;
double _h(int i) {
  final x = math.sin(i * 12.9898 + 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

Offset _pol(Offset c, double r, double a) => c + Offset(math.cos(a), math.sin(a)) * r;

Path _circle(Offset c, double r) => Path()..addOval(Rect.fromCircle(center: c, radius: math.max(.5, r)));

Path _star(Offset c, double ro, double ri, int n, [double rot = -math.pi / 2]) {
  final p = Path();
  for (var i = 0; i < n * 2; i++) {
    final q = _pol(c, i.isEven ? ro : ri, rot + i * math.pi / n);
    i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
  }
  return p..close();
}

Path _blob(Offset c, double r, {double k = 1, int seed = 0}) {
  final p = Path();
  for (var i = 0; i <= 48; i++) {
    final a = i / 48 * math.pi * 2;
    final rr = r * (1 + k * (.16 * math.sin(3 * a + seed) + .09 * math.cos(5 * a + 1 + seed)));
    final q = _pol(c, math.max(.5, rr), a);
    i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
  }
  return p..close();
}

Path _poly(List<Offset> pts, {bool close = false}) {
  final p = Path()..moveTo(pts.first.dx, pts.first.dy);
  for (final q in pts.skip(1)) {
    p.lineTo(q.dx, q.dy);
  }
  return close ? (p..close()) : p;
}

/// The sample picture every mask panel cuts: a vivid sunset over blue mountains and water.
const _mtn = [Offset(0, .7), Offset(.25, .45), Offset(.45, .68), Offset(.7, .5), Offset(1, .72)];

void _scene(Canvas c, Rect r) {
  c.drawRect(r, Paint()..shader = ui.Gradient.linear(r.topCenter, r.bottomCenter, const [_vi, _pk, _or], const [0, .55, 1]));
  c.drawCircle(Offset(r.left + r.width * .62, r.top + r.height * .42), r.height * .18, _f(_ye));
  final m = Path()..moveTo(r.left, r.bottom);
  for (final q in _mtn) {
    m.lineTo(r.left + q.dx * r.width, r.top + q.dy * r.height);
  }
  m
    ..lineTo(r.right, r.bottom)
    ..close();
  c.drawPath(m, _f(_bl));
  for (var i = 0; i < 3; i++) {
    final y = r.top + r.height * (.84 + i * .055);
    c.drawLine(Offset(r.left + r.width * (.1 + .12 * i), y), Offset(r.left + r.width * (.55 + .1 * i), y), _s(_cy, 2));
  }
}

Color _sceneAt(Offset p, Size s) {
  final x = p.dx / s.width, y = p.dy / s.height;
  if ((p - Offset(s.width * .62, s.height * .42)).distance < s.height * .18) return _ye;
  var my = _mtn.last.dy;
  for (var i = 0; i < _mtn.length - 1; i++) {
    final a = _mtn[i], b = _mtn[i + 1];
    if (x >= a.dx && x <= b.dx) my = _l(a.dy, b.dy, (x - a.dx) / (b.dx - a.dx));
  }
  if (y > my) return y > .84 && y < .96 && x > .1 && x < .75 ? _cy : _bl;
  return y < .55 ? Color.lerp(_vi, _pk, y / .55)! : Color.lerp(_pk, _or, (y - .55) / .45)!;
}

/// Paint [img] only where [mask] is (or is not, when [inv]).
void _masked(Canvas c, Rect r, void Function() img, void Function() mask, {bool inv = false}) {
  c.saveLayer(r, Paint());
  img();
  c.saveLayer(r, Paint()..blendMode = inv ? BlendMode.dstOut : BlendMode.dstIn);
  mask();
  c.restore();
  c.restore();
}

/// A stepped (terraced) soft edge: the shape grown from +soft/2 to -soft/2 in [n] crisp steps.
void _bands(Canvas c, Path Function(double g) shape, double soft, {int n = 5, Color col = _wh, BlendMode mode = BlendMode.srcOver}) {
  if (soft < .6) {
    c.drawPath(shape(0), _f(col)..blendMode = mode);
    return;
  }
  for (var i = 0; i < n; i++) {
    final g = soft / 2 - soft * i / (n - 1);
    c.drawPath(shape(g), _f(_a(col, i == n - 1 ? 1 : .32))..blendMode = mode);
  }
}

// ---------- path helpers (stroke sheet) ----------

Path _trim(Path p, double a, double b) {
  final out = Path();
  for (final m in p.computeMetrics()) {
    final s = m.length * a.clamp(0.0, 1.0), e = m.length * b.clamp(0.0, 1.0);
    if (e > s) out.addPath(m.extractPath(s, e), Offset.zero);
  }
  return out;
}

ui.Tangent? _at(Path p, double f) {
  final ms = p.computeMetrics().toList();
  if (ms.isEmpty) return null;
  final m = ms.first;
  return m.getTangentForOffset(m.length * f.clamp(0.0, 1.0));
}

void _dash(Canvas c, Path p, Paint pt, double on, double off, [double phase = 0]) {
  final per = on + off;
  for (final m in p.computeMetrics()) {
    for (var d = phase % per - per; d < m.length; d += per) {
      final a = math.max(0.0, d), b = math.min(m.length, d + on);
      if (b > a) c.drawPath(m.extractPath(a, b), pt);
    }
  }
}

void _rotRect(Canvas c, Offset at, double ang, Rect r, Paint p, {double rad = 2}) {
  c
    ..save()
    ..translate(at.dx, at.dy)
    ..rotate(ang);
  c.drawRRect(RRect.fromRectAndRadius(r, Radius.circular(rad)), p);
  c.restore();
}
