import 'dart:math' as math;

import 'package:flutter/foundation.dart' show listEquals;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../metrics.dart';
import '../theme.dart';
import 'drag.dart';

/// The faces a pointer turns and pushes: an angle in a ring, a pair on a
/// square.

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
