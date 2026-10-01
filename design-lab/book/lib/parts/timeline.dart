// The Timeline, drawn as DESIGN.md section 4 says. Rows, bars, keys, link lines and the grid are one style each.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../tokens.dart';

/// The open questions of the timeline, each one a value the owner picks in the real window (research/precedents.md, section 5).
enum KeyEdge { white, black, none }

enum GridTone { light, dark, none }

class TimelineTuning {
  const TimelineTuning({this.rowHeight = 23, this.zebra = 12, this.keyEdge = KeyEdge.white, this.grid = GridTone.light});
  final double rowHeight;
  final int zebra; // grey levels between the two row bands: 12 (concept art), 8, 4 (Blender's shipped zebra is about 4)
  final KeyEdge keyEdge;
  final GridTone grid;
  double get bar => rowHeight * .87;
  double get radius => rowHeight * .09;
  double get key => rowHeight * .33;
  Color get bandA => N.bandA;
  Color get bandB => Color.lerp(N.bandA, N.bandB, zebra / 12)!;
}

class Layer {
  const Layer(this.name, this.fam, this.start, this.end, [this.keys = const []]);
  final String name;
  final Fam fam;
  final double start, end; // frames
  final List<double> keys;
}

const sampleLayers = [
  Layer('Scatter', Fam.scatter, 10, 92, [10, 40, 70]),
  Layer('Stagger', Fam.stagger, 4, 118, [4, 36, 60]),
  Layer('Along Path', Fam.along, 14, 118, [14, 33, 60]),
  Layer('Face', Fam.face, 0, 76, [0, 34]),
  Layer('Follow', Fam.follow, 23, 106),
  Layer('Attach', Fam.attach, 18, 118, [18, 82]),
];

class TimelinePanel extends StatelessWidget {
  const TimelinePanel({super.key, this.layers = sampleLayers, this.selected = 2, this.head = 60, this.pixelsPerFrame = 6, this.fps = 30, this.tuning = const TimelineTuning()});
  final TimelineTuning tuning;
  final List<Layer> layers;
  final int selected, head, fps;
  final double pixelsPerFrame;

  @override
  Widget build(BuildContext context) => Container(
        color: N.g10,
        child: Column(children: [
          SizedBox(height: TL.ruler, child: CustomPaint(size: Size.infinite, painter: _Ruler(pixelsPerFrame, fps, head))),
          for (var i = 0; i < layers.length; i++)
            SizedBox(height: tuning.rowHeight, child: Container(
              color: i.isEven ? tuning.bandA : tuning.bandB,
              child: Stack(children: [
                Positioned.fill(child: LayerRow(layer: layers[i], selected: i == selected, head: head, ppf: pixelsPerFrame, fps: fps, tuning: tuning)),
                Positioned(left: 0, right: 0, bottom: 0, height: 1, child: const ColoredBox(color: N.rowLine)),
              ]),
            )),
          Expanded(child: CustomPaint(size: Size.infinite, painter: _Grid(pixelsPerFrame, fps, tuning.grid))),
        ]),
      );
}

class LayerRow extends StatelessWidget {
  const LayerRow({super.key, required this.layer, required this.selected, required this.head, required this.ppf, required this.fps, this.tuning = const TimelineTuning()});
  final TimelineTuning tuning;
  final Layer layer;
  final bool selected;
  final int head, fps;
  final double ppf;
  @override
  Widget build(BuildContext context) => FlexRow(
        children: [
          SizedBox(width: TL.label, child: Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: FlexRow(children: [
            Container(width: 8, height: 8, decoration: BoxDecoration(color: layer.fam.c, borderRadius: BorderRadius.circular(2))),
            const SizedBox(width: 6),
            Text(layer.name, style: T.name(selected ? N.g100 : N.g91)),
          ]))),
          Expanded(child: CustomPaint(size: Size.infinite, painter: _Lane(layer, selected, head, ppf, fps, tuning))),
        ],
      );
}

/// A plain horizontal flex, named so this file reads without Flutter's `Row` clashing with the token class.
class FlexRow extends StatelessWidget {
  const FlexRow({super.key, required this.children});
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: children);
}

double _major(double ppf, int fps) {
  final steps = {1, 2, 5, 10, fps ~/ 2, fps, 2 * fps, 5 * fps, 10 * fps, 30 * fps, 60 * fps}.where((f) => f > 0).toList()..sort();
  return steps.firstWhere((f) => f * ppf >= 70, orElse: () => steps.last).toDouble();
}

double _fine(double ppf, int fps, double major) {
  final steps = {1, 2, 5, 10, fps ~/ 2, fps, 2 * fps, 5 * fps, 10 * fps}.where((f) => f > 0 && f < major && major % f == 0 && f * ppf >= 12);
  var best = major;
  for (final f in steps) {
    if ((f * ppf - 30).abs() < (best * ppf - 30).abs()) best = f.toDouble();
  }
  return best;
}

void _gridLines(Canvas c, Size s, double ppf, int fps, GridTone tone, {double top = 0}) {
  if (tone == GridTone.none) return;
  final major = _major(ppf, fps), fine = _fine(ppf, fps, major);
  final p = Paint();
  for (var f = 0.0; f * ppf < s.width; f += fine) {
    final x = (f * ppf).roundToDouble();
    c.drawRect(Rect.fromLTWH(x, top, 1, s.height - top), p..color = tone == GridTone.dark ? (f % major == 0 ? N.g07 : N.g07.withValues(alpha: .55)) : (f % major == 0 ? N.glaze15 : N.glaze9));
  }
}

