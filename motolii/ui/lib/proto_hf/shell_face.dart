import 'package:flutter/widgets.dart';

import '../hf/metrics.dart' show Surface;

/// The Shell's own visible tree — the same one `proto_hf/main_shell.dart::Shell` draws, pulled out so both proto_hf
/// (fixtures in every slot) and production (Motolii Live in every slot) build the identical widget tree. This is
/// the layout/visual authority (topology, relative geometry, seat boundaries): nothing here reads fixture state or
/// EditorSession — every slot is handed a finished widget.
///
/// Geometry is the 1536×1024 reference frame's own rectangles (Browser 324, Stage 779, right seat 387 wide; main
/// row 632, Timeline row 291 tall), unscaled: this face is a fixed layout for now, not a resizable one — Dock
/// capability (split/resize/detach/persist) is a later, separate step that must not change what these seats look
/// like.
class ShellFace extends StatelessWidget {
  const ShellFace({super.key, required this.top, required this.browser, required this.stage, required this.right, required this.timeline});
  final Widget top, browser, stage, right, timeline;

  static const width = 1536.0, height = 1024.0;

  Widget _seat(double x, double y, double w, double h, Widget child) => Positioned(
        left: x, top: y, width: w, height: h,
        child: DecoratedBox(decoration: BoxDecoration(color: Surface.base, border: Border.all(color: Surface.divider)), child: child),
      );

  @override
  Widget build(BuildContext context) => SizedBox(
        width: width,
        height: height,
        child: Stack(children: [
          const Positioned.fill(child: ColoredBox(color: Surface.base)),
          Positioned(left: 0, top: 0, width: width, height: 62, child: top),
          _seat(10, 62, 324, 953, browser),
          Positioned(left: 344, top: 62, width: 779, height: 632, child: stage),
          _seat(1135, 62, 387, 631, right),
          Positioned(left: 344, top: 703, width: 1178, height: 291, child: timeline),
        ]),
      );
}
