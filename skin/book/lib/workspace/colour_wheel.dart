// colour_wheel.dart: the Colors shelf's wheel, flex_color_picker's own ColorWheelPicker (BSD-3, Rydmike): a hue ring
// with a square inside it for saturation and value, as CLIP STUDIO PAINT's Color Wheel palette (the user: "クリスタで
// 見かけたあれのままで大丈夫"). Material's, so it is given Material's dark theme here alone.
import 'package:flex_color_picker/flex_color_picker.dart' show ColorWheelPicker;
import 'package:flutter/widgets.dart';
import 'package:material_ui/material_ui.dart' show Theme, ThemeData;

class WsColourWheel extends StatelessWidget {
  const WsColourWheel({super.key, required this.color, required this.onChanged, this.onStart, this.onEnd, this.held = false});
  final Color color;
  final ValueChanged<Color> onChanged;
  final VoidCallback? onStart, onEnd;

  /// A hand is on it: it follows the hand, not the colour given (given from elsewhere, a swatch, it follows that).
  final bool held;

  @override
  Widget build(BuildContext context) => Theme(
    data: ThemeData(brightness: Brightness.dark),
    child: ColorWheelPicker(
      color: color,
      onChanged: onChanged,
      onChangeStart: (_) => onStart?.call(),
      onChangeEnd: (_) => onEnd?.call(),
      onWheel: (_) {},
      wheelWidth: 14,
      wheelSquarePadding: 6,
      wheelSquareBorderRadius: 3,
      shouldUpdate: !held,
    ),
  );
}
