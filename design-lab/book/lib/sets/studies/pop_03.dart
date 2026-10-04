// Pop 03: three sheets of 20 pop miniatures each (Noise, Warp / Distort, Shadow).
// Every panel is a CustomPainter fed by one shared clock per sheet and one tiny drag state per panel.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'pop_03_noise.dart';
part 'pop_03_warp.dart';
part 'pop_03_shadow.dart';

WidgetbookComponent pop03Set() => WidgetbookComponent(name: 'Pop 03', useCases: [
      WidgetbookUseCase(name: 'Noise x20', builder: (_) => const _Pop03Sheet(concept: 'Noise', panels: _noisePanels)),
      WidgetbookUseCase(name: 'Warp / Distort x20', builder: (_) => const _Pop03Sheet(concept: 'Warp', panels: _warpPanels)),
      WidgetbookUseCase(name: 'Shadow x20', builder: (_) => const _Pop03Sheet(concept: 'Shadow', panels: _shadowPanels)),
    ]);

const _kPanel = Color(0xFF1C1C1E), _kPanel2 = Color(0xFF2A2A2E), _kCream = Color(0xFFF4EBDD), _kWhite = Color(0xFFFFFFFF);
const _kOrange = Color(0xFFFF7A3D), _kYellow = Color(0xFFFFD23F), _kCyan = Color(0xFF3DD6F5), _kLime = Color(0xFF9BE564);
const _kPink = Color(0xFFFF5DA2), _kViolet = Color(0xFF9B7BFF), _kRed = Color(0xFFFF4B4B), _kBlue = Color(0xFF2E5BFF);
const _kInk = Color(0xFF1C1C1E), _kNight = Color(0xFF111114);

typedef _Draw = void Function(Canvas c, Size s, _St st, double t);

class _P {
  const _P(this.name, this.tags, this.draw, {this.init = const Offset(.5, .5), this.init0, this.bg = _kPanel, this.anim = true});
  final String name, tags;
  final _Draw draw;
  final Offset init;
  final Offset? init0;
  final Color bg;
  final bool anim;
}

/// Drag state, normalised to the panel: p is the finger (0..1), s0 where it went down.
class _St extends ChangeNotifier {
  _St(Offset init, [Offset? init0])
      : p = init,
        s0 = init0 ?? init;
  Offset p, s0, vel = Offset.zero;
  double spin = 0, rub = 0, downAt = -99, upAt = -99;
  bool down = false;
  final trail = <(Offset, Offset)>[];
  double get a => p.dx;
  double get b => 1 - p.dy;
  Offset at(Size s) => Offset(p.dx * s.width, p.dy * s.height);
  Offset at0(Size s) => Offset(s0.dx * s.width, s0.dy * s.height);
  void ping() => notifyListeners();
}

const _kW = 156.0, _kH = 120.0, _kGap = 8.0;

class _Pop03Sheet extends StatefulWidget {
  const _Pop03Sheet({required this.concept, required this.panels});
  final String concept;
  final List<_P> panels;
  @override
  State<_Pop03Sheet> createState() => _SheetState();
}

class _SheetState extends State<_Pop03Sheet> with SingleTickerProviderStateMixin {
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
      color: _kNight,
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
                          child: _Panel(spec: ps[r * 5 + k], n: r * 5 + k + 1, concept: widget.concept, clock: _clock),
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

class _Panel extends StatefulWidget {
  const _Panel({required this.spec, required this.n, required this.concept, required this.clock});
  final _P spec;
  final int n;
  final String concept;
  final ValueNotifier<double> clock;
  @override
  State<_Panel> createState() => _PanelState();
}

class _PanelState extends State<_Panel> {
  late final _St st = _St(widget.spec.init, widget.spec.init0);
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
    const l1 = TextStyle(fontSize: 10, height: 1.25, color: _kCream, fontWeight: FontWeight.w600, decoration: TextDecoration.none);
    const l2 = TextStyle(fontSize: 10, height: 1.25, color: Color(0xFF8C877F), decoration: TextDecoration.none);
    return SizedBox(
      width: _kW,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        GestureDetector(
          onPanDown: (d) {
            st
              ..down = true
              ..p = _norm(d.localPosition)
              ..s0 = st.p
              ..downAt = widget.clock.value;
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
              ..trail.add((st.p, Offset(d.delta.dx / _kW, d.delta.dy / _kH)));
            if (st.trail.length > 48) st.trail.removeAt(0);
            st.ping();
          },
          onPanEnd: (d) {
            st
              ..down = false
              ..vel = Offset(d.velocity.pixelsPerSecond.dx / _kW, d.velocity.pixelsPerSecond.dy / _kH)
              ..upAt = widget.clock.value;
            st.ping();
          },
          onPanCancel: () {
            st
              ..down = false
              ..upAt = widget.clock.value;
            st.ping();
          },
          child: ClipRRect(
            borderRadius: BorderRadius.circular(9),
            child: CustomPaint(size: const Size(_kW, _kH), painter: _Pt(widget.spec, st, widget.clock)),
          ),
        ),
        const SizedBox(height: 4),
        Text('${widget.concept} #${widget.n}  ${widget.spec.name}', maxLines: 1, overflow: TextOverflow.clip, softWrap: false, style: l1),
        Text(widget.spec.tags, maxLines: 1, overflow: TextOverflow.clip, softWrap: false, style: l2),
      ]),
    );
  }
}

