// Private building blocks of the inputs set. Colours and text come from tokens only.
import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../parts/controls.dart';
import '../../tokens.dart';
import 'text_hand.dart';
part 'inputs_parts_2.dart';
part 'inputs_parts_3.dart';
part 'inputs_parts_4.dart';

const kDur = Mo.dur;
const kEase = Mo.ease;
const double kH = 24; // [open] control height in the inspector (ValueWell uses 22)

/// Holds a value the use case can change by hand; the knob resets it when the knob moves.
class Live<V> extends StatefulWidget {
  const Live({super.key, required this.value, required this.builder});
  final V value;
  final Widget Function(BuildContext, V, ValueChanged<V>) builder;
  @override
  State<Live<V>> createState() => _LiveState<V>();
}

class _LiveState<V> extends State<Live<V>> {
  late V v = widget.value;
  @override
  void didUpdateWidget(Live<V> old) {
    super.didUpdateWidget(old);
    if (old.value != widget.value) v = widget.value;
  }

  @override
  Widget build(BuildContext context) => widget.builder(context, v, (n) => setState(() => v = n));
}

class Dim extends StatelessWidget {
  const Dim(this.off, this.child, {super.key});
  final bool off;
  final Widget child;
  @override
  Widget build(BuildContext context) => AnimatedOpacity(opacity: off ? .4 : 1, duration: kDur, curve: kEase, child: IgnorePointer(ignoring: off, child: child));
}

/// The one finish of a text-like well, in three looks.
BoxDecoration wellDeco(Look look, {bool hover = false, bool focus = false, bool invalid = false, double r = 4}) {
  Color fill, border;
  var glow = const <BoxShadow>[];
  switch (look) {
    case Look.concept:
      fill = hover && !focus ? N.g10 : N.g07;
      border = focus ? Role.selected : (hover ? N.g26 : N.g20);
    case Look.quiet:
      fill = focus ? N.g10 : (hover ? N.g15 : N.g13);
      border = focus ? N.g44 : fill;
    case Look.glow:
      fill = N.g07;
      border = focus ? Role.selected : (hover ? N.g38 : N.g20);
      if (focus) glow = [BoxShadow(color: Role.selected.withValues(alpha: .35), blurRadius: 8)];
  }
  if (invalid) border = Role.error;
  return BoxDecoration(color: fill, borderRadius: BorderRadius.circular(r), border: Border.all(color: border), boxShadow: glow);
}

/// A forced focus state: 2 px accent outline at 50 % with a 2 px gap. Layout does not move.
class FocusRing extends StatelessWidget {
  const FocusRing({super.key, required this.on, required this.child, this.radius = 4});
  final bool on;
  final double radius;
  final Widget child;
  @override
  Widget build(BuildContext context) => Stack(clipBehavior: Clip.none, children: [
        child,
        Positioned(
          left: -4,
          top: -4,
          right: -4,
          bottom: -4,
          child: IgnorePointer(
            child: AnimatedOpacity(
              opacity: on ? 1 : 0,
              duration: kDur,
              curve: kEase,
              child: DecoratedBox(decoration: BoxDecoration(borderRadius: BorderRadius.circular(radius + 4), border: Border.all(color: Role.selected.withValues(alpha: .5), width: 2))),
            ),
          ),
        ),
      ]);
}

class HitState {
  const HitState({this.hover = false, this.focus = false, this.down = false});
  final bool hover, focus, down;
}

/// Hover / focus / pressed in one place, plus the gestures a control needs.
class Hit extends StatefulWidget {
  const Hit({
    super.key,
    required this.builder,
    this.enabled = true,
    this.cursor = SystemMouseCursors.click,
    this.focusable = true,
    this.ringFromKeyboardOnly = true,
    this.onTap,
    this.onDoubleTap,
    this.onKey,
    this.onPanStart,
    this.onPanUpdate,
    this.onPanEnd,
    this.onPanDown,
  });
  final Widget Function(BuildContext, HitState) builder;
  final bool enabled, focusable, ringFromKeyboardOnly;
  final MouseCursor cursor;
  final VoidCallback? onTap, onDoubleTap;
  final KeyEventResult Function(KeyEvent)? onKey;
  final GestureDragStartCallback? onPanStart;
  final GestureDragUpdateCallback? onPanUpdate;
  final GestureDragEndCallback? onPanEnd;
  final GestureDragDownCallback? onPanDown;
  @override
  State<Hit> createState() => _HitWidgetState();
}

