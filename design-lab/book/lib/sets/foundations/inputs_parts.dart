// Private building blocks of the inputs set. Colours and text come from tokens only.
import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../parts/controls.dart';
import '../../tokens.dart';

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
        child: EditableText(
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
        ),
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

class _Vec3RowState extends State<Vec3Row> {
  late List<double> v = [widget.x, widget.y, widget.z];
  @override
  void didUpdateWidget(Vec3Row o) {
    super.didUpdateWidget(o);
    if (o.x != widget.x || o.y != widget.y || o.z != widget.z) v = [widget.x, widget.y, widget.z];
  }

  @override
  Widget build(BuildContext context) {
    // [open] axis colours: muted tints, shown only as a 2 px tick
    final axes = [('X', Fam.scatter.c.withValues(alpha: .6)), ('Y', Fam.along.c.withValues(alpha: .6)), ('Z', Fam.stagger.c.withValues(alpha: .6))];
    return Row(children: [
      for (var i = 0; i < 3; i++) ...[
        if (i > 0) const SizedBox(width: 4),
        Expanded(
          child: ScrubField(
            label: axes[i].$1,
            accent: axes[i].$2,
            axis: true,
            fill: false,
            value: v[i],
            decimals: widget.decimals,
            unit: widget.unit,
            look: widget.look,
            enabled: widget.enabled,
            onChanged: (n) => setState(() => v[i] = n),
          ),
        ),
      ],
    ]);
  }
}

// ---- 3. link / ratio -------------------------------------------------------------------------------------------------

class LinkToggle extends StatelessWidget {
  const LinkToggle({super.key, required this.on, this.onChanged, this.look = Look.concept});
  final bool on;
  final Look look;
  final ValueChanged<bool>? onChanged;
  @override
  Widget build(BuildContext context) => Hit(
        onTap: () => onChanged?.call(!on),
        onKey: (e) {
          if (e.logicalKey == LogicalKeyboardKey.space || e.logicalKey == LogicalKeyboardKey.enter) {
            onChanged?.call(!on);
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        builder: (ctx, h) {
          final Color fill = on ? (look == Look.glow ? C.mode.withValues(alpha: .18) : N.g20) : (h.hover ? N.g15 : (look == Look.quiet ? N.g13 : N.g10));
          final Color ink = on ? (look == Look.quiet ? N.g95 : C.mode) : (h.hover ? N.g95 : N.g56);
          return AnimatedContainer(
            duration: kDur,
            curve: kEase,
            width: kH,
            height: kH,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: h.down ? N.g26 : fill,
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: h.focus ? Role.selected : (on && look == Look.glow ? Role.selected.withValues(alpha: .6) : (look == Look.quiet ? fill : N.g20))),
              boxShadow: on && look == Look.glow ? [BoxShadow(color: C.mode.withValues(alpha: .3), blurRadius: 8)] : const [],
            ),
            child: GlyphIcon(on ? Glyph.linked : Glyph.unlinked, size: 14, color: ink),
          );
        },
      );
}

class RatioPair extends StatefulWidget {
  const RatioPair({super.key, required this.w, required this.h, required this.linked, this.look = Look.concept, this.enabled = true});
  final double w, h;
  final bool linked, enabled;
  final Look look;
  @override
  State<RatioPair> createState() => _RatioPairState();
}

class _RatioPairState extends State<RatioPair> {
  late double w = widget.w, h = widget.h;
  late bool linked = widget.linked;
  late double ratio = widget.w / widget.h;
  @override
  void didUpdateWidget(RatioPair o) {
    super.didUpdateWidget(o);
    if (o.w != widget.w || o.h != widget.h || o.linked != widget.linked) {
      w = widget.w;
      h = widget.h;
      linked = widget.linked;
      ratio = w / h;
    }
  }

  @override
  Widget build(BuildContext context) => Row(children: [
        Expanded(
          child: ScrubField(
            label: 'W', unit: 'px', decimals: 0, min: 1, perPx: 1, fill: false, value: w, look: widget.look, enabled: widget.enabled,
            onChanged: (n) => setState(() {
              w = n;
              if (linked) h = math.max(1, (n / ratio).roundToDouble());
            }),
          ),
        ),
        const SizedBox(width: 4),
        Dim(!widget.enabled, LinkToggle(on: linked, look: widget.look, onChanged: (v) => setState(() {
              linked = v;
              if (v) ratio = w / h;
            }))),
        const SizedBox(width: 4),
        Expanded(
          child: ScrubField(
            label: 'H', unit: 'px', decimals: 0, min: 1, perPx: 1, fill: false, value: h, look: widget.look, enabled: widget.enabled,
            onChanged: (n) => setState(() {
              h = n;
              if (linked) w = math.max(1, (n * ratio).roundToDouble());
            }),
          ),
        ),
      ]);
}

// ---- 4/5. text and search --------------------------------------------------------------------------------------------

class TextBox extends StatefulWidget {
  const TextBox({super.key, required this.text, this.hint = '', this.onChanged, this.onSubmit, this.look = Look.concept, this.enabled = true, this.leading, this.clearable = false, this.mono = false, this.invalid = false, this.maxLength, this.ring = false});
  final String text, hint;
  final ValueChanged<String>? onChanged, onSubmit;
  final Look look;
  final bool enabled, clearable, mono, invalid, ring;
  final Widget? leading;
  final int? maxLength;
  @override
  State<TextBox> createState() => _TextBoxState();
}

