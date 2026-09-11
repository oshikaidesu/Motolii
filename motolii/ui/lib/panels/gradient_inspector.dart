import 'dart:ui' show ViewFocusEvent, ViewFocusState;

import 'package:flutter/services.dart';
import 'package:flutter/material.dart';

import '../foundation/metrics.dart';
import '../foundation/theme.dart';
import '../foundation/panel_controls.dart';
import '../session/editor_session.dart';
import 'native_visual_sample.dart';

class GradientInspector extends StatefulWidget {
  const GradientInspector({
    super.key,
    required this.controller,
    required this.layer,
    required this.fill,
    this.inPanel = false,
  });
  final EditorSession controller;
  final Map<String, dynamic> layer, fill;

  /// Hosted in the Colors panel: the wheel is right below, so no link to it.
  final bool inPanel;
  @override
  State<GradientInspector> createState() => _GradientInspectorState();
}

class _GradientInspectorState extends State<GradientInspector>
    with WidgetsBindingObserver {
  int selected = 0;
  final _focus = FocusNode();
  List<Map<String, dynamic>>? _dragRows, _draft;
  double _dragX = 0;
  bool _ending = false;
  late final _queue = EditorPreviewQueue<Map<String, dynamic>>(
    (patch) => edit(patch, preview: true),
  );
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
    try {
      await _queue.finish(
        cancel,
        () => c.command(cancel ? 'cancelPreview' : 'commitPreview'),
      );
    } finally {
      _ending = false;
      _dragRows = null;
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
  Future<void> focus(int index) async {
    setState(() => selected = index);
    final stop = stops[index];
    await c.command('focusColor', {
      'layer': widget.layer['id'],
      'slot': stop['slot'] ?? widget.fill['slot'],
    });
    c.browserTab.value = 'Colors';
    await c.placePanel('Colors', 'show');
  }

  Color color(Map<String, dynamic> stop) {
    final rgba = (stop['rgba'] as List).cast<num>();
    return Color.from(
      alpha: rgba.length > 3 ? rgba[3].toDouble() : 1,
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
    final kind = widget.fill['kind'] ?? 'solid';
    final sampleStops = rows.length == 1
        ? [
            rows.first,
            {...rows.first, 'offset': 1.0},
          ]
        : rows;
    Widget sample(String type) => type == 'solid'
        ? ColoredBox(color: color(rows.first))
        : NativeVisualSample(
            controller: c,
            request: {
              'kind': 'gradient',
              'type': type,
              'stops': sampleStops,
              'angle': widget.fill['angle'] ?? 0,
            },
            fit: BoxFit.fill,
          );
    return Focus(
      focusNode: _focus,
      onKeyEvent: (_, event) {
        if (event is KeyDownEvent &&
            event.logicalKey == LogicalKeyboardKey.escape &&
            _dragRows != null) {
          _end(true);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              for (final type in ['solid', 'linear', 'radial', 'angular', 'diamond'])
                Expanded(
                  child: Padding(
                    padding: const EdgeInsets.only(right: EditorMetrics.s4),
                    child: EditorTooltip(
                      message:
                          '${type[0].toUpperCase()}${type.substring(1)} fill',
                      child: InkWell(
                        key: ValueKey('fill-mode:$type'),
                        onTap: !enabled
                            ? null
                            : () async {
                                if (type == 'solid') {
                                  await c.command('setFillMode', {
                                    'slot': widget.fill['slot'],
                                    'gradient': false,
                                  });
                                } else {
                                  await edit({'kind': type});
                                }
                                if (mounted) setState(() => selected = 0);
                              },
                        // The picture of the fill is the button, the chosen
                        // one ringed. A solid fill draws all three alike, so
                        // the word under each box tells them apart.
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            Container(
                              height: EditorMetrics.tall,
                              decoration: BoxDecoration(
                                border: kind == type
                                    ? Border.all(
                                        color: EditorTheme.accent,
                                        width: EditorMetrics.s2,
                                      )
                                    : Border.all(color: EditorTheme.border),
                              ),
                              child: sample(type),
                            ),
                            Text(
                              '${type[0].toUpperCase()}${type.substring(1)}',
                              textAlign: TextAlign.center,
                              style: Theme.of(context).textTheme.bodySmall
                                  ?.copyWith(
                                    color: kind == type
                                        ? EditorTheme.accent
                                        : EditorTheme.muted,
                                  ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
            ],
          ),
          const SizedBox(height: EditorMetrics.s8),
          if (kind != 'solid') ...[
            SizedBox(height: EditorMetrics.s36, child: sample(kind)),
            SizedBox(
              height: EditorMetrics.s22,
              child: LayoutBuilder(
                builder: (context, box) => Stack(
                  children: [
                    for (var i = 0; i < rows.length; i++)
                      Positioned(
                        left:
                            ((rows[i]['offset'] as num).toDouble() *
                            (box.maxWidth - EditorMetrics.s16)),
                        child: EditorTooltip(
                          message:
                              'Stop ${i + 1} · ${((rows[i]['offset'] as num) * 100).round()}%',
                          child: Listener(
                            onPointerCancel: (_) => _end(true),
                            child: GestureDetector(
                              onPanStart: !enabled
                                  ? null
                                  : (e) {
                                      if (_ending) return;
                                      _focus.requestFocus();
                                      _dragRows = stops;
                                      _dragX = e.globalPosition.dx;
                                      setState(() => selected = i);
                                    },
                              onPanUpdate: !enabled
                                  ? null
                                  : (e) {
                                      final base = _dragRows;
                                      if (base == null || _ending) return;
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
                                                      (box.maxWidth -
                                                          EditorMetrics.s16))
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
                                      setState(() => _draft = next);
                                      _queue.add({'stops': next});
                                    },
                              onPanEnd: (_) => _end(false),
                              onPanCancel: () => _end(true),
                              child: InkWell(
                                key: ValueKey('gradient-stop:$i'),
                                onTap: enabled ? () => focus(i) : null,
                                child: Container(
                                  width: EditorMetrics.s16,
                                  height: EditorMetrics.s16,
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
            ),
            Row(
              children: [
                Expanded(
                  child: EditorNumericField(
                    key: ValueKey('stop-position:$selected'),
                    value: (rows[selected]['offset'] as num).toDouble() * 100,
                    label: 'Stop position',
                    unit: '%',
                    decimals: 1,
                    speed: .2,
                    min: selected > 0
                        ? (rows[selected - 1]['offset'] as num).toDouble() * 100
                        : 0,
                    max: selected + 1 < rows.length
                        ? (rows[selected + 1]['offset'] as num).toDouble() * 100
                        : 100,
                    enabled: enabled,
                    onPreview: (value) => edit({
                      'stops': [
                        for (var i = 0; i < rows.length; i++)
                          {
                            ...rows[i],
                            if (i == selected) 'offset': value / 100,
                          },
                      ],
                    }, preview: true),
                    onCommit: (value) => edit({
                      'stops': [
                        for (var i = 0; i < rows.length; i++)
                          {
                            ...rows[i],
                            if (i == selected) 'offset': value / 100,
                          },
                      ],
                    }),
                    onFinish: () => c.command('commitPreview'),
                    onCancel: () => c.command('cancelPreview'),
                  ),
                ),
                const SizedBox(width: EditorMetrics.s6),
                EditorTooltip(
                  message: 'Add stop',
                  child: IconButton(
                    icon: const Icon(Icons.add, size: EditorMetrics.s14),
                    onPressed: !enabled || rows.length >= 32
                        ? null
                        : () async {
                            final stop = {
                              ...rows[selected],
                              'offset': selected + 1 < rows.length
                                  ? ((rows[selected]['offset'] as num) +
                                            (rows[selected + 1]['offset']
                                                as num)) /
                                        2
                                  : ((rows[selected - 1]['offset'] as num) +
                                            (rows[selected]['offset'] as num)) /
                                        2,
                            };
                            final next = [...rows, stop]
                              ..sort(
                                (a, b) => (a['offset'] as num).compareTo(
                                  b['offset'] as num,
                                ),
                              );
                            final index = next.indexOf(stop);
                            await edit({'stops': next});
                            if (mounted) setState(() => selected = index);
                          },
                  ),
                ),
                EditorTooltip(
                  message: 'Remove stop',
                  child: IconButton(
                    icon: const Icon(Icons.remove, size: EditorMetrics.s14),
                    onPressed: !enabled || rows.length <= 2
                        ? null
                        : () => edit({
                            'stops': [...rows]..removeAt(selected),
                          }),
                  ),
                ),
              ],
            ),
            const SizedBox(height: EditorMetrics.s6),
            Row(
              children: [
                const Icon(
                  Icons.rotate_right,
                  size: EditorMetrics.s16,
                  color: EditorTheme.muted,
                ),
                const SizedBox(width: EditorMetrics.s6),
                Expanded(
                  child: EditorNumericField(
                    value: (widget.fill['angle'] as num? ?? 0).toDouble(),
                    label: 'Gradient direction',
                    unit: '°',
                    enabled: enabled,
                    onPreview: (value) => edit({'angle': value}, preview: true),
                    onCommit: (value) => edit({'angle': value}),
                    onFinish: () => c.command('commitPreview'),
                    onCancel: () => c.command('cancelPreview'),
                  ),
                ),
              ],
            ),
          ] else if (!widget.inPanel)
            InkWell(
              onTap: enabled ? () => focus(0) : null,
              child: SizedBox(
                height: EditorMetrics.s22,
                child: Row(
                  children: [
                    Icon(
                      Icons.palette_outlined,
                      size: EditorMetrics.s16,
                      color: color(rows.first),
                    ),
                    const SizedBox(width: EditorMetrics.s6),
                    const Text('Choose color'),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }
}
