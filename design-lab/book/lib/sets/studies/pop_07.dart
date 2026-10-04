// set: Pop 07. Two SHEETS, each = one concept drawn as 20 different GUI ideas (5 x 4 panels, one screenshot shows all).
// Pop palette (flat vivid colour on dark panels, crisp vector lines), not the calm Role tokens. Each panel answers to a drag.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:widgetbook/widgetbook.dart';

part 'pop_07_blend.dart';
part 'pop_07_glass.dart';

WidgetbookComponent pop07Set() => WidgetbookComponent(name: 'Pop 07', useCases: [
      WidgetbookUseCase(name: 'Opacity / Blend x20', builder: (_) => _P7Sheet(concept: 'Opacity', specs: _blendSpecs)),
      WidgetbookUseCase(name: 'Glass / Refraction x20', builder: (_) => _P7Sheet(concept: 'Glass', specs: _glassSpecs)),
    ]);

const _k0 = Color(0xFF1C1C1E), _k1 = Color(0xFF2A2A2E), _cr = Color(0xFFF4EBDD), _wh = Color(0xFFFFFFFF);
const _or = Color(0xFFFF7A3D), _ye = Color(0xFFFFD23F), _cy = Color(0xFF3DD6F5), _li = Color(0xFF9BE564);
const _pk = Color(0xFFFF5DA2), _vi = Color(0xFF9B7BFF), _re = Color(0xFFFF4B4B), _bl = Color(0xFF2E5BFF);
const _ink = Color(0xFF141416);

Paint _f(Color c) => Paint()..color = c;
Paint _s(Color c, double w) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Color _o(Color c, double o) => c.withValues(alpha: o.clamp(0.0, 1.0));
double _h(int i) => (math.sin(i * 12.9898 + 4.1) * 43758.5453).abs() % 1;
double _cl(double v) => v.clamp(0.0, 1.0);

typedef _P7Paint = void Function(Canvas c, Size s, P7State st, double t);

class _P7Spec {
  const _P7Spec(this.name, this.tags, this.paint, {this.a = .5, this.b = .5, this.light = false, this.p = const Offset(.5, .5)});
  final String name, tags;
  final _P7Paint paint;
  final double a, b;
  final bool light;
  final Offset p;
}

/// Local toy state: a (drag right = up), b (drag up = up), p = last touch (0..1), rub = rubbed distance, spin = flickable position, marks = painted dabs.
class P7State {
  P7State(this.a, this.b, this.p);
  double a, b, rub = 0, spin = 0, spinV = 0;
  Offset p;
  bool down = false;
  final List<Offset> marks = [];
}

class _P7Sheet extends StatefulWidget {
  const _P7Sheet({required this.concept, required this.specs});
  final String concept;
  final List<_P7Spec> specs;
  @override
  State<_P7Sheet> createState() => _P7SheetSt();
}

class _P7SheetSt extends State<_P7Sheet> with SingleTickerProviderStateMixin {
  final _time = ValueNotifier<double>(0);
  late final List<P7State> _st = [for (final s in widget.specs) P7State(s.a, s.b, s.p)];
  late final Ticker _tk;
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tk = createTicker((d) {
      final dt = ((d - _last).inMicroseconds / 1e6).clamp(0.0, .05);
      _last = d;
      for (final s in _st) {
        if (!s.down) {
          s.spin += s.spinV * dt;
          s.spinV *= math.pow(.25, dt).toDouble();
        }
      }
      _time.value = d.inMicroseconds / 1e6;
    })
      ..start();
  }

  @override
  void dispose() {
    _tk.dispose();
    _time.dispose();
    super.dispose();
  }

  Widget _cell(int i) {
    final sp = widget.specs[i], st = _st[i];
    const size = Size(156, 118);
    return SizedBox(
      width: 156,
      height: 150,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        GestureDetector(
          onPanDown: (d) {
            st.down = true;
            st.spinV = 0;
            st.p = Offset(_cl(d.localPosition.dx / size.width), _cl(d.localPosition.dy / size.height));
          },
          onPanUpdate: (d) {
            st.a = _cl(st.a + d.delta.dx / 110);
            st.b = _cl(st.b - d.delta.dy / 90);
            st.p = Offset(_cl(d.localPosition.dx / size.width), _cl(d.localPosition.dy / size.height));
            st.rub = (st.rub + d.delta.distance / 900).clamp(0.0, 1.0);
            st.spin += d.delta.dx / 40;
            if (st.marks.isEmpty || (st.marks.last - st.p).distance > .03) {
              st.marks.add(st.p);
              if (st.marks.length > 160) st.marks.removeAt(0);
            }
          },
          onPanEnd: (d) {
            st.down = false;
            st.spinV = d.velocity.pixelsPerSecond.dx / 40;
          },
          onPanCancel: () => st.down = false,
          onDoubleTap: () {
            st
              ..a = sp.a
              ..b = sp.b
              ..p = sp.p
              ..rub = 0
              ..spin = 0
              ..spinV = 0
              ..marks.clear();
          },
          child: CustomPaint(size: size, painter: _P7Painter(sp, st, _time)),
        ),
        const SizedBox(height: 4),
        Text('${widget.concept} #${i + 1}  ${sp.name}',
            maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 10, height: 1.2, color: _cr, fontWeight: FontWeight.w600)),
        Text(sp.tags,
            maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 10, height: 1.2, color: Color(0xFF8E8A84))),
      ]),
    );
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: const Color(0xFF111113),
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(12),
            child: Column(mainAxisSize: MainAxisSize.min, children: [
              for (var r = 0; r < 4; r++)
                Padding(
                  padding: const EdgeInsets.only(bottom: 6),
                  child: Row(mainAxisSize: MainAxisSize.min, children: [
                    for (var c = 0; c < 5; c++) ...[if (c > 0) const SizedBox(width: 8), _cell(r * 5 + c)],
                  ]),
                ),
            ]),
          ),
        ),
      );
}

class _P7Painter extends CustomPainter {
  _P7Painter(this.sp, this.st, this.time) : super(repaint: time);
  final _P7Spec sp;
  final P7State st;
  final ValueNotifier<double> time;

  @override
  void paint(Canvas c, Size s) {
    final rr = RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(10));
    c.save();
    c.clipRRect(rr);
    c.drawRect(Offset.zero & s, _f(sp.light ? _cr : _k0));
    sp.paint(c, s, st, time.value);
    c.restore();
  }

  @override
  bool shouldRepaint(_P7Painter o) => true;
}