class _HitWidgetState extends State<Hit> {
  final _node = FocusNode();
  bool _h = false, _d = false, _f = false;
  @override
  void initState() {
    super.initState();
    _node.addListener(() => setState(() => _f = _node.hasFocus));
  }

  @override
  void dispose() {
    _node.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final on = widget.enabled;
    final ring = _f && on && (!widget.ringFromKeyboardOnly || FocusManager.instance.highlightMode == FocusHighlightMode.traditional);
    final st = HitState(hover: _h && on, focus: ring, down: _d && on);
    return Focus(
      focusNode: _node,
      canRequestFocus: on && widget.focusable,
      onKeyEvent: (n, e) => on && widget.onKey != null && (e is KeyDownEvent || e is KeyRepeatEvent) ? widget.onKey!(e) : KeyEventResult.ignored,
      child: MouseRegion(
        cursor: on ? widget.cursor : SystemMouseCursors.forbidden,
        onEnter: (_) => setState(() => _h = true),
        onExit: (_) => setState(() => _h = false),
        child: Listener(
          onPointerDown: (_) {
            setState(() => _d = true);
            if (on && widget.focusable) _node.requestFocus();
          },
          onPointerUp: (_) => setState(() => _d = false),
          onPointerCancel: (_) => setState(() => _d = false),
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: on ? widget.onTap : null,
            onDoubleTap: on ? widget.onDoubleTap : null,
            onPanDown: on ? widget.onPanDown : null,
            onPanStart: on ? widget.onPanStart : null,
            onPanUpdate: on ? widget.onPanUpdate : null,
            onPanEnd: on ? widget.onPanEnd : null,
            child: widget.builder(context, st),
          ),
        ),
      ),
    );
  }
}

// ---- glyphs ----------------------------------------------------------------------------------------------------------

enum Glyph { search, close, minus, plus, check, chevron, linked, unlinked, alignL, alignC, alignR, alignJ }

class GlyphIcon extends StatelessWidget {
  const GlyphIcon(this.g, {super.key, this.size = 14, this.color = N.g76, this.weight = 1.3});
  final Glyph g;
  final double size, weight;
  final Color color;
  @override
  Widget build(BuildContext context) => SizedBox.square(dimension: size, child: CustomPaint(painter: _GlyphPainter(g, color, weight)));
}

class _GlyphPainter extends CustomPainter {
  const _GlyphPainter(this.g, this.color, this.weight);
  final Glyph g;
  final Color color;
  final double weight;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width;
    Offset p(double x, double y) => Offset(x * w, y * w);
    final st = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = weight
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    switch (g) {
      case Glyph.search:
        cv.drawCircle(p(.42, .42), w * .26, st);
        cv.drawLine(p(.62, .62), p(.88, .88), st);
      case Glyph.close:
        cv.drawLine(p(.28, .28), p(.72, .72), st);
        cv.drawLine(p(.72, .28), p(.28, .72), st);
      case Glyph.minus:
        cv.drawLine(p(.25, .5), p(.75, .5), st);
      case Glyph.plus:
        cv.drawLine(p(.25, .5), p(.75, .5), st);
        cv.drawLine(p(.5, .25), p(.5, .75), st);
      case Glyph.check:
        cv.drawPath(Path()..moveTo(w * .2, w * .52)..lineTo(w * .42, w * .72)..lineTo(w * .8, w * .3), st);
      case Glyph.chevron:
        cv.drawPath(Path()..moveTo(w * .25, w * .38)..lineTo(w * .5, w * .64)..lineTo(w * .75, w * .38), st);
      case Glyph.linked:
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(w * .06, w * .34, w * .58, w * .66), Radius.circular(w * .16)), st);
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(w * .42, w * .34, w * .94, w * .66), Radius.circular(w * .16)), st);
      case Glyph.unlinked:
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(w * .04, w * .34, w * .44, w * .66), Radius.circular(w * .16)), st);
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(w * .56, w * .34, w * .96, w * .66), Radius.circular(w * .16)), st);
      case Glyph.alignL:
      case Glyph.alignC:
      case Glyph.alignR:
      case Glyph.alignJ:
        const ys = [.26, .42, .58, .74], ws = [.62, .42, .62, .34];
        for (var i = 0; i < 4; i++) {
          double x0, x1;
          switch (g) {
            case Glyph.alignL:
              x0 = .2;
              x1 = .2 + ws[i];
            case Glyph.alignC:
              x0 = .5 - ws[i] / 2;
              x1 = .5 + ws[i] / 2;
            case Glyph.alignR:
              x0 = .8 - ws[i];
              x1 = .8;
            default:
              x0 = .2;
              x1 = .8;
          }
          cv.drawLine(p(x0, ys[i]), p(x1, ys[i]), st);
        }
    }
  }

  @override
  bool shouldRepaint(_GlyphPainter o) => o.g != g || o.color != color || o.weight != weight;
}

