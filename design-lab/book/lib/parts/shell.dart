// The window: top bar, the three seats (browser, stage, inspector) and the timeline below. Stage chrome is here too.
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'browser.dart';
import 'controls.dart';
import 'glyphs.dart';
import 'inspector.dart';
import 'timeline.dart';

class TopBar extends StatelessWidget {
  const TopBar({super.key, this.mode = 0});
  final int mode;
  @override
  Widget build(BuildContext context) => Container(
        height: 48,
        color: N.g10,
        padding: const EdgeInsets.symmetric(horizontal: 14),
        child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
          Text('Motolii', style: T.title().copyWith(fontSize: 17, fontWeight: FontWeight.w700)),
          const SizedBox(width: 10),
          Text('Motion\nfor More Relations.', style: T.label(N.g56).copyWith(height: 1.25)),
          const Spacer(),
          Tool(size: 32, fill: C.play.withValues(alpha: .16), child: const _Play()),
          const SizedBox(width: 6),
          const Tool(size: 32, child: _Square()),
          const SizedBox(width: 6),
          Tool(size: 32, round: true, fill: C.record.withValues(alpha: .14), child: Container(width: 10, height: 10, decoration: const BoxDecoration(color: C.record, shape: BoxShape.circle))),
          const SizedBox(width: 18),
          Text('120.00', style: T.value(N.g91)),
          const SizedBox(width: 12),
          Text('00:02:13', style: T.value(N.g91)),
          const Spacer(),
          Segmented(items: const ['Edit', 'Play', 'Export'], index: mode),
          const Spacer(),
          Text('Less numbers.\nMore motion.', textAlign: TextAlign.right, style: T.label(N.g56).copyWith(height: 1.3)),
        ]),
      );
}

class _Play extends StatelessWidget {
  const _Play();
  @override
  Widget build(BuildContext context) => CustomPaint(size: const Size(12, 12), painter: _TriPainter());
}

class _TriPainter extends CustomPainter {
  @override
  void paint(Canvas c, Size s) => c.drawPath(Path()..moveTo(s.width * .15, 0)..lineTo(s.width, s.height / 2)..lineTo(s.width * .15, s.height)..close(), Paint()..color = C.play);
  @override
  bool shouldRepaint(_TriPainter o) => false;
}

class _Square extends StatelessWidget {
  const _Square();
  @override
  Widget build(BuildContext context) => Container(width: 11, height: 11, decoration: BoxDecoration(color: N.g63, borderRadius: BorderRadius.circular(2)));
}

/// The stage's own chrome: the tool column on its left and the zoom bar under it. The picture itself is the work, not chrome.
class StageChrome extends StatelessWidget {
  const StageChrome({super.key, this.zoom = 100});
  final int zoom;
  @override
  Widget build(BuildContext context) => Container(
        color: const Color(0xFF111111),
        child: Stack(children: [
          Positioned.fill(child: Padding(padding: const EdgeInsets.fromLTRB(58, 24, 24, 52), child: Center(child: AspectRatio(aspectRatio: 16 / 9, child: const _Artwork())))),
          Positioned(left: 10, top: 10, child: Container(
            padding: const EdgeInsets.all(4),
            decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20)),
            child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
              for (var i = 0; i < 6; i++) Padding(padding: const EdgeInsets.symmetric(vertical: 2), child: Tool(size: 28, on: i == 0, fill: i == 0 ? N.g20 : const Color(0x00000000), child: Glyph(const [G.select, G.move, G.rect, G.ellipse, G.pen, G.text][i], size: 16, color: i == 0 ? N.g95 : N.g63))),
            ]),
          )),
          Positioned(left: 10, bottom: 10, child: Container(
            height: 28,
            padding: const EdgeInsets.symmetric(horizontal: 10),
            alignment: Alignment.center,
            decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(7), border: Border.all(color: N.g20)),
            child: Text('$zoom%', style: T.value(N.g91)),
          )),
          Positioned(right: 10, bottom: 10, child: Container(
            height: 28,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            alignment: Alignment.center,
            decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(7), border: Border.all(color: N.g20)),
            child: Text('Camera View', style: T.name(N.g91)),
          )),
        ]),
      );
}

/// A stand-in for the work: a few shapes in the families' colours on a dark frame. The stage's chrome must stay quieter than this.
class _Artwork extends StatelessWidget {
  const _Artwork();
  @override
  Widget build(BuildContext context) => DecoratedBox(
        decoration: BoxDecoration(color: N.g07, border: Border.all(color: N.g20)),
        child: CustomPaint(painter: _ArtPainter()),
      );
}

class _ArtPainter extends CustomPainter {
  @override
  void paint(Canvas c, Size s) {
    final p = Paint();
    for (var i = 0; i < Fam.all.length; i++) {
      final f = Fam.all[i], x = s.width * (.18 + i * .13), y = s.height * (.3 + (i.isEven ? .22 : -.05));
      c.save();
      c.translate(x, y);
      c.rotate(.35 * (i - 2));
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: s.width * .16, height: s.width * .2), const Radius.circular(6)), p..color = f.c.withValues(alpha: .85));
      c.restore();
    }
  }

  @override
  bool shouldRepaint(_ArtPainter o) => false;
}

class Shell extends StatelessWidget {
  const Shell({super.key, this.look = Look.concept});
  final Look look;
  @override
  Widget build(BuildContext context) => Container(
        color: N.g07,
        child: Flex(direction: Axis.vertical, children: [
          const TopBar(),
          Container(height: 1, color: N.g20),
          Expanded(flex: 5, child: LayoutBuilder(builder: (context, box) => Flex(direction: Axis.horizontal, children: [
            // the browser gives way to the stage when the window is narrow
            if (box.maxWidth >= 1100) ...[BrowserPanel(look: look), Container(width: 1, color: N.g20)],
            const Expanded(child: StageChrome()),
            Container(width: 1, color: N.g20),
            InspectorPanel(look: look),
          ]))),
          Container(height: 1, color: N.g20),
          const Expanded(flex: 3, child: TimelinePanel()),
        ]),
      );
}
