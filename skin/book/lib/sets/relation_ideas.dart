// Tries for the Relations tab, side by side, not joined: one scene (six words in a row, a ring tipped back over it, the
// ring's strength 0 -> 1 -> 0 in turn) seen five ways, each borrowed from a tool people rate: a map (the tab now), nodes
// (Cavalry, Blender), a sequencer (Ableton's drum rack), an outliner (Blender), a map with its list. Each plays.
import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../tokens.dart';
import '../workspace/desk_kit.dart';
import '../workspace/ws.dart';

const _words = ['WE', 'MOVE', 'IN', 'TURN', 'ON', 'TIME'];
const _lag = .12, _length = 4.0;

/// The ring's strength at a moment: out with an overshoot, held, back home.
double _strength(double t) {
  if (t <= 0) return 0;
  if (t < 1.5) {
    final u = t / 1.5 - 1;
    return 1 + 2.7 * u * u * u + 1.7 * u * u;   // (an ease out with a back)
  }
  if (t < 2.5) return 1;
  if (t < 4) {
    final u = (t - 2.5) / 1.5;
    return 1 - u * u * (3 - 2 * u);
  }
  return 0;
}

double _s(int i, double t) => _strength(t - _lag * i);

typedef _P3 = ({double x, double y, double z});

_P3 _row(int i) => (x: -.75 + i * .3, y: 0, z: 0);
_P3 _ring(int i) {
  final a = i / _words.length * 2 * math.pi, tilt = 50 * math.pi / 180;
  return (x: .5 * math.sin(a), y: .5 * math.cos(a) * math.cos(tilt), z: .5 * math.cos(a) * math.sin(tilt));
}

_P3 _at(int i, double t) {
  final s = _s(i, t), a = _row(i), b = _ring(i);
  return (x: a.x + (b.x - a.x) * s, y: a.y + (b.y - a.y) * s, z: a.z + (b.z - a.z) * s);
}

Color _hue(int i) => wsLabel('${i + 1}');

/// A try's frame: its name, play, and the moment, over the view, which is drawn at that moment.
class _Try extends StatefulWidget {
  const _Try(this.name, this.view);
  final String name;
  final Widget Function(double t) view;
  @override
  State<_Try> createState() => _TryState();
}

class _TryState extends State<_Try> with SingleTickerProviderStateMixin {
  late final Ticker _tick;
  double _t = 0;
  bool _on = true;
  Duration _was = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tick = createTicker((d) {
      if (_on) setState(() => _t = (_t + (d - _was).inMicroseconds / 1e6) % (_length + 1));
      _was = d;
    })..start();
  }

  @override
  void dispose() {
    _tick.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: Grey.g07,
    child: Padding(
      padding: const EdgeInsets.all(12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          DeskStrip(
            left: [DeskLayerChip(widget.name, WsT.toneGroup)],
            right: [
              DeskChip(label: _on ? 'Pause' : 'Play', on: _on, onTap: () => setState(() => _on = !_on)),
              SizedBox(width: Shape.s(6)),
              SizedBox(
                width: 160,
                child: GestureDetector(
                  onHorizontalDragUpdate: (d) => setState(() {
                    _on = false;
                    _t = (d.localPosition.dx / 160 * _length).clamp(0, _length);
                  }),
                  child: CustomPaint(size: const Size(160, 18), painter: _Scrub(_t / _length)),
                ),
              ),
              SizedBox(width: Shape.s(6)),
              Text('${_t.clamp(0, _length).toStringAsFixed(2)} s', style: T.value(Grey.g76)),
            ],
          ),
          const SizedBox(height: WsT.gutter),
          Expanded(child: ColoredBox(color: WsT.card, child: widget.view(_t.clamp(0, _length)))),
        ],
      ),
    ),
  );
}

class _Scrub extends CustomPainter {
  const _Scrub(this.u);
  final double u;
  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromLTRBR(0, s.height / 2 - 2, s.width, s.height / 2 + 2, const Radius.circular(2)), Paint()..color = Grey.g26);
    c.drawCircle(Offset(u * s.width, s.height / 2), 5, Paint()..color = WsT.accent);
  }

  @override
  bool shouldRepaint(_Scrub o) => o.u != u;
}

