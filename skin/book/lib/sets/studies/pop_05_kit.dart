// Pop 05 kit: one sheet = 20 panels (5 x 4) for one concept, one shared Ticker per sheet.
// A panel owns two values a, b in 0..1, a mode m (tap cycles it when the panel has modes) and a phase ph that advances at rate(p) per second.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';

const kP0 = Color(0xFF1C1C1E), kP1 = Color(0xFF2A2A2E), kCream = Color(0xFFF4EBDD), kWhite = Color(0xFFFFFFFF);
const kOrange = Color(0xFFFF7A3D), kYellow = Color(0xFFFFD23F), kCyan = Color(0xFF3DD6F5), kLime = Color(0xFF9BE564);
const kPink = Color(0xFFFF5DA2), kViolet = Color(0xFF9B7BFF), kRed = Color(0xFFFF4B4B), kBlue = Color(0xFF2E5BFF);
const kInk = Color(0xFF1C1C1E);

/// How a drag turns into values. h: x -> a. v: y -> a. hv: x -> a, y -> b. spin: angle around centre -> a. spinB: angle -> b (wraps), y -> a.
enum G { h, v, hv, spin, spinB }

class PopP {
  PopP(this.a, this.b, this.m);
  double a, b, ph = 0, t = 0;
  int m;
  bool down = false;
}

class PopSpec {
  const PopSpec(this.name, this.tags, this.g, this.paint, {this.a = .5, this.b = .5, this.modes = 0, this.m = 0, this.wrap = false});
  final String name, tags;
  final G g;
  final void Function(Canvas c, Size s, PopP p) paint;
  final double a, b;
  final int modes, m;
  final bool wrap;
}

class PopSheet extends StatefulWidget {
  const PopSheet({super.key, required this.concept, required this.panels, this.rate});
  final String concept;
  final List<PopSpec> panels;
  final double Function(PopP p)? rate;
  @override
  State<PopSheet> createState() => _PopSheetState();
}

class _PopSheetState extends State<PopSheet> with SingleTickerProviderStateMixin {
  final time = ValueNotifier<double>(0);
  late final Ticker _tk;
  late final List<PopP> ps = [for (final s in widget.panels) PopP(s.a, s.b, s.m)];
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tk = createTicker((e) {
      final dt = ((e - _last).inMicroseconds / 1e6).clamp(0.0, .1);
      _last = e;
      for (final p in ps) {
        p.t += dt;
        p.ph += dt * (widget.rate?.call(p) ?? 1);
      }
      time.value = e.inMicroseconds / 1e6;
    })..start();
  }

  @override
  void dispose() {
    _tk.dispose();
    time.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: const Color(0xFF111113),
        child: Align(
          alignment: Alignment.topLeft,
          child: Padding(
            padding: const EdgeInsets.all(10),
            child: Wrap(spacing: 8, runSpacing: 6, children: [
              for (var i = 0; i < ps.length; i++) _Panel(concept: widget.concept, n: i + 1, spec: widget.panels[i], p: ps[i], time: time),
            ]),
          ),
        ),
      );
}

class _Panel extends StatefulWidget {
  const _Panel({required this.concept, required this.n, required this.spec, required this.p, required this.time});
  final String concept;
  final int n;
  final PopSpec spec;
  final PopP p;
  final ValueNotifier<double> time;
  @override
  State<_Panel> createState() => _PanelState();
}

class _PanelState extends State<_Panel> {
  static const art = Size(156, 120);
  double _ang = 0;

  double _angle(Offset l) => math.atan2(l.dy - art.height / 2, l.dx - art.width / 2);

  void _drag(DragUpdateDetails d) {
    final p = widget.p, s = widget.spec;
    double fit(double v) => s.wrap ? v - v.floorToDouble() : v.clamp(0.0, 1.0);
    setState(() {
      switch (s.g) {
        case G.h:
          p.a = fit(p.a + d.delta.dx / 130);
        case G.v:
          p.a = fit(p.a - d.delta.dy / 100);
        case G.hv:
          p.a = fit(p.a + d.delta.dx / 130);
          p.b = (p.b - d.delta.dy / 100).clamp(0.0, 1.0);
        case G.spin || G.spinB:
          final a1 = _angle(d.localPosition);
          var da = a1 - _ang;
          if (da > math.pi) da -= 2 * math.pi;
          if (da < -math.pi) da += 2 * math.pi;
          _ang = a1;
          if (s.g == G.spin) {
            p.a = fit(p.a + da / (2 * math.pi));
          } else {
            p.b = (p.b + da / (2 * math.pi)) % 1.0;
            p.a = (p.a - d.delta.dy / 160).clamp(0.0, 1.0);
          }
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final s = widget.spec, p = widget.p;
    return SizedBox(
      width: 156,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        GestureDetector(
          onPanStart: (d) => setState(() {
            p.down = true;
            _ang = _angle(d.localPosition);
          }),
          onPanUpdate: _drag,
          onPanEnd: (_) => setState(() => p.down = false),
          onTap: s.modes > 0 ? () => setState(() => p.m = (p.m + 1) % s.modes) : null,
          child: ClipRRect(
            borderRadius: BorderRadius.circular(6),
            child: CustomPaint(size: art, painter: _Painter(s, p, widget.time)),
          ),
        ),
        const SizedBox(height: 3),
        Text('${widget.concept} #${widget.n}  ${s.name}',
            maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 10, height: 1.2, color: kCream, fontWeight: FontWeight.w600)),
        Text(s.tags, maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 10, height: 1.2, color: Color(0xFF8E8A84))),
      ]),
    );
  }
}

class _Painter extends CustomPainter {
  _Painter(this.s, this.p, Listenable t) : super(repaint: t);
  final PopSpec s;
  final PopP p;
  @override
  void paint(Canvas c, Size size) => s.paint(c, size, p);
  @override
  bool shouldRepaint(_Painter o) => true;
}

// ---- drawing helpers ----
Paint fl(Color c) => Paint()..color = c;
Paint st(Color c, [double w = 2]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Color al(Color c, double o) => c.withValues(alpha: o.clamp(0.0, 1.0));
void bg(Canvas c, Size s, [Color col = kP1]) => c.drawRect(Offset.zero & s, fl(col));
void vgrad(Canvas c, Rect r, Color a, Color b) => c.drawRect(r, Paint()..shader = ui.Gradient.linear(r.topCenter, r.bottomCenter, [a, b]));
Path poly(List<Offset> pts, {bool close = true}) {
  final path = Path()..moveTo(pts.first.dx, pts.first.dy);
  for (final q in pts.skip(1)) {
    path.lineTo(q.dx, q.dy);
  }
  if (close) path.close();
  return path;
}

double lerp(double a, double b, double t) => a + (b - a) * t;
double frac(double x) => x - x.floorToDouble();
Offset pol(Offset c, double r, double a, [double sy = 1]) => c + Offset(math.cos(a) * r, math.sin(a) * r * sy);
double tri(double x) => 1 - (2 * frac(x) - 1).abs();
