import 'dart:async';
import 'dart:io';
import 'dart:math' as math;
import 'dart:ui' show ViewFocusEvent, ViewFocusState;

import 'package:flutter/gestures.dart';
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
      style: EditorTheme.menuSheet,
      menuChildren: [
        for (final e in choices)
          MenuItemButton(
            style: EditorTheme.menuRow,
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
    this.owner,
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

  /// Whose number this well shows. The same well stays in place while the
  /// panel is handed another owner's row — its keys hold the row, not the
  /// owner, so the render objects are kept — and a draft left open belongs to
  /// the owner it was typed for, so it is dropped rather than committed here.
  final Object? owner;
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
  bool _editing = false, _dragging = false, _ending = false;
  int? _pointer;
  double _start = 0, _startGlobalX = 0, _startGlobalY = 0;
  double? _shown;
  late final _queue = EditorPreviewQueue<double>((v) => widget.onPreview(v));
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

  @override
  void didUpdateWidget(covariant EditorNumericField old) {
    super.didUpdateWidget(old);
    if (old.owner != widget.owner && (_editing || _shown != null)) {
      setState(() {
        _editing = false;
        _error = null;
        _shown = null;
      });
    }
  }

  double _bounded(double n) => n
      .clamp(
        widget.min ?? double.negativeInfinity,
        widget.max ?? double.infinity,
      )
      .toDouble();
  void _open() {
    if (!widget.enabled || _ending) return;
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
    if (!_editing) return;
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
    _idle.requestFocus();
    widget.onCommit(_bounded(n));
  }

  void _tick(double n) => _queue.add(n);

  void _pointerDown(PointerDownEvent event) {
    if (!widget.enabled || _ending || _pointer != null || event.buttons != 1)
      return;
    _pointer = event.pointer;
    _start = widget.value;
    _startGlobalX = event.position.dx;
    _startGlobalY = event.position.dy;
    // A press lands on a two-finger scrub still settling: the press takes
    // the session over, and the release will finish it.
    if (_dragging) {
      _settle?.cancel();
      _settle = null;
      _rungBase = _shown ?? widget.value;
      _rungStartX = event.position.dx;
    }
    _idle.requestFocus();
  }

  /// The wheel: with the left button held, each notch steps the value by
  /// one unit (Shift ×10) — the mouse's way. A horizontal scroll needs no
  /// button and moves the value as a drag would — the trackpad's two-finger
  /// way — and finishes once the fingers rest. On macOS the trackpad's
  /// two fingers arrive as a pan gesture, not a scroll signal; a horizontal
  /// drag recognizer meets the surrounding list's vertical one in the arena,
  /// so sideways is the number's and up-and-down stays the list's.
  Timer? _settle;
  void _panUpdate(DragUpdateDetails details) {
    File('/tmp/motolii-diag.log').writeAsStringSync(
      '${DateTime.now().toIso8601String()} panUpdate dx=${details.delta.dx} dragging=$_dragging pointer=$_pointer ending=$_ending\n',
      mode: FileMode.append,
    ); // DIAG(temp)
    if (!widget.enabled || _editing || _ending || _pointer != null) return;
    final by = HardwareKeyboard.instance.isShiftPressed ? 10 : 1;
    // The pan is reported as the content's motion (natural scrolling), the
    // mirror of the fingers; fingers moving right raise the number.
    _nudge(
      -details.delta.dx * widget.speed * _rung * by,
      details.globalPosition.dx,
      settle: false,
    );
  }

  void _panEnd() {
    File('/tmp/motolii-diag.log').writeAsStringSync(
      '${DateTime.now().toIso8601String()} panEnd dragging=$_dragging pointer=$_pointer\n',
      mode: FileMode.append,
    ); // DIAG(temp)
    if (_pointer == null) _end(false);
  }

  void _pointerSignal(PointerSignalEvent event) {
    File('/tmp/motolii-diag.log').writeAsStringSync(
      '${DateTime.now().toIso8601String()} signal ${event.runtimeType} ${event is PointerScrollEvent ? event.scrollDelta : ''} kind=${event.kind}\n',
      mode: FileMode.append,
    ); // DIAG(temp)
    if (event is! PointerScrollEvent) return;
    if (!widget.enabled || _editing || _ending) return;
    final held = _pointer != null;
    final delta = event.scrollDelta;
    final horizontal = delta.dx.abs() > delta.dy.abs();
    if (!held && !horizontal) return;
    GestureBinding.instance.pointerSignalResolver.register(event, (_) {
      final by = HardwareKeyboard.instance.isShiftPressed ? 10 : 1;
      final step = horizontal
          ? delta.dx * widget.speed * _rung * by
          : -delta.dy.sign * widget.speed * _rung * by;
      _nudge(step, event.position.dx, settle: !held);
    });
  }

  void _nudge(double step, double x, {required bool settle}) {
    if (!_dragging) {
      widget.onBegin?.call();
      _rung = 1;
      setState(() => _dragging = true);
    }
    final n = _bounded((_shown ?? widget.value) + step);
    _rungBase = n;
    _rungStartX = x;
    setState(() => _shown = n);
    _tick(n);
    _settle?.cancel();
    _settle = settle
        ? Timer(const Duration(milliseconds: 300), () => _end(false))
        : null;
  }

  /// The drag's precision, picked by how far the pointer has moved up or
  /// down since it pressed: level is ×1, above is ×10, below ×0.1, further
  /// below ×0.01 (the Figma ladder). Changing rung mid-drag keeps the value.
  double _rung = 1;
  double _rungBase = 0, _rungStartX = 0;
  double _rungFor(double dy) => dy < -EditorMetrics.s32
      ? 10
      : dy < EditorMetrics.s32
      ? 1
      : dy < EditorMetrics.s70
      ? .1
      : .01;

  void _pointerMove(PointerMoveEvent event) {
    if (event.pointer != _pointer || event.buttons != 1) return;
    final displacement = event.position.dx - _startGlobalX;
    if (!_dragging) {
      if (displacement.abs() < 3) return;
      widget.onBegin?.call();
      _rung = 1;
      _rungBase = _start;
      _rungStartX = _startGlobalX;
      setState(() => _dragging = true);
    }
    final rung = _rungFor(event.position.dy - _startGlobalY);
    if (rung != _rung) {
      _rungBase = _shown ?? widget.value;
      _rungStartX = event.position.dx;
      _rung = rung;
    }
    final n = _bounded(
      _rungBase + (event.position.dx - _rungStartX) * widget.speed * _rung,
    );
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
    _settle?.cancel();
    _settle = null;
    _pointer = null;
    setState(() => _dragging = false);
    _ending = true;
    try {
      await _queue.finish(cancel, cancel ? widget.onCancel : widget.onFinish);
    } finally {
      _ending = false;
      if (mounted) setState(() => _shown = null);
    }
  }

  @override
  void dispose() {
    _settle?.cancel();
    if (_dragging) {
      _queue.finish(true, widget.onCancel);
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
      if (!_editing && !_dragging && !_ending && widget.enabled) {
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
              child: GestureDetector(
                supportedDevices: const {PointerDeviceKind.trackpad},
                onHorizontalDragUpdate: _panUpdate,
                onHorizontalDragEnd: (_) => _panEnd(),
                onHorizontalDragCancel: _panEnd,
                child: Listener(
                  behavior: HitTestBehavior.opaque,
                  onPointerDown: _pointerDown,
                  onPointerMove: _pointerMove,
                  onPointerUp: _pointerUp,
                  onPointerCancel: (event) => _pointerUp(event, cancel: true),
                  onPointerSignal: _pointerSignal,
                  child: EditorTooltip(
                    message: _dragging && _rung != 1
                        ? '${widget.label} ×$_rung'
                        : widget.label,
                    child: Container(
                      height: EditorMetrics.row,
                      decoration: BoxDecoration(
                        color: _dragging ? EditorTheme.hover : EditorTheme.app,
                        border: Border.all(color: EditorTheme.line),
                      ),
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
                                      // Resting at its default the number is
                                      // quiet; moved, it is ink.
                                      color: !widget.enabled
                                          ? EditorTheme.muted
                                          : widget.defaultValue != null &&
                                                (widget.value -
                                                            widget
                                                                .defaultValue!)
                                                        .abs() <
                                                    .0005
                                          ? EditorTheme.tab
                                          : EditorTheme.ink,
                                      // A number you can drag wears a dotted
                                      // underline, unless a track already says
                                      // so; a read-only one never does.
                                      decoration:
                                          widget.enabled &&
                                              !(widget.fill &&
                                                  widget.min != null &&
                                                  widget.max != null)
                                          ? TextDecoration.underline
                                          : TextDecoration.none,
                                      decorationStyle:
                                          TextDecorationStyle.dotted,
                                      decorationColor: EditorTheme.muted,
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

class EditorLamp extends StatefulWidget {
  const EditorLamp({
    super.key,
    required this.state,
    required this.child,
    this.onTap,
  });
  final KeyLamp state;
  final Widget child;

  /// Press the lamp to key the value at this frame, or take that key away.
  final VoidCallback? onTap;
  @override
  State<EditorLamp> createState() => _EditorLampState();
}

class _EditorLampState extends State<EditorLamp> {
  bool _hover = false;
  @override
  Widget build(BuildContext context) {
    final state = widget.state;
    final color = switch (state) {
      KeyLamp.none => null,
      KeyLamp.keyed => EditorTheme.keyAccent.withValues(alpha: .55),
      KeyLamp.now => EditorTheme.keyAccent,
      KeyLamp.draft => EditorTheme.keyAccent.withValues(alpha: .3),
    };
    // Unlit lamps show as a hollow ring only while the pointer is near, so
    // the corner stays quiet until it is wanted.
    final shown = color != null || (_hover && widget.onTap != null);
    final hit = widget.onTap != null;
    return MouseRegion(
      onEnter: (_) => setState(() => _hover = true),
      onExit: (_) => setState(() => _hover = false),
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          widget.child,
          if (shown || hit)
            Positioned(
              left: 0,
              top: 0,
              child: EditorTooltip(
                message: switch (state) {
                  KeyLamp.now => 'Key at this frame (press to remove)',
                  KeyLamp.draft => 'Keys exist; this change is not a key',
                  KeyLamp.keyed => 'Animated (press to key this frame)',
                  KeyLamp.none => 'Press to key this frame',
                },
                child: GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTap: widget.onTap,
                  child: Container(
                    width: EditorMetrics.s8,
                    height: EditorMetrics.s8,
                    alignment: Alignment.center,
                    child: shown
                        ? Container(
                            width: EditorMetrics.s5,
                            height: EditorMetrics.s5,
                            decoration: BoxDecoration(
                              color: color,
                              shape: BoxShape.circle,
                              border: color == null
                                  ? Border.all(color: EditorTheme.muted)
                                  : null,
                            ),
                          )
                        : null,
                  ),
                ),
              ),
            ),
        ],
      ),
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
    this.tint,
  });
  final bool on;
  final IconData glyph;
  final String label;
  final ValueChanged<bool>? onChanged;

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
          child: Icon(
            glyph,
            size: EditorMetrics.s16,
            color: !enabled
                ? EditorTheme.disabledInk
                : on
                ? tint ?? EditorTheme.accent
                : EditorTheme.muted,
          ),
        ),
      );
    }
    return EditorTooltip(
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
                color: on ? tint ?? EditorTheme.accent : EditorTheme.raised,
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

// Uses the numeric field's latest-pending-value rule for continuous controls.
class EditorPreviewQueue<T> {
  EditorPreviewQueue(this.send);
  final Future<void> Function(T) send;
  T? _pending;
  bool _sending = false;
  Future<void> _drained = Future<void>.value();

  /// Settles when nothing is left to send.
  Future<void> get drained => _drained;
  void add(T value) {
    _pending = value;
    if (_sending) return;
    _sending = true;
    _drained = () async {
      try {
        while (_pending != null) {
          final next = _pending as T;
          _pending = null;
          await send(next);
        }
      } finally {
        _sending = false;
      }
    }();
  }

  Future<void> finish(bool cancel, Future<void> Function() action) async {
    if (cancel) _pending = null;
    await _drained;
    await action();
  }
}

class _EditorDialState extends State<EditorDial> with WidgetsBindingObserver {
  double? _shown;
  double _lastAngle = 0;
  bool _dragging = false, _ending = false;
  late final _queue = EditorPreviewQueue<double>((v) => widget.onPreview(v));
  final _gestureFocus = FocusNode();
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) _end(true);
  }

  @override
  void didChangeViewFocus(ViewFocusEvent event) {
    if (event.state == ViewFocusState.unfocused) _end(true);
  }

  Future<void> _end(bool cancel) async {
    if (!_dragging) return;
    _dragging = false;
    _ending = true;
    try {
      await _queue.finish(cancel, cancel ? widget.onCancel : widget.onFinish);
    } finally {
      _ending = false;
      if (mounted) setState(() => _shown = null);
    }
  }

  @override
  void dispose() {
    if (_dragging) _queue.finish(true, widget.onCancel);
    WidgetsBinding.instance.removeObserver(this);
    _gestureFocus.dispose();
    super.dispose();
  }

  double _angleOf(Offset local) {
    final c = Offset(widget.size / 2, widget.size / 2);
    final d = local - c;
    return math.atan2(d.dy, d.dx) * 180 / math.pi;
  }

  @override
  Widget build(BuildContext context) => Focus(
    focusNode: _gestureFocus,
    onKeyEvent: (_, event) {
      if (event is KeyDownEvent &&
          event.logicalKey == LogicalKeyboardKey.escape &&
          _dragging) {
        _end(true);
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    },
    child: Listener(
      onPointerCancel: (_) => _end(true),
      child: EditorTooltip(
        message: 'Rotation',
        child: GestureDetector(
          onPanStart: widget.enabled
              ? (e) {
                  if (_ending) return;
                  _dragging = true;
                  _gestureFocus.requestFocus();
                  _lastAngle = _angleOf(e.localPosition);
                  _shown = widget.degrees;
                  widget.onBegin();
                }
              : null,
          onPanUpdate: widget.enabled
              ? (e) {
                  if (!_dragging) return;
                  final a = _angleOf(e.localPosition);
                  var delta = a - _lastAngle;
                  if (delta > 180) delta -= 360;
                  if (delta < -180) delta += 360;
                  _lastAngle = a;
                  setState(() => _shown = (_shown ?? widget.degrees) + delta);
                  _queue.add(_shown!);
                }
              : null,
          onPanEnd: widget.enabled ? (_) => _end(false) : null,
          onPanCancel: widget.enabled ? () => _end(true) : null,
          child: CustomPaint(
            size: Size.square(widget.size),
            painter: _DialPainter(
              _shown ?? widget.degrees,
              !widget.enabled
                  ? EditorTheme.muted
                  : widget.tint ?? EditorTheme.ink,
            ),
          ),
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
    this.onHover,
    this.cell = EditorMetrics.s14,
  });
  final List<double>? fraction;
  final double cell;
  final void Function(double x, double y)? onPick;

  /// The cell under the pointer, or null when it leaves — so the Stage can
  /// show where that pivot would land before it is chosen.
  final void Function(double x, double y, bool inside)? onHover;
  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: 'Anchor',
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final y in [0.0, .5, 1.0])
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              for (final x in [0.0, .5, 1.0])
                MouseRegion(
                  onEnter: (_) => onHover?.call(x, y, true),
                  onExit: (_) => onHover?.call(x, y, false),
                  child: GestureDetector(
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
    required this.children,
    this.leading,
    this.trailing,
    this.dim = false,
  });
  final String title;
  final List<Widget> children;
  final Widget? leading, trailing;

  /// The card's contents faded: what it holds is not applied right now.
  final bool dim;
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
            if (leading != null) leading!,
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
        if (dim)
          Opacity(
            opacity: .4,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: children,
            ),
          )
        else
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
  final Future<void> Function(double x, double y) onPreview;
  final Future<void> Function() onFinish, onCancel;
  @override
  State<EditorPad> createState() => _EditorPadState();
}

class _EditorPadState extends State<EditorPad> with WidgetsBindingObserver {
  Offset? _shown;
  bool _dragging = false, _ending = false;
  late final _queue = EditorPreviewQueue<Offset>(
    (v) => widget.onPreview(v.dx, v.dy),
  );
  final _gestureFocus = FocusNode();
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) _end(true);
  }

  @override
  void didChangeViewFocus(ViewFocusEvent event) {
    if (event.state == ViewFocusState.unfocused) _end(true);
  }

  Future<void> _end(bool cancel) async {
    if (!_dragging) return;
    _dragging = false;
    _ending = true;
    try {
      await _queue.finish(cancel, cancel ? widget.onCancel : widget.onFinish);
    } finally {
      _ending = false;
      if (mounted) setState(() => _shown = null);
    }
  }

  @override
  void dispose() {
    if (_dragging) _queue.finish(true, widget.onCancel);
    WidgetsBinding.instance.removeObserver(this);
    _gestureFocus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
    focusNode: _gestureFocus,
    onKeyEvent: (_, event) {
      if (event is KeyDownEvent &&
          event.logicalKey == LogicalKeyboardKey.escape &&
          _dragging) {
        _end(true);
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    },
    child: Listener(
      onPointerCancel: (_) => _end(true),
      child: EditorTooltip(
        message: 'Drag the point',
        child: MouseRegion(
          cursor: widget.enabled
              ? SystemMouseCursors.move
              : SystemMouseCursors.basic,
          child: GestureDetector(
            onPanStart: widget.enabled
                ? (_) {
                    if (_ending) return;
                    _dragging = true;
                    _gestureFocus.requestFocus();
                    _shown = Offset(widget.x, widget.y);
                    widget.onBegin();
                  }
                : null,
            onPanUpdate: widget.enabled
                ? (e) {
                    if (!_dragging) return;
                    final next =
                        (_shown ?? Offset(widget.x, widget.y)) +
                        e.delta * widget.speed;
                    setState(() => _shown = next);
                    _queue.add(next);
                  }
                : null,
            onPanEnd: widget.enabled ? (_) => _end(false) : null,
            onPanCancel: widget.enabled ? () => _end(true) : null,
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
  @override
  Widget build(BuildContext context) {
    final low = (min / base * 100).ceilToDouble();
    final high = (max / base * 100).floorToDouble();
    final percent = value / base * 100;
    void change(double n) =>
        onChanged(n.roundToDouble().clamp(low, high) * base / 100);
    Widget step(IconData icon, int delta, String suffix) => InkWell(
      key: ValueKey('$keyPrefix-$suffix'),
      onTap: () => change(percent.roundToDouble() + delta),
      child: SizedBox(
        width: EditorMetrics.row,
        child: Icon(icon, size: EditorMetrics.s14, color: EditorTheme.muted),
      ),
    );
    return Container(
      height: EditorMetrics.row,
      decoration: const BoxDecoration(
        border: Border(top: BorderSide(color: EditorTheme.line)),
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
              step(Icons.remove, -this.step, 'smaller'),
              if (box.maxWidth >= EditorMetrics.cell) ...[
                Expanded(
                  child: Slider(
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
              step(Icons.add, this.step, 'larger'),
            ],
          );
        },
      ),
    );
  }
}

/// The transparency grid: the picture editors' two greys, 8 px squares.
class CheckerPainter extends CustomPainter {
  const CheckerPainter({this.cell = 8});
  final double cell;
  static const light = Color(0xff8c8c8c), dark = Color(0xff666666);
  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, Paint()..color = light);
    final paint = Paint()..color = dark;
    for (var y = 0; y * cell < size.height; y++) {
      for (var x = (y % 2); x * cell < size.width; x += 2) {
        canvas.drawRect(Rect.fromLTWH(x * cell, y * cell, cell, cell), paint);
      }
    }
  }

  @override
  bool shouldRepaint(CheckerPainter old) => old.cell != cell;
}
