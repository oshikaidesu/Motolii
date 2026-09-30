// The top bar as a flexible row: the reference's pieces (brand, transport, readouts, marker, EDIT·PLAY·EXPORT, window
// keys, the motto) in its order, at the density owner's height, folding what can go when the window narrows instead
// of clipping it. The reference face (top.dart) stays for proto_hf; production draws this one over the same TopModel.
import 'package:flutter/widgets.dart';

import '../../hf/glyphs.dart';
import '../../hf/metrics.dart';
import '../../hf/shell/place.dart';
import 'top.dart';
import '../../hf/neutral.dart';

class TopBar extends StatelessWidget {
  const TopBar(this.m, {super.key, this.onModeAt});
  final TopModel m;

  /// A mode key pressed, with its own rectangle in global coordinates (a task opened from it anchors there).
  final void Function(TopMode mode, Rect key)? onModeAt;

  /// Which of Fit · Pin · Open has an operation (one without is drawn quiet, not as a live key).

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        final w = box.maxWidth;
        // what folds, first to last, as the window narrows: the motto, the brand's line, the duration readout
        final motto = w >= 1000, tagline = w >= 820, duration = w >= 680;
        return Container(
          height: Surface.topBar,
          padding: const EdgeInsets.symmetric(horizontal: Surface.panelInset),
          decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: H.rule))),
          child: Row(children: [
            Text('Motolii', style: H.s(14, w: FontWeight.w600, ls: -0.1, color: H.text2)),
            if (tagline) ...[
              const SizedBox(width: 9),
              Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text('Motion', style: Dn.label(N.g76)),
                const SizedBox(height: 1),
                Text('for More Relations.', style: Dn.label(N.g76)),
              ]),
            ],
            const Spacer(flex: 2),
            _Key(onTap: m.onPlay, fill: H.play, edge: H.play, child: const _Play()),
            _Key(onTap: m.onStop, fill: N.g10, edge: N.g20, child: Container(width: 7, height: 7, color: N.g56)),
            _Key(onTap: m.onAnimate, fill: N.g13, edge: N.g26, child: Container(width: 9, height: 9, decoration: const BoxDecoration(color: H.record, shape: BoxShape.circle))),
            const SizedBox(width: 10),
            ValueListenableBuilder<List<String>>(
              valueListenable: m.readouts,
              builder: (_, r, __) => Row(children: [
                Text(r[0], style: Dn.value()),
                if (duration) ...[
                  Container(width: 1, height: 10, margin: const EdgeInsets.symmetric(horizontal: 7), color: N.g44),
                  Text(r[1], style: Dn.value()),
                ],
                const SizedBox(width: 10),
                Text(r[2], style: Dn.value()),
              ]),
            ),
            _Glyph(HG.plus, size: 11, onTap: m.onMarker),
            const Spacer(flex: 2),
            _Modes(m, onModeAt),
            const Spacer(),
            for (final (g, f) in [(HG.fit, m.onFit), (HG.pin, m.onPin), (HG.folder, m.onOpen)])
              _Key(onTap: f, fill: H.raised, edge: H.rule, child: SizedBox(width: 13, height: 13, child: CustomPaint(painter: HgPainter(g, f != null ? N.g86 : N.g44, H.raised)))),
            if (motto) ...[
              const SizedBox(width: 12),
              Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.end, children: [
                Text('Less numbers.', style: Dn.value(H.text2).copyWith(fontSize: 10)),
                const SizedBox(height: 1),
                Text('More motion.', style: Dn.value(H.text2).copyWith(fontSize: 10)),
              ]),
            ],
          ]),
        );
      });
}

/// A transport or window key: [Surface.control] tall, a pointer target at least [Surface.hit]; quiet without an
/// operation.
class _Key extends StatelessWidget {
  const _Key({required this.onTap, required this.fill, required this.edge, required this.child});
  final VoidCallback? onTap;
  final Color fill, edge;
  final Widget child;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: onTap == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: onTap,
          child: Container(
            width: 26,
            height: Surface.control + 2,
            margin: const EdgeInsets.only(left: 4),
            alignment: Alignment.center,
            decoration: BoxDecoration(color: fill, border: Border.all(color: edge), borderRadius: BorderRadius.circular(3)),
            child: child,
          ),
        ),
      );
}

class _Play extends StatelessWidget {
  const _Play();
  @override
  Widget build(BuildContext context) => SizedBox(width: 9, height: 10, child: CustomPaint(painter: _PlayPainter()));
}

class _PlayPainter extends CustomPainter {
  @override
  void paint(Canvas cv, Size s) => cv.drawPath(Path()..moveTo(0, 0)..lineTo(s.width, s.height / 2)..lineTo(0, s.height)..close(), Paint()..color = N.g00);
  @override
  bool shouldRepaint(_PlayPainter o) => false;
}

class _Glyph extends StatelessWidget {
  const _Glyph(this.g, {required this.size, this.onTap});
  final HG g;
  final double size;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: onTap == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: onTap,
          child: SizedBox(width: Surface.hit, height: Surface.hit, child: Center(child: SizedBox(width: size, height: size, child: CustomPaint(painter: HgPainter(g, N.g82, H.window))))),
        ),
      );
}

/// EDIT · PLAY · EXPORT, the reference's three keys in one track; the lit one is the current mode.
class _Modes extends StatelessWidget {
  const _Modes(this.m, this.onModeAt);
  final TopModel m;
  final void Function(TopMode mode, Rect key)? onModeAt;
  @override
  Widget build(BuildContext context) => Container(
        height: Surface.control + 2,
        decoration: BoxDecoration(color: H.raised, border: Border.all(color: H.rule), borderRadius: BorderRadius.circular(3)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          for (final (mode, label) in const [(TopMode.edit, 'EDIT'), (TopMode.play, 'PLAY'), (TopMode.export, 'EXPORT')])
            Builder(
              builder: (context) => MouseRegion(
                cursor: m.onMode == null && onModeAt == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
                child: GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTap: () {
                    final box = context.findRenderObject() as RenderBox?;
                    if (onModeAt != null && box != null) {
                      onModeAt!(mode, box.localToGlobal(Offset.zero) & box.size);
                    } else {
                      m.onMode?.call(mode);
                    }
                  },
                  child: Container(
                    width: 54,
                    alignment: Alignment.center,
                    color: m.mode == mode ? H.mode : null,
                    child: Text(label, style: Dn.label(m.mode == mode ? N.g100 : H.text2, FontWeight.w600).copyWith(letterSpacing: .7)),
                  ),
                ),
              ),
            ),
        ]),
      );
}
