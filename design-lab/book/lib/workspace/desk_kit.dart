// The desk's shared small things: the tool catalog (name, hue, what it acts on), each tool's painted glyph, and the round
// touchables the desk's bodies use (chip, icon button, scrub field, readout).
import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_parts.dart' show Hov, Pop;
import '../tokens.dart';
import 'ws.dart';

/// One tool the desk can hold. [acts] is what it works on, said in two words.
class DeskTool {
  const DeskTool(this.name, this.tone, this.acts);
  final String name, acts;
  final Color tone;
}

const deskTools = [
  DeskTool('Ease', Pop.toneTime, 'Picked keys'),
  DeskTool('Depth', WsT.toneCamera, 'Camera'),
  DeskTool('Blend', Pop.toneBlend, 'Layer stack'),
  DeskTool('Align', Pop.toneLayout, 'Selection'),
  DeskTool('Sequence', Pop.tonePlace, 'Layers in time'),
  DeskTool('Path', Pop.tonePath, 'Motion path'),
  DeskTool('Wiggle', Pop.toneParticles, 'Noise'),
  DeskTool('Color', Pop.toneSolid, 'Swatches'),
  DeskTool('Audio', WsT.toneAudio, 'Beat markers'),
  DeskTool('Markers', Pop.toneTextAnim, 'Comp time'),
  DeskTool('Notes', Pop.toneText, 'Comments'),
  DeskTool('Links', Pop.toneLink, 'Relations'),
];

DeskTool deskTool(String name) => deskTools.firstWhere((t) => t.name == name, orElse: () => DeskTool(name, Grey.g63, ''));

/// What the selection alone would put on the desk (Ws.desk without a hand-opened tool).
String deskAuto(Ws ws) => ws.keys.isNotEmpty ? 'Ease' : (ws.layer?.kind == WsKind.camera ? 'Depth' : 'Tools');

/// A tool's sticker: a flat tile of its hue with the glyph knocked out in dark ink.
class DeskSticker extends StatelessWidget {
  const DeskSticker(this.name, {super.key, this.size = 16});
  final String name;
  final double size;
  @override
  Widget build(BuildContext context) {
    final tone = name == 'Tools' ? Grey.g76 : deskTool(name).tone;
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(color: tone, borderRadius: BorderRadius.circular(size * .3)),
      child: CustomPaint(painter: DeskGlyph(name, WsT.onAccent)),
    );
  }
}

/// Each tool drawn as one bold shape on a 20-unit grid.
class DeskGlyph extends CustomPainter {
  const DeskGlyph(this.name, this.ink);
  final String name;
  final Color ink;

