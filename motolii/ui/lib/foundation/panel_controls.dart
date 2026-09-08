import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'theme.dart';
import 'metrics.dart';

Widget panelButton(
  String label,
  VoidCallback? action, {
  String? tooltip,
  bool selected = false,
}) =>
    EditorButton(label, action, tooltip: tooltip ?? label, selected: selected);
Widget panelTitle(String title) => Container(
  height: EditorMetrics.s22,
  alignment: Alignment.centerLeft,
  padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
  decoration: const BoxDecoration(
    color: EditorTheme.raised,
    border: Border(bottom: BorderSide(color: EditorTheme.line)),
  ),
  child: Text(
    title,
    style: const TextStyle(
      fontSize: EditorMetrics.font,
      color: EditorTheme.ink,
    ),
  ),
);

/// A panel bar. Its Row keeps Spacer alignment while the content fits and
/// slides sideways when it does not, instead of overflowing the pane.
class EditorBar extends StatelessWidget {
  const EditorBar({
    super.key,
    required this.children,
    this.height = EditorMetrics.s22,
    this.padding = EdgeInsets.zero,
    this.decoration = const BoxDecoration(color: EditorTheme.panel),
  });
  final List<Widget> children;
  final double height;
  final EdgeInsetsGeometry padding;
  final BoxDecoration decoration;
  @override
  Widget build(BuildContext context) => Container(
    height: height,
    decoration: decoration,
    child: LayoutBuilder(
      builder: (context, box) => SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: ConstrainedBox(
          constraints: BoxConstraints(minWidth: box.maxWidth),
          child: IntrinsicWidth(
            child: Padding(
              padding: padding,
              child: Row(children: children),
            ),
          ),
        ),
      ),
    ),
  );
}

