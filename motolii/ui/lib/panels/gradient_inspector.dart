import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

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
    with
        WidgetsBindingObserver,
        EditorDragSession<Map<String, dynamic>, GradientInspector> {
  int selected = 0;
  final _focus = FocusNode();
  List<Map<String, dynamic>>? _dragRows, _draft;
  double _dragX = 0, _dragY = 0;
  bool _dropping = false;

  static const double _handle = EditorMetrics.s14,
      _dropReach = EditorMetrics.s32;

  @override
  Future<void> sendPreview(Map<String, dynamic> patch) =>
      c.command('previewProperties', {
        'edits': [patch],
      });
  @override
  Future<void> commitDrag() => c.command('commitPreview');

  /// A stop pulled off the bar is dropped once the preview is withdrawn.
  @override
  Future<void> cancelDrag() async {
    await c.command('cancelPreview');
    if (!_dropping) return;
    await edit({'removeStop': _dragRows![selected]['id']});
    if (mounted) setState(() => selected = 0);
  }

  @override
  bool get discardsPreview => _dropping;
  @override
  void dragStopped(bool cancel) {
    if (cancel) _dropping = false;
  }

  @override
  void dragSettled() {
    _dragRows = null;
    _dropping = false;
    setState(() => _draft = null);
  }

  @override
  void didUpdateWidget(covariant GradientInspector oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.layer['id'] != widget.layer['id'] ||
        (oldWidget.layer['locked'] != true && widget.layer['locked'] == true))
      endDrag(true);
  }

  @override
  void dispose() {
    _dropping = false;
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
      'slot': widget.fill['kind'] == 'solid'
          ? widget.fill['slot']
          : stops[index]['slot'],
    });
  }

  Future<void> add(double offset) async {
    if (stops.length >= 32) return;
    await edit({'addStop': offset});
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      final rows = stops;
      final index = rows.indexWhere(
        (s) => ((s['offset'] as num).toDouble() - offset).abs() < .0001,
      );
      if (index >= 0) setState(() => selected = index);
    });
  }

  void _moveStop(int i, Offset point, double span) {
    final base = _dragRows;
    if (base == null || ending) return;
    final away = (point.dy - _dragY).abs() > _dropReach && base.length > 2;
    final lower = i == 0 ? 0.0 : (base[i - 1]['offset'] as num).toDouble();
    final upper = i + 1 == base.length
        ? 1.0
        : (base[i + 1]['offset'] as num).toDouble();
    final offset =
        ((base[i]['offset'] as num).toDouble() + (point.dx - _dragX) / span)
            .clamp(lower, upper);
    final next = [
      for (var index = 0; index < base.length; index++)
        {...base[index], if (index == i) 'offset': offset},
    ];
    setState(() {
      _draft = next;
      _dropping = away;
    });
    queue.add({
      'layer': widget.layer['id'],
      'property': base[i]['positionProperty'],
      'value': offset,
    });
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
          endDrag(true);
          return KeyEventResult.handled;
        }
        if ((event.logicalKey == LogicalKeyboardKey.delete ||
            event.logicalKey == LogicalKeyboardKey.backspace)) {
          if (_dragRows != null) {
            endDrag(true);
          } else if (enabled && rows.length > 2) {
            edit({'removeStop': rows[selected]['id']});
            setState(() => selected = 0);
          }
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: LayoutBuilder(
        builder: (context, box) {
          final span = box.maxWidth - _handle;
          Offset local(Offset point) =>
              (context.findRenderObject()! as RenderBox).globalToLocal(point);
          return Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // The bar: press to add a stop. A solid is a bar of one colour;
              // adding a stop makes it a gradient.
              MouseRegion(
                cursor: enabled
                    ? solid
                          ? SystemMouseCursors.click
                          : SystemMouseCursors.precise
                    : MouseCursor.defer,
                child: EditorTooltip(
                  message: solid
                      ? 'Edit fill color in Colors'
                      : 'Click empty space to add a color stop',
                  child: GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onTapUp: !enabled
                        ? null
                        : solid
                        ? (_) => focus(0)
                        : (e) => add(
                            ((e.localPosition.dx - _handle / 2) / span).clamp(
                              0.0,
                              1.0,
                            ),
                          ),
                    child: Container(
                      key: const ValueKey('gradient-bar'),
                      height: EditorMetrics.s16,
                      margin: const EdgeInsets.symmetric(
                        horizontal: _handle / 2,
                      ),
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
                              'Stop ${i + 1} · click to edit in Colors · drag to move · drag off to remove',
                          child: Listener(
                            onPointerDown: (e) {
                              final point = local(e.position);
                              _dragX = point.dx;
                              _dragY = point.dy;
                            },
                            onPointerCancel: (_) => endDrag(true),
                            child: GestureDetector(
                              onTap: enabled
                                  ? () {
                                      _focus.requestFocus();
                                      focus(i);
                                    }
                                  : null,
                              onPanStart: !enabled || solid
                                  ? null
                                  : (e) {
                                      if (ending) return;
                                      _focus.requestFocus();
                                      _dragRows = stops;
                                      beginDrag();
                                      setState(() => selected = i);
                                      _moveStop(
                                        i,
                                        local(e.globalPosition),
                                        span,
                                      );
                                    },
                              onPanUpdate: !enabled || solid
                                  ? null
                                  : (e) => _moveStop(
                                      i,
                                      local(e.globalPosition),
                                      span,
                                    ),
                              onPanEnd: (_) => endDrag(false),
                              onPanCancel: () => endDrag(true),
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
