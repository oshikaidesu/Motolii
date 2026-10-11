part of 'op_03.dart';

const _kBg = Color(0xFF000000), _kSheet = Color(0xFF0B0B0C), _kFrame = Color(0xFF222222);
const _kBlue = Color(0xFF4F6CFF), _kGreen = Color(0xFF23E0A3), _kWhite = Color(0xFFE8E8EA), _kRed = Color(0xFFFF3363);
const _kPurple = Color(0xFF9B7BFF), _kDim = Color(0xFF3A3A40), _kGrey = Color(0xFF8A8A92);

typedef _Draw = void Function(Canvas c, Size s, _St st, double t);

class _P {
  const _P(this.name, this.tags, this.draw, {this.init = const Offset(.5, .5)});
  final String name, tags;
  final _Draw draw;
  final Offset init;
}

/// Drag state normalised to the panel. p = finger (0..1), s0 = where it went down,
/// spin = signed angle swept around the centre, rub = path length, flow() = flick with coast.
class _St extends ChangeNotifier {
  _St(Offset init)
      : p = init,
        s0 = init;
  Offset p, s0, vel = Offset.zero, _base = Offset.zero;
  double spin = 0, rub = 0, downAt = -99, upAt = -99;
  bool down = false;
  final trail = <Offset>[];
  double get a => p.dx;
  double get b => 1 - p.dy;
  Offset at(Size s) => Offset(p.dx * s.width, p.dy * s.height);
  double pinch(Size s) => ((p - s0).distance * s.height).clamp(0.0, s.width);
  Offset flow(double t) {
    if (down || upAt < 0) return _base;
    final k = 1 - math.exp(-(t - upAt) * 2.2);
    return _base + vel * (k / 2.2);
  }

  void bake(double t) => _base = flow(t);
  void ping() => notifyListeners();
}

const _kW = 156.0, _kH = 120.0, _kGap = 8.0;

class _OpSheet extends StatefulWidget {
  const _OpSheet({required this.concept, required this.panels});
  final String concept;
  final List<_P> panels;
  @override
  State<_OpSheet> createState() => _OpSheetState();
}

class _OpSheetState extends State<_OpSheet> with SingleTickerProviderStateMixin {
  final _clock = ValueNotifier<double>(0);
  late final Ticker _ticker = createTicker((e) => _clock.value = e.inMicroseconds / 1e6);

  @override
  void initState() {
    super.initState();
    _ticker.start();
  }

  @override
  void dispose() {
    _ticker.dispose();
    _clock.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ps = widget.panels;
    return ColoredBox(
      color: _kSheet,
      child: Align(
        alignment: Alignment.topCenter,
        child: FittedBox(
          fit: BoxFit.scaleDown,
          alignment: Alignment.topCenter,
          child: Padding(
            padding: const EdgeInsets.all(14),
            child: Column(mainAxisSize: MainAxisSize.min, children: [
              for (var r = 0; r < 4; r++)
                Padding(
                  padding: EdgeInsets.only(bottom: r < 3 ? _kGap : 0),
                  child: Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                    for (var k = 0; k < 5; k++)
                      if (r * 5 + k < ps.length)
                        Padding(
                          padding: EdgeInsets.only(right: k < 4 ? _kGap : 0),
                          child: _OpPanel(spec: ps[r * 5 + k], n: r * 5 + k + 1, concept: widget.concept, clock: _clock),
                        ),
                  ]),
                ),
            ]),
          ),
        ),
      ),
    );
  }
}

class _OpPanel extends StatefulWidget {
  const _OpPanel({required this.spec, required this.n, required this.concept, required this.clock});
  final _P spec;
  final int n;
  final String concept;
  final ValueNotifier<double> clock;
  @override
  State<_OpPanel> createState() => _OpPanelState();
}

class _OpPanelState extends State<_OpPanel> {
  late final _St st = _St(widget.spec.init);
  double _lastAng = 0;

  Offset _norm(Offset l) => Offset((l.dx / _kW).clamp(0.0, 1.0), (l.dy / _kH).clamp(0.0, 1.0));
  double _ang(Offset l) => math.atan2(l.dy - _kH / 2, l.dx - _kW / 2);