/// One value out of a short list. The sheet is the app's menu (menuTheme), so
/// rows are EditorMetrics.row high; DropdownButton cannot go under 48 and is
/// not used.
class EditorChoice<T> extends StatelessWidget {
  const EditorChoice({
    super.key,
    required this.value,
    required this.choices,
    required this.onChanged,
  });
  final T? value;
  final List<MapEntry<T, String>> choices;
  final ValueChanged<T>? onChanged;
  @override
  Widget build(BuildContext context) {
    final enabled = onChanged != null;
    final label = choices
        .where((e) => e.key == value)
        .map((e) => e.value)
        .firstOrNull;
    return MenuAnchor(
      crossAxisUnconstrained: false,
      menuChildren: [
        for (final e in choices)
          MenuItemButton(
            onPressed: enabled ? () => onChanged!(e.key) : null,
            child: Text(e.value, maxLines: 1, overflow: TextOverflow.ellipsis),
          ),
      ],
      builder: (context, menu, _) => GestureDetector(
        onTap: enabled ? (menu.isOpen ? menu.close : menu.open) : null,
        child: Container(
          height: EditorMetrics.row,
          padding: const EdgeInsets.only(left: EditorMetrics.s4),
          decoration: BoxDecoration(
            color: EditorTheme.app,
            border: Border.all(
              color: enabled ? EditorTheme.border : EditorTheme.line,
            ),
          ),
          child: Row(
            children: [
              Expanded(
                child: Text(
                  label ?? '',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: EditorMetrics.font,
                    color: enabled ? EditorTheme.ink : EditorTheme.muted,
                  ),
                ),
              ),
              const Icon(
                Icons.arrow_drop_down,
                size: EditorMetrics.s16,
                color: EditorTheme.muted,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class EditorDraftField extends StatefulWidget {
  const EditorDraftField({
    super.key,
    required this.value,
    required this.label,
    required this.onCommit,
    this.multiline = false,
    this.enabled = true,
    this.validator,
  });
  final String value, label;
  final bool multiline, enabled;
  final String? Function(String)? validator;
  final Future<void> Function(String) onCommit;
  @override
  State<EditorDraftField> createState() => _EditorDraftFieldState();
}

class _EditorDraftFieldState extends State<EditorDraftField> {
  late final TextEditingController _text = TextEditingController(
    text: widget.value,
  );
  late final FocusNode _focus = FocusNode(
    onKeyEvent: (_, event) {
      if (event is KeyDownEvent &&
          event.logicalKey == LogicalKeyboardKey.escape &&
          !_text.value.composing.isValid) {
        _cancelled = true;
        _text.text = widget.value;
        setState(() => _error = null);
        _focus.unfocus();
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    },
  )..addListener(_focusChanged);
  String? _error;
  bool _cancelled = false, _committing = false;
  void _focusChanged() {
    if (!_focus.hasFocus && !_cancelled) _commit();
    _cancelled = false;
  }

  void _commit() {
    final error = widget.validator?.call(_text.text);
    setState(() => _error = error);
    if (error != null) {
      _focus.requestFocus();
      return;
    }
    if (_text.text != widget.value && !_committing) {
      _committing = true;
      widget.onCommit(_text.text).whenComplete(() => _committing = false);
    }
  }

  @override
  void didUpdateWidget(covariant EditorDraftField old) {
    super.didUpdateWidget(old);
    if (!_focus.hasFocus && old.value != widget.value)
      _text.text = widget.value;
  }

  @override
  void dispose() {
    _focus.removeListener(_focusChanged);
    _focus.dispose();
    _text.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
    onKeyEvent: (_, event) {
      if (event is KeyDownEvent &&
          event.logicalKey == LogicalKeyboardKey.escape) {
        _cancelled = true;
        _text.text = widget.value;
        setState(() => _error = null);
        _focus.unfocus();
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    },
    child: TextField(
      controller: _text,
      focusNode: _focus,
      enabled: widget.enabled,
      minLines: 1,
      maxLines: widget.multiline ? 4 : 1,
      style: const TextStyle(
        fontSize: EditorMetrics.font,
        color: EditorTheme.ink,
      ),
      decoration: InputDecoration(
        isDense: true,
        contentPadding: const EdgeInsets.symmetric(
          horizontal: EditorMetrics.s5,
          vertical: EditorMetrics.s3,
        ),
        border: InputBorder.none,
        labelText: null,
        hintText: widget.label,
        errorText: _error,
      ),
      onSubmitted: (_) => _commit(),
      onChanged: (_) {
        if (_error != null) setState(() => _error = null);
      },
    ),
  );
}

class EditorNumericField extends StatefulWidget {
  const EditorNumericField({
    super.key,
    required this.value,
    required this.label,
    required this.onPreview,
    required this.onCommit,
    required this.onFinish,
    required this.onCancel,
    this.speed = 1,
    this.min,
    this.max,
    this.enabled = true,
    this.mixed = false,
    this.idleFocus,
    this.onBegin,
    this.fill = false,
    this.unit,
    this.decimals = 2,
    this.defaultValue,
    this.tint,
    this.track = TrackStyle.fill,
  });
  final double value, speed;
  final double? min, max;
  final String label;
  final bool enabled, mixed;

  /// Paint how far the value sits between min and max behind the number, so
  /// a bounded amount reads before the digits do.
  final bool fill;

  /// A small rider after the number: px, %, °.
  final String? unit;
  final int decimals;

  /// Where the value rests when untouched: a tick on the track, and the
  /// point a centre-zero track fills from.
  final double? defaultValue;

  /// The hue of the number's family; colours the track and the underline.
  final Color? tint;

  /// How the track tells the amount: a fill, a threshold, steps or a ruler.
  final TrackStyle track;
  final FocusNode? idleFocus;
  final Future<void> Function(double) onPreview, onCommit;
  final Future<void> Function() onFinish, onCancel;
  final VoidCallback? onBegin;
  @override
  State<EditorNumericField> createState() => _EditorNumericFieldState();
}

class _EditorNumericFieldState extends State<EditorNumericField> {
  final _text = TextEditingController();
  final _focus = FocusNode();
  final _ownIdle = FocusNode();
  FocusNode get _idle => widget.idleFocus ?? _ownIdle;
  bool _editing = false, _dragging = false, _sending = false;
  int? _pointer;
  double _start = 0, _startGlobalX = 0;
  double? _pending, _shown;
  Future<void> _drained = Future<void>.value();
  String? _error;
  @override
  void initState() {
    super.initState();
    _focus.addListener(_lost);
    _focus.onKeyEvent = (_, event) {
      if (event is KeyDownEvent &&
          event.logicalKey == LogicalKeyboardKey.escape) {
        setState(() {
          _editing = false;
          _error = null;
        });
        _idle.requestFocus();
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    };
  }

  void _lost() {
    if (!_focus.hasFocus && _editing) _commitText();
  }

  double _bounded(double n) => n
      .clamp(
        widget.min ?? double.negativeInfinity,
        widget.max ?? double.infinity,
      )
      .toDouble();
  void _open() {
    if (!widget.enabled) return;
    setState(() {
      _editing = true;
      _text.text = widget.mixed ? '' : widget.value.toString();
      _text.selection = TextSelection(
        baseOffset: 0,
        extentOffset: _text.text.length,
      );
    });
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _focus.requestFocus();
    });
  }

  void _commitText() {
    final n = double.tryParse(_text.text.trim());
    if (n == null || !n.isFinite) {
      setState(() => _error = 'Number required');
      _focus.requestFocus();
      return;
    }
    setState(() {
      _editing = false;
      _error = null;
    });
    widget.onCommit(_bounded(n));
  }

  void _tick(double n) {
    _pending = n;
    if (_sending) return;
    _sending = true;
    _drained = () async {
      try {
        while (_pending != null) {
          final value = _pending!;
          _pending = null;
          await widget.onPreview(value);
        }
      } finally {
        _sending = false;
      }
    }();
  }

  void _pointerDown(PointerDownEvent event) {
    if (!widget.enabled || _pointer != null || event.buttons != 1) return;
    _pointer = event.pointer;
    _start = widget.value;
    _startGlobalX = event.position.dx;
    _idle.requestFocus();
  }

  void _pointerMove(PointerMoveEvent event) {
    if (event.pointer != _pointer || event.buttons != 1) return;
    final displacement = event.position.dx - _startGlobalX;
    if (!_dragging) {
      if (displacement.abs() < 3) return;
      widget.onBegin?.call();
      setState(() => _dragging = true);
    }
    final n = _bounded(_start + displacement * widget.speed);
    setState(() => _shown = n);
    _tick(n);
  }

  void _pointerUp(PointerEvent event, {bool cancel = false}) {
    if (event.pointer != _pointer) return;
    if (_dragging) {
      _end(cancel);
    } else {
      _pointer = null;
    }
  }

  Future<void> _end(bool cancel) async {
    if (!_dragging) return;
    _pointer = null;
    setState(() => _dragging = false);
    if (cancel) _pending = null;
    await _drained;
    if (cancel)
      await widget.onCancel();
    else
      await widget.onFinish();
    if (mounted) setState(() => _shown = null);
  }

  @override
  void dispose() {
    if (_dragging) {
      _pending = null;
      final cancel = widget.onCancel;
      _drained.whenComplete(cancel);
    }
    _focus.removeListener(_lost);
    _focus.dispose();
    _ownIdle.dispose();
    _text.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
    focusNode: _idle,
    onKeyEvent: (_, event) {
      if (event is! KeyDownEvent) return KeyEventResult.ignored;
      if (event.logicalKey == LogicalKeyboardKey.escape) {
        if (_dragging) {
          _end(true);
          return KeyEventResult.handled;
        }
        if (_editing) {
          setState(() {
            _editing = false;
            _error = null;
          });
          return KeyEventResult.handled;
        }
      }
      if (!_editing && !_dragging && widget.enabled) {
        if (event.logicalKey == LogicalKeyboardKey.enter) {
          _open();
          return KeyEventResult.handled;
        }
        if (event.logicalKey == LogicalKeyboardKey.arrowUp ||
            event.logicalKey == LogicalKeyboardKey.arrowDown) {
          widget.onCommit(
            _bounded(
              widget.value +
                  (event.logicalKey == LogicalKeyboardKey.arrowUp ? 1 : -1) *
                      widget.speed *
                      (HardwareKeyboard.instance.isShiftPressed ? 10 : 1),
            ),
          );
          return KeyEventResult.handled;
        }
      }
      return KeyEventResult.ignored;
    },
    child: _editing
        ? TextField(
            controller: _text,
            focusNode: _focus,
            autofocus: true,
            style: const TextStyle(
              fontSize: EditorMetrics.font,
              color: EditorTheme.ink,
            ),
            decoration: InputDecoration(
              isDense: true,
              contentPadding: const EdgeInsets.symmetric(
                horizontal: EditorMetrics.s2,
                vertical: EditorMetrics.s2,
              ),
              border: InputBorder.none,
              errorText: _error,
            ),
            onSubmitted: (_) => _commitText(),
          )
        : MouseRegion(
            cursor: widget.enabled
                ? SystemMouseCursors.resizeLeftRight
                : SystemMouseCursors.basic,
            child: GestureDetector(
              onDoubleTap: _open,
              child: Listener(
                behavior: HitTestBehavior.opaque,
                onPointerDown: _pointerDown,
                onPointerMove: _pointerMove,
                onPointerUp: _pointerUp,
                onPointerCancel: (event) => _pointerUp(event, cancel: true),
                child: Tooltip(
                  message: widget.label,
                  child: Container(
                    height: EditorMetrics.s18,
                    color: _dragging ? EditorTheme.hover : EditorTheme.app,
                    child: Stack(
                      fit: StackFit.expand,
                      children: [
                        if (widget.fill &&
                            widget.min != null &&
                            widget.max != null)
                          CustomPaint(
                            painter: _TrackPainter(
                              value: _shown ?? widget.value,
                              min: widget.min!,
                              max: widget.max!,
                              rest: widget.defaultValue,
                              tint: widget.tint ?? EditorTheme.raised,
                              style: widget.track,
                            ),
                          ),
                        Padding(
                          padding: const EdgeInsets.symmetric(
                            horizontal: EditorMetrics.s2,
                          ),
                          child: Row(
                            mainAxisAlignment: MainAxisAlignment.end,
                            children: [
                              Flexible(
                                child: Text(
                                  widget.mixed && _shown == null
                                      ? '—'
                                      : (_shown ?? widget.value)
                                            .toStringAsFixed(widget.decimals),
                                  maxLines: 1,
                                  overflow: TextOverflow.clip,
                                  style: TextStyle(
                                    fontSize: EditorMetrics.font,
                                    fontFeatures: const [
                                      FontFeature.tabularFigures(),
                                    ],
                                    color: widget.enabled
                                        ? EditorTheme.ink
                                        : EditorTheme.muted,
                                    // A number you can drag wears a dotted
                                    // underline; a read-only one does not.
                                    decoration: widget.enabled
                                        ? TextDecoration.underline
                                        : TextDecoration.none,
                                    decorationStyle: TextDecorationStyle.dotted,
                                    decorationColor:
                                        widget.tint ?? EditorTheme.muted,
                                  ),
                                ),
                              ),
                              // The rider keeps its slot even when empty, so
                              // digits line up down a column of wells.
                              if (widget.unit != null) ...[
                                const SizedBox(width: EditorMetrics.s2),
                                SizedBox(
                                  width: EditorMetrics.s12,
                                  child: Text(
                                    widget.unit!,
                                    style: const TextStyle(
                                      fontSize: EditorMetrics.micro,
                                      color: EditorTheme.muted,
                                    ),
                                  ),
                                ),
                              ],
                            ],
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ),
  );
}

/// How a value stands to time, shown as a lamp in the control's corner
/// (the Ableton convention): unlit = no keys, lit = keys, bright = a key at
/// this frame, ember = keys exist but the value was touched without Animate.
enum KeyLamp { none, keyed, now, draft }

KeyLamp keyLampOf(Map<String, dynamic>? row, {bool draft = false}) {
  if (row == null) return KeyLamp.none;
  final keys = row['keys'] as List? ?? const [];
  if (keys.isEmpty) return KeyLamp.none;
  if (draft) return KeyLamp.draft;
  return row['keyedNow'] == true ? KeyLamp.now : KeyLamp.keyed;
}

class EditorLamp extends StatelessWidget {
  const EditorLamp({super.key, required this.state, required this.child});
  final KeyLamp state;
  final Widget child;
  @override
  Widget build(BuildContext context) {
    final color = switch (state) {
      KeyLamp.none => null,
      KeyLamp.keyed => EditorTheme.accent.withValues(alpha: .55),
      KeyLamp.now => EditorTheme.accent,
      KeyLamp.draft => EditorTheme.accent.withValues(alpha: .3),
    };
    return Stack(
      clipBehavior: Clip.none,
      children: [
        child,
        if (color != null)
          Positioned(
            left: -1,
            top: -1,
            child: Tooltip(
              message: switch (state) {
                KeyLamp.now => 'Key at this frame',
                KeyLamp.draft => 'Keys exist; this change is not a key',
                _ => 'Animated',
              },
              child: Container(
                width: EditorMetrics.s5,
                height: EditorMetrics.s5,
                decoration: BoxDecoration(color: color, shape: BoxShape.circle),
              ),
            ),
          ),
      ],
    );
  }
}

/// On / off with the result drawn beside it, so the switch says what it does
/// without a word.
class EditorSwitch extends StatelessWidget {
  const EditorSwitch({
    super.key,
    required this.on,
    required this.glyph,
    required this.label,
    required this.onChanged,
    this.compact = false,
  });
  final bool on;
  final IconData glyph;
  final String label;
  final ValueChanged<bool>? onChanged;

  /// Glyph only, lit when on — for a slot too narrow for the track.
  final bool compact;
  @override
  Widget build(BuildContext context) {
    final enabled = onChanged != null;
    if (compact) {
      return Tooltip(
        message: label,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: enabled ? () => onChanged!(!on) : null,
          child: Icon(
            glyph,
            size: EditorMetrics.s16,
            color: !enabled
                ? EditorTheme.disabledInk
                : on
                ? EditorTheme.accent
                : EditorTheme.muted,
          ),
        ),
      );
    }
    return Tooltip(
      message: label,
      child: GestureDetector(
        onTap: enabled ? () => onChanged!(!on) : null,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            AnimatedContainer(
              duration: const Duration(milliseconds: 120),
              width: EditorMetrics.s22,
              height: EditorMetrics.s12,
              padding: const EdgeInsets.all(EditorMetrics.s2),
              decoration: BoxDecoration(
                color: on ? EditorTheme.accent : EditorTheme.raised,
                borderRadius: BorderRadius.circular(EditorMetrics.s6),
              ),
              alignment: on ? Alignment.centerRight : Alignment.centerLeft,
              child: Container(
                width: EditorMetrics.s8,
                height: EditorMetrics.s8,
                decoration: BoxDecoration(
                  color: on ? EditorTheme.tabInk : EditorTheme.ink,
                  shape: BoxShape.circle,
                ),
              ),
            ),
            const SizedBox(width: EditorMetrics.s4),
            Icon(
              glyph,
              size: EditorMetrics.s14,
              color: !enabled
                  ? EditorTheme.disabledInk
                  : on
                  ? EditorTheme.ink
                  : EditorTheme.muted,
            ),
          ],
        ),
      ),
    );
  }
}

/// An angle as a needle in a ring; drag to turn it. Preview while dragging,
/// finish on release, the same route as the numeric field.
class EditorDial extends StatefulWidget {
  const EditorDial({
    super.key,
    required this.degrees,
    required this.onBegin,
    required this.onPreview,
    required this.onFinish,
    required this.onCancel,
    this.enabled = true,
    this.size = EditorMetrics.s22,
    this.tint,
  });
  final double degrees, size;
  final bool enabled;
  final Color? tint;
  final VoidCallback onBegin;
  final Future<void> Function(double) onPreview;
  final Future<void> Function() onFinish, onCancel;
  @override
  State<EditorDial> createState() => _EditorDialState();
}

class _EditorDialState extends State<EditorDial> {
  double? _shown;
  double _lastAngle = 0;
  double _angleOf(Offset local) {
    final c = Offset(widget.size / 2, widget.size / 2);
    final d = local - c;
    return math.atan2(d.dy, d.dx) * 180 / math.pi;
  }

  @override
  Widget build(BuildContext context) => Tooltip(
    message: 'Rotation',
    child: GestureDetector(
      onPanStart: widget.enabled
          ? (e) {
              _lastAngle = _angleOf(e.localPosition);
              _shown = widget.degrees;
              widget.onBegin();
            }
          : null,
      onPanUpdate: widget.enabled
          ? (e) {
              final a = _angleOf(e.localPosition);
              var delta = a - _lastAngle;
              if (delta > 180) delta -= 360;
              if (delta < -180) delta += 360;
              _lastAngle = a;
              setState(() => _shown = (_shown ?? widget.degrees) + delta);
              widget.onPreview(_shown!);
            }
          : null,
      onPanEnd: widget.enabled
          ? (_) {
              widget.onFinish();
              setState(() => _shown = null);
            }
          : null,
      onPanCancel: widget.enabled
          ? () {
              widget.onCancel();
              setState(() => _shown = null);
            }
          : null,
      child: CustomPaint(
        size: Size.square(widget.size),
        painter: _DialPainter(
          _shown ?? widget.degrees,
          !widget.enabled ? EditorTheme.muted : widget.tint ?? EditorTheme.ink,
        ),
      ),
    ),
  );
}

class _DialPainter extends CustomPainter {
  const _DialPainter(this.degrees, this.ink);
  final double degrees;
  final Color ink;
  @override
  void paint(Canvas canvas, Size size) {
    final c = size.center(Offset.zero);
    final r = size.width / 2 - 1;
    canvas.drawCircle(
      c,
      r,
      Paint()
        ..color = EditorTheme.app
        ..style = PaintingStyle.fill,
    );
    canvas.drawCircle(
      c,
      r,
      Paint()
        ..color = EditorTheme.border
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1,
    );
    final a = (degrees - 90) * math.pi / 180;
    canvas.drawLine(
      c,
      c + Offset(math.cos(a), math.sin(a)) * (r - 2),
      Paint()
        ..color = ink
        ..strokeWidth = 1.5
        ..strokeCap = StrokeCap.round,
    );
  }

  @override
  bool shouldRepaint(_DialPainter old) =>
      old.degrees != degrees || old.ink != ink;
}

/// Where the layer turns and scales from: nine places, the current one lit.
class EditorAnchorGrid extends StatelessWidget {
  const EditorAnchorGrid({
    super.key,
    required this.fraction,
    required this.onPick,
    this.cell = EditorMetrics.s14,
  });
  final List<double>? fraction;
  final double cell;
  final void Function(double x, double y)? onPick;
  @override
  Widget build(BuildContext context) => Tooltip(
    message: 'Anchor',
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final y in [0.0, .5, 1.0])
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              for (final x in [0.0, .5, 1.0])
                GestureDetector(
                  onTap: onPick == null ? null : () => onPick!(x, y),
                  child: Container(
                    width: cell,
                    height: cell,
                    margin: const EdgeInsets.all(1),
                    decoration: BoxDecoration(
                      color:
                          fraction != null &&
                              (fraction![0] - x).abs() < .05 &&
                              (fraction![1] - y).abs() < .05
                          ? EditorTheme.accent
                          : onPick == null
                          ? EditorTheme.raised
                          : EditorTheme.border,
                      borderRadius: BorderRadius.circular(EditorMetrics.s2),
                    ),
                  ),
                ),
            ],
          ),
      ],
    ),
  );
}

/// A group of controls on one raised sheet, named by a glyph and a kicker.
class EditorCard extends StatelessWidget {
  const EditorCard({
    super.key,
    required this.title,
    required this.glyph,
    required this.children,
    this.trailing,
  });
  final String title;
  final IconData glyph;
  final List<Widget> children;
  final Widget? trailing;
  @override
  Widget build(BuildContext context) => Container(
    margin: const EdgeInsets.fromLTRB(
      EditorMetrics.s6,
      EditorMetrics.s6,
      EditorMetrics.s6,
      0,
    ),
    padding: const EdgeInsets.all(EditorMetrics.s6),
    decoration: BoxDecoration(
      color: EditorTheme.panel,
      borderRadius: BorderRadius.circular(EditorMetrics.s3),
    ),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Icon(glyph, size: EditorMetrics.s12, color: EditorTheme.muted),
            const SizedBox(width: EditorMetrics.s4),
            Expanded(
              child: Text(
                title.toUpperCase(),
                style: const TextStyle(
                  fontSize: EditorMetrics.micro,
                  letterSpacing: 1,
                  color: EditorTheme.muted,
                ),
              ),
            ),
            if (trailing != null) trailing!,
          ],
        ),
        const SizedBox(height: EditorMetrics.s6),
        ...children,
      ],
    ),
  );
}

/// Two values as one point on a square. Dragging moves the point by the
/// pointer's own distance (no range needed); the dot shows where the pair
/// stands within [span] of the centre.
class EditorPad extends StatefulWidget {
  const EditorPad({
    super.key,
    required this.x,
    required this.y,
    required this.onBegin,
    required this.onPreview,
    required this.onFinish,
    required this.onCancel,
    this.enabled = true,
    this.size = EditorMetrics.s70,
    this.span = EditorMetrics.s200,
    this.speed = 1,
    this.tint,
  });
  final double x, y, size, span, speed;
  final bool enabled;
  final Color? tint;
  final VoidCallback onBegin;
  final void Function(double x, double y) onPreview;
  final Future<void> Function() onFinish, onCancel;
  @override
  State<EditorPad> createState() => _EditorPadState();
}

class _EditorPadState extends State<EditorPad> {
  Offset? _shown;
  @override
  Widget build(BuildContext context) => Tooltip(
    message: 'Drag the point',
    child: MouseRegion(
      cursor: widget.enabled
          ? SystemMouseCursors.move
          : SystemMouseCursors.basic,
      child: GestureDetector(
        onPanStart: widget.enabled
            ? (_) {
                _shown = Offset(widget.x, widget.y);
                widget.onBegin();
              }
            : null,
        onPanUpdate: widget.enabled
            ? (e) {
                final next =
                    (_shown ?? Offset(widget.x, widget.y)) +
                    e.delta * widget.speed;
                setState(() => _shown = next);
                widget.onPreview(next.dx, next.dy);
              }
            : null,
        onPanEnd: widget.enabled
            ? (_) {
                widget.onFinish();
                setState(() => _shown = null);
              }
            : null,
        onPanCancel: widget.enabled
            ? () {
                widget.onCancel();
                setState(() => _shown = null);
              }
            : null,
        child: CustomPaint(
          size: Size.square(widget.size),
          painter: _PadPainter(
            _shown ?? Offset(widget.x, widget.y),
            widget.span,
            !widget.enabled
                ? EditorTheme.muted
                : widget.tint ?? EditorTheme.accent,
          ),
        ),
      ),
    ),
  );
}

class _PadPainter extends CustomPainter {
  const _PadPainter(this.at, this.span, this.dot);
  final Offset at;
  final double span;
  final Color dot;
  @override
  void paint(Canvas canvas, Size size) {
    final r = RRect.fromRectAndRadius(
      Offset.zero & size,
      const Radius.circular(EditorMetrics.s3),
    );
    canvas.drawRRect(r, Paint()..color = EditorTheme.app);
    final c = size.center(Offset.zero);
    final hair = Paint()
      ..color = EditorTheme.line
      ..strokeWidth = 1;
    canvas.drawLine(Offset(c.dx, 0), Offset(c.dx, size.height), hair);
    canvas.drawLine(Offset(0, c.dy), Offset(size.width, c.dy), hair);
    final half = size.width / 2 - EditorMetrics.s4;
    final p = Offset(
      c.dx + (at.dx / span * half).clamp(-half, half),
      c.dy + (at.dy / span * half).clamp(-half, half),
    );
    canvas.drawLine(c, p, Paint()..color = EditorTheme.border);
    canvas.drawCircle(p, EditorMetrics.s4, Paint()..color = dot);
    canvas.drawRRect(
      r,
      Paint()
        ..color = EditorTheme.border
        ..style = PaintingStyle.stroke,
    );
  }

  @override
  bool shouldRepaint(_PadPainter old) =>
      old.at != at || old.span != span || old.dot != dot;
}

/// How a track tells its amount, by what the number is for.
enum TrackStyle {
  /// So much of the reach, from where it rests.
  fill,

  /// What lies above this passes: the far side is lit.
  level,

  /// Whole steps, one mark each.
  steps,

  /// A length along a ruler: fill under ticks.
  ruler,
}

/// The amount behind a bounded number, in the family's hue, with a tick
/// where the rest point is.
class _TrackPainter extends CustomPainter {
  const _TrackPainter({
    required this.value,
    required this.min,
    required this.max,
    required this.tint,
    required this.style,
    this.rest,
  });
  final double value, min, max;
  final double? rest;
  final Color tint;
  final TrackStyle style;
  double _x(double v, double w) =>
      ((v - min) / (max - min)).clamp(0.0, 1.0) * w;
  @override
  void paint(Canvas canvas, Size size) {
    final wash = Paint()..color = tint.withValues(alpha: .28);
    final b = _x(value, size.width);
    switch (style) {
      case TrackStyle.fill:
      case TrackStyle.ruler:
        final zero = rest != null && rest! > min && rest! < max ? rest! : min;
        final a = _x(zero, size.width);
        canvas.drawRect(
          Rect.fromLTRB(math.min(a, b), 0, math.max(a, b), size.height),
          wash,
        );
        if (style == TrackStyle.ruler) {
          final tick = Paint()..color = tint.withValues(alpha: .5);
          for (var i = 1; i < 8; i++) {
            final x = size.width * i / 8;
            canvas.drawLine(
              Offset(x, size.height - (i.isEven ? 5 : 3)),
              Offset(x, size.height),
              tick,
            );
          }
        }
      case TrackStyle.level:
        canvas.drawRect(Rect.fromLTRB(b, 0, size.width, size.height), wash);
        canvas.drawLine(
          Offset(b, 0),
          Offset(b, size.height),
          Paint()
            ..color = tint
            ..strokeWidth = 1.5,
        );
      case TrackStyle.steps:
        final span = (max - min).round().clamp(1, 12);
        final cell = size.width / span;
        final lit = (value - min).round().clamp(0, span);
        for (var i = 0; i < span; i++) {
          canvas.drawRect(
            Rect.fromLTRB(
              i * cell + 1,
              size.height - 4,
              (i + 1) * cell - 1,
              size.height - 1,
            ),
            i < lit ? (Paint()..color = tint) : wash,
          );
        }
    }
    if (rest != null) {
      final x = _x(rest!, size.width);
      canvas.drawLine(
        Offset(x, 0),
        Offset(x, size.height),
        Paint()
          ..color = EditorTheme.border
          ..strokeWidth = 1,
      );
    }
  }

  @override
  bool shouldRepaint(_TrackPainter old) =>
      old.value != value ||
      old.min != min ||
      old.max != max ||
      old.rest != rest ||
      old.tint != tint ||
      old.style != style;
}
