import 'package:flutter/widgets.dart';

import '../../session/editor_session.dart';

import '../../session/color_edit.dart' show colorOf;
export '../../session/color_edit.dart' show rgbaOf, colorOf;
import '../native_visual_sample.dart';

/// What a swatch is made of: the four numbers a stored colour carries, the
/// Color they make, and the strip that draws one — solid or gradient.

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