/// 1. The map: the world turned a little from above, the ring and the row as broken lines, each word a dot with its
/// order number, the ring's things lit as much as the ring holds them.
class _Map extends CustomPainter {
  const _Map(this.t);
  final double t;
  @override
  void paint(Canvas c, Size s) {
    const yaw = .45, pitch = -.55;
    Offset on(_P3 p) {
      final x = p.x * math.cos(yaw) + p.z * math.sin(yaw), z = -p.x * math.sin(yaw) + p.z * math.cos(yaw);
      final y = p.y * math.cos(pitch) - z * math.sin(pitch), k = math.min(s.width, s.height) * .75;
      return Offset(s.width / 2 + x * k, s.height / 2 - y * k);
    }

    final line = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    Path through(List<_P3> ps, {bool close = false}) => Path()..addPolygon([for (final p in ps) on(p)], close);
    _broken(c, through([for (var i = 0; i < _words.length; i++) _row(i)]), line..color = Grey.g44);
    _broken(c, through([for (var k = 0; k < 48; k++) (x: .5 * math.sin(k / 48 * 2 * math.pi), y: .5 * math.cos(k / 48 * 2 * math.pi) * math.cos(50 * math.pi / 180), z: .5 * math.cos(k / 48 * 2 * math.pi) * math.sin(50 * math.pi / 180))], close: true), line..color = WsT.toneGroup);
    for (var i = 0; i < _words.length; i++) {
      final p = on(_at(i, t));
      c.drawCircle(p, 6, Paint()..color = _hue(i));
      _label(c, '${i + 1}', p + const Offset(9, -9), Grey.g76);
    }
  }

  @override
  bool shouldRepaint(_Map o) => o.t != t;
}

void _broken(Canvas c, Path p, Paint ink) {
  for (final m in p.computeMetrics()) {
    for (var d = 0.0; d < m.length; d += 9) {
      c.drawPath(m.extractPath(d, math.min(d + 5, m.length)), ink);
    }
  }
}

void _label(Canvas c, String text, Offset at, Color ink, {double ax = 0}) {
  final p = TextPainter(text: TextSpan(text: text, style: T.label(ink)), textDirection: TextDirection.ltr)..layout();
  p.paint(c, at - Offset(p.width * ax, p.height / 2));
  p.dispose();
}

/// 2. Nodes: the relations as boxes on the left, the words on the right, a wire from each relation to each of its
/// things, lit as much as it holds that thing now: in turn shows as the wires lighting one after another.
class _Nodes extends CustomPainter {
  const _Nodes(this.t);
  final double t;
  @override
  void paint(Canvas c, Size s) {
    final ring = Rect.fromLTWH(24, s.height * .22, 110, 34), row = Rect.fromLTWH(24, s.height * .62, 110, 34);
    for (final (r, name, hue) in [(ring, 'around', WsT.toneGroup), (row, 'row', Grey.g56)]) {
      c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(6)), Paint()..color = Grey.g20);
      c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(6)), Paint()..color = hue..style = PaintingStyle.stroke..strokeWidth = 1.5);
      _label(c, name, r.centerLeft + const Offset(10, 0), Grey.g91);
    }
    for (var i = 0; i < _words.length; i++) {
      final y = s.height * (.1 + .8 * i / (_words.length - 1)), box = Rect.fromLTWH(s.width - 130, y - 13, 100, 26);
      for (final (from, lit) in [(row.centerRight, .35), (ring.centerRight, _s(i, t).clamp(0.0, 1.0))]) {
        final to = box.centerLeft, mid = (from.dx + to.dx) / 2;
        c.drawPath(Path()..moveTo(from.dx, from.dy)..cubicTo(mid, from.dy, mid, to.dy, to.dx, to.dy), Paint()..color = _hue(i).withValues(alpha: .12 + .88 * lit)..style = PaintingStyle.stroke..strokeWidth = 1 + 2 * lit);
      }
      c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(6)), Paint()..color = _hue(i));
      _label(c, '${i + 1}  ${_words[i]}', box.centerLeft + const Offset(8, 0), Grey.g07);
    }
  }

  @override
  bool shouldRepaint(_Nodes o) => o.t != t;
}

