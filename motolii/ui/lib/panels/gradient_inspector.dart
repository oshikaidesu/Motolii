import 'dart:ui' show ViewFocusEvent, ViewFocusState;

import 'package:flutter/services.dart';
import 'package:flutter/material.dart';

import '../foundation/metrics.dart';
import '../foundation/theme.dart';
import '../foundation/panel_controls.dart';
import '../session/editor_session.dart';
import 'native_visual_sample.dart';

/// The fill's value on the sheet: one bar, its stops as handles. A handle is
/// the stop's swatch — press it and the Browser's wheel turns to it, drag it
/// along to move it, drag it off the bar to drop it; press the bar to add a
/// stop. The kind and the blend are definitions and live on the Colors shelf.
class GradientInspector extends StatefulWidget {
  const GradientInspector({
    super.key,
    required this.controller,
    required this.layer,
    required this.fill,
  });
  final EditorSession controller;
  final Map<String, dynamic> layer, fill;
  @override
  State<GradientInspector> createState() => _GradientInspectorState();
}

class _GradientInspectorState extends State<GradientInspector>
    with WidgetsBindingObserver {
  int selected = 0;
  final _focus = FocusNode();
  List<Map<String, dynamic>>? _dragRows, _draft;
  double _dragX = 0, _dragY = 0;
  bool _ending = false, _dropping = false;
  late final _queue = EditorPreviewQueue<Map<String, dynamic>>(
    (patch) => edit(patch, preview: true),
  );

  static const double _handle = EditorMetrics.s14,
      _dropReach = EditorMetrics.s32;

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
    if (_dragRows == null || _ending) return;
    _ending = true;
    final drop = _dropping && !cancel;
    try {
      await _queue.finish(
        cancel,
        () => c.command(cancel ? 'cancelPreview' : 'commitPreview'),
      );
      if (drop) {
        final rows = [...stops]..removeAt(selected);
        await edit({'stops': rows});
        if (mounted) setState(() => selected = 0);
      }
    } finally {
      _ending = false;
      _dragRows = null;
      _dropping = false;
      if (mounted) setState(() => _draft = null);
    }
  }

  @override
  void dispose() {
    if (_dragRows != null && !_ending)
      _queue.finish(true, () => c.command('cancelPreview'));
    WidgetsBinding.instance.removeObserver(this);
    _focus.dispose();
    super.dispose();
  }

  EditorSession get c => widget.controller;
  List<Map<String, dynamic>> get stops =>
      EditorSession.maps(widget.fill['stops'])
        ..sort((a, b) => (a['offset'] as num).compareTo(b['offset'] as num));
  bool get enabled =>
      widget.layer['locked'] != true && c.supports('setGradient');
  Future<void> edit(Map<String, dynamic> patch, {bool preview = false}) =>
      c.command('setGradient', {
        'slot': widget.fill['slot'],
        ...patch,
        'preview': preview,
      });

  /// Choosing a stop is focusing its colour: the Browser's wheel turns to it.
  Future<void> focus(int index) async {
    setState(() => selected = index);
    if (!c.supports('focusColor')) return;
    await c.focusColor({
      'layer': widget.layer['id'],
      'slot': stops[index]['slot'],
    });
  }

  /// A new stop where the bar was pressed, coloured like the bar there.
  Future<void> add(double offset) async {
    final rows = stops;
    if (rows.length >= 32) return;
    var before = rows.first, after = rows.last;
    for (final r in rows) {
      final o = (r['offset'] as num).toDouble();
      if (o <= offset) before = r;
    }
    for (final r in rows.reversed) {
      final o = (r['offset'] as num).toDouble();
      if (o >= offset) after = r;
    }
    final a = (before['offset'] as num).toDouble(),
        b = (after['offset'] as num).toDouble();
    final u = b > a ? ((offset - a) / (b - a)).clamp(0.0, 1.0) : 0.0;
    final ca = (before['rgba'] as List).cast<num>(),
        cb = (after['rgba'] as List).cast<num>();
    final rgba = [for (var k = 0; k < 3; k++) ca[k] + (cb[k] - ca[k]) * u, 1.0];
    final next = [
      ...rows,
      {'offset': offset, 'rgba': rgba},
    ]..sort((x, y) => (x['offset'] as num).compareTo(y['offset'] as num));
    final index = next.indexWhere((r) => r['offset'] == offset);
    await edit({'stops': next});
    if (mounted) setState(() => selected = index);
  }

  Color color(Map<String, dynamic> stop) {
    final rgba = (stop['rgba'] as List).cast<num>();
    return Color.from(
      alpha: 1,
      red: rgba[0].toDouble(),
      green: rgba[1].toDouble(),
      blue: rgba[2].toDouble(),
    );
  }

  @override
  Widget build(BuildContext context) {
    final rows = _draft ?? stops;
    if (rows.isEmpty) return const SizedBox();
    selected = selected.clamp(0, rows.length - 1);
    final solid = widget.fill['kind'] == 'solid';
    final sampleStops = rows.length == 1
        ? [
            rows.first,
            {...rows.first, 'offset': 1.0},
          ]
        : rows;
    return Focus(
      focusNode: _focus,
      onKeyEvent: (_, event) {
        if (event is! KeyDownEvent) return KeyEventResult.ignored;
        if (event.logicalKey == LogicalKeyboardKey.escape &&
            _dragRows != null) {
          _end(true);
          return KeyEventResult.handled;
        }
        if ((event.logicalKey == LogicalKeyboardKey.delete ||
                event.logicalKey == LogicalKeyboardKey.backspace) &&
            enabled &&
            rows.length > 2) {
          edit({
            'stops': [...rows]..removeAt(selected),
          });
          setState(() => selected = 0);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: LayoutBuilder(
        builder: (context, box) {
          final span = box.maxWidth - _handle;
          return Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // The bar: press to add a stop. A solid is a bar of one colour;
              // adding a stop makes it a gradient.
              GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTapUp: !enabled
                    ? null
                    : (e) => add(
                        ((e.localPosition.dx - _handle / 2) / span).clamp(
                          0.0,
                          1.0,
                        ),
                      ),
                child: Container(
                  key: const ValueKey('gradient-bar'),
                  height: EditorMetrics.s16,
                  margin: const EdgeInsets.symmetric(horizontal: _handle / 2),
                  decoration: BoxDecoration(
                    border: Border.all(color: EditorTheme.line),
                    borderRadius: BorderRadius.circular(EditorMetrics.s2),
                  ),
                  clipBehavior: Clip.antiAlias,
                  child: solid
                      ? ColoredBox(color: color(rows.first))
                      : NativeVisualSample(
                          controller: c,
                          request: {
                            'kind': 'gradient',
                            'type': 'linear',
                            'stops': sampleStops,
                            'blend': widget.fill['blend'],
                          },
                          fit: BoxFit.fill,
                        ),
                ),
              ),
              // The handles: each is its stop's swatch.
              SizedBox(
                height: _handle + EditorMetrics.s2,
                child: Stack(
                  clipBehavior: Clip.none,
                  children: [
                    for (var i = 0; i < rows.length; i++)
                      Positioned(
                        left: (rows[i]['offset'] as num).toDouble() * span,
                        top: 0,
                        child: EditorTooltip(
                          message:
                              'Stop ${i + 1} · ${((rows[i]['offset'] as num) * 100).round()}% · drag off to remove',
                          child: Listener(
                            onPointerCancel: (_) => _end(true),
                            child: GestureDetector(
                              onTap: enabled ? () => focus(i) : null,
                              onPanStart: !enabled || solid
                                  ? null
                                  : (e) {
                                      if (_ending) return;
                                      _focus.requestFocus();
                                      _dragRows = stops;
                                      _dragX = e.globalPosition.dx;
                                      _dragY = e.globalPosition.dy;
                                      setState(() => selected = i);
                                    },
                              onPanUpdate: !enabled || solid
                                  ? null
                                  : (e) {
                                      final base = _dragRows;
                                      if (base == null || _ending) return;
                                      final away =
                                          (e.globalPosition.dy - _dragY).abs() >
                                              _dropReach &&
                                          base.length > 2;
                                      final lower = i == 0
                                          ? 0.0
                                          : (base[i - 1]['offset'] as num)
                                                .toDouble();
                                      final upper = i + 1 == base.length
                                          ? 1.0
                                          : (base[i + 1]['offset'] as num)
                                                .toDouble();
                                      final offset =
                                          ((base[i]['offset'] as num)
                                                      .toDouble() +
                                                  (e.globalPosition.dx -
                                                          _dragX) /
                                                      span)
                                              .clamp(lower, upper);
                                      final next = [
                                        for (
                                          var index = 0;
                                          index < base.length;
                                          index++
                                        )
                                          {
                                            ...base[index],
                                            if (index == i) 'offset': offset,
                                          },
                                      ];
                                      setState(() {
                                        _draft = next;
                                        _dropping = away;
                                      });
                                      _queue.add({'stops': next});
                                    },
                              onPanEnd: (_) => _end(false),
                              onPanCancel: () => _end(true),
                              child: Opacity(
                                opacity: _dropping && selected == i ? .35 : 1,
                                child: Container(
                                  key: ValueKey('gradient-stop:$i'),
                                  width: _handle,
                                  height: _handle,
                                  decoration: BoxDecoration(
                                    color: color(rows[i]),
                                    border: Border.all(
                                      color: selected == i
                                          ? EditorTheme.accent
                                          : EditorTheme.ink,
                                      width: EditorMetrics.s2,
                                    ),
                                    borderRadius: BorderRadius.circular(
                                      EditorMetrics.s3,
                                    ),
                                  ),
                                ),
                              ),
                            ),
                          ),
                        ),
                      ),
                  ],
                ),
              ),
            ],
          );
        },
      ),
    );
  }
}
