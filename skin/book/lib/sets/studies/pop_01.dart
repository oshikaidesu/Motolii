// set: Pop 01. Three sheets (Glow, Scatter, Stagger), each 20 panels of one concept, from sensible to pushed to the extreme,
// in the pop palette (flat vivid colour on dark panels, crisp vector, playful physics). One Widgetbook screen shows a whole sheet.
// Every panel answers a drag: horizontal -> a, vertical -> b, plus tap / flick (seed), rub (energy that decays), spin, paint (trail).
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'pop_01_glow.dart';
part 'pop_01_scatter.dart';
part 'pop_01_stagger.dart';

WidgetbookComponent pop01Set() => WidgetbookComponent(name: 'Pop 01', useCases: [
      WidgetbookUseCase(name: 'Glow x20', builder: (c) => const PopSheet(concept: 'Glow', specs: _glowSpecs)),
      WidgetbookUseCase(name: 'Scatter x20', builder: (c) => const PopSheet(concept: 'Scatter', specs: _scatterSpecs)),
      WidgetbookUseCase(name: 'Stagger x20', builder: (c) => const PopSheet(concept: 'Stagger', specs: _staggerSpecs)),
    ]);

abstract final class _K {
  static const ink0 = Color(0xFF1C1C1E), ink1 = Color(0xFF2A2A2E), cream = Color(0xFFF4EBDD), white = Color(0xFFFFFFFF);
  static const orange = Color(0xFFFF7A3D), yellow = Color(0xFFFFD23F), cyan = Color(0xFF3DD6F5), lime = Color(0xFF9BE564);
  static const pink = Color(0xFFFF5DA2), violet = Color(0xFF9B7BFF), red = Color(0xFFFF4B4B), blue = Color(0xFF2E5BFF);
  static const dim = Color(0xFF3A3A42), dim2 = Color(0xFF4C4C56), ground = Color(0xFF111113), tag = Color(0xFF8E8A84);
  static const vivid = [yellow, cyan, pink, lime, orange, violet, red, white];
}

typedef PopPaint = void Function(Canvas c, Size s, PV v);

class PopSpec {
  const PopSpec(this.name, this.tags, this.paint, {this.bg = _K.ink0, this.a = .5, this.b = .5});
  final String name, tags;
  final PopPaint paint;
  final Color bg;
  final double a, b;
}

/// One panel's local state. a/b: drag axes 0..1; seed: tap or flick; energy: rub, decays; spin: turning around the centre; trail: painted points.
class PV {
  PV(this.a, this.b);
  double a, b, t = 0, energy = 0, spin = 0;
  int seed = 0;
  Offset? p;
  bool down = false;
  final trail = <Offset?>[];
  Offset pt(Size s) => p ?? s.center(Offset.zero);
}

const _panelW = 156.0, _panelH = 116.0;

class PopSheet extends StatefulWidget {
  const PopSheet({super.key, required this.concept, required this.specs});
  final String concept;
  final List<PopSpec> specs;
  @override
  State<PopSheet> createState() => _PopSheetState();
}

class _PopSheetState extends State<PopSheet> with SingleTickerProviderStateMixin {
  late final _v = [for (final s in widget.specs) PV(s.a, s.b)];
  final _frame = ValueNotifier<int>(0);
  late final Ticker _tk = createTicker(_tick);
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tk.start();
  }

  void _tick(Duration e) {
    final dt = (e - _last).inMicroseconds / 1e6;
    _last = e;
    final k = math.exp(-dt * 1.1);
    for (final v in _v) {
      v.t = e.inMicroseconds / 1e6;
      if (!v.down) v.energy *= k;
    }
    _frame.value++;
  }

  @override
  void dispose() {
    _tk.dispose();
    _frame.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    Widget cell(int i) => _Panel(index: i, concept: widget.concept, spec: widget.specs[i], v: _v[i], frame: _frame);
    return ColoredBox(
      color: _K.ground,
      child: Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(14),
          child: Column(mainAxisSize: MainAxisSize.min, children: [
            for (var r = 0; r < (widget.specs.length / 5).ceil(); r++) ...[
              if (r > 0) const SizedBox(height: 10),
              Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                for (var col = 0; col < 5 && r * 5 + col < widget.specs.length; col++) ...[
                  if (col > 0) const SizedBox(width: 8),
                  cell(r * 5 + col),
                ],
              ]),
            ],
          ]),
        ),
      ),
    );
  }
}

