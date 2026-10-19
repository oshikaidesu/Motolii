// The whole window drawn larger or smaller in 1% steps (Ableton's Zoom Display, a browser's ⌘+ / ⌘−): laid out in a box
// 1/zoom the window's size and drawn zoom times larger, so every part grows alike, stays sharp (Flutter draws vectors) and
// takes the hand where it is drawn; the stage is told the finer pixels. Kept in ~/.motolii/window.xml, beside the theme.
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import '../window_file.dart';

abstract final class WsZoom {
  static final value = ValueNotifier<double>((double.tryParse(WindowFile.get('Zoom') ?? '') ?? 1).clamp(.5, 2.0));

  /// The zoom set (to the nearest 1%, 50% to 200%) and, unless [keep] is false, kept for the next time.
  static void set(double z, {bool keep = true}) {
    value.value = ((z * 100).round() / 100).clamp(.5, 2.0);
    if (keep) WindowFile.set('Zoom', value.value.toStringAsFixed(2));
  }

  static void by(int percent) => set(value.value + percent / 100);
}

/// The zoom in the top bar, quiet: scrubbed sideways it moves 1% every 3 px (kept when let go), tapped twice it is 100% again.
class WsZoomReadout extends StatefulWidget {
  const WsZoomReadout({super.key});
  @override
  State<WsZoomReadout> createState() => _WsZoomReadoutState();
}

class _WsZoomReadoutState extends State<WsZoomReadout> {
  double _from = 1, _dx = 0;
  @override
  Widget build(BuildContext context) => ValueListenableBuilder<double>(
    valueListenable: WsZoom.value,
    builder: (context, z, _) => MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      child: GestureDetector(
        key: const ValueKey('ws-zoom'),
        behavior: HitTestBehavior.opaque,
        onHorizontalDragStart: (_) {
          _from = z;
          _dx = 0;
        },
        onHorizontalDragUpdate: (e) => WsZoom.set(_from + ((_dx += e.delta.dx) / 3).round() / 100, keep: false),
        onHorizontalDragEnd: (_) => WsZoom.set(WsZoom.value.value),
        onDoubleTap: () => WsZoom.set(1),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
          child: Text('${(z * 100).round()}%', style: T.micro(z == 1 ? Grey.g56 : Grey.g91)),
        ),
      ),
    ),
  );
}

/// [child] drawn at [WsZoom.value].
class WsZoomed extends StatelessWidget {
  const WsZoomed({super.key, required this.child});
  final Widget child;
  @override
  Widget build(BuildContext context) => ValueListenableBuilder<double>(
    valueListenable: WsZoom.value,
    builder: (context, z, _) => LayoutBuilder(
      builder: (context, box) {
        final mq = MediaQuery.of(context), size = Size(box.maxWidth / z, box.maxHeight / z);
        return ClipRect(
          child: OverflowBox(
            alignment: Alignment.topLeft,
            minWidth: size.width,
            maxWidth: size.width,
            minHeight: size.height,
            maxHeight: size.height,
            child: Transform.scale(
              scale: z,
              alignment: Alignment.topLeft,
              child: MediaQuery(data: mq.copyWith(size: size, devicePixelRatio: mq.devicePixelRatio * z), child: child), // (finer pixels: the stage renders sharp)
            ),
          ),
        );
      },
    ),
  );
}
