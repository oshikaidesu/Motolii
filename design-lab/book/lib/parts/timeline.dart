// The Timeline, drawn as DESIGN.md section 4 says. Rows, bars, keys, link lines and the grid are one style each.
import 'dart:math' as math;

import 'package:flutter/services.dart' show HardwareKeyboard;
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
  const Layer(this.name, this.fam, this.start, this.end, [this.keys = const [], this.pick = const {}]);
  final Set<double> pick; // the picked keys (frames): filled with the accent, a white edge
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
  const TimelinePanel({
    super.key,
    this.layers = sampleLayers,
    this.selected = 2,
    this.head = 60,
    this.pixelsPerFrame = 6,
    this.scroll = 0,
    this.fps = 30,
    this.tuning = const TimelineTuning(),
    this.picks = const {},
    this.hover,
    this.ring,
    this.onPick,
    this.onHover,
    this.onEmpty,
    this.onKey,
    this.onScrub,
  });
  final TimelineTuning tuning;
  final List<Layer> layers;
  final int selected, head, fps;
  final double pixelsPerFrame;
  final double scroll; // the frame at the left edge of the lanes (the view is panned and zoomed by the owner)

  /// Interaction (all optional; without them the panel is the still picture it was). [picks] = every selected row, [hover] = the row under the pointer
  /// (in this or any other panel), [ring] = the row that shows the keyboard focus ring. A click on a label, a bar or any part of a row calls [onPick];
  /// a click on the ruler or on the ground below the rows calls [onEmpty].
  final Set<int> picks;
  final int? hover, ring;
  final void Function(int index, bool add)? onPick;
  final ValueChanged<int?>? onHover;
  final VoidCallback? onEmpty;

  /// A press on a key diamond (row, frame); a press or drag on the ruler scrubs the playhead to a whole frame.
  final void Function(int row, double frame)? onKey;
  final ValueChanged<int>? onScrub;
  int _frameAt(double x) => math.max(0, ((x - TL.label) / pixelsPerFrame + scroll).round());

  @override
  Widget build(BuildContext context) => Container(
        color: N.g10,
        child: Column(children: [
          GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTapDown: onScrub != null ? (d) => onScrub!(_frameAt(d.localPosition.dx)) : (onEmpty == null ? null : (_) => onEmpty!()),
            onHorizontalDragUpdate: onScrub == null ? null : (d) => onScrub!(_frameAt(d.localPosition.dx)),
            child: SizedBox(height: TL.ruler, child: CustomPaint(size: Size.infinite, painter: _Ruler(pixelsPerFrame, fps, head, scroll))),
          ),
          for (var i = 0; i < layers.length; i++) _row(i),
          Expanded(child: GestureDetector(behavior: HitTestBehavior.opaque, onTapDown: onEmpty == null ? null : (_) => onEmpty!(), child: CustomPaint(size: Size.infinite, painter: _Grid(pixelsPerFrame, fps, tuning.grid, scroll)))),
        ]),
      );

  Widget _row(int i) {
    final sel = i == selected || picks.contains(i), hov = hover == i, interactive = onPick != null;
    Widget row = SizedBox(
      height: tuning.rowHeight,
      child: Container(
        color: i.isEven ? tuning.bandA : tuning.bandB,
        child: Stack(children: [
          // the label cell's state (DESIGN.md: rows and cells = a g20 fill and a 2px g95 tick; hover = g15), eased by the one shared motion
          if (interactive || onHover != null) ...[
            Positioned(left: 0, top: 0, bottom: 0, width: TL.label, child: AnimatedContainer(duration: Mo.dur, curve: Mo.ease, color: sel ? N.g20 : (hov ? N.g15 : const Color(0x00262626)))),
            Positioned(left: 0, top: 0, bottom: 0, width: 2, child: AnimatedContainer(duration: Mo.dur, curve: Mo.ease, color: sel ? N.g95 : const Color(0x00F2F2F2))),
          ],
          Positioned.fill(child: LayerRow(layer: layers[i], selected: sel, hover: hov, head: head, ppf: pixelsPerFrame, fps: fps, tuning: tuning, scroll: scroll)),
          Positioned(left: 0, right: 0, bottom: 0, height: 1, child: const ColoredBox(color: N.rowLine)),
          if (ring == i) Positioned.fill(child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: Role.selected, width: 1))))),
        ]),
      ),
    );
    if (!interactive && onHover == null) return row;
    row = GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTapDown: onPick == null
          ? null
          : (d) {
              final x = d.localPosition.dx - TL.label;
              if (onKey != null && x > 0) {
                for (final k in layers[i].keys) {
                  if ((x - k * pixelsPerFrame).abs() <= 7) {
                    onKey!(i, k);
                    return;
                  }
                }
              }
              onPick!(i, HardwareKeyboard.instance.isShiftPressed || HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed);
            },
      child: row,
    );
    return MouseRegion(opaque: false, cursor: SystemMouseCursors.click, onEnter: (_) => onHover?.call(i), onExit: (_) => onHover?.call(null), child: row);
  }
}