  @override
  void paint(Canvas c, Size s) {
    final u = s.width / 20;
    Offset p(double x, double y) => Offset(x * u, y * u);
    final fill = Paint()..color = ink;
    final line = Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.8 * u
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    RRect bar(double x, double y, double w, double h) => RRect.fromRectAndRadius(Rect.fromLTWH(x * u, y * u, w * u, h * u), Radius.circular(u));
    switch (name) {
      case 'Ease':
        c.drawPath(
          Path()
            ..moveTo(4 * u, 15 * u)
            ..cubicTo(11 * u, 15 * u, 9 * u, 5 * u, 16 * u, 5 * u),
          line,
        );
        c.drawCircle(p(4, 15), 2 * u, fill);
        c.drawCircle(p(16, 5), 2 * u, fill);
      case 'Depth':
        c.drawPath(
          Path()
            ..moveTo(10 * u, 16 * u)
            ..lineTo(4 * u, 4 * u)
            ..lineTo(16 * u, 4 * u)
            ..close(),
          line..strokeWidth = 1.4 * u,
        );
        c.drawRRect(bar(6, 6.5, 8, 2), fill);
        c.drawRRect(bar(7.5, 10.5, 5, 2), fill);
      case 'Blend':
        c.drawCircle(p(8, 10), 4.5 * u, fill);
        c.drawCircle(p(12.5, 10), 4.5 * u, line..strokeWidth = 1.5 * u);
      case 'Align':
        c.drawLine(p(4.5, 3.5), p(4.5, 16.5), line);
        c.drawRRect(bar(6.5, 5, 9, 3), fill);
        c.drawRRect(bar(6.5, 11.5, 6, 3), fill);
      case 'Sequence':
        for (var i = 0; i < 3; i++) {
          c.drawRRect(bar(3.5 + i * 3.5, 4 + i * 4.3, 8, 3), fill);
        }
      case 'Path':
        c.drawPath(
          Path()
            ..moveTo(4 * u, 15 * u)
            ..quadraticBezierTo(4 * u, 5 * u, 10 * u, 9 * u)
            ..quadraticBezierTo(16 * u, 13 * u, 16 * u, 4 * u),
          line..strokeWidth = 1.5 * u,
        );
        c.drawCircle(p(4, 15), 2 * u, fill);
        c.drawRRect(bar(14, 2.5, 4, 4), fill);
      case 'Wiggle':
        c.drawPath(
          Path()
            ..moveTo(3 * u, 10 * u)
            ..lineTo(6 * u, 5 * u)
            ..lineTo(9 * u, 14 * u)
            ..lineTo(12 * u, 6 * u)
            ..lineTo(15 * u, 13 * u)
            ..lineTo(17 * u, 9 * u),
          line,
        );
      case 'Color':
        c.drawCircle(p(7, 7.5), 3.2 * u, fill);
        c.drawCircle(p(13, 7.5), 3.2 * u, line..strokeWidth = 1.4 * u);
        c.drawCircle(p(10, 13), 3.2 * u, fill);
      case 'Audio':
        const hs = <double>[4, 8, 12, 7, 10, 5];
        for (final (i, h) in hs.indexed) {
          c.drawRRect(bar(3 + i * 2.5, 10 - h / 2, 1.6, h), fill);
        }
      case 'Markers':
        c.drawLine(p(3, 15), p(17, 15), line..strokeWidth = 1.4 * u);
        for (final x in [5.0, 13.0]) {
          c.drawPath(
            Path()
              ..moveTo(x * u, 12 * u)
              ..lineTo((x - 2.5) * u, 9 * u)
              ..lineTo((x - 2.5) * u, 4 * u)
              ..lineTo((x + 2.5) * u, 4 * u)
              ..lineTo((x + 2.5) * u, 9 * u)
              ..close(),
            fill,
          );
        }
      case 'Notes':
        c.drawRRect(bar(4, 3.5, 12, 13), line..strokeWidth = 1.4 * u);
        c.drawRRect(bar(6.5, 7, 7, 1.6), fill);
        c.drawRRect(bar(6.5, 10.5, 5, 1.6), fill);
      case 'Links':
        c.drawRRect(bar(2.5, 7, 9, 6), line..strokeWidth = 1.5 * u);
        c.drawRRect(bar(8.5, 7, 9, 6), line);
      case 'Tools':
        for (var i = 0; i < 4; i++) {
          c.drawRRect(bar(4 + (i % 2) * 6.5, 4 + (i ~/ 2) * 6.5, 5.5, 5.5), fill);
        }
      default:
        c.drawCircle(p(10, 10), 3 * u, fill);
    }
  }

  @override
  bool shouldRepaint(DeskGlyph o) => o.name != name || o.ink != ink;
}