class _TextBoxState extends State<TextBox> {
  late final _c = TextEditingController(text: widget.text);
  final _n = FocusNode();
  bool _hover = false, _focus = false;
  @override
  void initState() {
    super.initState();
    _c.addListener(() => setState(() {}));
    _n.addListener(() {
      setState(() => _focus = _n.hasFocus);
      if (!_n.hasFocus) widget.onSubmit?.call(_c.text);
    });
  }

  @override
  void didUpdateWidget(TextBox o) {
    super.didUpdateWidget(o);
    if (widget.text != _c.text && (!_n.hasFocus || widget.text != o.text)) _c.text = widget.text;
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final style = widget.mono ? T.value(N.g95) : T.name(N.g95);
    return Dim(
      !widget.enabled,
      FocusRing(
        on: widget.ring,
        child: MouseRegion(
        cursor: SystemMouseCursors.text,
        onEnter: (_) => setState(() => _hover = true),
        onExit: (_) => setState(() => _hover = false),
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: _n.requestFocus,
          child: AnimatedContainer(
            duration: kDur,
            curve: kEase,
            height: kH,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: wellDeco(widget.look, hover: _hover, focus: _focus || widget.ring, invalid: widget.invalid),
            child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
              if (widget.leading != null) ...[widget.leading!, const SizedBox(width: 6)],
              Expanded(
                child: Stack(alignment: Alignment.centerLeft, children: [
                  if (_c.text.isEmpty) IgnorePointer(child: Text(widget.hint, style: T.name(N.g44), maxLines: 1)),
                  EditableText(
                    controller: _c,
                    focusNode: _n,
                    style: style,
                    cursorColor: Role.selected,
                    backgroundCursorColor: N.g26,
                    selectionColor: Role.selected.withValues(alpha: .35),
                    maxLines: 1,
                    cursorWidth: 1,
                    readOnly: !widget.enabled,
                    inputFormatters: widget.maxLength == null ? null : [LengthLimitingTextInputFormatter(widget.maxLength)],
                    onChanged: widget.onChanged,
                    onSubmitted: (s) {
                      widget.onSubmit?.call(s);
                      _n.requestFocus();
                    },
                  ),
                ]),
              ),
              if (widget.invalid) const ErrMark(lead: 6),
              if (widget.clearable && _c.text.isNotEmpty)
                Hit(
                  focusable: false,
                  onTap: () {
                    _c.clear();
                    widget.onChanged?.call('');
                    _n.requestFocus();
                  },
                  builder: (ctx, h) => AnimatedContainer(
                    duration: kDur,
                    curve: kEase,
                    width: 16,
                    height: 16,
                    alignment: Alignment.center,
                    decoration: BoxDecoration(color: h.down ? N.g38 : (h.hover ? N.g26 : N.g20), borderRadius: BorderRadius.circular(8)),
                    child: GlyphIcon(Glyph.close, size: 10, color: h.hover ? N.g95 : N.g63, weight: 1.2),
                  ),
                ),
            ]),
          ),
        ),
      ),
      ),
    );
  }
}

// ---- 6. popover anchor and dropdown ----------------------------------------------------------------------------------

class PopAnchor extends StatefulWidget {
  const PopAnchor({super.key, required this.anchor, required this.menu, this.width, this.initialOpen = false});
  final Widget Function(BuildContext, bool open, VoidCallback toggle) anchor;
  final Widget Function(BuildContext, VoidCallback close) menu;
  final double? width;
  final bool initialOpen;
  @override
  State<PopAnchor> createState() => _PopAnchorState();
}

class _PopAnchorState extends State<PopAnchor> {
  final _link = LayerLink();
  final _ctl = OverlayPortalController();
  bool _open = false;
  void _set(bool o) {
    if (o == _open) return;
    setState(() => _open = o);
    o ? _ctl.show() : _ctl.hide();
  }

  @override
  void initState() {
    super.initState();
    if (widget.initialOpen) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _set(true);
      });
    }
  }

  @override
  void didUpdateWidget(PopAnchor o) {
    super.didUpdateWidget(o);
    if (o.initialOpen != widget.initialOpen) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _set(widget.initialOpen);
      });
    }
  }

  @override
  Widget build(BuildContext context) => CompositedTransformTarget(
        link: _link,
        child: OverlayPortal(
          controller: _ctl,
          overlayChildBuilder: (_) {
            final w = widget.width ?? (this.context.findRenderObject() as RenderBox?)?.size.width ?? 160;
            return Stack(children: [
              Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.translucent, onTap: () => _set(false), child: const SizedBox.expand())),
              CompositedTransformFollower(
                link: _link,
                showWhenUnlinked: false,
                targetAnchor: Alignment.bottomLeft,
                followerAnchor: Alignment.topLeft,
                offset: const Offset(0, 4),
                child: Align(
                  alignment: Alignment.topLeft,
                  child: SizedBox(
                    width: w,
                    child: TweenAnimationBuilder<double>(
                      tween: Tween(begin: 0, end: 1),
                      duration: kDur,
                      curve: kEase,
                      builder: (c, t, child) => Opacity(opacity: t, child: Transform.translate(offset: Offset(0, (1 - t) * -4), child: child)),
                      child: widget.menu(context, () => _set(false)),
                    ),
                  ),
                ),
              ),
            ]);
          },
          child: widget.anchor(context, _open, () => _set(!_open)),
        ),
      );
}

