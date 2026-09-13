import 'package:flutter/material.dart';

import 'metrics.dart';
import 'theme.dart';
import 'panel_controls.dart';

/// A colour value's row: the swatch, the hex, the alpha. This is the value's
/// home, not the picker — the swatch hands the focus to the Browser's wheel
/// through [onFocus], and the hex and the alpha write the value directly.
class EditorColorField extends StatefulWidget {
  const EditorColorField({
    super.key,
    required this.value,
    required this.label,
    required this.onPreview,
    required this.onFinish,
    required this.onCancel,
    this.onFocus,
    this.enabled = true,
    this.allowAlpha = true,
  });
  final Color value;
  final String label;
  final bool enabled, allowAlpha;
  final Future<void> Function(Color) onPreview;
  final Future<void> Function() onFinish, onCancel;
  final VoidCallback? onFocus;
  @override
  State<EditorColorField> createState() => _EditorColorFieldState();
}

class _EditorColorFieldState extends State<EditorColorField> {
  bool _changing = false, _ending = false;
  Color? _draft;
  late final _queue = EditorPreviewQueue<Color>(
    (value) => widget.onPreview(value),
  );

  void _change(Color value) {
    if (_ending || !widget.enabled) return;
    setState(() {
      _draft = value;
      _changing = true;
    });
    _queue.add(value);
  }

  Future<void> _finish(bool cancel) async {
    if (!_changing || _ending) return;
    _ending = true;
    try {
      await _queue.finish(cancel, cancel ? widget.onCancel : widget.onFinish);
    } finally {
      _ending = false;
      _changing = false;
      if (mounted) setState(() => _draft = null);
    }
  }

  @override
  void dispose() {
    if (_changing && !_ending) _queue.finish(true, widget.onCancel);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final value = _draft ?? widget.value;
    final swatch = Swatch(color: value, size: EditorMetrics.row);
    return Row(
      children: [
        widget.onFocus == null || !widget.enabled
            ? swatch
            : EditorTooltip(
                message: 'Pick ${widget.label} in the Browser',
                child: InkWell(onTap: widget.onFocus, child: swatch),
              ),
        const SizedBox(width: EditorMetrics.s4),
        SizedBox(
          width: EditorMetrics.s70,
          child: EditorDraftField(
            key: ValueKey('hex:${value.toARGB32() & 0xffffff}'),
            value: hexOf(value),
            label: '${widget.label} hex',
            enabled: widget.enabled,
            validator: (v) =>
                parseHex(v) == null ? 'Use three or six hex digits' : null,
            onCommit: (v) async {
              _change(parseHex(v)!.withValues(alpha: value.a));
              await _finish(false);
            },
          ),
        ),
        if (widget.allowAlpha) ...[
          const SizedBox(width: EditorMetrics.s4),
          const Icon(
            Icons.opacity,
            size: EditorMetrics.s14,
            color: EditorTheme.muted,
          ),
          Expanded(
            child: Slider(
              value: value.a,
              onChanged: widget.enabled
                  ? (a) => _change(value.withValues(alpha: a))
                  : null,
              onChangeEnd: (_) => _finish(false),
            ),
          ),
        ],
      ],
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
      border: Border.all(color: EditorTheme.border),
    ),
    child: CustomPaint(
      painter: const CheckerPainter(cell: EditorMetrics.s4),
      child: ColoredBox(color: color),
    ),
  );
}