class _Ruler extends CustomPainter {
  const _Ruler(this.ppf, this.fps, this.head);
  final double ppf;
  final int fps, head;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = N.g07);
    final x0 = TL.label, major = _major(ppf, fps), headX = x0 + head * ppf;
    for (var f = 0.0; x0 + f * ppf < s.width; f += major) {
      final x = x0 + f * ppf;
      final skip = (x + 18 - headX).abs() < 30; // a label under the head's pill is hidden, not overdrawn

      c.drawRect(Rect.fromLTWH(x.roundToDouble(), s.height - 6, 1, 6), Paint()..color = N.g44);
      if (skip) continue;
      final sec = (f / fps).floor(), fr = (f % fps).round();
      final label = major < fps ? '${(sec % 60).toString().padLeft(2, '0')}:${fr.toString().padLeft(2, '0')}' : '${(sec ~/ 60).toString().padLeft(2, '0')}:${(sec % 60).toString().padLeft(2, '0')}';
      final tp = TextPainter(text: TextSpan(text: label, style: T.value(N.g63).copyWith(fontSize: 9.5)), textDirection: TextDirection.ltr)..layout();
      tp.paint(c, Offset(x + 3, 3));
      tp.dispose();
    }
    // the playhead's head and its timecode, so the playhead reads in the ruler as well as in the lanes
    final hx = x0 + head * ppf, sec = head ~/ fps, fr = head % fps;
    final label = TextPainter(text: TextSpan(text: '${sec.toString().padLeft(2, '0')}:${fr.toString().padLeft(2, '0')}', style: T.value(N.g10).copyWith(fontSize: 10)), textDirection: TextDirection.ltr)..layout();
    final r = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(hx, s.height / 2 - 1), width: label.width + 10, height: 14), const Radius.circular(3));
    c.drawRRect(r, Paint()..color = C.playhead);
    label.paint(c, Offset(hx - label.width / 2, s.height / 2 - 1 - label.height / 2));
    label.dispose();
  }

  @override
  bool shouldRepaint(_Ruler o) => o.ppf != ppf || o.fps != fps || o.head != head;
}

class _Grid extends CustomPainter {
  const _Grid(this.ppf, this.fps, this.tone);
  final GridTone tone;
  final double ppf;
  final int fps;
  @override
  void paint(Canvas c, Size s) {
    c.save();
    c.translate(TL.label, 0);
    _gridLines(c, Size(s.width - TL.label, s.height), ppf, fps, tone);
    c.restore();
  }

  @override
  bool shouldRepaint(_Grid o) => o.ppf != ppf || o.fps != fps || o.tone != tone;
}

class _Lane extends CustomPainter {
  const _Lane(this.l, this.selected, this.head, this.ppf, this.fps, this.t);
  final TimelineTuning t;
  final Layer l;
  final bool selected;
  final int head, fps;
  final double ppf;

  @override
  void paint(Canvas c, Size s) {
    final cy = s.height / 2, fill = Paint();
    _gridLines(c, s, ppf, fps, t.grid);
    // the bar: family colour, calmed at rest, flat, a faint light line on the top and the bottom edge, never a shadow
    final tone = HSLColor.fromColor(l.fam.c);
    final colour = selected ? l.fam.c : tone.withSaturation(tone.saturation * .85).withLightness(tone.lightness * .93).toColor();
    final bar = Rect.fromLTRB(l.start * ppf, cy - t.bar / 2, math.max(l.end * ppf, l.start * ppf + 3), cy + t.bar / 2);
    final rr = RRect.fromRectAndRadius(bar, Radius.circular(t.radius));
    c.drawRRect(rr, fill..color = colour);
    c.save();
    c.clipRRect(rr);
    c.drawRect(Rect.fromLTWH(bar.left, bar.top, bar.width, 1), fill..color = N.g100.withValues(alpha: .16));
    c.drawRect(Rect.fromLTWH(bar.left, bar.bottom - 1, bar.width, 1), fill..color = N.g100.withValues(alpha: .16));
    c.restore();
    if (l.keys.length > 1) {
      c.drawRect(Rect.fromLTRB(l.keys.first * ppf, cy - .5, l.keys.last * ppf, cy + .5), fill..color = N.g100.withValues(alpha: selected ? .9 : .7));
    }
    for (final k in l.keys) {
      final at = k.round() == head, r = t.key / 2 * (at ? 1.2 : 1);
      final p = Path()..moveTo(k * ppf, cy - r)..lineTo(k * ppf + r, cy)..lineTo(k * ppf, cy + r)..lineTo(k * ppf - r, cy)..close();
      c.drawPath(p, fill..color = at ? C.playhead : N.g95.withValues(alpha: .9));
      if (t.keyEdge != KeyEdge.none) c.drawPath(p, Paint()..style = PaintingStyle.stroke..strokeWidth = at ? 1.5 : .75..color = t.keyEdge == KeyEdge.black ? N.g00.withValues(alpha: .85) : N.g100.withValues(alpha: at ? 1 : .6));
    }
    c.drawRect(Rect.fromLTWH(head * ppf, 0, 1, s.height), fill..color = C.playhead);
  }

  @override
  bool shouldRepaint(_Lane o) => true;
}