// ---- inline editing --------------------------------------------------------------------------------------------------

/// A one-line editor that opens focused with everything selected; Enter or blur commits, Escape cancels.
class InlineEdit extends StatefulWidget {
  const InlineEdit({super.key, required this.text, required this.onDone, this.style, this.align = TextAlign.right});
  final String text;
  final TextStyle? style;
  final TextAlign align;
  final void Function(String text, bool commit) onDone;
  @override
  State<InlineEdit> createState() => _InlineEditState();
}

class _InlineEditState extends State<InlineEdit> {
  late final _c = TextEditingController(text: widget.text);
  final _n = FocusNode();
  bool _done = false;
  @override
  void initState() {
    super.initState();
    _c.selection = TextSelection(baseOffset: 0, extentOffset: widget.text.length);
    _n.addListener(() {
      if (!_n.hasFocus) _finish(true);
    });
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _n.requestFocus();
    });
  }

  void _finish(bool commit) {
    if (_done) return;
    _done = true;
    widget.onDone(_c.text, commit);
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
        canRequestFocus: false,
        skipTraversal: true,
        onKeyEvent: (n, e) {
          if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
            _finish(false);
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: TextHand(field: (k) => EditableText(key: k, rendererIgnoresPointer: true,
          controller: _c,
          focusNode: _n,
          style: widget.style ?? T.value(N.g100),
          cursorColor: Role.selected,
          backgroundCursorColor: N.g26,
          selectionColor: Role.selected.withValues(alpha: .35),
          textAlign: widget.align,
          maxLines: 1,
          cursorWidth: 1,
          onSubmitted: (_) => _finish(true),
        )),
      );
}

// ---- 1. scrub number -------------------------------------------------------------------------------------------------

class ScrubField extends StatefulWidget {
  const ScrubField({
    super.key,
    required this.label,
    required this.value,
    this.onChanged,
    this.min,
    this.max,
    this.perPx,
    this.decimals = 1,
    this.unit = '',
    this.accent,
    this.axis = false,
    this.fill = true,
    this.look = Look.concept,
    this.enabled = true,
  });
  final String label, unit;
  final double value;
  final double? min, max, perPx;
  final int decimals;
  final Color? accent;
  final bool axis, fill, enabled;
  final Look look;
  final ValueChanged<double>? onChanged;
  @override
  State<ScrubField> createState() => _ScrubFieldState();
}

class _ScrubFieldState extends State<ScrubField> {
  bool _edit = false, _drag = false;
  double _acc = 0;

  bool get _bounded => widget.min != null && widget.max != null;
  double get _per => widget.perPx ?? (_bounded ? (widget.max! - widget.min!) / 200 : 1);
  double _clamp(double v) => math.min(widget.max ?? double.infinity, math.max(widget.min ?? double.negativeInfinity, v));
  double _round(double v) {
    final f = math.pow(10, widget.decimals).toDouble();
    return (v * f).round() / f;
  }

  void _commit(String s) {
    final v = double.tryParse(s.replaceAll(widget.unit, '').trim());
    if (v != null) widget.onChanged?.call(_round(_clamp(v)));
  }

