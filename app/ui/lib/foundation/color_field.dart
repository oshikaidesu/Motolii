import 'package:flutter/widgets.dart';

import 'leaves.dart';
import 'metrics.dart';
import 'theme.dart';
import 'panel_controls.dart';

/// The colour atom: one swatch that is the value. It types nothing and
/// opens nothing — pressing it hands the focus to the Browser's wheel, which
/// is the only picker. Every colour-typed value on a sheet is this.
class EditorColorField extends StatelessWidget {
  const EditorColorField({
    super.key,
    required this.value,
    required this.label,
    this.onFocus,
    this.enabled = true,
    this.size = EditorMetrics.row,
  });
  final Color value;
  final String label;
  final bool enabled;
  final double size;
  final VoidCallback? onFocus;

  @override
  Widget build(BuildContext context) {
    final swatch = Swatch(color: value, size: size);
    if (onFocus == null || !enabled) return swatch;
    return EditorTooltip(
      message: 'Pick $label in the Browser',
      child: EditorPress(onTap: onFocus, child: swatch),
    );
  }
}

/// `#rrggbb` of the colour; the alpha rides on its own control.
String hexOf(Color c) =>
    '#${(c.toARGB32() & 0xffffff).toRadixString(16).padLeft(6, '0')}';

/// Three or six hex digits, with or without `#`, as an opaque colour.
Color? parseHex(String text) {
  var raw = text.trim().replaceFirst('#', '');
  if (raw.length == 3) raw = raw.split('').map((s) => '$s$s').join();
  final n = raw.length == 6 ? int.tryParse(raw, radix: 16) : null;
  return n == null ? null : Color(0xff000000 | n);
}

/// A colour on the checker, so transparency reads as transparency.
class Swatch extends StatelessWidget {
  const Swatch({super.key, required this.color, required this.size});
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => Container(
    width: size,
    height: size,
    clipBehavior: Clip.antiAlias,
    decoration: BoxDecoration(
      borderRadius: BorderRadius.circular(EditorMetrics.s4),
      border: Border.all(color: EditorTheme.of(context).border),
    ),
    child: CustomPaint(
      painter: const CheckerPainter(cell: EditorMetrics.s4),
      child: ColoredBox(color: color),
    ),
  );
}
