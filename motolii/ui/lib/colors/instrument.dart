import 'dart:math' as math;

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import 'color_field.dart' show parseHex;
import 'hsv_triangle.dart';
import 'shelf.dart';
import '../browser/parts.dart';
import '../browser/panel_chrome.dart' show GlyphBox;
import '../theme/glyphs.dart';
import '../session/color_edit.dart';
import '../session/editor_session.dart';
import '../theme/neutral.dart';
import '../theme/metrics.dart' show Dn, Surface;

/// The Colors instrument over the session: the reference's wheel, hex and two bars, editing the colour target (the
/// Inspector's focused colour, else the selection's fill) through [ColorEdit] — the same preview, commit and cancel
/// Classic's wheel uses. The glyph by the hex picks the next Stage click's colour; Esc cancels a drag or the pick.
class LiveColorInstrument extends StatefulWidget {
  const LiveColorInstrument({super.key, required this.c, required this.wheel});
  final EditorSession c;
  final double wheel;
  @override
  State<LiveColorInstrument> createState() => _LiveColorInstrumentState();
}

class _LiveColorInstrumentState extends State<LiveColorInstrument> {
  static const _watched = [
    'colorTarget',
    'selectedIds',
    'capabilities',
  ];
  EditorSession get c => widget.c;
  late final ColorEdit edit = ColorEdit(c)
    ..onPick = (rgba) => setState(() => edit.unbound = rgba);
  final focus = FocusNode(debugLabel: 'colors');
  final hex = TextEditingController();
  final hexFocus = FocusNode(debugLabel: 'hex');
  String? part; // 'hue', 'sv', 'value', 'alpha'

  /// The inner shape, the desk setting Classic's wheel keeps too.
  bool get _triangle => c.deskWork.value['colorShape'] == 'triangle';

  @override
  void initState() {
    super.initState();
    _absorb();
    c.slice('liveColors', _watched, derived: () => colorReading(c)).addListener(_absorb);
    c.eyedropper.addListener(_redraw);
    c.deskWork.addListener(_redraw);
    edit.addListener(_redraw);
    hexFocus.addListener(() {
      if (!hexFocus.hasFocus) _typed(hex.text);
    });
  }

  @override
  void dispose() {
    c.slice('liveColors', _watched, derived: () => colorReading(c)).removeListener(_absorb);
    c.eyedropper.removeListener(_redraw);
    c.deskWork.removeListener(_redraw);
    edit.dispose();
    focus.dispose();
    hex.dispose();
    hexFocus.dispose();
    super.dispose();
  }

  void _redraw() {
    if (!mounted) return;
    setState(() {});
    if (!hexFocus.hasFocus) hex.text = hexText(colorOf(edit.value));
  }

  void _absorb() {
    edit
      ..retarget(colorTarget(c))
      ..enabled = c.supports('setColor');
    _redraw();
  }

  void _typed(String text) {
    final col = parseHex(text);
    if (col == null) {
      hex.text = hexText(colorOf(edit.value));
      return;
    }
    edit.setNow([col.r, col.g, col.b, edit.value[3]]);
  }

  void _wheel(Offset p, {bool start = false}) {
    final size = Size.square(widget.wheel);
    final (centre, rInner, _, square) = WheelPainter.geometry(size);
    final hsv = edit.hsv;
    final corners = _triangle ? WheelPainter.triangleAt(size, hsv.hue) : null;
    if (start) {
      focus.requestFocus();
      edit.begin();
      final inner = corners == null
          ? square.contains(p)
          : (insideTriangle(p, corners) - p).distance <= 4;
      part = inner
          ? 'sv'
          : ((p - centre).distance >= rInner - 2 ? 'hue' : null);
    }
    if (part == 'sv' && corners != null) {
      edit.previewHsv(pickInTriangle(p, corners, hsv));
    } else if (part == 'sv') {
      edit.previewHsv(
        hsv
            .withSaturation(
              ((p.dx - square.left) / square.width).clamp(0.0, 1.0),
            )
            .withValue(
              (1 - (p.dy - square.top) / square.height).clamp(0.0, 1.0),
            ),
      );
    } else if (part == 'hue') {
      final a = math.atan2(p.dy - centre.dy, p.dx - centre.dx) * 180 / math.pi;
      edit.previewHsv(hsv.withHue((a + 360) % 360));
    }
  }

  void _bar(String which, Offset p, {bool start = false}) {
    if (start) {
      focus.requestFocus();
      edit.begin();
      part = which;
    }
    final t = (1 - p.dy / widget.wheel).clamp(0.0, 1.0);
    if (which == 'value') {
      edit.previewHsv(edit.hsv.withValue(t));
    } else if (edit.target?['alpha'] == true || edit.target == null) {
      edit.previewAlpha(t);
    }
  }

  void _end() {
    part = null;
    edit.commit();
  }

  void _cancel() {
    part = null;
    edit.cancel();
  }

