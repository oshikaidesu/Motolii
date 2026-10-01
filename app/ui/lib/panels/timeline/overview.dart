import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import 'ink.dart';

/// 帯の上の縮図(レンズ)の絵 — 全ての層の配置と、今どこを見ているかの窓。
class ArrangementOverview extends CustomPainter {
  final EditorTheme colors;

  const ArrangementOverview({
    this.colors = EditorTheme.chromatic,
    required this.layers,
    required this.duration,
    required this.extent,
    required this.frame,
    required this.offset,
    required this.scale,
    required this.viewportWidth,
  });
  final List<Map<String, dynamic>> layers;
  final int duration, extent, frame;
  final double offset, scale, viewportWidth;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, fillPaint(colors.app));
    final unit = size.width / math.max(1, extent);
    final lane = 12 / math.max(1, layers.length);
    canvas.save();
    canvas.clipRect(Offset.zero & size);
    for (var i = 0; i < layers.length; i++) {
      final l = layers[i];
      canvas.drawRect(
        Rect.fromLTWH(
          (l['start'] as num? ?? 0) * unit,
          3 + i * lane,
          (l['duration'] as num? ?? 1) * unit,
          math.max(1, lane - 1),
        ),
        fillPaint(
          l['hidden'] == true ? colors.border : colors.timelineColor(l['id']),
        ),
      );
    }
    final left = (offset / scale * unit).clamp(0.0, size.width);
    final right = ((offset + viewportWidth) / scale * unit).clamp(
      left,
      size.width,
    );
    canvas.drawRect(
      Rect.fromLTRB(left, 1, right, 17),
      fillPaint(EditorTheme.white.withValues(alpha: .10)),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTRB(left, 1, right, 17),
        const Radius.circular(EditorMetrics.s3),
      ),
      strokePaint(colors.tab, 1.5),
    );
    // 尺の印(壁ではない)と、今のコマ。
    if (extent > duration) {
      final x = duration * unit;
      canvas.drawLine(Offset(x, 1), Offset(x, 17), linePaint(colors.muted, 1));
    }
    final now = frame * unit;
    canvas.drawLine(
      Offset(now, 0),
      Offset(now, size.height),
      linePaint(colors.keyAccent, 1.5),
    );
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant ArrangementOverview old) =>
      colors != old.colors ||
      !identical(layers, old.layers) ||
      duration != old.duration ||
      extent != old.extent ||
      frame != old.frame ||
      offset != old.offset ||
      scale != old.scale ||
      viewportWidth != old.viewportWidth;
}