/// A floating surface: the only kind of thing in the inputs that carries a shadow.
class FloatCard extends StatelessWidget {
  const FloatCard({super.key, required this.child, this.padding = const EdgeInsets.all(4)});
  final Widget child;
  final EdgeInsets padding;
  @override
  Widget build(BuildContext context) => Container(
        padding: padding,
        decoration: BoxDecoration(
          color: N.g13,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: N.g20),
          boxShadow: [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 16, offset: const Offset(0, 6))],
        ),
        child: child,
      );
}

class Dropdown extends StatelessWidget {
  const Dropdown({super.key, required this.items, required this.index, this.onChanged, this.look = Look.concept, this.enabled = true, this.initialOpen = false, this.ring = false});
  final List<String> items;
  final int index;
  final ValueChanged<int>? onChanged;
  final Look look;
  final bool enabled, initialOpen, ring;
  @override
  Widget build(BuildContext context) => Dim(
        !enabled,
        PopAnchor(
          initialOpen: initialOpen,
          anchor: (ctx, open, toggle) => Hit(
            onTap: toggle,
            ringFromKeyboardOnly: true,
            onKey: (e) {
              final k = e.logicalKey;
              if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.space || k == LogicalKeyboardKey.arrowDown) {
                if (!open) toggle();
                return KeyEventResult.handled;
              }
              if (k == LogicalKeyboardKey.escape && open) {
                toggle();
                return KeyEventResult.handled;
              }
              return KeyEventResult.ignored;
            },
            builder: (c, h) => FocusRing(
              on: ring || h.focus,
              child: AnimatedContainer(
                duration: kDur,
                curve: kEase,
                height: kH,
                padding: const EdgeInsets.only(left: 8, right: 6),
                decoration: wellDeco(look, hover: h.hover || h.down, focus: h.focus || open || ring),
                child: Row(children: [
                  Expanded(child: Text(items[index], style: T.name(N.g95), maxLines: 1, overflow: TextOverflow.ellipsis)),
                  AnimatedRotation(turns: open ? .5 : 0, duration: kDur, curve: kEase, child: GlyphIcon(Glyph.chevron, size: 14, color: h.hover || open ? N.g95 : N.g56)),
                ]),
              ),
            ),
          ),
          menu: (ctx, close) => FloatCard(
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              for (var i = 0; i < items.length; i++)
                Hit(
                  focusable: false,
                  onTap: () {
                    onChanged?.call(i);
                    close();
                  },
                  builder: (c, h) => AnimatedContainer(
                    duration: kDur,
                    curve: kEase,
                    height: kH,
                    padding: const EdgeInsets.symmetric(horizontal: 4),
                    decoration: BoxDecoration(color: h.hover ? N.g20 : null, borderRadius: BorderRadius.circular(4)),
                    child: Row(children: [
                      SizedBox(width: 16, child: i == index ? GlyphIcon(Glyph.check, size: 12, color: C.mode) : null),
                      const SizedBox(width: 4),
                      Expanded(child: Text(items[i], style: T.name(i == index || h.hover ? N.g95 : N.g76))),
                    ]),
                  ),
                ),
            ]),
          ),
        ),
      );
}

// ---- 7. checkbox and radio -------------------------------------------------------------------------------------------

KeyEventResult _activate(KeyEvent e, VoidCallback f) {
  if (e.logicalKey == LogicalKeyboardKey.space || e.logicalKey == LogicalKeyboardKey.enter) {
    f();
    return KeyEventResult.handled;
  }
  return KeyEventResult.ignored;
}

/// Shared look of the 14 px mark of a checkbox or radio.
BoxDecoration _markDeco(Look look, {required bool on, required HitState h, required bool round}) {
  Color fill, border;
  var glow = const <BoxShadow>[];
  switch (look) {
    case Look.concept:
      fill = on ? C.mode : (h.hover ? N.g10 : N.g07);
      border = h.focus ? N.g100 : (on ? Role.selected : (h.hover ? N.g44 : N.g26));
    case Look.quiet:
      fill = on ? N.g20 : (h.hover ? N.g15 : N.g13);
      border = h.focus ? N.g100 : (on ? N.g63 : N.g20);
    case Look.glow:
      fill = on ? C.mode.withValues(alpha: .22) : N.g07;
      border = h.focus ? N.g100 : (on ? Role.selected : (h.hover ? N.g44 : N.g26));
      if (on) glow = [BoxShadow(color: C.mode.withValues(alpha: .35), blurRadius: 6)];
  }
  if (h.down) fill = N.g26;
  return BoxDecoration(color: fill, shape: round ? BoxShape.circle : BoxShape.rectangle, borderRadius: round ? null : BorderRadius.circular(3), border: Border.all(color: border), boxShadow: glow);
}

class CheckBox extends StatelessWidget {
  const CheckBox({super.key, required this.label, required this.value, this.onChanged, this.look = Look.concept, this.enabled = true});
  final String label;
  final bool? value; // null = mixed
  final ValueChanged<bool>? onChanged;
  final Look look;
  final bool enabled;
  @override
  Widget build(BuildContext context) {
    void flip() => onChanged?.call(value != true);
    return Dim(
      !enabled,
      Hit(
        onTap: flip,
        onKey: (e) => _activate(e, flip),
        builder: (c, h) {
          final on = value != false;
          final ink = look == Look.glow ? C.mode : (look == Look.quiet ? N.g95 : N.g100);
          return Row(mainAxisSize: MainAxisSize.min, children: [
            AnimatedContainer(
              duration: kDur,
              curve: kEase,
              width: 14,
              height: 14,
              alignment: Alignment.center,
              decoration: _markDeco(look, on: on, h: h, round: false),
              child: AnimatedOpacity(opacity: on ? 1 : 0, duration: kDur, curve: kEase, child: GlyphIcon(value == null ? Glyph.minus : Glyph.check, size: 12, color: ink, weight: 1.6)),
            ),
            const SizedBox(width: 8),
            Text(label, style: T.name(h.hover || on ? N.g95 : N.g76)),
          ]);
        },
      ),
    );
  }
}