class _Pt extends CustomPainter {
  _Pt(this.spec, this.st, this.clock) : super(repaint: spec.anim ? Listenable.merge([st, clock]) : st);
  final _P spec;
  final _St st;
  final ValueNotifier<double> clock;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, _f(spec.bg));
    spec.draw(c, s, st, clock.value);
  }

  @override
  bool shouldRepaint(_Pt o) => o.spec != spec;
}

// ---- small helpers -------------------------------------------------------------------------

Paint _f(Color c) => Paint()..color = c;
Paint _s(Color c, double w) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Paint _soft(Color c, double sigma) {
  final p = Paint()..color = c;
  if (sigma > .3) p.maskFilter = MaskFilter.blur(BlurStyle.normal, sigma);
  return p;
}

Color _al(Color c, double o) => c.withValues(alpha: o.clamp(0.0, 1.0));
Color _mix(Color a, Color b, double t) => Color.lerp(a, b, t.clamp(0.0, 1.0))!;
Color _ramp(List<Color> cs, double t) {
  final x = t.clamp(0.0, .9999) * (cs.length - 1);
  final i = x.floor();
  return _mix(cs[i], cs[i + 1], x - i);
}

Path _poly(List<Offset> pts, {bool close = true}) {
  final p = Path()..addPolygon(pts, close);
  return p;
}

Offset _rot(Offset v, double a) {
  final c = math.cos(a), s = math.sin(a);
  return Offset(v.dx * c - v.dy * s, v.dx * s + v.dy * c);
}

Offset _dir(double a) => Offset(math.cos(a), math.sin(a));
double _lerp(double a, double b, double t) => a + (b - a) * t;
double _sat(double x) => x.clamp(0.0, 1.0);

/// Deterministic hash in -1..1.
double _hash(int x, int y) {
  var h = (x * 374761393 + y * 668265263) & 0xffffffff;
  h = ((h ^ (h >> 13)) * 1274126177) & 0xffffffff;
  h = h ^ (h >> 16);
  return (h & 0xffff) / 0xffff * 2 - 1;
}

/// Smooth value noise in about -1..1.
double _vn(double x, double y) {
  final xi = x.floor(), yi = y.floor();
  final fx = x - xi, fy = y - yi;
  final u = fx * fx * (3 - 2 * fx), v = fy * fy * (3 - 2 * fy);
  return _lerp(_lerp(_hash(xi, yi), _hash(xi + 1, yi), u), _lerp(_hash(xi, yi + 1), _hash(xi + 1, yi + 1), u), v);
}

double _fb(double x, double y) => _vn(x, y) * .68 + _vn(x * 2.13 + 5.2, y * 2.13 + 1.7) * .32;

/// Warp falloff: 1 at the centre, 0 at radius R.
double _fall(double d, double r) {
  if (d >= r) return 0;
  final q = 1 - (d / r) * (d / r);
  return q * q;
}

/// Bulge (S>0) or pinch (S<0) plus twist T inside radius R around c.
Offset _tw(Offset q, Offset c, double r, double s, double t) {
  final v = q - c, k = _fall(v.distance, r);
  return c + _rot(v, t * k) * (1 + s * k);
}
