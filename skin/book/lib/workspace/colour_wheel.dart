// colour_wheel.dart: the Colors shelf's wheel, a hue ring with a square inside it for saturation and value, as CLIP
// STUDIO PAINT's Color Wheel palette (the user: "クリスタで見かけたあれのままで大丈夫"): flutter_colorpicker's own
// HueRingPicker (MIT, mchome), its ring and its square stacked as it stacks them (the square 1/1.6 of the ring), without
// its hex field (the shelf has its own). Not flex_color_picker's ColorWheelPicker: it takes the hand's place on the
// screen less its own corner, so in the window drawn smaller (WsZoom, 76%) a press missed it (the user, "カラーサークルが
// 選べん"); these take it by globalToLocal, through any scale.
import 'package:flutter/widgets.dart';
import 'package:flutter_colorpicker/flutter_colorpicker.dart' show ColorPickerArea, ColorPickerHueRing, PaletteType;

class WsColourWheel extends StatelessWidget {
  const WsColourWheel({super.key, required this.color, required this.onChanged, this.onStart, this.onEnd});
  final HSVColor color;
  final ValueChanged<HSVColor> onChanged;

  /// A hand down on it, and up again (Flutter's Listener: it sees the hand without taking it from the ring or square).
  final VoidCallback? onStart, onEnd;

  static const _ring = 14.0;

  @override
  Widget build(BuildContext context) => Listener(
    onPointerDown: (_) => onStart?.call(),
    onPointerUp: (_) => onEnd?.call(),
    onPointerCancel: (_) => onEnd?.call(),
    child: LayoutBuilder(
      builder: (context, box) {
        final s = box.biggest.shortestSide - _ring;
        return Stack(
          alignment: Alignment.center,
          children: [
            // (the ring is stroked on the edge of its box, half of it outside, as HueRingPicker pads it; that half takes
            //  no press there, so an unseen ring the whole size takes them: the hue is the angle from the middle, the same)
            SizedBox.square(dimension: s, child: IgnorePointer(child: ColorPickerHueRing(color, onChanged, strokeWidth: _ring))),
            SizedBox.square(dimension: s + _ring, child: Opacity(opacity: 0, child: ColorPickerHueRing(color, onChanged, strokeWidth: _ring))),
            SizedBox.square(
              dimension: s / 1.6,
              child: ClipRRect(borderRadius: const BorderRadius.all(Radius.circular(3)), child: ColorPickerArea(color, onChanged, PaletteType.hsv)),
            ),
          ],
        );
      },
    ),
  );
}