TextStyle _lab(Color c, FontWeight w) => TextStyle(
    fontFamily: 'Inter', fontFamilyFallback: const ['.AppleSystemUIFont'], fontSize: 10, fontWeight: w, color: c, height: 1.25, decoration: TextDecoration.none);

class _Panel extends StatelessWidget {
  const _Panel({required this.index, required this.concept, required this.spec, required this.v, required this.frame});
  final int index;
  final String concept;
  final PopSpec spec;
  final PV v;
  final Listenable frame;

  @override
  Widget build(BuildContext context) {
    final size = const Size(_panelW, _panelH);
    final ctr = size.center(Offset.zero);
    Offset? prev;
    return SizedBox(
      width: _panelW,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapUp: (d) {
            v.p = d.localPosition;
            v.seed++;
          },
          onPanStart: (d) {
            v.down = true;
            v.p = prev = d.localPosition;
            if (v.trail.isNotEmpty) v.trail.add(null);
            v.trail.add(d.localPosition);
          },
          onPanUpdate: (d) {
            final q = d.localPosition, o = prev ?? q;
            v.a = (v.a + d.delta.dx / 130).clamp(0.0, 1.0);
            v.b = (v.b - d.delta.dy / 100).clamp(0.0, 1.0);
            v.energy = math.min(1.0, v.energy + d.delta.distance / 260);
            var da = (q - ctr).direction - (o - ctr).direction;
            if (da > math.pi) da -= 2 * math.pi;
            if (da < -math.pi) da += 2 * math.pi;
            if ((q - ctr).distance > 8) v.spin += da;
            v.p = prev = q;
            v.trail.add(q);
            while (v.trail.length > 90) {
              v.trail.removeAt(0);
            }
          },
          onPanEnd: (d) {
            v.down = false;
            if (d.velocity.pixelsPerSecond.distance > 700) v.seed++;
          },
          child: ClipRRect(
            borderRadius: BorderRadius.circular(10),
            child: CustomPaint(size: size, painter: _PP(spec, v, frame)),
          ),
        ),
        const SizedBox(height: 5),
        Text('$concept #${index + 1}  ${spec.name}', maxLines: 1, overflow: TextOverflow.ellipsis, style: _lab(_K.cream, FontWeight.w600)),
        Text(spec.tags, maxLines: 1, overflow: TextOverflow.ellipsis, style: _lab(_K.tag, FontWeight.w400)),
      ]),
    );
  }
}

class _PP extends CustomPainter {
  _PP(this.spec, this.v, Listenable frame) : super(repaint: frame);
  final PopSpec spec;
  final PV v;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = spec.bg);
    spec.paint(c, s, v);
  }

  @override
  bool shouldRepaint(_PP o) => o.spec != spec || o.v != v;
}

// ---- small shared drawing helpers ----