class LayerRow extends StatelessWidget {
  const LayerRow({super.key, required this.layer, required this.selected, required this.head, required this.ppf, required this.fps, this.tuning = const TimelineTuning(), this.hover = false, this.scroll = 0});
  final TimelineTuning tuning;
  final Layer layer;
  final bool selected, hover;
  final int head, fps;
  final double ppf, scroll;
  @override
  Widget build(BuildContext context) => FlexRow(
        children: [
          SizedBox(width: TL.label, child: Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: FlexRow(children: [
            Container(width: 8, height: 8, decoration: BoxDecoration(color: layer.fam.c, borderRadius: BorderRadius.circular(2))),
            const SizedBox(width: 6),
            Text(layer.name, style: T.name(selected ? N.g100 : N.g91)),
          ]))),
          Expanded(child: TweenAnimationBuilder<double>(
            tween: Tween(end: selected ? 1.0 : (hover ? .5 : 0.0)),
            duration: Mo.dur,
            curve: Mo.ease,
            builder: (_, level, _) => CustomPaint(size: Size.infinite, painter: _Lane(layer, selected, head, ppf, fps, tuning, level, scroll)),
          )),
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
  const _Ruler(this.ppf, this.fps, this.head, this.scroll);
  final double ppf, scroll;
  final int fps, head;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = N.g07);
    c.save();
    c.clipRect(Rect.fromLTWH(TL.label, 0, s.width - TL.label, s.height));
    final x0 = TL.label - scroll * ppf, major = _major(ppf, fps);
    final hx = x0 + head * ppf, hsec = head ~/ fps, hfr = head % fps;
    final label = TextPainter(text: TextSpan(text: '${hsec.toString().padLeft(2, '0')}:${hfr.toString().padLeft(2, '0')}', style: T.value(N.g10).copyWith(fontSize: 10)), textDirection: TextDirection.ltr)..layout();
    final pill = label.width + 10;
    for (var f = 0.0; x0 + f * ppf < s.width; f += major) {
      final x = x0 + f * ppf;
      c.drawRect(Rect.fromLTWH(x.roundToDouble(), s.height - 6, 1, 6), Paint()..color = N.g44);
      final sec = (f / fps).floor(), fr = (f % fps).round();
      final text = major < fps ? '${(sec % 60).toString().padLeft(2, '0')}:${fr.toString().padLeft(2, '0')}' : '${(sec ~/ 60).toString().padLeft(2, '0')}:${(sec % 60).toString().padLeft(2, '0')}';
      final tp = TextPainter(text: TextSpan(text: text, style: T.value(N.g63).copyWith(fontSize: 9.5)), textDirection: TextDirection.ltr)..layout();
      // a label that would touch the head's pill is hidden, not overdrawn
      if (x + 3 + tp.width > hx - pill / 2 - 3 && x + 3 < hx + pill / 2 + 3) {
        tp.dispose();
        continue;
      }
      tp.paint(c, Offset(x + 3, 3));
      tp.dispose();
    }
    // the playhead's head and its timecode, so the playhead reads in the ruler as well as in the lanes
    c.restore();
    // the chip is drawn outside the clip, so at frame 0 it is whole (the label column's ruler is empty); it is hidden only when scrolled clear away
    if (hx + pill / 2 > TL.label - 24 && hx - pill / 2 < s.width) {
      final r = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(hx, s.height / 2 - 1), width: pill, height: 14), const Radius.circular(3));
      c.drawRRect(r, Paint()..color = C.playhead);
      label.paint(c, Offset(hx - label.width / 2, s.height / 2 - 1 - label.height / 2));
    }
    label.dispose();
  }

  @override
  bool shouldRepaint(_Ruler o) => o.ppf != ppf || o.fps != fps || o.head != head || o.scroll != scroll;
}

