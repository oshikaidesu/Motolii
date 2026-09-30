import 'dart:ui' as ui show ViewFocusEvent, ViewFocusState;

import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';

import '../../../colors/color_field.dart';
import '../../../theme/material_icons.dart';
import '../../../controls/leaves.dart';
import '../../../theme/editor_metrics.dart';
import '../../../controls/panel.dart';
import '../../../theme/editor_theme.dart';
import '../../../session/editor_session.dart';
import '../../../session/color_edit.dart';
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
  String? dragPart;
  final pickerFocus = FocusNode();
  late final edit = ColorEdit(widget.controller)..addListener(_changed);

  void _changed() {
    if (mounted) setState(() {});
  }

  void _configure() => edit
    ..retarget(widget.target)
    ..enabled = widget.enabled
    ..unbound = widget.unbound
    ..onPick = widget.onPick;

  @override
  void initState() {
    super.initState();
    _configure();
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

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    edit.dispose();
    pickerFocus.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant ColorPicker oldWidget) {
    super.didUpdateWidget(oldWidget);
    _configure();
  }

  List<double> get value => edit.value;

  /// Set by the last layout; sampling reads the same geometry the paint used.
  ColorWheel wheel = ColorWheel(EditorMetrics.thumb, 'square');
  String get shape =>
      widget.controller.deskWork.value['colorShape'] as String? ?? 'square';

  void sample(Offset p) {
    if (!widget.enabled) return;
    pickerFocus.requestFocus();
    final hsv = edit.hsv;
    dragPart ??= wheel.hitsInner(p, hsv) ? 'sv' : 'hue';
    edit.previewHsv(
      dragPart == 'sv' ? wheel.pickInner(p, hsv) : hsv.withHue(wheel.hueAt(p)),
    );
  }

  Future<void> commit() {
    dragPart = null;
    return edit.commit();
  }

  Future<void> cancel() {
    dragPart = null;
    return edit.cancel();
  }

  @override
  Widget build(BuildContext context) {
    final color = colorOf(value);
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
                    edit.begin();
                    sample(e.localPosition);
                  },
                  onPanUpdate: (e) => sample(e.localPosition),
                  onPanEnd: (_) => commit(),
                  onPanCancel: cancel,
                  onTapUp: (e) {
                    edit.begin();
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
                        hue: edit.hsv.hue,
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
                            await edit.setNow([c.r, c.g, c.b, value[3]]);
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
                            onTap: edit.toggleEyedropper,
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
                                    edit.begin();
                                    edit.previewAlpha(a);
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
