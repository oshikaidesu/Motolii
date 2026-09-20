import 'dart:async';
import 'dart:math' as math;
import 'dart:ui' show ViewFocusEvent, ViewFocusState;

import 'package:flutter/foundation.dart' show ValueListenable, listEquals;
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import 'glyphs.dart';
import 'leaves.dart';
import 'metrics.dart';
import 'theme.dart';

export 'leaves.dart' show EditorChoice;

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

Widget panelButton(
  String label,
  VoidCallback? action, {
  String? tooltip,
  bool selected = false,
}) =>
    EditorButton(label, action, tooltip: tooltip ?? label, selected: selected);
Widget panelTitle(String title) => Builder(
  builder: (context) => Container(
    height: EditorMetrics.s22,
    alignment: Alignment.centerLeft,
    padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
    decoration: BoxDecoration(
      color: EditorTheme.of(context).raised,
      border: Border(bottom: BorderSide(color: EditorTheme.of(context).line)),
    ),
    child: Text(
      title,
      style: TextStyle(
        fontSize: EditorMetrics.font,
        color: EditorTheme.of(context).ink,
      ),
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
    this.decoration,
  });
  final List<Widget> children;
  final double height;
  final EdgeInsetsGeometry padding;
  final BoxDecoration? decoration;
  @override
  Widget build(BuildContext context) => Container(
    height: height,
    decoration:
        decoration ?? BoxDecoration(color: EditorTheme.of(context).panel),
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

/// The one box a field sits in, at rest and while typing. The theme draws no
/// frame around a TextField, so a field has this border and no other: line at
/// rest, accent while it owns the keys, error while it refuses a draft. A tap
/// anywhere in the box hands the keys to [focus].
class EditorFieldFrame extends StatelessWidget {
  const EditorFieldFrame({
    super.key,
    required this.child,
    this.focus,
    this.error = false,
    this.height = EditorMetrics.row,
    this.minHeight,
    this.maxHeight,
    this.padding = const EdgeInsets.symmetric(horizontal: EditorMetrics.s5),
    this.color,
  });
  final Widget child;
  final FocusNode? focus;
  final bool error;
  final double? height, minHeight, maxHeight;
  final EdgeInsets padding;
  final Color? color;

  Widget _box(BuildContext context, bool focused) => GestureDetector(
    behavior: HitTestBehavior.translucent,
    onTap: focus?.requestFocus,
    child: Container(
      height: height,
      constraints: minHeight == null && maxHeight == null
          ? null
          : BoxConstraints(
              minHeight: minHeight ?? 0,
              maxHeight: maxHeight ?? double.infinity,
            ),
      padding: padding,
      decoration: BoxDecoration(
        color: color ?? EditorTheme.of(context).app,
        border: Border.all(
          color: error
              ? EditorTheme.of(context).error
              : focused
              ? EditorTheme.of(context).accent
              : EditorTheme.of(context).line,
        ),
      ),
      child: child,
    ),
  );

  @override
  Widget build(BuildContext context) => focus == null
      ? _box(context, false)
      : ListenableBuilder(
          listenable: focus!,
          builder: (_, __) => _box(context, focus!.hasFocus),
        );
}

/// The line that hides the seldom-used rows of a card: a chevron, the word,
/// a rule. The same line on every card, so the eye learns it once.
class EditorFold extends StatelessWidget {
  const EditorFold({required this.open, required this.onTap});
  final bool open;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: open ? 'Hide the advanced controls' : 'Show the advanced controls',
    child: GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      child: Row(
        children: [
          Icon(
            open ? Glyph.expand_more : Glyph.chevron_right,
            size: EditorMetrics.s14,
            color: EditorTheme.of(context).muted,
          ),
          const SizedBox(width: EditorMetrics.s2),
          Text(
            'Advanced',
            style: TextStyle(
              fontSize: EditorMetrics.dense,
              color: EditorTheme.of(context).muted,
            ),
          ),
          const SizedBox(width: EditorMetrics.s6),
          const Expanded(child: EditorRule(height: 1)),
        ],
      ),
    ),
  );
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
  Widget build(BuildContext context) => EditorTooltip(
    message: _error ?? widget.label,
    child: EditorFieldFrame(
      focus: _focus,
      error: _error != null,
      height: widget.multiline ? null : EditorMetrics.row,
      minHeight: widget.multiline ? EditorMetrics.row : null,
      child: EditorTextField(
        controller: _text,
        focusNode: _focus,
        enabled: widget.enabled,
        minLines: 1,
        maxLines: widget.multiline ? 4 : 1,
        style: TextStyle(
          fontSize: EditorMetrics.font,
          color: EditorTheme.of(context).ink,
        ),
        hint: widget.label,
        onSubmitted: (_) => _commit(),
        onChanged: (_) {
          if (_error != null) setState(() => _error = null);
        },
      ),
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

/// The ladder while a scrub is on it: the four rungs, the one the pointer
/// is on in ink, the rest faint (Lumit's value field shows the same four).
class _RungPill extends StatelessWidget {
  const _RungPill({required this.rung});
  final double rung;
  static const _rungs = [10.0, 1.0, .1, .01];
  static String _name(double r) => r == 10
      ? '×10'
      : r == 1
      ? '×1'
      : r == .1
      ? '×0.1'
      : '×0.01';
  @override
  Widget build(BuildContext context) => Container(
    constraints: const BoxConstraints(minHeight: EditorMetrics.control),
    padding: const EdgeInsets.symmetric(
      horizontal: EditorMetrics.s8,
      vertical: EditorMetrics.s4,
    ),
    decoration: BoxDecoration(
      color: EditorTheme.of(context).tooltip,
      borderRadius: BorderRadius.all(Radius.circular(EditorMetrics.s4)),
    ),
    child: Text.rich(
      TextSpan(
        children: [
          for (final r in _rungs) ...[
            if (r != _rungs.first) const TextSpan(text: '  '),
            TextSpan(
              text: _name(r),
              style: TextStyle(
                color: r == rung
                    ? EditorTheme.black
                    : EditorTheme.of(context).disabledInk,
                fontWeight: r == rung ? FontWeight.w600 : FontWeight.w400,
              ),
            ),
          ],
        ],
      ),
      style: const TextStyle(
        fontSize: EditorMetrics.s12,
        fontFeatures: [FontFeature.tabularFigures()],
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
      KeyLamp.keyed => EditorTheme.of(context).keyAccent.withValues(alpha: .55),
      KeyLamp.now => EditorTheme.of(context).keyAccent,
      KeyLamp.draft => EditorTheme.of(context).keyAccent.withValues(alpha: .3),
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
                                  ? Border.all(
                                      color: EditorTheme.of(context).muted,
                                    )
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
                duration: const Duration(milliseconds: 120),
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
/// drag → preview → commit, once, for every continuous control. The field,
/// the dial, the pad and the gradient stops mix this in and keep only their
/// own value shape: what a preview sends, what a commit and a cancel run, and
/// what to clear once the drag has settled. The window losing focus, or the
/// app leaving the foreground, cancels the drag in flight.
mixin EditorDragSession<T, W extends StatefulWidget>
    on State<W>, WidgetsBindingObserver {
  late final queue = EditorPreviewQueue<T>(sendPreview);

  /// True from [beginDrag] until [endDrag] takes the drag over.
  bool dragging = false;

  /// True while the commit or cancel of the last drag is in flight; a new
  /// drag waits for it.
  bool ending = false;

  Future<void> sendPreview(T value);
  Future<void> commitDrag();
  Future<void> cancelDrag();

  /// Whether the window's focus and the app's lifecycle end the drag.
  bool get watchesWindow => true;

  /// A commit that throws away what is still queued and runs [cancelDrag]
  /// instead (a stop pulled off its bar).
  bool get discardsPreview => false;

  /// Right after the drag stops, before the commit or cancel goes out.
  void dragStopped(bool cancel) {}

  /// Once the commit or cancel has settled and the control is still mounted.
  void dragSettled() {}

  void beginDrag() => dragging = true;

  Future<void> endDrag(bool cancel) async {
    if (!dragging) return;
    dragging = false;
    dragStopped(cancel);
    ending = true;
    final discard = cancel || discardsPreview;
    try {
      await queue.finish(discard, discard ? cancelDrag : commitDrag);
    } finally {
      ending = false;
      if (mounted) dragSettled();
    }
  }

  @override
  void initState() {
    super.initState();
    if (watchesWindow) WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    if (dragging) queue.finish(true, cancelDrag);
    if (watchesWindow) WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) endDrag(true);
  }

  @override
  void didChangeViewFocus(ViewFocusEvent event) {
    if (event.state == ViewFocusState.unfocused) endDrag(true);
  }
}

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

class _EditorDialState extends State<EditorDial>
    with WidgetsBindingObserver, EditorDragSession<double, EditorDial> {
  double? _shown;
  double _lastAngle = 0;
  final _gestureFocus = FocusNode();
  @override
  Future<void> sendPreview(double value) => widget.onPreview(value);
  @override
  Future<void> commitDrag() => widget.onFinish();
  @override
  Future<void> cancelDrag() => widget.onCancel();
  @override
  void dragSettled() => setState(() => _shown = null);

  @override
  void dispose() {
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
          dragging) {
        endDrag(true);
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    },
    child: Listener(
      onPointerCancel: (_) => endDrag(true),
      child: EditorTooltip(
        message: 'Rotation',
        child: GestureDetector(
          onPanStart: widget.enabled
              ? (e) {
                  if (ending) return;
                  dragging = true;
                  _gestureFocus.requestFocus();
                  _lastAngle = _angleOf(e.localPosition);
                  _shown = widget.degrees;
                  widget.onBegin();
                }
              : null,
          onPanUpdate: widget.enabled
              ? (e) {
                  if (!dragging) return;
                  final a = _angleOf(e.localPosition);
                  var delta = a - _lastAngle;
                  if (delta > 180) delta -= 360;
                  if (delta < -180) delta += 360;
                  _lastAngle = a;
                  setState(() => _shown = (_shown ?? widget.degrees) + delta);
                  queue.add(_shown!);
                }
              : null,
          onPanEnd: widget.enabled ? (_) => endDrag(false) : null,
          onPanCancel: widget.enabled ? () => endDrag(true) : null,
          child: CustomPaint(
            size: Size.square(widget.size),
            painter: _DialPainter(
              colors: EditorTheme.of(context),
              _shown ?? widget.degrees,
              !widget.enabled
                  ? EditorTheme.of(context).muted
                  : widget.tint ?? EditorTheme.of(context).ink,
            ),
          ),
        ),
      ),
    ),
  );
}

class _DialPainter extends CustomPainter {
  final EditorTheme colors;

  const _DialPainter(
    this.degrees,
    this.ink, {
    this.colors = EditorTheme.chromatic,
  });
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
        ..color = colors.app
        ..style = PaintingStyle.fill,
    );
    canvas.drawCircle(
      c,
      r,
      Paint()
        ..color = colors.border
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
      colors != old.colors || old.degrees != degrees || old.ink != ink;
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
                            ? EditorTheme.of(context).accent
                            : onPick == null
                            ? EditorTheme.of(context).raised
                            : EditorTheme.of(context).border,
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
    this.expanded = true,
    this.onToggle,
  });
  final String title;
  final List<Widget> children;
  final Widget? leading, trailing;
  final bool expanded;
  final VoidCallback? onToggle;

  /// The card's contents faded: what it holds is not applied right now.
  final bool dim;
  @override
  Widget build(BuildContext context) => DefaultTextStyle(
    style: EditorTheme.of(context).text,
    child: Container(
      padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
      decoration: BoxDecoration(
        color: EditorTheme.of(context).panel,
        border: Border(bottom: BorderSide(color: EditorTheme.of(context).line)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _SectionHead(
            title: title,
            leading: leading,
            trailing: trailing,
            expanded: expanded,
            onToggle: onToggle,
          ),
          if (expanded) ...[
            // The head sits close to its rows; the rows are a step apart; the
            // card ends a step past its last row, whatever that row is.
            const SizedBox(height: EditorMetrics.s2),
            if (dim)
              Opacity(
                opacity: .4,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  spacing: EditorMetrics.s4,
                  children: children,
                ),
              )
            else
              Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                spacing: EditorMetrics.s4,
                children: children,
              ),
            const SizedBox(height: EditorMetrics.s8),
          ],
        ],
      ),
    ),
  );
}

/// A section's head row: its name, the fold mark when it folds, and what the
/// caller puts at either end.
class _SectionHead extends StatelessWidget {
  const _SectionHead({
    required this.title,
    required this.leading,
    required this.trailing,
    required this.expanded,
    required this.onToggle,
  });
  final String title;
  final Widget? leading, trailing;
  final bool expanded;
  final VoidCallback? onToggle;
  @override
  Widget build(BuildContext context) => SizedBox(
    height: EditorMetrics.row,
    child: Row(
      children: [
        if (leading != null) leading!,
        Expanded(
          child: Semantics(
            expanded: expanded,
            child: EditorPress(
              onTap: onToggle,
              child: SizedBox(
                height: EditorMetrics.row,
                child: Row(
                  children: [
                    if (onToggle != null)
                      Transform.translate(
                        offset: const Offset(-1.5, 0),
                        child: Icon(
                          expanded ? Glyph.expand_more : Glyph.chevron_right,
                          size: EditorMetrics.s14,
                          color: EditorTheme.of(context).muted,
                        ),
                      ),
                    Expanded(
                      child: Text(
                        title.toUpperCase(),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(
                          fontSize: EditorMetrics.micro,
                          letterSpacing: 1,
                          color: EditorTheme.of(context).muted,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
        if (trailing != null) trailing!,
      ],
    ),
  );
}

/// Two values as one point on a square. Dragging moves the point by the
/// pointer's own distance (no range needed); the dot shows where the pair
/// stands within [span] of the centre.
///
/// With [unit] the pair is a fraction of the square (0..1 each way, top-left
/// at the origin) and [snaps] are the places the pair may land: the sent
/// value is always the nearest snap, the dot follows the pointer until it is
/// let go (Shift holds the dot on the snaps too). [bars] paints the snaps of
/// one axis as columns — the alignment box in its "auto gap" mode.
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
    this.unit = false,
    this.snaps,
    this.bars,
  });
  final double x, y, size, span, speed;
  final bool enabled, unit;
  final List<Offset>? snaps;
  final Axis? bars;
  final Color? tint;

  /// The snap nearest to [at]; [at] itself when there are none.
  Offset snapped(Offset at) {
    final snaps = this.snaps;
    if (snaps == null || snaps.isEmpty) return at;
    var best = snaps.first;
    for (final s in snaps) {
      if ((s - at).distanceSquared < (best - at).distanceSquared) best = s;
    }
    return best;
  }

  double get _inner => size - EditorMetrics.s4 * 2;
  final VoidCallback onBegin;
  final Future<void> Function(double x, double y) onPreview;
  final Future<void> Function() onFinish, onCancel;
  @override
  State<EditorPad> createState() => _EditorPadState();
}

class _EditorPadState extends State<EditorPad>
    with WidgetsBindingObserver, EditorDragSession<Offset, EditorPad> {
  Offset? _shown;
  final _gestureFocus = FocusNode();
  @override
  Future<void> sendPreview(Offset value) =>
      widget.onPreview(value.dx, value.dy);
  @override
  Future<void> commitDrag() => widget.onFinish();
  @override
  Future<void> cancelDrag() => widget.onCancel();
  @override
  void dragSettled() => setState(() => _shown = null);

  @override
  void dispose() {
    _gestureFocus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
    focusNode: _gestureFocus,
    onKeyEvent: (_, event) {
      if (event is KeyDownEvent &&
          event.logicalKey == LogicalKeyboardKey.escape &&
          dragging) {
        endDrag(true);
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    },
    child: Listener(
      onPointerCancel: (_) => endDrag(true),
      child: EditorTooltip(
        message: 'Drag the point',
        child: MouseRegion(
          cursor: widget.enabled
              ? SystemMouseCursors.move
              : SystemMouseCursors.basic,
          child: GestureDetector(
            onPanStart: widget.enabled
                ? (_) {
                    if (ending) return;
                    dragging = true;
                    _gestureFocus.requestFocus();
                    _shown = Offset(widget.x, widget.y);
                    widget.onBegin();
                  }
                : null,
            onPanUpdate: widget.enabled
                ? (e) {
                    if (!dragging) return;
                    var next =
                        (_shown ?? Offset(widget.x, widget.y)) +
                        e.delta *
                            (widget.unit
                                ? widget.speed / widget._inner
                                : widget.speed);
                    if (widget.unit) {
                      next = Offset(
                        next.dx.clamp(0.0, 1.0),
                        next.dy.clamp(0.0, 1.0),
                      );
                    }
                    final sent = widget.snapped(next);
                    setState(
                      () => _shown =
                          HardwareKeyboard.instance.isShiftPressed &&
                              widget.snaps != null
                          ? sent
                          : next,
                    );
                    queue.add(sent);
                  }
                : null,
            onPanEnd: widget.enabled ? (_) => endDrag(false) : null,
            onPanCancel: widget.enabled ? () => endDrag(true) : null,
            child: CustomPaint(
              size: Size.square(widget.size),
              painter: _PadPainter(
                colors: EditorTheme.of(context),
                _shown ?? Offset(widget.x, widget.y),
                widget.span,
                !widget.enabled
                    ? EditorTheme.of(context).muted
                    : widget.tint ?? EditorTheme.of(context).accent,
                unit: widget.unit,
                snaps: widget.snaps,
                bars: widget.bars,
              ),
            ),
          ),
        ),
      ),
    ),
  );
}

class _PadPainter extends CustomPainter {
  final EditorTheme colors;

  const _PadPainter(
    this.at,
    this.span,
    this.dot, {
    this.colors = EditorTheme.chromatic,
    this.unit = false,
    this.snaps,
    this.bars,
  });
  final Offset at;
  final double span;
  final Color dot;
  final bool unit;
  final List<Offset>? snaps;
  final Axis? bars;
  @override
  void paint(Canvas canvas, Size size) {
    final r = RRect.fromRectAndRadius(
      Offset.zero & size,
      const Radius.circular(EditorMetrics.s3),
    );
    canvas.drawRRect(r, Paint()..color = colors.app);
    final c = size.center(Offset.zero);
    final hair = Paint()
      ..color = colors.line
      ..strokeWidth = 1;
    final half = size.width / 2 - EditorMetrics.s4;
    // A unit pad maps 0..1 onto the inner square; the snaps are its marks.
    Offset place(Offset v) => unit
        ? Offset(
            EditorMetrics.s4 + v.dx.clamp(0.0, 1.0) * half * 2,
            EditorMetrics.s4 + v.dy.clamp(0.0, 1.0) * half * 2,
          )
        : Offset(
            c.dx + (v.dx / span * half).clamp(-half, half),
            c.dy + (v.dy / span * half).clamp(-half, half),
          );
    if (snaps case final marks?) {
      final mark = Paint()..color = colors.border;
      for (final s in marks) {
        final q = place(s);
        if (bars == Axis.horizontal) {
          canvas.drawRect(
            Rect.fromCenter(center: q, width: EditorMetrics.s2, height: half),
            mark,
          );
        } else if (bars == Axis.vertical) {
          canvas.drawRect(
            Rect.fromCenter(center: q, width: half, height: EditorMetrics.s2),
            mark,
          );
        } else {
          // A cross, not a dot: nine of them have to read as places to land
          // at this size, and a one-pixel dot does not.
          final pen = Paint()
            ..color = colors.muted
            ..strokeWidth = 1;
          canvas.drawLine(
            q - const Offset(EditorMetrics.s3, 0),
            q + const Offset(EditorMetrics.s3, 0),
            pen,
          );
          canvas.drawLine(
            q - const Offset(0, EditorMetrics.s3),
            q + const Offset(0, EditorMetrics.s3),
            pen,
          );
        }
      }
    } else {
      canvas.drawLine(Offset(c.dx, 0), Offset(c.dx, size.height), hair);
      canvas.drawLine(Offset(0, c.dy), Offset(size.width, c.dy), hair);
    }
    final p = place(at);
    if (!unit) canvas.drawLine(c, p, Paint()..color = colors.border);
    canvas.drawCircle(p, EditorMetrics.s4, Paint()..color = dot);
    canvas.drawRRect(
      r,
      Paint()
        ..color = colors.border
        ..style = PaintingStyle.stroke,
    );
  }

  @override
  bool shouldRepaint(_PadPainter old) =>
      colors != old.colors ||
      old.at != at ||
      old.span != span ||
      old.dot != dot ||
      old.unit != unit ||
      old.bars != bars ||
      !listEquals(old.snaps, snaps);
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
  final EditorTheme colors;

  const _TrackPainter({
    this.colors = EditorTheme.chromatic,
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
          ..color = colors.border
          ..strokeWidth = 1,
      );
    }
  }

  @override
  bool shouldRepaint(_TrackPainter old) =>
      colors != old.colors ||
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
