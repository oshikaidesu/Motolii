// The top bar as a flexible row: the reference's pieces (brand, transport, readouts, marker, EDIT·PLAY·EXPORT, window
// keys, the motto) in its order, at the density owner's height, folding what can go when the window narrows instead
// of clipping it. The reference face (top.dart) stays for proto_hf; production draws this one over the same TopModel.
import 'package:flutter/widgets.dart';

import '../glyphs.dart';
import '../metrics.dart';
import 'place.dart';
import 'top.dart';

class TopBar extends StatelessWidget {
  const TopBar(this.m, {super.key, this.onModeAt, this.keyEnabled = const [true, true, true]});
  final TopModel m;

  /// A mode key pressed, with its own rectangle in global coordinates (a task opened from it anchors there).
  final void Function(int mode, Rect key)? onModeAt;

  /// Which of Fit · Pin · Open has an operation (one without is drawn quiet, not as a live key).
  final List<bool> keyEnabled;

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        final w = box.maxWidth;
        // what folds, first to last, as the window narrows: the motto, the brand's line, the duration readout
        final motto = w >= 1240, tagline = w >= 1060, duration = w >= 900;
        return Container(
          height: UiMetrics.topBar,
          padding: const EdgeInsets.symmetric(horizontal: UiMetrics.pad),
          decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: H.rule))),
          child: Row(children: [
            Text('Motolii', style: H.s(18, w: FontWeight.w600, ls: -0.3, color: H.text2)),
            if (tagline) ...[
              const SizedBox(width: 12),
              Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text('Motion', style: H.s(10.5, color: const Color(0xFFBDBEC0))),
                Text('for More Relations.', style: H.s(10.5, color: const Color(0xFFBDBEC0))),
              ]),
            ],
            const Spacer(flex: 2),
            _Key(onTap: m.onPlay, fill: H.play, edge: H.play, child: const _Play()),
            _Key(onTap: m.onStop, fill: const Color(0xFF1C1C1C), edge: const Color(0xFF363636), child: Container(width: 9, height: 9, color: const Color(0xFF969697))),
            _Key(onTap: m.onAnimate, fill: const Color(0xFF1F1F1F), edge: const Color(0xFF3C3C3C), child: Container(width: 11, height: 11, decoration: const BoxDecoration(color: H.record, shape: BoxShape.circle))),
            const SizedBox(width: 14),
            ValueListenableBuilder<List<String>>(
              valueListenable: m.readouts,
              builder: (_, r, __) => Row(children: [
                Text(r[0], style: H.m(12, color: const Color(0xFFD8D9D8))),
                if (duration) ...[
                  Container(width: 1, height: 12, margin: const EdgeInsets.symmetric(horizontal: 10), color: const Color(0xFF6A6A6A)),
                  Text(r[1], style: H.m(12, color: const Color(0xFFD8D9D8))),
                ],
                const SizedBox(width: 14),
                Text(r[2], style: H.m(12, color: const Color(0xFFD8D9D8))),
              ]),
            ),
            _Glyph(HG.plus, size: 13, onTap: m.onMarker),
            const Spacer(flex: 2),
            _Modes(m, onModeAt),
            const Spacer(),
            for (final (i, g) in const [HG.fit, HG.pin, HG.folder].indexed)
              _Key(onTap: keyEnabled[i] && m.onKey != null ? () => m.onKey!(i) : null, fill: H.raised, edge: H.rule, child: SizedBox(width: 16, height: 16, child: CustomPaint(painter: HgPainter(g, keyEnabled[i] ? const Color(0xFFD6D8D8) : const Color(0xFF6A6A6C), H.raised)))),
            if (motto) ...[
              const SizedBox(width: 16),
              Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.end, children: [
                Text('Less numbers.', style: H.m(10.5, color: H.text2)),
                Text('More motion.', style: H.m(10.5, color: H.text2)),
              ]),
            ],
          ]),
        );
      });
}

/// A transport or window key: [UiMetrics.control] tall, a pointer target at least [UiMetrics.hit]; quiet without an
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
            width: 32,
            height: UiMetrics.control + 2,
            margin: const EdgeInsets.only(left: 5),
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
  Widget build(BuildContext context) => SizedBox(width: 11, height: 12, child: CustomPaint(painter: _PlayPainter()));
}

class _PlayPainter extends CustomPainter {
  @override
  void paint(Canvas cv, Size s) => cv.drawPath(Path()..moveTo(0, 0)..lineTo(s.width, s.height / 2)..lineTo(0, s.height)..close(), Paint()..color = const Color(0xFF040709));
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
          child: SizedBox(width: UiMetrics.hit + 4, height: UiMetrics.hit + 4, child: Center(child: SizedBox(width: size, height: size, child: CustomPaint(painter: HgPainter(g, const Color(0xFFCFCFCF), H.window))))),
        ),
      );
}

/// EDIT · PLAY · EXPORT, the reference's three keys in one track; the lit one is the current mode.
class _Modes extends StatelessWidget {
  const _Modes(this.m, this.onModeAt);
  final TopModel m;
  final void Function(int mode, Rect key)? onModeAt;
  @override
  Widget build(BuildContext context) => Container(
        height: UiMetrics.control + 2,
        decoration: BoxDecoration(color: H.raised, border: Border.all(color: H.rule), borderRadius: BorderRadius.circular(3)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          for (final (i, label) in const ['EDIT', 'PLAY', 'EXPORT'].indexed)
            Builder(
              builder: (context) => MouseRegion(
                cursor: m.onMode == null && onModeAt == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
                child: GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTap: () {
                    final box = context.findRenderObject() as RenderBox?;
                    if (onModeAt != null && box != null) {
                      onModeAt!(i, box.localToGlobal(Offset.zero) & box.size);
                    } else {
                      m.onMode?.call(i);
                    }
                  },
                  child: Container(
                    width: 70,
                    alignment: Alignment.center,
                    color: m.mode == i ? H.mode : null,
                    child: Text(label, style: H.s(11, w: FontWeight.w600, ls: .8, color: m.mode == i ? const Color(0xFFFCFCFE) : H.text2)),
                  ),
                ),
              ),
            ),
        ]),
      );
}
