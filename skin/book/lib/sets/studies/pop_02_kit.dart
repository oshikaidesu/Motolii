part of 'pop_02.dart';

const _k1 = Color(0xFF1C1C1E), _k2 = Color(0xFF2A2A2E), _cr = Color(0xFFF4EBDD), _wh = Color(0xFFFFFFFF);
const _or = Color(0xFFFF7A3D), _ye = Color(0xFFFFD23F), _cy = Color(0xFF3DD6F5), _li = Color(0xFF9BE564);
const _pi = Color(0xFFFF5DA2), _vi = Color(0xFF9B7BFF), _re = Color(0xFFFF4B4B), _bl = Color(0xFF2E5BFF);
const _pop = [_or, _ye, _cy, _li, _pi, _vi, _re, _bl];

/// Every painter draws in this fixed 156 x 120 space.
const _pw = 156.0, _ph = 120.0, _pc = Offset(78, 60);
const _full = Rect.fromLTWH(0, 0, _pw, _ph);

Paint _f(Color c) => Paint()..color = c;
Paint _s(Color c, [double w = 1.5]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Paint _bf(Color c, double sigma) => Paint()
  ..color = c
  ..maskFilter = sigma > .25 ? MaskFilter.blur(BlurStyle.normal, sigma) : null;
Color _al(Color c, double o) => c.withValues(alpha: (c.a * o).clamp(0.0, 1.0));
Color _mix(Color a, Color b, double t) => Color.lerp(a, b, t.clamp(0.0, 1.0))!;
Color _dk(Color c, double t) => _mix(c, const Color(0xFF000000), t);
double _cl(double x) => x.clamp(0.0, 1.0).toDouble();
double _h(int i) {
  final x = math.sin(i * 12.9898 + 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

/// 1 at the centre, 0 at r; k < 1 holds then drops, k > 1 peaks.
double _fall(double d, double r, double k) => d >= r ? 0 : math.pow(1 - d / r, k).toDouble();
double _k(double b) => math.pow(4, (b - .5) * 2).toDouble();
Offset _pol(double a, double r) => Offset(math.cos(a) * r, math.sin(a) * r);
Offset _nrm(Offset o) => o.distance < 1e-6 ? const Offset(1, 0) : o / o.distance;

Path _poly(List<Offset> pts) => Path()..addPolygon(pts, true);
Path _star(Offset c, double r, int n, [double inner = .45, double rot = -math.pi / 2]) => _poly([
      for (var i = 0; i < n * 2; i++) c + _pol(rot + i * math.pi / n, i.isEven ? r : r * inner),
    ]);

/// Local state of one panel: a / b are drag accumulators (right = a up, up = b up), p the last touch, trail the painted path (null = pen up),
/// dir the smoothed drag direction, ang / w a spin around the panel centre that keeps going after a flick.
class _St extends ChangeNotifier {
  _St(this.a, this.b, this.p, List<Offset> seed) {
    trail.addAll(seed);
    p0 = p;
  }
  double a, b;
  Offset p, p0 = Offset.zero, pull = Offset.zero;
  bool down = false;
  final trail = <Offset?>[];
  Offset dir = const Offset(1, 0);
  double ang = 0, w = 0, now = 0, tUp = -99;

  Offset _clip(Offset q) => Offset(q.dx.clamp(0.0, _pw), q.dy.clamp(0.0, _ph));

  void start(Offset q) {
    down = true;
    p = p0 = _clip(q);
    if (trail.isNotEmpty) trail.add(null);
    trail.add(p);
    notifyListeners();
  }

  void move(Offset q, Offset d) {
    a = _cl(a + d.dx / 110);
    b = _cl(b - d.dy / 90);
    p = _clip(q);
    pull = p - p0;
    trail.add(p);
    while (trail.length > 260) {
      trail.removeAt(0);
    }
    if (d.distance > .3) dir = _nrm(dir * .75 + _nrm(d) * .25);
    final r0 = q - d - _pc, r1 = q - _pc;
    if (r0.distance > 6 && r1.distance > 6) {
      var da = r1.direction - r0.direction;
      if (da > math.pi) da -= 2 * math.pi;
      if (da < -math.pi) da += 2 * math.pi;
      ang += da;
      w = w * .5 + da * 30;
    }
    notifyListeners();
  }

  void end() {
    down = false;
    tUp = now;
    notifyListeners();
  }

  void step(double dt) {
    now += dt;
    if (!down && w.abs() > .01) {
      ang += w * dt;
      w *= math.exp(-dt * .9);
    }
  }

  /// Trail points with pen-ups dropped, newest last.
  Iterable<Offset> get pts => trail.whereType<Offset>();
}

typedef _Paint = void Function(Canvas c, _St s, double t);

class _Pn {
  const _Pn(this.name, this.tags, this.paint, {this.bg = _k1, this.a = .5, this.b = .5, this.p = _pc, this.seed = const []});
  final String name, tags;
  final _Paint paint;
  final Color bg;
  final double a, b;
  final Offset p;
  final List<Offset> seed;
}

class _Sheet extends StatefulWidget {
  const _Sheet({required this.concept, required this.panels});
  final String concept;
  final List<_Pn> panels;
  @override
  State<_Sheet> createState() => _SheetState();
}

class _SheetState extends State<_Sheet> with SingleTickerProviderStateMixin {
  late final Ticker _tk;
  final _t = ValueNotifier<double>(0);
  late final List<_St> _st = [for (final p in widget.panels) _St(p.a, p.b, p.p, p.seed)];
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tk = createTicker((e) {
      final dt = ((e - _last).inMicroseconds / 1e6).clamp(0.0, .05);
      _last = e;
      for (final s in _st) {
        s.step(dt);
      }
      _t.value = e.inMicroseconds / 1e6;
    })
      ..start();
  }

  @override
  void dispose() {
    _tk.dispose();
    _t.dispose();
    for (final s in _st) {
      s.dispose();
    }
    super.dispose();
  }

  Widget _cell(int i) {
    final pn = widget.panels[i], s = _st[i];
    return SizedBox(
      width: 156,
      height: 150,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        ClipRRect(
          borderRadius: BorderRadius.circular(8),
          child: GestureDetector(
            onPanStart: (d) => s.start(d.localPosition),
            onPanUpdate: (d) => s.move(d.localPosition, d.delta),
            onPanEnd: (_) => s.end(),
            onPanCancel: s.end,
            child: RepaintBoundary(child: CustomPaint(size: const Size(_pw, _ph), painter: _Pa(pn, s, _t))),
          ),
        ),
        const SizedBox(height: 4),
        Text('${widget.concept} #${i + 1}  ${pn.name}',
            maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 10, height: 1.25, color: _cr)),
        Text(pn.tags,
            maxLines: 1, overflow: TextOverflow.ellipsis, style: TextStyle(fontSize: 10, height: 1.25, color: _al(_cr, .5))),
      ]),
    );
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: const Color(0xFF111113),
        child: Align(
          alignment: Alignment.topLeft,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              for (var r = 0; r < 4; r++)
                Padding(
                  padding: EdgeInsets.only(bottom: r < 3 ? 8 : 0),
                  child: Row(mainAxisSize: MainAxisSize.min, children: [
                    for (var col = 0; col < 5; col++) ...[
                      if (col > 0) const SizedBox(width: 8),
                      if (r * 5 + col < widget.panels.length) _cell(r * 5 + col) else const SizedBox(width: 156),
                    ],
                  ]),
                ),
            ]),
          ),
        ),
      );
}

class _Pa extends CustomPainter {
  _Pa(this.pn, this.s, this.t) : super(repaint: Listenable.merge([t, s]));
  final _Pn pn;
  final _St s;
  final ValueNotifier<double> t;

  @override
  void paint(Canvas c, Size size) {
    c.save();
    c.clipRect(Offset.zero & size);
    c.scale(size.width / _pw, size.height / _ph);
    c.drawRect(_full, _f(pn.bg));
    pn.paint(c, s, t.value);
    c.restore();
  }

  @override
  bool shouldRepaint(_Pa o) => o.pn != pn || o.s != s;
}