/// A round header chip: a mark and a word. [onTap] makes it a button.
class DeskChip extends StatelessWidget {
  const DeskChip({super.key, required this.label, this.dot, this.onTap, this.on = false});
  final String label;
  final Color? dot;
  final VoidCallback? onTap;
  final bool on;
  @override
  Widget build(BuildContext context) => Hov(
    onTap: onTap,
    cursor: onTap == null ? MouseCursor.defer : SystemMouseCursors.click,
    builder: (_, h) => Container(
      height: 18,
      padding: const EdgeInsets.symmetric(horizontal: 6),
      decoration: BoxDecoration(color: on ? WsT.accent : (h && onTap != null ? WsT.raised : WsT.well), borderRadius: BorderRadius.circular(WsT.chipRadius)),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (dot != null) ...[
            Container(
              width: 6,
              height: 6,
              decoration: BoxDecoration(color: on ? WsT.onAccent : dot, shape: BoxShape.circle),
            ),
            const SizedBox(width: 5),
          ],
          Flexible(
            child: Text(
              label,
              maxLines: 1,
              softWrap: false,
              overflow: TextOverflow.ellipsis,
              style: T.micro(on ? WsT.onAccent : (h && onTap != null ? Grey.g95 : Grey.g76)),
            ),
          ),
        ],
      ),
    ),
  );
}

/// A square-ish round button holding one glyph.
class DeskIconButton extends StatelessWidget {
  const DeskIconButton({super.key, required this.glyph, required this.onTap, this.on = false, this.size = 20});
  final CustomPainter Function(Color ink) glyph;
  final VoidCallback onTap;
  final bool on;
  final double size;
  @override
  Widget build(BuildContext context) => Hov(
    onTap: onTap,
    builder: (_, h) => Container(
      width: size,
      height: size,
      decoration: BoxDecoration(color: on ? WsT.accent : (h ? WsT.raised : WsT.well), borderRadius: BorderRadius.circular(WsT.radius)),
      child: CustomPaint(painter: glyph(on ? WsT.onAccent : (h ? Grey.g95 : Grey.g76))),
    ),
  );
}

/// A field you scrub sideways: label left, value right, a fill bar in the field's [tone] shows where the value sits in its range.
class DeskScrub extends StatefulWidget {
  const DeskScrub({
    super.key,
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.onChanged,
    required this.format,
    this.perPx = 1,
    this.tone = WsT.accent,
    this.enabled = true,
    this.pad = 7,
  });
  final String label;
  final double value, min, max, perPx, pad;
  final ValueChanged<double> onChanged;
  final String Function(double) format;
  final Color tone;
  final bool enabled;
  @override
  State<DeskScrub> createState() => _DeskScrubState();
}