  @override
  void dispose() {
    st.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    const l1 = TextStyle(fontSize: 9.5, height: 1.3, color: _kWhite, letterSpacing: .2, decoration: TextDecoration.none);
    const l2 = TextStyle(fontSize: 9, height: 1.3, color: Color(0xFF6E6E76), letterSpacing: .2, decoration: TextDecoration.none);
    final t = widget.clock;
    return SizedBox(
      width: _kW,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        GestureDetector(
          onPanDown: (d) {
            st
              ..bake(t.value)
              ..down = true
              ..p = _norm(d.localPosition)
              ..s0 = st.p
              ..downAt = t.value
              ..trail.clear();
            _lastAng = _ang(d.localPosition);
            st.ping();
          },
          onPanUpdate: (d) {
            final a = _ang(d.localPosition);
            var da = a - _lastAng;
            if (da > math.pi) da -= 2 * math.pi;
            if (da < -math.pi) da += 2 * math.pi;
            _lastAng = a;
            st
              ..p = _norm(d.localPosition)
              ..spin += da
              ..rub += d.delta.distance / _kH
              .._base += Offset(d.delta.dx / _kW, d.delta.dy / _kH)
              ..trail.add(st.p);
            if (st.trail.length > 60) st.trail.removeAt(0);
            st.ping();
          },
          onPanEnd: (d) {
            st
              ..down = false
              ..vel = Offset(d.velocity.pixelsPerSecond.dx / _kW, d.velocity.pixelsPerSecond.dy / _kH)
              ..upAt = t.value;
            st.ping();
          },
          onPanCancel: () {
            st
              ..down = false
              ..vel = Offset.zero
              ..upAt = t.value;
            st.ping();
          },
          child: CustomPaint(size: const Size(_kW, _kH), painter: _OpPt(widget.spec, st, t)),
        ),
        const SizedBox(height: 4),
        Text('${widget.concept} #${widget.n}  ${widget.spec.name}', maxLines: 1, overflow: TextOverflow.clip, softWrap: false, style: l1),
        Text(widget.spec.tags, maxLines: 1, overflow: TextOverflow.clip, softWrap: false, style: l2),
      ]),
    );
  }
}

class _OpPt extends CustomPainter {
  _OpPt(this.spec, this.st, this.clock) : super(repaint: Listenable.merge([st, clock]));
  final _P spec;
  final _St st;
  final ValueNotifier<double> clock;
  @override
  void paint(Canvas c, Size s) {
    final r = Offset.zero & s;
    c.drawRect(r, Paint()..color = _kBg);
    c.save();
    c.clipRect(r);
    spec.draw(c, s, st, clock.value);
    c.restore();
    c.drawRect(r.deflate(.5), _s(_kFrame, 1));
  }

  @override
  bool shouldRepaint(_OpPt o) => o.spec != spec;
}

// ---- drawing helpers ------------------------------------------------------------------------

