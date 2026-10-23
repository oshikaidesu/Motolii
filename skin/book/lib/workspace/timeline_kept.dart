// The stage's pictures kept on disk for playing (the inside's stage_kept.h), marked under the ruler's work-area band as
// After Effects marks its cached frames: a thin line where a played picture is put, not drawn.
import 'package:flutter/foundation.dart' show ValueListenable;
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'timeline_paint.dart';
import 'ws.dart';

class WtKeptPainter extends CustomPainter {
  WtKeptPainter({required this.view, required this.kept}) : super(repaint: kept);
  final WtView view;
  final ValueListenable<List<(int, int)>> kept;

  @override
  void paint(Canvas c, Size s) {
    final ink = Paint()..color = WsT.accent;
    for (final (from, to) in kept.value) {
      c.drawRect(Rect.fromLTRB(view.x(from).clamp(0, s.width), WtM.band, view.x(to).clamp(0, s.width), WtM.band + Shape.z(3)), ink);
    }
  }

  @override
  bool shouldRepaint(WtKeptPainter o) => o.view != view || o.kept != kept;
}