double _cl(double x) => x.clamp(0.0, 1.0);
Color _al(Color c, double o) => c.withValues(alpha: _cl(c.a * o));
Paint _f(Color c, [double o = 1]) => Paint()..color = _al(c, o);
Paint _s(Color c, double w, [double o = 1]) => Paint()
  ..color = _al(c, o)
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Color _mix(Color a, Color b, double t) => Color.lerp(a, b, _cl(t))!;
double _h(int i) {
  final x = math.sin(i * 12.9898 + 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

double _hs(int seed, int i) => _h(seed * 7919 + i * 131 + 17);
double _eo(double x) => 1 - math.pow(1 - _cl(x), 3).toDouble();

/// Back-out: arrives with a small overshoot.
double _bo(double x) {
  final t = _cl(x) - 1;
  return 1 + t * t * (2.7 * t + 1.7);
}

/// A random point in the unit disc (denser towards the centre).
Offset _rnd2(int seed, int i) {
  final a = _hs(seed, i) * math.pi * 2, r = math.pow(_hs(seed, i + 977), .7).toDouble();
  return Offset(math.cos(a), math.sin(a)) * r;
}

Offset _dir(double a) => Offset(math.cos(a), math.sin(a));

void _glow(Canvas c, Offset o, double r, Color col, double k) {
  if (r < .5 || k <= 0) return;
  c.drawCircle(o, r, Paint()..shader = ui.Gradient.radial(o, r, [_al(col, k), _al(col, k * .3), _al(col, 0)], const [0, .42, 1]));
}

Path _polyPath(List<Offset> pts, {bool close = true}) {
  final p = Path()..moveTo(pts.first.dx, pts.first.dy);
  for (final q in pts.skip(1)) {
    p.lineTo(q.dx, q.dy);
  }
  if (close) p.close();
  return p;
}

Path _star(Offset o, double r0, double r1, int n, [double rot = -math.pi / 2]) =>
    _polyPath([for (var i = 0; i < n * 2; i++) o + _dir(rot + i * math.pi / n) * (i.isEven ? r0 : r1)]);

/// Isometric projection around an origin, k px per unit.
class _Iso {
  const _Iso(this.o, this.k);
  final Offset o;
  final double k;
  Offset p(double x, double y, [double z = 0]) => Offset(o.dx + (x - y) * k * .866, o.dy + (x + y) * k * .5 - z * k);
  void box(Canvas c, double x, double y, double sx, double sy, double z0, double z1, Color top, Color left, Color right) {
    c.drawPath(_polyPath([p(x, y + sy, z0), p(x + sx, y + sy, z0), p(x + sx, y + sy, z1), p(x, y + sy, z1)]), _f(left));
    c.drawPath(_polyPath([p(x + sx, y, z0), p(x + sx, y + sy, z0), p(x + sx, y + sy, z1), p(x + sx, y, z1)]), _f(right));
    c.drawPath(_polyPath([p(x, y, z1), p(x + sx, y, z1), p(x + sx, y + sy, z1), p(x, y + sy, z1)]), _f(top));
  }
}

/// The shared sample picture: sky, sun, two mountains, a field.
void _pic(Canvas c, Rect r) {
  c.drawRect(r, _f(_K.cyan));
  c.drawCircle(Offset(r.left + r.width * .72, r.top + r.height * .32), r.height * .16, _f(_K.yellow));
  c.drawPath(_polyPath([Offset(r.left, r.bottom), Offset(r.left + r.width * .3, r.top + r.height * .35), Offset(r.left + r.width * .6, r.bottom)]), _f(_K.violet));
  c.drawPath(_polyPath([Offset(r.left + r.width * .35, r.bottom), Offset(r.left + r.width * .62, r.top + r.height * .5), Offset(r.right, r.bottom)]), _f(_K.blue));
  c.drawRect(Rect.fromLTRB(r.left, r.bottom - r.height * .2, r.right, r.bottom), _f(_K.lime));
}

// ---- stagger timing ----

/// Rank of each item in the arrival order. Order mode cycles with the seed (tap): along, centre-out, edges-in, random.
/// Horizontal drag below the middle reverses the direction.
List<int> _ranks(PV v, int n, [List<double>? keys]) {
  final k = keys ?? [for (var i = 0; i < n; i++) n == 1 ? 0.0 : i / (n - 1)];
  final mode = v.seed % 4;
  double key(int i) => switch (mode) { 1 => (k[i] - .5).abs(), 2 => -(k[i] - .5).abs(), 3 => _hs(v.seed, i), _ => k[i] };
  final idx = [for (var i = 0; i < n; i++) i]..sort((x, y) => key(x).compareTo(key(y)));
  final ord = v.a < .5 && mode != 3 ? idx.reversed.toList() : idx;
  final r = List.filled(n, 0);
  for (var j = 0; j < n; j++) {
    r[ord[j]] = j;
  }
  return r;
}

/// Progress 0..1 of the item with this rank, on a loop. Vertical drag = the gap between arrivals.
double _st(PV v, int rank, int n, {double dur = .5}) {
  final off = .05 + v.b * .3, total = (n - 1) * off + dur + 1.3;
  return _cl((v.t % total - .4 - rank * off) / dur);
}