  @override
  Widget build(BuildContext context) {
    final accent = widget.accent ?? C.mode;
    final look = widget.look;
    final hasFill = widget.fill && _bounded;
    final t = hasFill ? ((widget.value - widget.min!) / (widget.max! - widget.min!)).clamp(0.0, 1.0) : 0.0;
    return Dim(
      !widget.enabled,
      Hit(
        cursor: _edit ? SystemMouseCursors.text : SystemMouseCursors.resizeLeftRight,
        onDoubleTap: () => setState(() => _edit = true),
        onPanStart: (_) => setState(() {
          _drag = true;
          _acc = widget.value;
        }),
        onPanUpdate: (d) {
          if (_edit) return;
          _acc = _clamp(_acc + d.delta.dx * _per * (HardwareKeyboard.instance.isShiftPressed ? .1 : 1));
          widget.onChanged?.call(_round(_acc));
        },
        onPanEnd: (_) => setState(() => _drag = false),
        onKey: (e) {
          final k = e.logicalKey;
          if (k == LogicalKeyboardKey.enter) {
            setState(() => _edit = true);
            return KeyEventResult.handled;
          }
          final dir = k == LogicalKeyboardKey.arrowRight || k == LogicalKeyboardKey.arrowUp ? 1 : (k == LogicalKeyboardKey.arrowLeft || k == LogicalKeyboardKey.arrowDown ? -1 : 0);
          if (dir == 0) return KeyEventResult.ignored;
          widget.onChanged?.call(_round(_clamp(widget.value + dir * _per * (HardwareKeyboard.instance.isShiftPressed ? 1 : 10))));
          return KeyEventResult.handled;
        },
        builder: (ctx, h) {
          final lit = h.hover || _drag;
          final fillA = look == Look.quiet ? [.2, .2] : (look == Look.glow ? [.25, .7] : [.28, .55]);
          return AnimatedContainer(
            duration: kDur,
            curve: kEase,
            height: kH,
            decoration: wellDeco(look, hover: lit && !_edit, focus: h.focus || _edit || (look == Look.glow && _drag)),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(3),
              child: Stack(children: [
                if (hasFill)
                  Positioned.fill(
                    child: FractionallySizedBox(
                      alignment: Alignment.centerLeft,
                      widthFactor: t,
                      child: DecoratedBox(
                        decoration: BoxDecoration(
                          gradient: LinearGradient(colors: [accent.withValues(alpha: fillA[0] + (lit ? .06 : 0)), accent.withValues(alpha: fillA[1] + (lit ? .06 : 0))]),
                          border: look == Look.glow ? Border(right: BorderSide(color: accent, width: 1.5)) : null,
                        ),
                      ),
                    ),
                  ),
                if (look == Look.concept || widget.axis) Positioned(left: 0, top: 4, bottom: 4, width: 2, child: DecoratedBox(decoration: BoxDecoration(color: accent, borderRadius: BorderRadius.circular(1)))),
                Positioned.fill(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 8),
                    child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
                      Text(widget.label, style: widget.axis ? T.label(N.g63) : T.label(N.g76)),
                      const Spacer(),
                      if (_edit)
                        SizedBox(
                          width: 72,
                          child: InlineEdit(text: widget.value.toStringAsFixed(widget.decimals), onDone: (s, ok) {
                            setState(() => _edit = false);
                            if (ok) _commit(s);
                          }),
                        )
                      else ...[
                        Text(widget.value.toStringAsFixed(widget.decimals), style: T.value(N.g95)),
                        if (widget.unit.isNotEmpty) Text(' ${widget.unit}', style: T.value(N.g56)),
                      ],
                    ]),
                  ),
                ),
              ]),
            ),
          );
        },
      ),
    );
  }
}

// ---- 2. vec3 ---------------------------------------------------------------------------------------------------------

class Vec3Row extends StatefulWidget {
  const Vec3Row({super.key, required this.x, required this.y, required this.z, this.decimals = 1, this.unit = '', this.look = Look.concept, this.enabled = true});
  final double x, y, z;
  final int decimals;
  final String unit;
  final Look look;
  final bool enabled;
  @override
  State<Vec3Row> createState() => _Vec3RowState();
}
