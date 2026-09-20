import 'dart:ui' as ui show ViewFocusEvent, ViewFocusState;

import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';

import '../../foundation/color_field.dart';
import '../../foundation/glyphs.dart';
import '../../foundation/leaves.dart';
import '../../foundation/metrics.dart';
import '../../foundation/panel_controls.dart';
import '../../foundation/theme.dart';
import '../../session/editor_session.dart';
import 'color_values.dart';
import 'color_wheel.dart';

/// The wheel: hue on the ring, saturation and value inside. It edits the
/// focused slot as a draft while the pointer is down and writes once on
/// release; with nothing focused it only feeds the stops bar.
class ColorPicker extends StatefulWidget {
  const ColorPicker({
    required this.controller,
    required this.target,
    required this.enabled,
    required this.size,
    required this.unbound,
    required this.onPick,
  });
  final EditorSession controller;
  final Map<String, dynamic>? target;
  final bool enabled;

  /// The wheel's colour while nothing is focused; the shelf keeps it.
  final List<double> unbound;

  /// The wheel's side; the shelf has already fitted it to the panel.
  final double size;
  final ValueChanged<List<double>> onPick;
  @override
  State<ColorPicker> createState() => _ColorPickerState();
}

class _ColorPickerState extends State<ColorPicker> with WidgetsBindingObserver {
  List<double>? draft;
  String? dragPart;
  double? rememberedHue;
  bool previewUsed = false, ending = false, gestureCancelled = false;
  final pickerFocus = FocusNode();
  late final queue = EditorPreviewQueue<Map<String, dynamic>>(
    (patch) => widget.controller.command('previewColor', patch),
  );
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) cancel();
  }

  @override
  void didChangeViewFocus(ui.ViewFocusEvent event) {
    if (event.state == ui.ViewFocusState.unfocused) cancel();
  }

  bool get canPreview =>
      widget.controller.supports('previewColor') && widget.enabled;

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    if (previewUsed && !ending)
      queue.finish(true, () => widget.controller.command('cancelPreview'));
    pickerFocus.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant ColorPicker oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!sameValue(oldWidget.target?['slot'], widget.target?['slot']) ||
        oldWidget.target?['layer'] != widget.target?['layer']) {
      cancel();
    }
  }

  List<double> get value =>
      draft ??
      (widget.target == null ? widget.unbound : rgbaOf(widget.target!['rgba']));

  /// Set by the last layout; sampling reads the same geometry the paint used.
  ColorWheel wheel = ColorWheel(EditorMetrics.thumb, 'square');
  String get shape =>
      widget.controller.deskWork.value['colorShape'] as String? ?? 'square';

  void sample(Offset p) {
    if (ending || gestureCancelled || !widget.enabled) return;
    pickerFocus.requestFocus();
    final v = value;
    var hsv = HSVColor.fromColor(colorOf(v));
    if (dragPart == null) {
      if (hsv.saturation > 1 / 255) rememberedHue = hsv.hue;
      hsv = hsv.withHue(rememberedHue ?? hsv.hue);
      dragPart = wheel.hitsInner(p, hsv) ? 'sv' : 'hue';
    } else if (dragPart == 'sv') {
      // White and black have no hue. Keep the hue that this gesture began
      // with instead of deriving 0° (red) from an achromatic RGB snapshot.
      hsv = hsv.withHue(rememberedHue ?? hsv.hue);
    }
    final updated = dragPart == 'sv'
        ? wheel.pickInner(p, hsv)
        : hsv.withHue(wheel.hueAt(p));
    rememberedHue = updated.hue;
    final c = updated.toColor();
    final next = [c.r, c.g, c.b, v[3]];
    setState(() => draft = next);
    if (widget.target == null) {
      widget.onPick(next);
    } else if (canPreview) {
      previewUsed = true;
      queue.add({
        if (widget.target!['layer'] != null) 'layer': widget.target!['layer'],
        'slot': widget.target!['slot'],
        'rgba': next,
      });
    }
  }

  Future<void> commit() async {
    final target = widget.target;
    final next = draft;
    draft = null;
    dragPart = null;
    final used = previewUsed;
    previewUsed = false;
    if (next == null) return;
    if (target == null) {
      setState(() {});
      return;
    }
    if (!widget.enabled) return;
    ending = true;
    try {
      await queue.finish(
        false,
        () => used
            ? widget.controller.command('commitPreview')
            : widget.controller.command('setColor', {
                if (target['layer'] != null) 'layer': target['layer'],
                'slot': target['slot'],
                'rgba': next,
              }),
      );
    } finally {
      ending = false;
      if (mounted) setState(() {});
    }
  }

  Future<void> cancel() async {
    if (ending) return;
    gestureCancelled = true;
    final used = previewUsed;
    previewUsed = false;
    setState(() {
      draft = null;
      dragPart = null;
    });
    if (!used) return;
    ending = true;
    try {
      await queue.finish(
        true,
        () => widget.controller.command('cancelPreview'),
      );
    } finally {
      ending = false;
    }
  }

  @override
  Widget build(BuildContext context) {
    final color = colorOf(value);
    final hsv = HSVColor.fromColor(color);
    if (dragPart == null && hsv.saturation > 1 / 255) {
      rememberedHue = hsv.hue;
    }
    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.escape): () {
          widget.controller.eyedropper.value = false;
          cancel();
        },
      },
      child: Focus(
        focusNode: pickerFocus,
        child: Builder(
          builder: (context) {
            wheel = ColorWheel(widget.size, shape);
            return Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                GestureDetector(
                  onPanStart: (e) {
                    gestureCancelled = false;
                    sample(e.localPosition);
                  },
                  onPanUpdate: (e) => sample(e.localPosition),
                  onPanEnd: (_) => commit(),
                  onPanCancel: cancel,
                  onTapUp: (e) {
                    gestureCancelled = false;
                    sample(e.localPosition);
                    commit();
                  },
                  child: SizedBox(
                    width: wheel.side,
                    height: wheel.side,
                    child: CustomPaint(
                      painter: ColorWheelPainter(
                        colors: EditorTheme.of(context),
                        color,
                        wheel,
                        hue: rememberedHue,
                      ),
                    ),
                  ),
                ),
                Container(
                  width: wheel.side,
                  height: EditorMetrics.s23,
                  margin: const EdgeInsets.only(top: EditorMetrics.s6),
                  padding: const EdgeInsets.symmetric(
                    horizontal: EditorMetrics.s5,
                  ),
                  decoration: BoxDecoration(
                    color: EditorTheme.of(context).line,
                    borderRadius: BorderRadius.circular(EditorMetrics.s5),
                  ),
                  child: Row(
                    children: [
                      Swatch(color: color, size: EditorMetrics.s14),
                      const SizedBox(width: EditorMetrics.s4),
                      Expanded(
                        child: EditorDraftField(
                          key: ValueKey(
                            'browser:hex:${widget.target?['layer']}:${widget.target?['slot']}',
                          ),
                          value: hexOf(color),
                          label: 'hex',
                          enabled: widget.enabled,
                          validator: (v) => parseHex(v) == null
                              ? 'Use three or six hex digits'
                              : null,
                          onCommit: (v) async {
                            final c = parseHex(v)!;
                            final rgba = [c.r, c.g, c.b, value[3]];
                            if (widget.target == null) {
                              widget.onPick(rgba);
                              return;
                            }
                            await widget.controller.command('setColor', {
                              if (widget.target!['layer'] != null)
                                'layer': widget.target!['layer'],
                              'slot': widget.target!['slot'],
                              'rgba': rgba,
                            });
                          },
                        ),
                      ),
                      const SizedBox(width: EditorMetrics.s4),
                      ValueListenableBuilder<bool>(
                        valueListenable: widget.controller.eyedropper,
                        builder: (context, on, _) => EditorTooltip(
                          message: on
                              ? 'Click the Stage to pick a colour · Esc cancels'
                              : 'Pick a colour from the Stage',
                          child: EditorPress(
                            key: const ValueKey('browser:eyedropper'),
                            onTap: () =>
                                widget.controller.eyedropper.value = !on,
                            child: Container(
                              padding: const EdgeInsets.all(EditorMetrics.s3),
                              decoration: BoxDecoration(
                                color: on
                                    ? EditorTheme.of(context).hover
                                    : null,
                                borderRadius: BorderRadius.circular(
                                  EditorMetrics.s3,
                                ),
                              ),
                              child: Icon(
                                Glyph.colorize,
                                size: EditorMetrics.s14,
                                color: on
                                    ? EditorTheme.of(context).accent
                                    : EditorTheme.of(context).muted,
                              ),
                            ),
                          ),
                        ),
                      ),
                      EditorTooltip(
                        message: shape == 'square'
                            ? 'Switch to triangle'
                            : 'Switch to square',
                        child: EditorPress(
                          key: const ValueKey('browser:color-shape'),
                          onTap: () => widget.controller.storeDesk(
                            'colorShape',
                            shape == 'square' ? 'triangle' : 'square',
                          ),
                          child: Container(
                            height: EditorMetrics.row,
                            alignment: Alignment.center,
                            padding: const EdgeInsets.symmetric(
                              horizontal: EditorMetrics.s3,
                            ),
                            child: Text(
                              shape == 'square' ? 'Square' : 'Triangle',
                              style: TextStyle(
                                fontSize: EditorMetrics.micro,
                                color: EditorTheme.of(context).muted,
                              ),
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                SizedBox(
                  width: wheel.side,
                  height: EditorMetrics.s14,
                  child: ValueListenableBuilder<bool>(
                    valueListenable: widget.controller.eyedropper,
                    builder: (context, on, _) => Text(
                      on ? 'PICK STAGE · ESC TO CANCEL' : 'HEX · 3 OR 6 DIGITS',
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: TextStyle(
                        fontSize: EditorMetrics.micro,
                        color: on
                            ? EditorTheme.of(context).accent
                            : EditorTheme.of(context).muted,
                      ),
                    ),
                  ),
                ),
                // The alpha, only where the slot carries one (text, params).
                if (widget.target?['alpha'] == true)
                  SizedBox(
                    width: wheel.side,
                    height: EditorMetrics.s23,
                    child: Row(
                      children: [
                        Icon(
                          Glyph.opacity,
                          size: EditorMetrics.s14,
                          color: EditorTheme.of(context).muted,
                        ),
                        Expanded(
                          child: EditorSlider(
                            value: value[3],
                            onChanged: !widget.enabled
                                ? null
                                : (a) {
                                    final v = value;
                                    final next = [v[0], v[1], v[2], a];
                                    setState(() => draft = next);
                                    if (canPreview) {
                                      previewUsed = true;
                                      queue.add({
                                        if (widget.target!['layer'] != null)
                                          'layer': widget.target!['layer'],
                                        'slot': widget.target!['slot'],
                                        'rgba': next,
                                      });
                                    }
                                  },
                            onChangeEnd: (_) => commit(),
                          ),
                        ),
                      ],
                    ),
                  ),
              ],
            );
          },
        ),
      ),
    );
  }
}
