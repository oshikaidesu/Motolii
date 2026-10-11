// Pop 09: two sheets of 20 pop miniatures each (Rotation / Spin, Scale / Stretch). One sheet = one screenshot:
// 5 x 4 panels of 156 x 116 with a 2-line label under each. A panel paints from two values (a, b), an integrated phase
// (so a speed change never jumps) and the pointer; the gesture kind decides how a drag moves the values. One Ticker per sheet.
import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'pop_09_rot.dart';
part 'pop_09_scale.dart';

WidgetbookComponent pop09Set() => WidgetbookComponent(name: 'Pop 09', useCases: [
      WidgetbookUseCase(name: 'Rotation / Spin x20', builder: (c) => _P9Sheet(concept: 'Rotation', panels: _rotPanels())),
      WidgetbookUseCase(name: 'Scale / Stretch x20', builder: (c) => _P9Sheet(concept: 'Scale', panels: _scalePanels())),
    ]);

const _k0 = Color(0xFF1C1C1E), _k1 = Color(0xFF2A2A2E), _cr = Color(0xFFF4EBDD), _wh = Color(0xFFFFFFFF);
const _or = Color(0xFFFF7A3D), _ye = Color(0xFFFFD23F), _cy = Color(0xFF3DD6F5), _li = Color(0xFF9BE564);
const _pk = Color(0xFFFF5DA2), _vi = Color(0xFF9B7BFF), _rd = Color(0xFFFF4B4B), _bl = Color(0xFF2E5BFF);

const double _cw = 156, _ch = 116;
const _ctr = Offset(_cw / 2, _ch / 2);
const _tau = math.pi * 2;

/// xy: dx -> a, up -> b. spin: angle round the centre -> a (turns, wraps), radius -> b. flick: spin that keeps its throw.
/// flickY: up -> a (turns), keeps its throw. rub: path length pumps b (leaks by `decay`), dx -> a. pinch: distance from the
/// centre -> a. stretch: away from the centre on each axis -> a (x) and b (y). corner: the pointer is the bottom-right corner.
/// wind: the drag's heading -> a (turns).
enum _G { xy, spin, flick, flickY, rub, pinch, stretch, corner, wind }

typedef _Paint9 = void Function(Canvas c, Size s, _S st, double t);

class _Pan9 {
  const _Pan9(this.name, this.tags, this.g, this.paint,
      {this.a = .5, this.b = .5, this.fric = .4, this.decay = 0, this.spd});
  final String name, tags;
  final _G g;
  final _Paint9 paint;
  final double a, b, fric, decay;

  /// Turns per second the panel's phase advances, from its state.
  final double Function(_S st)? spd;
}

class _S {
  _S(this.a, this.b);
  double a, b, va = 0, w = 0, ph = 0, rel = -9;
  final List<Offset> pts = [];
  Offset? at;
  bool down = false;
}

