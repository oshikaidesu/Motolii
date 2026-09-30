import 'package:flutter/foundation.dart' show ValueListenable;
import 'package:flutter/widgets.dart';

import '../../theme/editor_metrics.dart';
import '../../theme/editor_theme.dart';

/// On and off, and the lamps that say how a value stands to time and to a
/// selection.

/// One item's view of a selection signal: rebuilds only when [test] flips for
/// this item, so a pick in a shelf of hundreds redraws the two it touches.
class Picked<T> extends StatefulWidget {
  const Picked({
    super.key,
    required this.of,
    required this.test,
    required this.builder,
  });
  final ValueListenable<T> of;
  final bool Function(T value) test;
  final Widget Function(bool picked) builder;
  @override
  State<Picked<T>> createState() => _PickedState<T>();
}

class _PickedState<T> extends State<Picked<T>> {
  late bool on = widget.test(widget.of.value);

  void _check() {
    final now = widget.test(widget.of.value);
    if (now != on) setState(() => on = now);
  }

  @override
  void initState() {
    super.initState();
    widget.of.addListener(_check);
  }

  @override
  void didUpdateWidget(Picked<T> old) {
    super.didUpdateWidget(old);
    if (old.of != widget.of) {
      old.of.removeListener(_check);
      widget.of.addListener(_check);
    }
    on = widget.test(widget.of.value);
  }

  @override
  void dispose() {
    widget.of.removeListener(_check);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.builder(on);
}




/// On / off with the result drawn beside it, so the switch says what it does
/// without a word.
class EditorSwitch extends StatelessWidget {
  static const minExpandedWidth =
      EditorMetrics.s22 + EditorMetrics.s4 + EditorMetrics.s14;
  const EditorSwitch({
    super.key,
    required this.on,
    required this.glyph,
    required this.label,
    required this.onChanged,
    this.compact = false,
    this.tint,
    this.ink,
  });
  final bool on;
  final IconData glyph;
  final String label;
  final ValueChanged<bool>? onChanged;

  /// The glyph's colour, on or off, when the switch sits on a head that is
  /// not the panel grey.
  final Color? ink;

  /// The lit colour; the accent unless the switch belongs to another mode.
  final Color? tint;

  /// Glyph only, lit when on — for a slot too narrow for the track.
  final bool compact;
  @override
  Widget build(BuildContext context) {
    final enabled = onChanged != null;
    if (compact) {
      return EditorTooltip(
        message: label,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: enabled ? () => onChanged!(!on) : null,
          child: SizedBox.square(
            dimension: EditorMetrics.row,
            child: Icon(
              glyph,
              size: EditorMetrics.s16,
              color: !enabled
                  ? EditorTheme.of(context).disabledInk
                  : on
                  ? tint ?? EditorTheme.of(context).accent
                  : EditorTheme.of(context).muted,
            ),
          ),
        ),
      );
    }
    return EditorTooltip(
      message: label,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: enabled ? () => onChanged!(!on) : null,
        child: SizedBox(
          height: EditorMetrics.row,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              AnimatedContainer(
                // a switch the hand just pressed shows its new state at once
                duration: Duration.zero,
                width: EditorMetrics.s22,
                height: EditorMetrics.s12,
                padding: const EdgeInsets.all(EditorMetrics.s2),
                decoration: BoxDecoration(
                  color: on
                      ? tint ?? EditorTheme.of(context).accent
                      : EditorTheme.of(context).raised,
                ),
                alignment: on ? Alignment.centerRight : Alignment.centerLeft,
                child: Container(
                  width: EditorMetrics.s8,
                  height: EditorMetrics.s8,
                  color: on
                      ? EditorTheme.of(context).tabInk
                      : EditorTheme.of(context).ink,
                ),
              ),
              const SizedBox(width: EditorMetrics.s4),
              Icon(
                glyph,
                size: EditorMetrics.s14,
                color: !enabled
                    ? EditorTheme.of(context).disabledInk
                    : ink ??
                          (on
                              ? EditorTheme.of(context).ink
                              : EditorTheme.of(context).muted),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
