import 'package:flutter/widgets.dart';

import '../glyphs.dart';
import '../leaves.dart';
import '../metrics.dart';
import '../theme.dart';
import 'numeric.dart';

/// The editor's one scale, what it is set by, and the greys behind what is
/// see-through.

/// The editor's one scale, set above the Navigator so pages and their menus,
/// dialogs and drawers all grow together.
class EditorScale extends InheritedNotifier<ValueNotifier<double>> {
  const EditorScale({
    super.key,
    required ValueNotifier<double> super.notifier,
    required super.child,
  });
  static ValueNotifier<double>? of(BuildContext context) =>
      context.getInheritedWidgetOfExactType<EditorScale>()?.notifier;
}

/// A logical viewport whose painted and hit-tested bounds fill its parent.
class EditorScaledViewport extends StatelessWidget {
  const EditorScaledViewport({
    super.key,
    required this.scale,
    required this.child,
  });
  final double scale;
  final Widget child;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) => OverflowBox(
      alignment: Alignment.topLeft,
      minWidth: box.maxWidth / scale,
      maxWidth: box.maxWidth / scale,
      minHeight: box.maxHeight / scale,
      maxHeight: box.maxHeight / scale,
      child: Transform.scale(
        scale: scale,
        alignment: Alignment.topLeft,
        child: child,
      ),
    ),
  );
}

class EditorPercentField extends StatefulWidget {
  const EditorPercentField({
    super.key,
    required this.value,
    required this.min,
    required this.max,
    required this.onChanged,
    this.label = 'Scale',
  });
  final double value, min, max;
  final String label;
  final ValueChanged<double> onChanged;
  @override
  State<EditorPercentField> createState() => _EditorPercentFieldState();
}

class _EditorPercentFieldState extends State<EditorPercentField> {
  double? _start;
  @override
  Widget build(BuildContext context) {
    Future<void> change(double n) async => widget.onChanged(
      n.roundToDouble().clamp(
        widget.min.ceilToDouble(),
        widget.max.floorToDouble(),
      ),
    );
    return SizedBox(
      width: EditorMetrics.s64,
      child: EditorNumericField(
        value: widget.value,
        label: widget.label,
        min: widget.min.ceilToDouble(),
        max: widget.max.floorToDouble(),
        decimals: 0,
        unit: '%',
        speed: 1,
        onBegin: () => _start = widget.value,
        onPreview: change,
        onCommit: change,
        onFinish: () async {
          _start = null;
        },
        onCancel: () async {
          if (_start != null) widget.onChanged(_start!);
          _start = null;
        },
      ),
    );
  }
}

/// Sizes are percentages of the panel's default size.
class EditorZoomBar extends StatelessWidget {
  const EditorZoomBar({
    super.key,
    required this.value,
    required this.min,
    required this.max,
    required this.onChanged,
    required this.keyPrefix,
    required this.base,
    this.step = 1,
  });
  final double value, min, max, base;
  final ValueChanged<double> onChanged;
  final String keyPrefix;

  /// Percent moved by one press of − or +.
  final int step;

  /// The width at which the bar shows its slider: both presses, the percent
  /// field, and a slider long enough to grab. Narrower, the field stands alone.
  static const sliderRoom =
      EditorMetrics.row * 2 + EditorMetrics.field + EditorMetrics.s48;
  @override
  Widget build(BuildContext context) {
    final low = (min / base * 100).ceilToDouble();
    final high = (max / base * 100).floorToDouble();
    final percent = value / base * 100;
    void change(double n) =>
        onChanged(n.roundToDouble().clamp(low, high) * base / 100);
    Widget step(IconData icon, int delta, String suffix) => EditorPress(
      key: ValueKey('$keyPrefix-$suffix'),
      onTap: () => change(percent.roundToDouble() + delta),
      child: SizedBox.square(
        dimension: EditorMetrics.row,
        child: Icon(
          icon,
          size: EditorMetrics.s14,
          color: EditorTheme.of(context).muted,
        ),
      ),
    );
    return Container(
      height: EditorMetrics.row,
      foregroundDecoration: BoxDecoration(
        border: Border(top: BorderSide(color: EditorTheme.of(context).line)),
      ),
      child: LayoutBuilder(
        builder: (context, box) {
          final field = EditorPercentField(
            key: ValueKey('$keyPrefix-percent'),
            value: percent,
            min: low,
            max: high,
            onChanged: change,
            label: 'Size',
          );
          return Row(
            children: [
              step(Glyph.remove, -this.step, 'smaller'),
              if (box.maxWidth >= sliderRoom) ...[
                Expanded(
                  child: EditorSlider(
                    min: low,
                    max: high,
                    divisions: (high - low).round(),
                    value: percent.clamp(low, high),
                    onChanged: change,
                  ),
                ),
                field,
              ] else
                Expanded(child: field),
              step(Glyph.add, this.step, 'larger'),
            ],
          );
        },
      ),
    );
  }
}

/// The transparency grid: the picture editors' two greys, 8 px squares.
class CheckerPainter extends CustomPainter {
  const CheckerPainter({this.cell = 8, this.ink = EditorInk.dark});
  final double cell;
  final EditorInk ink;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, Paint()..color = ink.checkerLight);
    final paint = Paint()..color = ink.checkerDark;
    for (var y = 0; y * cell < size.height; y++) {
      for (var x = (y % 2); x * cell < size.width; x += 2) {
        canvas.drawRect(Rect.fromLTWH(x * cell, y * cell, cell, cell), paint);
      }
    }
  }

  @override
  bool shouldRepaint(CheckerPainter old) => old.cell != cell || old.ink != ink;
}