  Widget _drag(Widget child, void Function(Offset, {bool start}) at) =>
      GestureDetector(
        behavior: HitTestBehavior.opaque,
        onPanStart: (e) => at(e.localPosition, start: true),
        onPanUpdate: (e) => at(e.localPosition),
        onPanEnd: (_) => _end(),
        onPanCancel: _cancel,
        onTapDown: (e) => at(e.localPosition, start: true),
        onTapUp: (_) => _end(),
        child: child,
      );

  @override
  Widget build(BuildContext context) {
    final wheel = widget.wheel;
    final hsv = edit.hsv;
    final color = colorOf(edit.value);
    final picking = c.eyedropper.value;
    final alpha = edit.target == null || edit.target?['alpha'] == true;
    return Focus(
      focusNode: focus,
      onKeyEvent: (_, e) {
        if (e is! KeyDownEvent || e.logicalKey != LogicalKeyboardKey.escape)
          return KeyEventResult.ignored;
        c.eyedropper.value = false;
        _cancel();
        return KeyEventResult.handled;
      },
      child: Opacity(
        opacity: edit.enabled || edit.target == null ? 1 : .5,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Column(
                  children: [
                    _drag(
                      SizedBox(
                        key: const ValueKey('hf-color-wheel'),
                        width: wheel,
                        height: wheel,
                        child: CustomPaint(painter: WheelPainter(hsv, _triangle)),
                      ),
                      _wheel,
                    ),
                  ],
                ),
                const SizedBox(width: Surface.sectionGap),
                SizedBox(
                  height: wheel,
                  child: Row(
                    children: [
                      _drag(
                        SizedBox(
                          key: const ValueKey('hf-color-value'),
                          width: 7.5,
                          height: wheel,
                          child: CustomPaint(
                            painter: ColorBar(
                              0,
                              HSVColor.fromAHSV(
                                1,
                                hsv.hue,
                                hsv.saturation,
                                1,
                              ).toColor(),
                              1.0 - hsv.value,
                            ),
                          ),
                        ),
                        (p, {start = false}) => _bar('value', p, start: start),
                      ),
                      const SizedBox(width: Surface.inlineGap),
                      Opacity(
                        opacity: alpha ? 1 : .4,
                        child: _drag(
                          SizedBox(
                            key: const ValueKey('hf-color-alpha'),
                            width: 7.5,
                            height: wheel,
                            child: CustomPaint(
                              painter: ColorBar(1, color, 1.0 - edit.value[3]),
                            ),
                          ),
                          (p, {start = false}) =>
                              _bar('alpha', p, start: start),
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 9),
                // the readout (Swiss): what the wheel edits in small caps, then its value, both flush left
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      if (edit.target case final target?)
                        Text(
                          colorTargetTitle(c, target),
                          key: const ValueKey('hf-color-target'),
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                          style: sans(Dn.labelSize, c: N.g63, w: FontWeight.w500),
                        ),
                      const SizedBox(height: 4.5),
                      Row(children: [
                        Expanded(
                          child: EditableText(
                            key: const ValueKey('hf-color-hex'),
                            controller: hex,
                            focusNode: hexFocus,
                            style: sans(Dn.nameSize, c: N.g95, w: FontWeight.w600),
                            cursorColor: N.g82,
                            backgroundCursorColor: N.g00,
                            onSubmitted: _typed,
                          ),
                        ),
                        GestureDetector(
                          key: const ValueKey('hf-eyedropper'),
                          behavior: HitTestBehavior.opaque,
                          onTap: () {
                            focus.requestFocus();
                            edit.toggleEyedropper();
                          },
                          child: GlyphBox(HG.composite, size: 12, color: picking ? N.g95 : Surface.muted),
                        ),
                      ]),
                      // the same colour in the other numbers people ask for, in the space beside the wheel
                      const SizedBox(height: Surface.sectionGap),
                      for (final (a, b) in [
                        ('R ${(color.r * 255).round()}', 'H ${hsv.hue.round()}°'),
                        ('G ${(color.g * 255).round()}', 'S ${(hsv.saturation * 100).round()}'),
                        ('B ${(color.b * 255).round()}', 'V ${(hsv.value * 100).round()}'),
                        if (alpha) ('A ${(edit.value[3] * 100).round()}%', ''),
                      ])
                        Padding(
                          padding: const EdgeInsets.only(top: 1),
                          child: Row(children: [
                            SizedBox(width: 30, child: Text(a, softWrap: false, style: sans(Dn.labelSize, c: N.g56))),
                            Text(b, softWrap: false, style: sans(Dn.labelSize, c: N.g56)),
                          ]),
                        ),
                      // while armed, what to do next
                      if (picking)
                        Padding(
                          padding: const EdgeInsets.only(top: 4.5),
                          child: Text('Click the Stage to pick · Esc cancels', key: const ValueKey('hf-eyedropper-hint'), maxLines: 2, style: sans(Dn.microSize, c: Surface.muted)),
                        ),
                    ],
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