/// 3. The sequencer: a line a word, the ring's strength drawn as the same clip on each, so much later each, the
/// playhead across; as a drum rack plays one pattern down its pads.
class _Sequencer extends CustomPainter {
  const _Sequencer(this.t);
  final double t;
  @override
  void paint(Canvas c, Size s) {
    const left = 90.0, top = 10.0;
    final w = s.width - left - 10, h = (s.height - top - 10) / _words.length;
    double x(double sec) => left + sec / _length * w;
    for (var i = 0; i < _words.length; i++) {
      final y0 = top + i * h;
      c.drawRect(Rect.fromLTWH(left, y0 + 1, w, h - 2), Paint()..color = i.isEven ? Grey.g10 : Grey.g13);
      _label(c, '${i + 1}  ${_words[i]}', Offset(8, y0 + h / 2), _hue(i));
      final clip = Path()..moveTo(x(0), y0 + h - 2);
      for (var k = 0; k <= 80; k++) {
        final sec = k / 80 * _length;
        clip.lineTo(x(sec), y0 + h - 2 - (h - 6) * _s(i, sec).clamp(0.0, 1.2) / 1.2);
      }
      clip
        ..lineTo(x(_length), y0 + h - 2)
        ..close();
      c.drawPath(clip, Paint()..color = _hue(i).withValues(alpha: .45));
    }
    c.drawLine(Offset(x(t), top), Offset(x(t), top + h * _words.length), Paint()..color = Grey.g95..strokeWidth = 1.5);
  }

  @override
  bool shouldRepaint(_Sequencer o) => o.t != t;
}

/// 4. The outliner: each relation a heading, its things under it in order, each with a meter of how much the relation
/// holds it now (as a mixer's level), the lag said once.
Widget _outliner(double t) => ListView(
  padding: const EdgeInsets.all(10),
  children: [
    for (final (name, hue, held) in [('around', WsT.toneGroup, (int i) => _s(i, t)), ('row', Grey.g56, (int i) => 1.0 - _s(i, t).clamp(0.0, 1.0))]) ...[
      Padding(
        padding: const EdgeInsets.only(top: 6, bottom: 4),
        child: Row(children: [Container(width: 8, height: 8, decoration: BoxDecoration(color: hue, shape: BoxShape.circle)), const SizedBox(width: 8), Text(name, style: T.label(Grey.g95)), const Spacer(), Text('Lag ${_lag.toStringAsFixed(2)} s', style: T.label(Grey.g56))]),
      ),
      for (var i = 0; i < _words.length; i++)
        Padding(
          padding: const EdgeInsets.only(left: 16, top: 2, bottom: 2),
          child: Row(
            children: [
              SizedBox(width: 18, child: Text('${i + 1}', style: T.label(Grey.g56))),
              Container(width: 8, height: 8, decoration: BoxDecoration(color: _hue(i), borderRadius: BorderRadius.circular(2))),
              const SizedBox(width: 8),
              SizedBox(width: 56, child: Text(_words[i], style: T.label(Grey.g91))),
              Expanded(child: LayoutBuilder(builder: (_, b) => Stack(children: [Container(height: 6, color: Grey.g20), Container(height: 6, width: b.maxWidth * held(i).clamp(0.0, 1.0), color: _hue(i))]))),
            ],
          ),
        ),
    ],
  ],
);

WidgetbookComponent relationIdeasSet() => WidgetbookComponent(
  name: 'Relations tab ideas',
  useCases: [
    WidgetbookUseCase(name: '1 Map', builder: (c) => _Try('Map', (t) => CustomPaint(painter: _Map(t), size: Size.infinite))),
    WidgetbookUseCase(name: '2 Nodes', builder: (c) => _Try('Nodes', (t) => CustomPaint(painter: _Nodes(t), size: Size.infinite))),
    WidgetbookUseCase(name: '3 Sequencer', builder: (c) => _Try('Sequencer', (t) => CustomPaint(painter: _Sequencer(t), size: Size.infinite))),
    WidgetbookUseCase(name: '4 Outliner', builder: (c) => _Try('Outliner', _outliner)),
    WidgetbookUseCase(
      name: '5 Map and list',
      builder: (c) => _Try('Map and list', (t) => Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [Expanded(flex: 3, child: CustomPaint(painter: _Map(t))), SizedBox(width: 260, child: _outliner(t))])),
    ),
  ],
);