class _Grid extends CustomPainter {
  const _Grid(this.ppf, this.fps, this.tone, this.scroll);
  final GridTone tone;
  final double ppf, scroll;
  final int fps;
  @override
  void paint(Canvas c, Size s) {
    c.save();
    c.clipRect(Rect.fromLTWH(TL.label, 0, s.width - TL.label, s.height));
    c.translate(TL.label - scroll * ppf, 0);
    _gridLines(c, Size(s.width - TL.label + scroll * ppf, s.height), ppf, fps, tone);
    c.restore();
  }

  @override
  bool shouldRepaint(_Grid o) => o.ppf != ppf || o.fps != fps || o.tone != tone || o.scroll != scroll;
}

class _Lane extends CustomPainter {
  const _Lane(this.l, this.selected, this.head, this.ppf, this.fps, this.t, this.level, [this.scroll = 0]);
  final double scroll;
  final double level; // 0 at rest, .5 hovered, 1 selected: the bar's colour eases between them (DESIGN.md: rest s x.85 l x.93, hover s x.95, selected as is)
  final TimelineTuning t;
  final Layer l;
  final bool selected;
  final int head, fps;
  final double ppf;

  @override
  void paint(Canvas c, Size s) {
    final cy = s.height / 2, fill = Paint();
    c.save();
    c.clipRect(Offset.zero & s);
    c.translate(-scroll * ppf, 0);
    _gridLines(c, Size(s.width + scroll * ppf, s.height), ppf, fps, t.grid);
    // the bar: family colour, calmed at rest, flat, a faint light line on the top and the bottom edge, never a shadow
    final tone = HSLColor.fromColor(l.fam.c);
    final rest = tone.withSaturation(tone.saturation * .85).withLightness(tone.lightness * .93).toColor(), hov = tone.withSaturation(tone.saturation * .95).withLightness(tone.lightness * .96).toColor();
    final colour = level <= .5 ? Color.lerp(rest, hov, level * 2)! : Color.lerp(hov, l.fam.c, (level - .5) * 2)!;
    final bar = Rect.fromLTRB(l.start * ppf, cy - t.bar / 2, math.max(l.end * ppf, l.start * ppf + 3), cy + t.bar / 2);
    final rr = RRect.fromRectAndRadius(bar, Radius.circular(t.radius));
    c.drawRRect(rr, fill..color = colour);
    c.save();
    c.clipRRect(rr);
    c.drawRect(Rect.fromLTWH(bar.left, bar.top, bar.width, 1), fill..color = N.g100.withValues(alpha: .16));
    c.drawRect(Rect.fromLTWH(bar.left, bar.bottom - 1, bar.width, 1), fill..color = N.g100.withValues(alpha: .16));
    c.restore();
    if (level > .5) c.drawRRect(rr.deflate(.5), Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = N.g95.withValues(alpha: (level - .5) * 2)); // the selected bar's own outline
    if (l.keys.length > 1) {
      c.drawRect(Rect.fromLTRB(l.keys.first * ppf, cy - .5, l.keys.last * ppf, cy + .5), fill..color = N.g100.withValues(alpha: selected ? .9 : .7));
    }
    for (final k in l.keys) {
      final at = k.round() == head, picked = l.pick.contains(k), r = t.key / 2 * (at || picked ? 1.2 : 1);
      final p = Path()..moveTo(k * ppf, cy - r)..lineTo(k * ppf + r, cy)..lineTo(k * ppf, cy + r)..lineTo(k * ppf - r, cy)..close();
      c.drawPath(p, fill..color = picked ? Role.selected : (at ? C.playhead : Role.of(N.g95, Role.key).withValues(alpha: .9)));
      if (picked) {
        c.drawPath(p, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.5..color = N.g100);
      } else if (Role.keyEdge != null) {
        c.drawPath(p, Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = Role.keyEdge!);
      } else if (t.keyEdge != KeyEdge.none) {
        c.drawPath(p, Paint()..style = PaintingStyle.stroke..strokeWidth = at ? 1.5 : .75..color = t.keyEdge == KeyEdge.black ? N.g00.withValues(alpha: .85) : N.g100.withValues(alpha: at ? 1 : .6));
      }
    }
    c.drawRect(Rect.fromLTWH(head * ppf, 0, 1, s.height), fill..color = C.playhead);
    c.restore();
  }

  @override
  bool shouldRepaint(_Lane o) => true;
}