Paint _s(Color c, [double w = 1.1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Paint _f(Color c) => Paint()..color = c;
Color _al(Color c, double o) => c.withValues(alpha: o.clamp(0.0, 1.0));

void _ln(Canvas c, Offset a, Offset b, Color col, [double w = 1.1]) => c.drawLine(a, b, _s(col, w));
void _pl(Canvas c, List<Offset> pts, Color col, {double w = 1.1, bool close = false}) {
  if (pts.length < 2) return;
  c.drawPath(Path()..addPolygon(pts, close), _s(col, w));
}

void _o(Canvas c, Offset p, double r, Color col, [double w = 1.1]) => c.drawCircle(p, r, _s(col, w));
void _dot(Canvas c, Offset p, double r, Color col) => c.drawCircle(p, r, _f(col));
void _ov(Canvas c, Offset p, double rx, double ry, Color col, [double w = 1.1]) =>
    c.drawOval(Rect.fromCenter(center: p, width: rx * 2, height: ry * 2), _s(col, w));

void _dash(Canvas c, Offset a, Offset b, Color col, {double on = 1.4, double off = 2.6, double w = 1}) {
  final d = b - a, len = d.distance;
  if (len < .1) return;
  final u = d / len, p = _s(col, w);
  for (var x = 0.0; x < len; x += on + off) {
    c.drawLine(a + u * x, a + u * math.min(x + on, len), p);
  }
}

/// A closed curve r(theta) around o, as a polyline.
List<Offset> _loop(Offset o, double Function(double th) r, {int n = 64, double sy = 1}) => [
      for (var i = 0; i < n; i++) o + Offset(math.cos(i / n * 2 * math.pi), math.sin(i / n * 2 * math.pi) * sy) * r(i / n * 2 * math.pi)
    ];

Offset _dir(double a) => Offset(math.cos(a), math.sin(a));
Offset _rot(Offset v, double a) {
  final cs = math.cos(a), sn = math.sin(a);
  return Offset(v.dx * cs - v.dy * sn, v.dx * sn + v.dy * cs);
}

double _lerp(double a, double b, double t) => a + (b - a) * t;
double _sat(double x) => x.clamp(0.0, 1.0);
double _g(double d, double r) => math.exp(-(d * d) / math.max(r * r, 1e-6));

/// Isometric projection: x right-down, y left-down, z up.
Offset _iso(Offset o, double k, double x, double y, double z) => o + Offset((x - y) * k * .866, (x + y) * k * .5 - z * k);

// ---- noise ---------------------------------------------------------------------------------

double _hash(int x, int y) {
  var h = (x * 374761393 + y * 668265263) & 0xffffffff;
  h = ((h ^ (h >> 13)) * 1274126177) & 0xffffffff;
  h = h ^ (h >> 16);
  return (h & 0xffff) / 0xffff * 2 - 1;
}

double _vn(double x, double y) {
  final xi = x.floor(), yi = y.floor();
  final fx = x - xi, fy = y - yi;
  final u = fx * fx * (3 - 2 * fx), v = fy * fy * (3 - 2 * fy);
  return _lerp(_lerp(_hash(xi, yi), _hash(xi + 1, yi), u), _lerp(_hash(xi, yi + 1), _hash(xi + 1, yi + 1), u), v);
}

double _fbm(double x, double y, [int oct = 3]) {
  var s = 0.0, a = .6, f = 1.0;
  for (var i = 0; i < oct; i++) {
    s += a * _vn(x * f + i * 17.3, y * f + i * 5.1);
    a *= .5;
    f *= 2.03;
  }
  return s;
}

// ---- monoline type (OP-1 like): glyphs on a 4 x 6 grid, y down -------------------------------

const _glyphs = <String, List<List<double>>>{
  '0': [
    [1, 0, 3, 0, 3.6, .25, 3.9, .7, 4, 1.3, 4, 4.7, 3.9, 5.3, 3.6, 5.75, 3, 6, 1, 6, .4, 5.75, .1, 5.3, 0, 4.7, 0, 1.3, .1, .7, .4, .25, 1, 0]
  ],
  '1': [
    [2, 0, 2, 6]
  ],
  '2': [
    [0, 1, 1, 0, 3, 0, 4, 1, 4, 2, 0, 6, 4, 6]
  ],
  '3': [
    [0, 0, 4, 0, 2, 2.5, 3, 2.5, 4, 3.5, 4, 5, 3, 6, 1, 6, 0, 5]
  ],
  '4': [
    [3, 6, 3, 0, 0, 4.4, 4, 4.4]
  ],
  '5': [
    [4, 0, .5, 0, 0, 2.6, 3, 2.4, 4, 3.4, 4, 5, 3, 6, 1, 6, 0, 5.2]
  ],
  '6': [
    [3.4, 0, 1, 2.4, 0, 4, 0, 5, 1, 6, 3, 6, 4, 5, 4, 4, 3, 3, 1, 3, 0, 4]
  ],
  '7': [
    [0, 0, 4, 0, 1.4, 6]
  ],
  '8': [
    [2, 2.9, 1, 2.75, .5, 2.4, .3, 1.5, .5, .5, 1.1, .05, 2, 0, 2.9, .05, 3.5, .5, 3.7, 1.5, 3.5, 2.4, 3, 2.75, 2, 2.9],
    [2, 2.9, .9, 3.1, .25, 3.7, 0, 4.5, .25, 5.4, 1, 5.9, 2, 6, 3, 5.9, 3.75, 5.4, 4, 4.5, 3.75, 3.7, 3.1, 3.1, 2, 2.9]
  ],
  '9': [
    [.6, 6, 3, 3.6, 4, 2, 4, 1, 3, 0, 1, 0, 0, 1, 0, 2, 1, 3, 3, 3, 4, 2]
  ],
  'A': [
    [0, 6, 2, 0, 4, 6],
    [.7, 4, 3.3, 4]
  ],
  'B': [
    [0, 0, 0, 6, 3, 6, 4, 5, 4, 4, 3, 3, 0, 3],
    [0, 0, 3, 0, 3.7, .7, 3.7, 2.3, 3, 3]
  ],
  'C': [
    [4, 1, 3, 0, 1, 0, 0, 1, 0, 5, 1, 6, 3, 6, 4, 5]
  ],
  'D': [
    [0, 0, 0, 6, 2.5, 6, 4, 4.5, 4, 1.5, 2.5, 0, 0, 0]
  ],
  'E': [
    [4, 0, 0, 0, 0, 6, 4, 6],
    [0, 3, 3, 3]
  ],
  'F': [
    [4, 0, 0, 0, 0, 6],
    [0, 3, 3, 3]
  ],
  'G': [
    [4, 1, 3, 0, 1, 0, 0, 1, 0, 5, 1, 6, 3, 6, 4, 5, 4, 3.5, 2.5, 3.5]
  ],
  'H': [
    [0, 0, 0, 6],
    [4, 0, 4, 6],
    [0, 3, 4, 3]
  ],
  'I': [
    [.6, 0, .6, 6]
  ],
  'J': [
    [4, 0, 4, 5, 3, 6, 1, 6, 0, 5]
  ],
  'K': [
    [0, 0, 0, 6],
    [4, 0, 0, 3.6],
    [1.4, 2.6, 4, 6]
  ],
  'L': [
    [0, 0, 0, 6, 4, 6]
  ],
  'M': [
    [0, 6, 0, 0, 2, 4, 4, 0, 4, 6]
  ],
  'N': [
    [0, 6, 0, 0, 4, 6, 4, 0]
  ],
  'O': [
    [1, 0, 3, 0, 3.6, .25, 3.9, .7, 4, 1.3, 4, 4.7, 3.9, 5.3, 3.6, 5.75, 3, 6, 1, 6, .4, 5.75, .1, 5.3, 0, 4.7, 0, 1.3, .1, .7, .4, .25, 1, 0]
  ],
  'P': [
    [0, 6, 0, 0, 3, 0, 4, 1, 4, 2, 3, 3, 0, 3]
  ],
  'Q': [
    [1, 0, 3, 0, 4, 1, 4, 5, 3, 6, 1, 6, 0, 5, 0, 1, 1, 0],
    [2.6, 4.6, 4, 6]
  ],
  'R': [
    [0, 6, 0, 0, 3, 0, 4, 1, 4, 2, 3, 3, 0, 3],
    [2, 3, 4, 6]
  ],
  'S': [
    [4, 1, 3, 0, 1, 0, 0, 1, 0, 2, 1, 3, 3, 3, 4, 4, 4, 5, 3, 6, 1, 6, 0, 5]
  ],
  'T': [
    [0, 0, 4, 0],
    [2, 0, 2, 6]
  ],
  'U': [
    [0, 0, 0, 5, 1, 6, 3, 6, 4, 5, 4, 0]
  ],
  'V': [
    [0, 0, 2, 6, 4, 0]
  ],
  'W': [
    [0, 0, 1, 6, 2, 2, 3, 6, 4, 0]
  ],
  'X': [
    [0, 0, 4, 6],
    [4, 0, 0, 6]
  ],
  'Y': [
    [0, 0, 2, 3, 4, 0],
    [2, 3, 2, 6]
  ],
  'Z': [
    [0, 0, 4, 0, 0, 6, 4, 6]
  ],
  '-': [
    [1, 3, 3, 3]
  ],
  '+': [
    [0.5, 3, 3.5, 3],
    [2, 1.5, 2, 4.5]
  ],
  '/': [
    [3.5, 0, .5, 6]
  ],
  '%': [
    [4, 0, 0, 6],
    [0, 0, 1, 0, 1, 1, 0, 1, 0, 0],
    [3, 5, 4, 5, 4, 6, 3, 6, 3, 5]
  ],
  '.': [
    [2, 5.8, 2, 6]
  ],
  ':': [
    [2, 1.8, 2, 2],
    [2, 4.8, 2, 5]
  ],
  '°': [
    [1, 0, 2, 0, 2, 1, 1, 1, 1, 0]
  ],
  '>': [
    [0, 1, 4, 3, 0, 5]
  ],
  '<': [
    [4, 1, 0, 3, 4, 5]
  ],
};

double _adv(String ch) => ch == 'I' || ch == 'i' || ch == '.' || ch == ':' ? 2.8 : 5.6;
double _tw(String s, double h) => s.isEmpty ? 0 : h / 6 * (s.split('').fold(0.0, (a, ch) => a + _adv(ch)) - 1.6);

/// Monoline text. h = cap height. align: 0 left, .5 centre, 1 right. warp bends every point (subdivided).
void _t(Canvas c, String s, Offset at, double h, Color col, {double w = 1, double align = 0, Offset Function(Offset)? warp}) {
  final u = h / 6, p = _s(col, w);
  var x = at.dx - _tw(s, h) * align;
  for (final ch in s.toUpperCase().split('')) {
    final g = _glyphs[ch];
    if (g != null) {
      for (final line in g) {
        final path = Path();
        for (var i = 0; i < line.length; i += 2) {
          final q = Offset(x + line[i] * u, at.dy + line[i + 1] * u);
          if (warp == null) {
            i == 0 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
          } else if (i == 0) {
            final r = warp(q);
            path.moveTo(r.dx, r.dy);
          } else {
            final q0 = Offset(x + line[i - 2] * u, at.dy + line[i - 1] * u);
            final n = math.max(1, ((q - q0).distance / 2.5).ceil());
            for (var k = 1; k <= n; k++) {
              final r = warp(Offset.lerp(q0, q, k / n)!);
              path.lineTo(r.dx, r.dy);
            }
          }
        }
        c.drawPath(path, p);
      }
    }
    x += u * _adv(ch);
  }
}

/// Polyline with every point bent by warp (segments subdivided to ~2px).
void _plw(Canvas c, List<Offset> pts, Offset Function(Offset) warp, Color col, {double w = 1.1, bool close = false}) {
  if (pts.length < 2) return;
  final src = close ? [...pts, pts.first] : pts;
  final out = <Offset>[warp(src.first)];
  for (var i = 1; i < src.length; i++) {
    final n = math.max(1, ((src[i] - src[i - 1]).distance / 2).ceil());
    for (var k = 1; k <= n; k++) {
      out.add(warp(Offset.lerp(src[i - 1], src[i], k / n)!));
    }
  }
  _pl(c, out, col, w: w);
}

/// Tiny label over a thin numeral, OP-1 readout style.
void _kv(Canvas c, Offset at, String label, num v, Color col, {double h = 14, double align = 0}) {
  _t(c, label, at, 4.2, col, w: .9, align: align);
  _t(c, v.round().toString().padLeft(2, '0'), at + Offset(0, 7), h, col, w: 1, align: align);
}

/// Tiny caps label.
void _lb(Canvas c, String s, Offset at, Color col, {double align = 0}) => _t(c, s, at, 4.2, col, w: .9, align: align);

/// Small stick figure: head at h, facing right. pose 0..1 swings the arms.
void _fig(Canvas c, Offset feet, double k, Color col, {double arm = 0, double w = 1}) {
  final hip = feet + Offset(0, -9 * k), neck = hip + Offset(0, -9 * k), head = neck + Offset(0, -3.4 * k);
  _o(c, head, 3 * k, col, w);
  _ln(c, neck, hip, col, w);
  _pl(c, [feet + Offset(-3 * k, 0), hip, feet + Offset(3 * k, 0)], col, w: w);
  _pl(c, [neck + _rot(Offset(0, 7 * k), .5 + arm), neck, neck + _rot(Offset(0, 7 * k), -.5 - arm)], col, w: w);
}