double _wr(double x) => x - x.floorToDouble();
double _cl(double v, [double lo = 0, double hi = 1]) => v < lo ? lo : (v > hi ? hi : v);
double _lr(double a, double b, double t) => a + (b - a) * t;
double _rn(int i) {
  final x = math.sin(i * 12.9898 + 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

Offset _pol(Offset c, double r, double ang) => c + Offset(math.cos(ang), math.sin(ang)) * r;
Paint _f(Color c) => Paint()..color = c;
Paint _s(Color c, [double w = 2]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
void _bg(Canvas c, Size s, Color col) => c.drawRect(Offset.zero & s, _f(col));
Color _al(Color c, double a) => c.withValues(alpha: _cl(a));
Path _poly(List<Offset> p) => Path()..addPolygon(p, true);
void _rr(Canvas c, Rect r, double rad, Paint p) => c.drawRRect(RRect.fromRectAndRadius(r, Radius.circular(rad)), p);

Path _star(Offset c, double r, double rot, {int k = 5, double inner = .45}) => _poly([
      for (var i = 0; i < k * 2; i++) _pol(c, i.isEven ? r : r * inner, rot - math.pi / 2 + i * math.pi / k),
    ]);

Path _gear(Offset c, double r, int teeth, double rot) {
  final p = <Offset>[];
  for (var i = 0; i < teeth; i++) {
    final a0 = rot + i * _tau / teeth, st = _tau / teeth;
    p
      ..add(_pol(c, r * .8, a0))
      ..add(_pol(c, r, a0 + st * .18))
      ..add(_pol(c, r, a0 + st * .48))
      ..add(_pol(c, r * .8, a0 + st * .66));
  }
  return _poly(p);
}

/// A tiny sample picture (sky, sun, two hills) filling [r]; the thing a user rotates or scales in a real work.
void _pic(Canvas c, Rect r, {Color sky = _cy}) {
  c.save();
  c.clipRect(r);
  c.drawRect(r, _f(sky));
  c.drawCircle(Offset(r.left + r.width * .72, r.top + r.height * .32), r.shortestSide * .16, _f(_ye));
  c.drawOval(Rect.fromCenter(center: Offset(r.left + r.width * .25, r.bottom), width: r.width * 1.1, height: r.height * .9), _f(_li));
  c.drawOval(Rect.fromCenter(center: Offset(r.left + r.width * .85, r.bottom + r.height * .05), width: r.width * .9, height: r.height * .7),
      _f(_bl));
  c.restore();
}

class _P9Sheet extends StatefulWidget {
  const _P9Sheet({required this.concept, required this.panels});
  final String concept;
  final List<_Pan9> panels;
  @override
  State<_P9Sheet> createState() => _P9SheetState();
}

class _P9SheetState extends State<_P9Sheet> with SingleTickerProviderStateMixin {
  late final Ticker _tk;
  final _t = ValueNotifier<double>(0);
  late final List<_S> _st = [for (final p in widget.panels) _S(p.a, p.b)];
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tk = createTicker(_tick)..start();
  }

  void _tick(Duration d) {
    final dt = ((d - _last).inMicroseconds / 1e6).clamp(0.0, .1);
    _last = d;
    for (var i = 0; i < _st.length; i++) {
      final p = widget.panels[i], s = _st[i];
      if (p.spd != null) s.ph += p.spd!(s) * dt;
      s.w *= math.pow(.02, dt).toDouble();
      if (p.decay > 0 && !s.down) s.b = _cl(s.b - p.decay * dt);
      if ((p.g == _G.flick || p.g == _G.flickY) && !s.down && s.va.abs() > 1e-4) {
        s.a = _wr(s.a + s.va * dt);
        s.w = s.va;
        s.va *= math.pow(p.fric, dt).toDouble();
      }
    }
    _t.value = d.inMicroseconds / 1e6;
  }

  @override
  void dispose() {
    _tk.dispose();
    _t.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ps = widget.panels;
    return ColoredBox(
      color: const Color(0xFF111113),
      child: Center(
        child: FittedBox(
          fit: BoxFit.scaleDown,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(mainAxisSize: MainAxisSize.min, children: [
              for (var r = 0; r < (ps.length + 4) ~/ 5; r++)
                Padding(
                  padding: EdgeInsets.only(top: r == 0 ? 0 : 8),
                  child: Row(mainAxisSize: MainAxisSize.min, children: [
                    for (var k = 0; k < 5 && r * 5 + k < ps.length; k++)
                      Padding(
                        padding: EdgeInsets.only(left: k == 0 ? 0 : 8),
                        child: _P9Tile(n: r * 5 + k + 1, concept: widget.concept, p: ps[r * 5 + k], st: _st[r * 5 + k], t: _t),
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

const _lab1 = TextStyle(fontSize: 10, height: 1.25, color: _cr, fontWeight: FontWeight.w600, decoration: TextDecoration.none);
const _lab2 = TextStyle(fontSize: 10, height: 1.25, color: Color(0x99F4EBDD), decoration: TextDecoration.none);

class _P9Tile extends StatelessWidget {
  const _P9Tile({required this.n, required this.concept, required this.p, required this.st, required this.t});
  final int n;
  final String concept;
  final _Pan9 p;
  final _S st;
  final ValueNotifier<double> t;

  void _start(DragStartDetails d) {
    st
      ..down = true
      ..at = d.localPosition
      ..va = 0;
    st.pts
      ..clear()
      ..add(d.localPosition);
    if (p.g == _G.corner) _corner(d.localPosition);
  }

  void _corner(Offset q) {
    st.a = _cl((q.dx - 14) / (_cw - 28));
    st.b = _cl((q.dy - 12) / (_ch - 24));
  }

  void _move(DragUpdateDetails d) {
    final q = d.localPosition, dx = d.delta.dx / _cw, dy = -d.delta.dy / _ch, o = q - d.delta - _ctr, n = q - _ctr;
    st.at = q;
    if ((q - st.pts.last).distance > 3) st.pts.add(q);
    if (st.pts.length > 40) st.pts.removeAt(0);
    switch (p.g) {
      case _G.xy:
        st.a = _cl(st.a + dx);
        st.b = _cl(st.b + dy);
      case _G.spin || _G.flick:
        var da = n.direction - o.direction;
        if (da > math.pi) da -= _tau;
        if (da < -math.pi) da += _tau;
        st.a = _wr(st.a + da / _tau);
        st.w = da / _tau * 60;
        if (p.g == _G.spin) st.b = _cl(st.b + (n.distance - o.distance) / (_ch * .5));
      case _G.flickY:
        st.a = _wr(st.a + dy * .5);
        st.w = dy * 30;
      case _G.rub:
        st.a = _cl(st.a + dx);
        st.b = _cl(st.b + d.delta.distance / (_cw * 3));
      case _G.pinch:
        st.a = _cl(st.a + (n.distance - o.distance) / (_ch * .45));
      case _G.stretch:
        st.a = _cl(st.a + dx * (n.dx >= 0 ? 2 : -2));
        st.b = _cl(st.b - dy * (n.dy >= 0 ? 2 : -2));
      case _G.corner:
        _corner(q);
      case _G.wind:
        if (d.delta.distance > .5) {
          var da = _wr(d.delta.direction / _tau) - st.a;
          if (da > .5) da -= 1;
          if (da < -.5) da += 1;
          st.a = _wr(st.a + da * .35);
          st.b = _cl(st.b + d.delta.distance / 200);
        }
    }
  }

  void _end(DragEndDetails d) {
    st
      ..down = false
      ..rel = t.value;
    final v = d.velocity.pixelsPerSecond;
    if (p.g == _G.flick) {
      final r = (st.at ?? _ctr) - _ctr;
      if (r.distanceSquared > 25) st.va = (r.dx * v.dy - r.dy * v.dx) / r.distanceSquared / _tau;
    } else if (p.g == _G.flickY) {
      st.va = -v.dy / _ch * .5;
    }
  }

  @override
  Widget build(BuildContext context) => SizedBox(
        width: _cw,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          GestureDetector(
            onPanStart: _start,
            onPanUpdate: _move,
            onPanEnd: _end,
            onTap: () => st.rel = t.value,
            child: ClipRRect(
              borderRadius: BorderRadius.circular(8),
              child: CustomPaint(size: const Size(_cw, _ch), painter: _P9Painter(p, st, t)),
            ),
          ),
          const SizedBox(height: 5),
          Text('$concept #$n  ${p.name}', style: _lab1, maxLines: 1, overflow: TextOverflow.ellipsis, softWrap: false),
          Text(p.tags, style: _lab2, maxLines: 1, overflow: TextOverflow.ellipsis, softWrap: false),
        ]),
      );
}

class _P9Painter extends CustomPainter {
  _P9Painter(this.p, this.st, this.t) : super(repaint: t);
  final _Pan9 p;
  final _S st;
  final ValueNotifier<double> t;
  @override
  void paint(Canvas canvas, Size size) => p.paint(canvas, size, st, t.value);
  @override
  bool shouldRepaint(covariant _P9Painter old) => true;
}