class RadioDot extends StatelessWidget {
  const RadioDot({super.key, required this.label, required this.on, this.onTap, this.look = Look.concept, this.enabled = true});
  final String label;
  final bool on, enabled;
  final Look look;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => Dim(
        !enabled,
        Hit(
          onTap: onTap,
          onKey: (e) => _activate(e, () => onTap?.call()),
          builder: (c, h) => Row(mainAxisSize: MainAxisSize.min, children: [
            AnimatedContainer(
              duration: kDur,
              curve: kEase,
              width: 14,
              height: 14,
              alignment: Alignment.center,
              decoration: _markDeco(look, on: on, h: h, round: true),
              child: AnimatedContainer(
                duration: kDur,
                curve: kEase,
                width: on ? 6 : 0,
                height: on ? 6 : 0,
                decoration: BoxDecoration(shape: BoxShape.circle, color: look == Look.glow ? C.mode : N.g100),
              ),
            ),
            const SizedBox(width: 8),
            Text(label, style: T.name(h.hover || on ? N.g95 : N.g76)),
          ]),
        ),
      );
}

// ---- 8/9. slider and range -------------------------------------------------------------------------------------------

class TickSlider extends StatefulWidget {
  const TickSlider({super.key, required this.lo, required this.hi, this.range = false, this.min = 0, this.max = 100, this.ticks = 10, this.snap = false, this.onChanged, this.look = Look.concept, this.enabled = true});
  final double lo, hi, min, max;
  final bool range, snap, enabled;
  final int ticks;
  final Look look;
  final void Function(double lo, double hi)? onChanged;
  @override
  State<TickSlider> createState() => _TickSliderState();
}

class _TickSliderState extends State<TickSlider> {
  int _active = 1;
  bool _drag = false;
  static const _thumb = 12.0;

  double _val(double x, double w) {
    final t = ((x - _thumb / 2) / (w - _thumb)).clamp(0.0, 1.0);
    var v = widget.min + t * (widget.max - widget.min);
    if (widget.snap && widget.ticks > 0) {
      final st = (widget.max - widget.min) / widget.ticks;
      v = widget.min + ((v - widget.min) / st).round() * st;
    }
    return v;
  }

  void _apply(double v) {
    if (widget.range) {
      _active == 0 ? widget.onChanged?.call(math.min(v, widget.hi), widget.hi) : widget.onChanged?.call(widget.lo, math.max(v, widget.lo));
    } else {
      widget.onChanged?.call(widget.min, v);
    }
  }

  @override
  Widget build(BuildContext context) {
    final span = widget.max - widget.min;
    final look = widget.look;
    return Dim(
      !widget.enabled,
      LayoutBuilder(builder: (context, box) {
        final w = box.maxWidth, inner = w - _thumb;
        double tx(double v) => (v - widget.min) / span * inner;
        final lo = widget.range ? widget.lo : widget.min, hi = widget.hi;
        return Hit(
          cursor: SystemMouseCursors.grab,
          onPanDown: (d) {
            if (widget.range) {
              final x = d.localPosition.dx;
              _active = (x - (tx(lo) + _thumb / 2)).abs() <= (x - (tx(hi) + _thumb / 2)).abs() ? 0 : 1;
              if ((tx(lo) - tx(hi)).abs() < 1) _active = x < w / 2 ? 0 : 1;
            }
            setState(() => _drag = true);
            _apply(_val(d.localPosition.dx, w));
          },
          onPanUpdate: (d) => _apply(_val(d.localPosition.dx, w)),
          onPanEnd: (_) => setState(() => _drag = false),
          onTap: () => setState(() => _drag = false),
          onKey: (e) {
            final k = e.logicalKey;
            final dir = k == LogicalKeyboardKey.arrowRight || k == LogicalKeyboardKey.arrowUp ? 1 : (k == LogicalKeyboardKey.arrowLeft || k == LogicalKeyboardKey.arrowDown ? -1 : 0);
            if (dir == 0) return KeyEventResult.ignored;
            final st = widget.snap && widget.ticks > 0 ? span / widget.ticks : span / 100;
            final cur = _active == 0 && widget.range ? lo : hi;
            _apply((cur + dir * st * (HardwareKeyboard.instance.isShiftPressed ? 10 : 1)).clamp(widget.min, widget.max));
            return KeyEventResult.handled;
          },
          builder: (c, h) {
            final lit = h.hover || _drag || h.focus;
            final railFill = look == Look.quiet ? N.g63 : C.mode;
            Widget thumb(double v, bool isActive) {
              final ring = lit && (isActive || !widget.range);
              final base = look == Look.quiet ? N.g76 : N.g95;
              return Positioned(
                left: tx(v),
                top: 4,
                width: _thumb,
                height: _thumb,
                child: AnimatedContainer(
                  duration: kDur,
                  curve: kEase,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: base,
                    border: Border.all(color: ring ? (look == Look.quiet ? N.g100 : Role.selected) : base, width: 2),
                    boxShadow: look == Look.glow && ring ? [BoxShadow(color: C.mode.withValues(alpha: .55), blurRadius: 8)] : const [],
                  ),
                ),
              );
            }

            return SizedBox(
              height: 24,
              child: Stack(clipBehavior: Clip.none, children: [
                Positioned(left: _thumb / 2, right: _thumb / 2, top: 8, height: 4, child: DecoratedBox(decoration: BoxDecoration(color: look == Look.quiet ? N.g20 : N.g15, borderRadius: BorderRadius.circular(2), border: look == Look.concept ? Border.all(color: N.g20) : null))),
                Positioned(left: _thumb / 2 + tx(lo), width: tx(hi) - tx(lo), top: 8, height: 4, child: DecoratedBox(decoration: BoxDecoration(color: railFill, borderRadius: BorderRadius.circular(2)))),
                for (var i = 0; i <= widget.ticks; i++)
                  Positioned(
                    left: _thumb / 2 + inner * i / widget.ticks - .5,
                    top: 18,
                    width: 1,
                    height: 4,
                    child: ColoredBox(color: (i / widget.ticks) * span + widget.min >= lo - 1e-9 && (i / widget.ticks) * span + widget.min <= hi + 1e-9 ? N.g76 : N.g38),
                  ),
                if (widget.range) thumb(lo, _active == 0),
                thumb(hi, _active == 1),
              ]),
            );
          },
        );
      }),
    );
  }
}