class _DeskScrubState extends State<DeskScrub> {
  bool _hot = false, _drag = false;
  @override
  Widget build(BuildContext context) {
    final w = widget, f = ((w.value - w.min) / (w.max - w.min)).clamp(0.0, 1.0);
    final live = _hot || _drag;
    return MouseRegion(
      cursor: w.enabled ? SystemMouseCursors.resizeLeftRight : MouseCursor.defer,
      onEnter: (_) => setState(() => _hot = true),
      onExit: (_) => setState(() => _hot = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onHorizontalDragStart: w.enabled ? (_) => setState(() => _drag = true) : null,
        onHorizontalDragUpdate: w.enabled ? (d) => w.onChanged((w.value + d.delta.dx * w.perPx).clamp(w.min, w.max)) : null,
        onHorizontalDragEnd: w.enabled ? (_) => setState(() => _drag = false) : null,
        child: Container(
          height: 22,
          decoration: BoxDecoration(color: live && w.enabled ? WsT.raised : WsT.well, borderRadius: BorderRadius.circular(WsT.radius)),
          clipBehavior: Clip.antiAlias,
          child: Stack(
            children: [
              if (w.enabled)
                Positioned(
                  left: 4,
                  right: 4,
                  bottom: 2,
                  height: 2,
                  child: FractionallySizedBox(
                    alignment: Alignment.centerLeft,
                    widthFactor: f,
                    child: DecoratedBox(
                      decoration: BoxDecoration(color: _drag ? WsT.accent : w.tone, borderRadius: BorderRadius.circular(1)),
                    ),
                  ),
                ),
              Positioned.fill(
                left: w.pad,
                right: w.pad,
                child: Row(
                  children: [
                    Text(w.label, style: T.label(w.enabled ? Grey.g63 : Grey.g56)),
                    const SizedBox(width: 4),
                    Expanded(
                      child: Text(
                        w.format(w.value),
                        maxLines: 1,
                        softWrap: false,
                        overflow: TextOverflow.clip,
                        textAlign: TextAlign.right,
                        style: T.value(w.enabled ? (_drag ? Grey.g100 : Grey.g91) : Grey.g56).copyWith(fontSize: 10.5),
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// The strip over a tool's body: what it works on at the left (cut off, never wrapped, when narrow), actions at the right.
class DeskStrip extends StatelessWidget {
  const DeskStrip({super.key, required this.left, this.right = const []});
  final List<Widget> left, right;
  @override
  Widget build(BuildContext context) => Container(
    height: 26,
    color: WsT.card,
    padding: const EdgeInsets.symmetric(horizontal: WsT.inset),
    child: Row(
      children: [
        Expanded(
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            physics: const NeverScrollableScrollPhysics(),
            child: Row(children: left),
          ),
        ),
        if (right.isNotEmpty) const SizedBox(width: WsT.gap),
        ...right,
      ],
    ),
  );
}

/// A flat chip in a layer's hue with its name in dark ink.
class DeskLayerChip extends StatelessWidget {
  const DeskLayerChip(this.name, this.tone, {super.key});
  final String name;
  final Color tone;
  @override
  Widget build(BuildContext context) => Container(
    height: 16,
    padding: const EdgeInsets.symmetric(horizontal: 6),
    alignment: Alignment.center,
    decoration: BoxDecoration(color: tone, borderRadius: BorderRadius.circular(WsT.chipRadius)),
    child: Text(name, style: T.micro(WsT.onAccent).copyWith(fontWeight: FontWeight.w700)),
  );
}

/// A caption over a section of the desk's side column.
class DeskCaption extends StatelessWidget {
  const DeskCaption(this.text, {super.key, this.tone, this.trailing});
  final String text;
  final Color? tone;
  final String? trailing;
  @override
  Widget build(BuildContext context) => SizedBox(
    height: 14,
    child: Row(
      children: [
        if (tone != null) ...[Container(width: 3, height: 10, color: tone), const SizedBox(width: 5)],
        Text(text.toUpperCase(), style: T.micro(Grey.g76).copyWith(letterSpacing: 1, fontWeight: FontWeight.w700)),
        const SizedBox(width: 6),
        Expanded(
          child: Text(
            trailing ?? '',
            maxLines: 1,
            softWrap: false,
            overflow: TextOverflow.fade,
            textAlign: TextAlign.right,
            style: T.value(Grey.g56).copyWith(fontSize: 10),
          ),
        ),
      ],
    ),
  );
}

/// A big number with its unit and a quiet line under it.
class DeskReadout extends StatelessWidget {
  const DeskReadout({super.key, required this.value, this.unit = '', this.sub, this.ink});
  final String value, unit;
  final String? sub;
  final Color? ink;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    mainAxisSize: MainAxisSize.min,
    children: [
      Row(
        crossAxisAlignment: CrossAxisAlignment.baseline,
        textBaseline: TextBaseline.alphabetic,
        children: [
          Text(value, style: T.value(ink ?? Grey.g95).copyWith(fontSize: 18, fontWeight: FontWeight.w700, letterSpacing: -.5)),
          if (unit.isNotEmpty) ...[const SizedBox(width: 2), Text(unit, style: T.label(Grey.g63))],
        ],
      ),
      if (sub != null) ...[const SizedBox(height: 3), Text(sub!, style: T.label(Grey.g56))],
    ],
  );
}

/// Draws a line of text in a painter, anchored by [ax] / [ay] (0 left/top, 1 right/bottom).
void deskText(Canvas c, String s, Offset at, TextStyle st, {double ax = 0, double ay = 0}) {
  final tp = TextPainter(
    text: TextSpan(text: s, style: st),
    textDirection: TextDirection.ltr,
  )..layout();
  tp.paint(c, at - Offset(tp.width * ax, tp.height * ay));
  tp.dispose();
}
