import 'package:flutter/widgets.dart';

import '../../session/editor_session.dart';
import '../native_visual_sample.dart';

/// What a swatch is made of: the four numbers a stored colour carries, the
/// Color they make, and the strip that draws one — solid or gradient.

List<double> rgbaOf(dynamic raw) {
  final values = (raw as List? ?? [1, 0, 0, 1])
      .map((v) => (v as num).toDouble())
      .toList();
  if (values.length < 4) values.add(1);
  final scale = values.any((v) => v > 1) ? 255.0 : 1.0;
  return values.take(4).map((v) => (v / scale).clamp(0.0, 1.0)).toList();
}

Color colorOf(List<double> rgba) =>
    Color.from(alpha: rgba[3], red: rgba[0], green: rgba[1], blue: rgba[2]);

/// A solid, or the same strip a saved gradient was made from.
Widget gradientBox(
  EditorSession controller,
  List<List<double>> stops, {
  String? blend,
}) => stops.length > 1
    ? NativeVisualSample(
        controller: controller,
        request: {
          'kind': 'gradient',
          'type': 'linear',
          'stops': stops,
          if (blend != null) 'blend': blend,
        },
        fit: BoxFit.fill,
      )
    : ColoredBox(color: colorOf(stops.single));