// ---- 11/12. colour ---------------------------------------------------------------------------------------------------

Color? parseHex(String s) {
  var t = s.trim().replaceAll('#', '');
  if (t.length == 3) t = t.split('').map((c) => c + c).join();
  if (t.length != 6) return null;
  final v = int.tryParse(t, radix: 16);
  return v == null ? null : Color(0xFF000000 | v);
}

String toHex(Color c) => (c.toARGB32() & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase();

class HsvPicker extends StatelessWidget {
  const HsvPicker({super.key, required this.hsv, required this.onChanged});
  final HSVColor hsv;
  final ValueChanged<HSVColor> onChanged;
  @override
  Widget build(BuildContext context) {
    const svH = 112.0, hueH = 12.0;
    return LayoutBuilder(builder: (context, box) {
      final w = box.maxWidth;
      void sv(Offset p) => onChanged(hsv.withSaturation((p.dx / w).clamp(0.0, 1.0)).withValue((1 - p.dy / svH).clamp(0.0, 1.0)));
      void hue(Offset p) => onChanged(hsv.withHue((p.dx / w).clamp(0.0, 1.0) * 359.99));
      final pure = HSVColor.fromAHSV(1, hsv.hue, 1, 1).toColor();
      return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        MouseRegion(
          cursor: SystemMouseCursors.precise,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanDown: (d) => sv(d.localPosition),
            onPanUpdate: (d) => sv(d.localPosition),
            child: SizedBox(
              height: svH,
              child: Stack(clipBehavior: Clip.none, children: [
                Positioned.fill(
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(4),
                    child: DecoratedBox(
                      decoration: BoxDecoration(gradient: LinearGradient(colors: [N.g100, pure])),
                      child: const DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [Color(0x00000000), N.g00]))),
                    ),
                  ),
                ),
                Positioned(
                  left: hsv.saturation * w - 6,
                  top: (1 - hsv.value) * svH - 6,
                  width: 12,
                  height: 12,
                  child: DecoratedBox(decoration: BoxDecoration(shape: BoxShape.circle, color: hsv.toColor(), border: Border.all(color: N.g100, width: 2))),
                ),
              ]),
            ),
          ),
        ),
        const SizedBox(height: 8),
        MouseRegion(
          cursor: SystemMouseCursors.precise,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanDown: (d) => hue(d.localPosition),
            onPanUpdate: (d) => hue(d.localPosition),
            child: SizedBox(
              height: hueH,
              child: Stack(clipBehavior: Clip.none, children: [
                Positioned.fill(
                  child: DecoratedBox(
                    decoration: BoxDecoration(
                      borderRadius: BorderRadius.circular(4),
                      gradient: LinearGradient(colors: [for (var i = 0; i <= 6; i++) HSVColor.fromAHSV(1, i * 60.0 % 360, 1, 1).toColor()]),
                    ),
                  ),
                ),
                Positioned(
                  left: hsv.hue / 360 * (w - 6),
                  top: -2,
                  width: 6,
                  height: hueH + 4,
                  child: DecoratedBox(decoration: BoxDecoration(color: pure, borderRadius: BorderRadius.circular(3), border: Border.all(color: N.g100, width: 1.5))),
                ),
              ]),
            ),
          ),
        ),
      ]);
    });
  }
}

class HexColorField extends StatefulWidget {
  const HexColorField({super.key, required this.hex, this.look = Look.concept, this.enabled = true, this.initialOpen = false, this.ring = false});
  final String hex;
  final Look look;
  final bool enabled, initialOpen, ring;
  @override
  State<HexColorField> createState() => _HexColorFieldState();
}

class _HexColorFieldState extends State<HexColorField> {
  late HSVColor hsv = HSVColor.fromColor(parseHex(widget.hex) ?? N.g95);
  late String typed = widget.hex.replaceAll('#', '').toUpperCase();
  @override
  void didUpdateWidget(HexColorField o) {
    super.didUpdateWidget(o);
    if (o.hex != widget.hex) {
      hsv = HSVColor.fromColor(parseHex(widget.hex) ?? N.g95);
      typed = widget.hex.replaceAll('#', '').toUpperCase();
    }
  }

