import 'package:flutter/widgets.dart';

import '../../theme/metrics.dart';
import '../../theme/editor_theme.dart';
import 'numeric.dart';

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
      width: Surface.px(64),
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
