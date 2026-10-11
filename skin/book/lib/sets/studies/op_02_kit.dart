part of 'op_02.dart';

// OP-1 screen language: black ground, monoline drawing, four encoder colours (blue, green, white, red) + purple and a dim guide grey.
const _bg = Color(0xFF0B0B0C), _frame = Color(0xFF222222);
const _cb = Color(0xFF4F6CFF), _cg = Color(0xFF23E0A3), _cw = Color(0xFFE8E8EA), _cr = Color(0xFFFF3363);
const _cp = Color(0xFF9B7BFF), _cd = Color(0xFF3A3A40);

/// Every painter draws in this fixed 156 x 120 space.
const _pw = 156.0, _ph = 120.0, _pc = Offset(78, 60);

/// Unknown family falls back to the platform sans in the app; a render check can load a face under this name.
const op02FontFamily = 'Op02';

Paint _sp(Color c, [double w = 1.2]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Paint _fp(Color c) => Paint()..color = c;
Color _al(Color c, double o) => c.withValues(alpha: (c.a * o).clamp(0.0, 1.0));
Color _mx(Color a, Color b, double t) => Color.lerp(a, b, t.clamp(0.0, 1.0))!;
double _cl(double x) => x.clamp(0.0, 1.0).toDouble();
double _h(int i) {
  final x = math.sin(i * 12.9898 + 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

/// 1 at the centre, 0 at r; k < 1 holds then drops, k > 1 peaks.
double _fall(double d, double r, double k) => d >= r ? 0 : math.pow(1 - d / r, k).toDouble();

/// b in 0..1 to a curve exponent 0.25..4.
double _kx(double b) => math.pow(4, (b - .5) * 2).toDouble();
Offset _pol(double a, double r) => Offset(math.cos(a) * r, math.sin(a) * r);
String _n2(double v) => v.round().clamp(0, 99).toString().padLeft(2, '0');

void _ln(Canvas c, Offset a, Offset b, Color col, [double w = 1.2]) => c.drawLine(a, b, _sp(col, w));
void _ci(Canvas c, Offset p, double r, Color col, [double w = 1.2]) => c.drawCircle(p, r, _sp(col, w));
void _dt(Canvas c, Offset p, double r, Color col) => c.drawCircle(p, r, _fp(col));
void _pl(Canvas c, List<Offset> pts, Color col, {double w = 1.2, bool close = false}) {
  if (pts.length < 2) return;
  c.drawPath(Path()..addPolygon(pts, close), _sp(col, w));
}

void _rc(Canvas c, Rect r, Color col, [double w = 1.2]) => c.drawRect(r, _sp(col, w));
void _dash(Canvas c, Offset a, Offset b, Color col, {double on = 1.5, double off = 2.5, double w = 1}) {
  final d = (b - a).distance;
  if (d < .5) return;
  final u = (b - a) / d, p = _sp(col, w);
  for (var s = 0.0; s < d; s += on + off) {
    c.drawLine(a + u * s, a + u * math.min(d, s + on), p);
  }
}

/// Dotted guide made of single points, the OP-1 way of marking a vertical.
void _dots(Canvas c, Offset a, Offset b, Color col, [double gap = 3]) {
  final d = (b - a).distance;
  if (d < .5) return;
  final u = (b - a) / d, p = _fp(col);
  for (var s = 0.0; s <= d; s += gap) {
    c.drawCircle(a + u * s, .55, p);
  }
}

final _tcache = <String, TextPainter>{};
TextPainter _tp(String s, double size, Color col, double wght, double ls) {
  final cq = col.withAlpha(((col.a * 16).round() * 16).clamp(0, 255));
  final k = '$s|$size|${cq.toARGB32()}|$wght|$ls';
  var t = _tcache[k];
  if (t == null) {
    if (_tcache.length > 900) _tcache.clear();
    final fw = FontWeight.values[((wght / 100).round() - 1).clamp(0, 8)];
    t = TextPainter(
      text: TextSpan(
          text: s,
          style: TextStyle(
              fontFamily: op02FontFamily,
              fontSize: size,
              color: cq,
              fontWeight: fw,
              fontVariations: [ui.FontVariation('wght', wght)],
              letterSpacing: ls,
              height: 1)),
      textDirection: TextDirection.ltr,
    )..layout();
    _tcache[k] = t;
  }
  return t;
}

/// ax / ay anchor: 0 left/top, .5 centre, 1 right/bottom.
void _tx(Canvas c, String s, Offset p, double size, Color col, {double wght = 500, double ls = .4, double ax = 0, double ay = 0}) {
  final t = _tp(s, size, col, wght, ls);
  t.paint(c, p - Offset(t.width * ax, t.height * ay));
}

/// Tiny uppercase encoder label.
void _lab(Canvas c, String s, Offset p, Color col, {double ax = 0, double ay = 0}) =>
    _tx(c, s, p, 7, col, wght: 600, ls: .9, ax: ax, ay: ay);

/// Large thin numeral readout.
void _num(Canvas c, String s, Offset p, double size, Color col, {double ax = 0, double ay = 0}) =>
    _tx(c, s, p, size, col, wght: 100, ls: -.3, ax: ax, ay: ay);

/// Isometric projection: x to the lower right, y to the lower left, z up.
Offset _iso(Offset o, double x, double y, double z) => o + Offset((x - y) * .866, (x + y) * .5 - z);
void _cube(Canvas c, Offset o, double x, double y, double z, double sx, double sy, double sz, Color col, [double w = 1]) {
  Offset q(double a, double b, double e) => _iso(o, x + a * sx, y + b * sy, z + e * sz);
  final p = _sp(col, w);
  final top = Path()..addPolygon([q(0, 0, 1), q(1, 0, 1), q(1, 1, 1), q(0, 1, 1)], true);
  c.drawPath(top, p);
  for (final v in [q(1, 0, 0), q(1, 1, 0), q(0, 1, 0)]) {
    c.drawLine(v, v - Offset(0, sz), p);
  }
  c.drawPath(Path()..addPolygon([q(1, 0, 0), q(1, 1, 0), q(0, 1, 0)], false), p);
}

/// Panel input. a / b: drag (right = a up, up = b up). z: pinch (a mouse drags it vertically). spin: angle swept around the
/// panel centre, keeps turning after a flick. rub: grows with scrubbing, fades. fx / fy: a position you can throw; it slides
/// and bounces. kick: impulse of the last release, fades. ink: the stroke being drawn.
class _St extends ChangeNotifier {
  _St(this.a, this.b, this.z, this.fx, this.fy);
  double a, b, z, fx, fy;
  Offset p = _pc;
  bool down = false;
  double spin = 0, w = 0, rub = 0, kick = 0, now = 0, lastScale = 1;
  Offset v = Offset.zero, lastD = Offset.zero;
  List<Offset> ink = [];

  Offset _clip(Offset q) => Offset(q.dx.clamp(0.0, _pw), q.dy.clamp(0.0, _ph));

  void start(Offset q) {
    down = true;
    p = _clip(q);
    ink = [p];
    v = Offset.zero;
    w = 0;
    lastScale = 1;
    notifyListeners();
  }

  void move(Offset q, Offset d, double scale, int fingers) {
    a = _cl(a + d.dx / 120);
    b = _cl(b - d.dy / 90);
    if (fingers > 1) {
      z = _cl(z + (scale - lastScale) * .8);
      lastScale = scale;
    } else {
      z = _cl(z - d.dy / 90);
    }
    fx = _cl(fx + d.dx / _pw);
    fy = _cl(fy + d.dy / _ph);
    final r0 = p - _pc, np = _clip(q), r1 = np - _pc;
    if (r0.distance > 6 && r1.distance > 6) {
      var da = r1.direction - r0.direction;
      if (da > math.pi) da -= 2 * math.pi;
      if (da < -math.pi) da += 2 * math.pi;
      spin += da;
    }
    if (d.distance > .5 && lastD.distance > .5 && (d.dx * lastD.dx + d.dy * lastD.dy) < 0) rub = _cl(rub + .12);
    rub = _cl(rub + d.distance / 900);
    if (d.distance > .5) lastD = d;
    p = np;
    ink.add(p);
    if (ink.length > 220) ink.removeAt(0);
    notifyListeners();
  }

  void end(Offset vel) {
    down = false;
    v = Offset(vel.dx / _pw, vel.dy / _ph);
    kick = _cl(vel.distance / 1400);
    final r = p - _pc;
    if (r.distance > 8) w = ((r.dx * vel.dy - r.dy * vel.dx) / (r.distance * r.distance)).clamp(-14.0, 14.0);
    notifyListeners();
  }

  void step(double dt) {
    now += dt;
    if (down) return;
    if (w.abs() > .01) {
      spin += w * dt;
      w *= math.exp(-dt * 1.2);
    }
    if (v.distance > .01) {
      fx += v.dx * dt;
      fy += v.dy * dt;
      if (fx < 0 || fx > 1) v = Offset(-v.dx, v.dy);
      if (fy < 0 || fy > 1) v = Offset(v.dx, -v.dy);
      fx = _cl(fx);
      fy = _cl(fy);
      v *= math.exp(-dt * 1.6);
    }
    rub *= math.exp(-dt * .35);
    kick *= math.exp(-dt * .9);
  }
}

typedef _Draw = void Function(Canvas c, _St s, double t);

class _Pn {
  const _Pn(this.name, this.tags, this.draw, {this.a = .5, this.b = .5, this.z = .5, this.fx = .5, this.fy = .5});
  final String name, tags;
  final _Draw draw;
  final double a, b, z, fx, fy;
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
  late final List<_St> _st = [for (final p in widget.panels) _St(p.a, p.b, p.z, p.fx, p.fy)];
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
      width: _pw + 2,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        DecoratedBox(
          decoration: BoxDecoration(border: Border.all(color: _frame)),
          child: Padding(
            padding: const EdgeInsets.all(1),
            child: GestureDetector(
              onScaleStart: (d) => s.start(d.localFocalPoint),
              onScaleUpdate: (d) => s.move(d.localFocalPoint, d.focalPointDelta, d.scale, d.pointerCount),
              onScaleEnd: (d) => s.end(d.velocity.pixelsPerSecond),
              child: RepaintBoundary(child: CustomPaint(size: const Size(_pw, _ph), painter: _Pa(pn, s, _t))),
            ),
          ),
        ),
        const SizedBox(height: 4),
        Text('${widget.concept} #${i + 1}  ${pn.name}',
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(fontFamily: op02FontFamily, fontSize: 9.5, height: 1.25, color: _cw)),
        Text(pn.tags,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(fontFamily: op02FontFamily, fontSize: 8.5, height: 1.25, color: Color(0xFF7A7A82))),
      ]),
    );
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: const Color(0xFF000000),
        child: Align(
          alignment: Alignment.topLeft,
          child: FittedBox(
            fit: BoxFit.scaleDown,
            alignment: Alignment.topLeft,
            child: Padding(
              padding: const EdgeInsets.all(10),
              child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                for (var r = 0; r < 4; r++)
                  Padding(
                    padding: EdgeInsets.only(bottom: r < 3 ? 8 : 0),
                    child: Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                      for (var col = 0; col < 5; col++) ...[
                        if (col > 0) const SizedBox(width: 8),
                        if (r * 5 + col < widget.panels.length) _cell(r * 5 + col) else const SizedBox(width: _pw + 2),
                      ],
                    ]),
                  ),
              ]),
            ),
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
    c.drawRect(Offset.zero & size, _fp(_bg));
    pn.draw(c, s, t.value);
    c.restore();
  }

  @override
  bool shouldRepaint(_Pa o) => o.pn != pn || o.s != s;
}