  void _pick(HSVColor n) => setState(() {
        hsv = n;
        typed = toHex(n.toColor());
      });

  @override
  Widget build(BuildContext context) {
    final col = hsv.toColor();
    final look = widget.look;
    return Dim(
      !widget.enabled,
      Row(children: [
        PopAnchor(
          width: 184,
          initialOpen: widget.initialOpen,
          anchor: (ctx, open, toggle) => Hit(
            onTap: toggle,
            onKey: (e) => _activate(e, toggle),
            builder: (c, h) => FocusRing(
              on: widget.ring || h.focus,
              child: AnimatedContainer(
              duration: kDur,
              curve: kEase,
              width: kH,
              height: kH,
              padding: const EdgeInsets.all(2),
              decoration: BoxDecoration(
                color: look == Look.quiet ? N.g13 : N.g07,
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: h.focus || open || widget.ring ? Role.selected : (h.hover ? N.g44 : (look == Look.quiet ? N.g13 : N.g20))),
                boxShadow: look == Look.glow && open ? [BoxShadow(color: C.mode.withValues(alpha: .35), blurRadius: 8)] : const [],
              ),
              child: DecoratedBox(decoration: BoxDecoration(color: col, borderRadius: BorderRadius.circular(2))),
            ),
            ),
          ),
          menu: (ctx, close) => FloatCard(
            padding: const EdgeInsets.all(8),
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              HsvPicker(hsv: hsv, onChanged: _pick),
              const SizedBox(height: 8),
              Row(children: [
                for (final f in Fam.all) ...[
                  Hit(
                    focusable: false,
                    onTap: () => _pick(HSVColor.fromColor(f.c)),
                    builder: (c, h) => AnimatedContainer(
                      duration: kDur,
                      curve: kEase,
                      width: 20,
                      height: 20,
                      decoration: BoxDecoration(color: f.c, borderRadius: BorderRadius.circular(4), border: Border.all(color: h.hover ? N.g100 : N.g13, width: 1.5)),
                    ),
                  ),
                  if (f != Fam.all.last) const Spacer(),
                ],
              ]),
            ]),
          ),
        ),
        const SizedBox(width: 4),
        Expanded(
          child: TextBox(
            text: typed,
            mono: true,
            look: look,
            ring: widget.ring,
            maxLength: 7,
            leading: Text('#', style: T.value(N.g56)),
            invalid: parseHex(typed) == null,
            onChanged: (s) => setState(() {
              typed = s.replaceAll('#', '').toUpperCase();
              final c = parseHex(typed);
              if (c != null && typed.length == 6) hsv = HSVColor.fromColor(c);
            }),
            onSubmit: (s) => setState(() => typed = toHex(hsv.toColor())),
          ),
        ),
      ]),
    );
  }
}

// ---- 13. tags --------------------------------------------------------------------------------------------------------

class TagInput extends StatefulWidget {
  const TagInput({super.key, required this.tags, this.look = Look.concept, this.enabled = true});
  final List<String> tags;
  final Look look;
  final bool enabled;
  @override
  State<TagInput> createState() => _TagInputState();
}

class _TagInputState extends State<TagInput> {
  late List<String> tags = [...widget.tags];
  final _c = TextEditingController();
  final _n = FocusNode();
  bool _hover = false, _focus = false;
  @override
  void initState() {
    super.initState();
    _n.addListener(() => setState(() => _focus = _n.hasFocus));
  }

  @override
  void didUpdateWidget(TagInput o) {
    super.didUpdateWidget(o);
    if (o.tags.join('|') != widget.tags.join('|')) tags = [...widget.tags];
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  void _add() {
    final t = _c.text.replaceAll(',', '').trim();
    _c.clear();
    if (t.isNotEmpty && !tags.contains(t)) setState(() => tags.add(t));
  }

  @override
  Widget build(BuildContext context) {
    final quiet = widget.look == Look.quiet;
    return Dim(
      !widget.enabled,
      MouseRegion(
        cursor: SystemMouseCursors.text,
        onEnter: (_) => setState(() => _hover = true),
        onExit: (_) => setState(() => _hover = false),
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: _n.requestFocus,
          child: AnimatedContainer(
            duration: kDur,
            curve: kEase,
            constraints: const BoxConstraints(minHeight: kH),
            padding: const EdgeInsets.all(3),
            decoration: wellDeco(widget.look, hover: _hover, focus: _focus),
            child: Wrap(spacing: 4, runSpacing: 4, crossAxisAlignment: WrapCrossAlignment.center, children: [
              for (final t in tags)
                Container(
                  height: 16,
                  padding: const EdgeInsets.only(left: 6, right: 2),
                  decoration: BoxDecoration(color: quiet ? N.g20 : N.g15, borderRadius: BorderRadius.circular(3), border: Border.all(color: quiet ? N.g20 : N.g20)),
                  child: Row(mainAxisSize: MainAxisSize.min, children: [
                    Text(t, style: T.label(N.g91)),
                    const SizedBox(width: 2),
                    Hit(
                      focusable: false,
                      onTap: () => setState(() => tags.remove(t)),
                      builder: (c, h) => AnimatedContainer(
                        duration: kDur,
                        curve: kEase,
                        width: 12,
                        height: 12,
                        alignment: Alignment.center,
                        decoration: BoxDecoration(color: h.hover ? N.g38 : const Color(0x00000000), borderRadius: BorderRadius.circular(2)),
                        child: GlyphIcon(Glyph.close, size: 8, color: h.hover ? N.g100 : N.g56, weight: 1.2),
                      ),
                    ),
                  ]),
                ),
              SizedBox(
                width: 72,
                height: 16,
                child: Focus(
                  canRequestFocus: false,
                  skipTraversal: true,
                  onKeyEvent: (n, e) {
                    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.backspace && _c.text.isEmpty && tags.isNotEmpty) {
                      setState(() => tags.removeLast());
                      return KeyEventResult.handled;
                    }
                    return KeyEventResult.ignored;
                  },
                  child: Stack(alignment: Alignment.centerLeft, children: [
                    if (!_focus && tags.isEmpty) IgnorePointer(child: Text('Add tag', style: T.label(N.g44))),
                    Padding(
                      padding: const EdgeInsets.only(left: 2),
                      child: EditableText(
                        controller: _c,
                        focusNode: _n,
                        style: T.label(N.g95),
                        cursorColor: Role.selected,
                        backgroundCursorColor: N.g26,
                        selectionColor: Role.selected.withValues(alpha: .35),
                        maxLines: 1,
                        cursorWidth: 1,
                        onChanged: (s) {
                          if (s.endsWith(',')) _add();
                        },
                        onEditingComplete: _add,
                      ),
                    ),
                  ]),
                ),
              ),
            ]),
          ),
        ),
      ),
    );
  }
}

