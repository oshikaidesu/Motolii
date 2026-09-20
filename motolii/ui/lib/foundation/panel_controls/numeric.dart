import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../leaves.dart';
import '../metrics.dart';
import '../theme.dart';
import 'drag.dart';
import 'fields.dart';

part 'numeric_paint.dart';

/// The well a number is scrubbed, typed and read in.

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
    this.zeroWord,
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

  /// A small rider after the number: px, %, °, or a short plain word the
  /// field names itself by (cols, rows, s) — Figma's way of labelling a
  /// number inside its own box instead of in a column beside it.
  final String? unit;

  /// How many digits after the point the number reads at; 0 for a count,
  /// which must read as a whole number (2, not 2.00).
  final int decimals;

  /// The word this number reads as at zero, when zero has a meaning of its
  /// own rather than a size: grid rows at 0 are `auto`. The word replaces
  /// the digits for reading only — the field still scrubs and types as the
  /// number it is.
  final String? zeroWord;

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

class _EditorNumericFieldState extends State<EditorNumericField>
    with WidgetsBindingObserver, EditorDragSession<double, EditorNumericField> {
  final _text = TextEditingController();
  final _focus = FocusNode();
  final _ownIdle = FocusNode();
  FocusNode get _idle => widget.idleFocus ?? _ownIdle;
  bool _editing = false;
  int? _pointer;
  double _start = 0, _startGlobalX = 0, _startGlobalY = 0;
  double? _shown;

  /// The ladder's pill hangs over the well while a scrub is off the ×1 rung.
  final _well = LayerLink();
  final _pill = OverlayPortalController();
  void _showRung() {
    final wanted = dragging && _rung != 1;
    if (wanted && !_pill.isShowing) _pill.show();
    if (!wanted && _pill.isShowing) _pill.hide();
  }

  // The field is the one control a lost window does not cancel: a two-finger
  // scrub settles on its own timer.
  @override
  bool get watchesWindow => false;
  @override
  Future<void> sendPreview(double value) => widget.onPreview(value);
  @override
  Future<void> commitDrag() => widget.onFinish();
  @override
  Future<void> cancelDrag() => widget.onCancel();
  @override
  void dragStopped(bool cancel) {
    _settle?.cancel();
    _settle = null;
    _pointer = null;
    setState(() {});
    _showRung();
  }

  @override
  void dragSettled() => setState(() => _shown = null);
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

  /// A grabbed well with no track of its own turns to its family's flat
  /// colour, ink reversed: the touch answers the instant it lands.
  bool get _flooded =>
      dragging &&
      widget.enabled &&
      widget.tint != null &&
      !(widget.fill && widget.min != null && widget.max != null);

  double _bounded(double n) => n
      .clamp(
        widget.min ?? double.negativeInfinity,
        widget.max ?? double.infinity,
      )
      .toDouble();

  /// What the well reads right now. Typing starts from this exact string —
  /// the number must not turn into a different one under the caret (0.00 →
  /// 0.0, -19 → -18.999999).
  String get _reading =>
      (_shown ?? widget.value).toStringAsFixed(widget.decimals);

  /// What the well shows when it is not being typed into: the digits, or
  /// the word zero stands for on this row.
  String get _shownText {
    final word = widget.zeroWord;
    if (word != null && (_shown ?? widget.value).abs() < .0005) return word;
    return _reading;
  }

  void _open() {
    if (!widget.enabled || ending) return;
    setState(() {
      _editing = true;
      _text.text = widget.mixed ? '' : _reading;
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

  void _tick(double n) => queue.add(n);

  void _pointerDown(PointerDownEvent event) {
    if (!widget.enabled || ending || _pointer != null || event.buttons != 1)
      return;
    _pointer = event.pointer;
    _start = widget.value;
    _startGlobalX = event.position.dx;
    _startGlobalY = event.position.dy;
    // A press lands on a two-finger scrub still settling: the press takes
    // the session over, and the release will finish it.
    if (dragging) {
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
    if (!widget.enabled || _editing || ending || _pointer != null) return;
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
    if (_pointer == null) endDrag(false);
  }

  void _pointerSignal(PointerSignalEvent event) {
    if (event is! PointerScrollEvent) return;
    if (!widget.enabled || _editing || ending) return;
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
    if (!dragging) {
      widget.onBegin?.call();
      _rung = 1;
      setState(() => dragging = true);
    }
    final n = _bounded((_shown ?? widget.value) + step);
    _rungBase = n;
    _rungStartX = x;
    setState(() => _shown = n);
    _tick(n);
    _settle?.cancel();
    _settle = settle
        ? Timer(const Duration(milliseconds: 300), () => endDrag(false))
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
    if (!dragging) {
      if (displacement.abs() < 3) return;
      widget.onBegin?.call();
      _rung = 1;
      _rungBase = _start;
      _rungStartX = _startGlobalX;
      setState(() => dragging = true);
    }
    final rung = _rungFor(event.position.dy - _startGlobalY);
    if (rung != _rung) {
      _rungBase = _shown ?? widget.value;
      _rungStartX = event.position.dx;
      _rung = rung;
      _showRung();
    }
    final n = _bounded(
      _rungBase + (event.position.dx - _rungStartX) * widget.speed * _rung,
    );
    setState(() => _shown = n);
    _tick(n);
  }

  void _pointerUp(PointerEvent event, {bool cancel = false}) {
    if (event.pointer != _pointer) return;
    if (dragging) {
      endDrag(cancel);
    } else {
      _pointer = null;
    }
  }

  @override
  void dispose() {
    _settle?.cancel();
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
        if (dragging) {
          endDrag(true);
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
      if (!_editing && !dragging && !ending && widget.enabled) {
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
    // Typing happens inside the well, not in place of it: the same box, the
    // same digits, right-aligned under the same rider. Only the caret is new.
    child: _editing
        ? EditorTooltip(
            message: _error ?? widget.label,
            child: EditorFieldFrame(
              focus: _focus,
              error: _error != null,
              padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s2),
              child: Row(
                children: [
                  Expanded(
                    child: EditorTextField(
                      controller: _text,
                      focusNode: _focus,
                      autofocus: true,
                      textAlign: TextAlign.right,
                      style: TextStyle(
                        fontSize: EditorMetrics.font,
                        color: EditorTheme.of(context).ink,
                        fontFeatures: [FontFeature.tabularFigures()],
                      ),
                      cursorWidth: 1,
                      cursorColor: EditorTheme.of(context).ink,
                      onSubmitted: (_) => _commitText(),
                    ),
                  ),
                  if (widget.unit != null) ...[
                    const SizedBox(width: EditorMetrics.s2),
                    ConstrainedBox(
                      constraints: const BoxConstraints(
                        minWidth: EditorMetrics.s12,
                      ),
                      child: Text(
                        widget.unit!,
                        style: TextStyle(
                          fontSize: EditorMetrics.micro,
                          color: EditorTheme.of(context).muted,
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
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
                    message: dragging && _rung != 1
                        ? '${widget.label} ×$_rung'
                        : widget.enabled
                        ? '${widget.label} · drag to adjust · double-click or Enter to type'
                        : widget.label,
                    child: OverlayPortal(
                      controller: _pill,
                      overlayChildBuilder: (_) => CompositedTransformFollower(
                        link: _well,
                        targetAnchor: Alignment.topCenter,
                        followerAnchor: Alignment.bottomCenter,
                        offset: const Offset(0, -EditorMetrics.s4),
                        child: IgnorePointer(
                          child: Align(
                            alignment: Alignment.bottomCenter,
                            child: _RungPill(rung: _rung),
                          ),
                        ),
                      ),
                      child: CompositedTransformTarget(
                        link: _well,
                        child: Container(
                          height: EditorMetrics.row,
                          decoration: BoxDecoration(
                            color: _flooded
                                ? widget.tint
                                : dragging
                                ? EditorTheme.of(context).hover
                                : EditorTheme.of(context).app,
                            // The family's rule down the left edge: a flat
                            // colour the eye can follow down a column of wells,
                            // drawn by the well's own border.
                            border: widget.tint != null && widget.enabled
                                ? Border(
                                    left: BorderSide(
                                      color: widget.tint!,
                                      width: EditorMetrics.s3,
                                    ),
                                    top: BorderSide(
                                      color: EditorTheme.of(context).line,
                                    ),
                                    right: BorderSide(
                                      color: EditorTheme.of(context).line,
                                    ),
                                    bottom: BorderSide(
                                      color: EditorTheme.of(context).line,
                                    ),
                                  )
                                : Border.all(
                                    color: EditorTheme.of(context).line,
                                  ),
                          ),
                          child: Stack(
                            fit: StackFit.expand,
                            children: [
                              if (widget.fill &&
                                  widget.min != null &&
                                  widget.max != null)
                                CustomPaint(
                                  painter: _TrackPainter(
                                    colors: EditorTheme.of(context),
                                    value: _shown ?? widget.value,
                                    min: widget.min!,
                                    max: widget.max!,
                                    rest: widget.defaultValue,
                                    tint:
                                        widget.tint ??
                                        EditorTheme.of(context).raised,
                                    style: widget.track,
                                  ),
                                ),
                              Padding(
                                padding: const EdgeInsets.symmetric(
                                  horizontal: EditorMetrics.s4,
                                ),
                                child: Row(
                                  mainAxisAlignment: MainAxisAlignment.end,
                                  children: [
                                    Flexible(
                                      child: Text(
                                        widget.mixed && _shown == null
                                            ? '—'
                                            : _shownText,
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
                                              ? EditorTheme.of(context).muted
                                              : _flooded
                                              ? EditorTheme.of(context).tabInk
                                              : widget.defaultValue != null &&
                                                    (widget.value -
                                                                widget
                                                                    .defaultValue!)
                                                            .abs() <
                                                        .0005
                                              ? EditorTheme.of(context).tab
                                              : EditorTheme.of(context).ink,
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
                                          decorationColor: EditorTheme.of(
                                            context,
                                          ).muted,
                                        ),
                                      ),
                                    ),
                                    // The rider keeps its slot even when empty, so
                                    // digits line up down a column of wells.
                                    if (widget.unit != null) ...[
                                      const SizedBox(width: EditorMetrics.s2),
                                      ConstrainedBox(
                                        constraints: const BoxConstraints(
                                          minWidth: EditorMetrics.s12,
                                        ),
                                        child: Text(
                                          widget.unit!,
                                          style: TextStyle(
                                            fontSize: EditorMetrics.micro,
                                            color: EditorTheme.of(context)
                                                .muted,
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
            ),
          ),
  );
}