// ---- 14. stepper -----------------------------------------------------------------------------------------------------

class _RepeatBtn extends StatefulWidget {
  const _RepeatBtn({required this.glyph, required this.step, required this.enabled, required this.look});
  final Glyph glyph;
  final bool Function() step;
  final bool enabled;
  final Look look;
  @override
  State<_RepeatBtn> createState() => _RepeatBtnState();
}

class _RepeatBtnState extends State<_RepeatBtn> {
  Timer? _t;
  void _stop() {
    _t?.cancel();
    _t = null;
  }

  void _start() {
    if (!widget.step()) return;
    _t = Timer(const Duration(milliseconds: 380), () {
      _t = Timer.periodic(const Duration(milliseconds: 70), (t) {
        if (!widget.step()) _stop();
      });
    });
  }

  @override
  void dispose() {
    _stop();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Hit(
        enabled: widget.enabled,
        builder: (c, h) => Listener(
          onPointerDown: (_) => _start(),
          onPointerUp: (_) => _stop(),
          onPointerCancel: (_) => _stop(),
          child: AnimatedContainer(
            duration: kDur,
            curve: kEase,
            width: kH - 2,
            alignment: Alignment.center,
            color: h.down ? N.g26 : (h.hover ? N.g20 : const Color(0x00000000)),
            child: GlyphIcon(widget.glyph, size: 12, color: !widget.enabled ? N.g38 : (h.hover ? N.g100 : N.g76), weight: 1.4),
          ),
        ),
      );
}

class StepperField extends StatelessWidget {
  const StepperField({super.key, required this.value, required this.min, required this.max, this.step = 1, this.unit = '', this.onChanged, this.look = Look.concept, this.enabled = true});
  final double value, min, max, step;
  final String unit;
  final ValueChanged<double>? onChanged;
  final Look look;
  final bool enabled;
  @override
  Widget build(BuildContext context) {
    bool move(int dir) {
      final n = (value + dir * step * (HardwareKeyboard.instance.isShiftPressed ? 10 : 1)).clamp(min, max).toDouble();
      if (n == value) return false;
      onChanged?.call(n);
      return true;
    }

    return Dim(
      !enabled,
      Container(
        height: kH,
        decoration: wellDeco(look),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(3),
          child: Row(children: [
            _RepeatBtn(glyph: Glyph.minus, step: () => move(-1), enabled: value > min, look: look),
            const SizedBox(width: 1, child: ColoredBox(color: N.g20)),
            Expanded(child: Center(child: Text.rich(TextSpan(children: [TextSpan(text: value.toStringAsFixed(step < 1 ? 1 : 0), style: T.value(N.g95)), if (unit.isNotEmpty) TextSpan(text: ' $unit', style: T.value(N.g56))])))),
            const SizedBox(width: 1, child: ColoredBox(color: N.g20)),
            _RepeatBtn(glyph: Glyph.plus, step: () => move(1), enabled: value < max, look: look),
          ]),
        ),
      ),
    );
  }
}

// ---- 15. toggle button group -----------------------------------------------------------------------------------------

class ToggleGroup extends StatelessWidget {
  const ToggleGroup({super.key, required this.icons, required this.selected, this.multi = false, this.onChanged, this.look = Look.concept, this.enabled = true});
  final List<Widget Function(Color)> icons;
  final Set<int> selected;
  final bool multi, enabled;
  final Look look;
  final ValueChanged<Set<int>>? onChanged;
  @override
  Widget build(BuildContext context) => Dim(
        !enabled,
        Container(
          padding: const EdgeInsets.all(2),
          decoration: BoxDecoration(color: look == Look.quiet ? N.g13 : N.g07, borderRadius: BorderRadius.circular(6), border: look == Look.concept ? Border.all(color: N.g20) : null),
          child: Row(mainAxisSize: MainAxisSize.min, children: [
            for (var i = 0; i < icons.length; i++) ...[
              if (i > 0) const SizedBox(width: 2),
              Hit(
                onTap: () {
                  final n = {...selected};
                  if (multi) {
                    n.contains(i) ? n.remove(i) : n.add(i);
                  } else {
                    n..clear()..add(i);
                  }
                  onChanged?.call(n);
                },
                onKey: (e) => _activate(e, () {
                  final n = {...selected};
                  if (multi) {
                    n.contains(i) ? n.remove(i) : n.add(i);
                  } else {
                    n..clear()..add(i);
                  }
                  onChanged?.call(n);
                }),
                builder: (c, h) {
                  final on = selected.contains(i);
                  final Color fill = on ? N.g20 : (h.hover ? N.g15 : const Color(0x00000000));
                  final Color ink = on || h.hover ? N.g95 : N.g56;
                  return AnimatedContainer(
                    duration: kDur,
                    curve: kEase,
                    width: 28,
                    height: kH - 4,
                    alignment: Alignment.center,
                    decoration: BoxDecoration(
                      color: h.down ? N.g26 : fill,
                      borderRadius: BorderRadius.circular(4),
                      border: Border.all(color: h.focus ? Role.selected : const Color(0x00000000)),
                    ),
                    child: Stack(alignment: Alignment.center, children: [
                      icons[i](ink),
                      Positioned(left: 6, right: 6, bottom: 1, height: 1, child: AnimatedOpacity(opacity: on ? 1 : 0, duration: kDur, curve: kEase, child: ColoredBox(color: Role.selected))),
                    ]),
                  );
                },
              ),
            ],
          ]),
        ),
      );
}

// ---- 16. curve picker ------------------------------------------------------------------------------------------------

const curvePresets = <(String, double, double, double, double)>[
  ('Linear', 0, 0, 1, 1),
  ('In', .42, 0, 1, 1),
  ('Out', 0, 0, .58, 1),
  ('In-Out', .42, 0, .58, 1),
  ('Back Out', .34, 1.56, .64, 1),
  ('Back In', .36, 0, .66, -.56),
];

class _CurvePainter extends CustomPainter {
  const _CurvePainter(this.c, this.ink);
  final (String, double, double, double, double) c;
  final Color ink;
  @override
  void paint(Canvas cv, Size s) {
    const padX = 10.0, padY = 13.0;
    final inner = s.height - padY * 2;
    double px(double x) => padX + x * (s.width - padX * 2);
    double py(double y) => padY + (1 - y) * inner;
    final base = Paint()
      ..color = N.g20
      ..strokeWidth = 1;
    cv.drawLine(Offset(padX, py(0)), Offset(s.width - padX, py(0)), base);
    cv.drawLine(Offset(padX, py(1)), Offset(s.width - padX, py(1)), base);
    final st = Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5
      ..strokeCap = StrokeCap.round;
    cv.drawPath(Path()..moveTo(px(0), py(0))..cubicTo(px(c.$2), py(c.$3), px(c.$4), py(c.$5), px(1), py(1)), st);
    final dot = Paint()..color = ink;
    cv.drawCircle(Offset(px(0), py(0)), 2, dot);
    cv.drawCircle(Offset(px(1), py(1)), 2, dot);
  }

  @override
  bool shouldRepaint(_CurvePainter o) => o.c != c || o.ink != ink;
}

class CurvePicker extends StatelessWidget {
  const CurvePicker({super.key, required this.index, this.onChanged, this.look = Look.concept, this.enabled = true});
  final int index;
  final ValueChanged<int>? onChanged;
  final Look look;
  final bool enabled;
  @override
  Widget build(BuildContext context) {
    String f(double v) => v.toStringAsFixed(2);
    final s = curvePresets[index];
    return Dim(
      !enabled,
      Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
        Wrap(spacing: 6, runSpacing: 8, children: [
          for (var i = 0; i < curvePresets.length; i++)
            Hit(
              onTap: () => onChanged?.call(i),
              onKey: (e) => _activate(e, () => onChanged?.call(i)),
              builder: (c, h) {
                final on = i == index;
                final Color fill = on ? N.g20 : (look == Look.quiet ? (h.hover ? N.g15 : N.g13) : (h.hover ? N.g10 : N.g07));
                final Color border = h.focus ? N.g100 : (on ? Role.selected : (h.hover ? N.g38 : (look == Look.quiet ? fill : N.g20)));
                return Column(mainAxisSize: MainAxisSize.min, children: [
                  AnimatedContainer(
                    duration: kDur,
                    curve: kEase,
                    width: 64,
                    height: 48,
                    decoration: BoxDecoration(
                      color: h.down ? N.g15 : fill,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: border),
                      boxShadow: look == Look.glow && on ? [BoxShadow(color: C.mode.withValues(alpha: .35), blurRadius: 8)] : const [],
                    ),
                    child: CustomPaint(painter: _CurvePainter(curvePresets[i], on ? N.g100 : (h.hover ? N.g91 : N.g63))),
                  ),
                  const SizedBox(height: 4),
                  Text(curvePresets[i].$1, style: T.micro(on ? N.g95 : N.g56)),
                ]);
              },
            ),
        ]),
        const SizedBox(height: 12),
        Text('cubic-bezier(${f(s.$2)}, ${f(s.$3)}, ${f(s.$4)}, ${f(s.$5)})', style: T.value(N.g63), maxLines: 1),
      ]),
    );
  }
}
