import 'dart:convert';
import 'dart:ui' show ViewFocusEvent, ViewFocusState;
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';

class EaseDesk extends StatefulWidget {
  const EaseDesk({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<EaseDesk> createState() => _EaseDeskState();
}

class _EaseDeskState extends State<EaseDesk> with WidgetsBindingObserver {
  EditorSession get c => widget.controller;
  Map<String, dynamic> _shape = {'kind': 'Linear'};
  Map<String, dynamic>? _original;
  int? _pointer, _handle;
  int _epoch = 0, _focused = 0;
  String _target = '', _source = '';
  String? _hover;
  bool _free = false, _valid = true;
  Future<void> _pending = Future.value();
  final _focus = FocusNode();
  final _presetFocus = FocusNode();
  String get _selection => jsonEncode(c.state['selectedKeys'] ?? []);
  List<Map<String, dynamic>> get _segments {
    final selected = EditorSession.maps(c.state['selectedKeys']);
    final result = <Map<String, dynamic>>[];
    for (final layer in c.layers) {
      for (final row in [
        ...EditorSession.maps(layer['properties']),
        for (final fx in EditorSession.maps(layer['effects']))
          ...EditorSession.maps(fx['params']),
      ]) {
        final chosen = selected
            .where(
              (k) =>
                  k['layer'] == layer['id'] &&
                  (k['property'] == null || k['property'] == row['id']),
            )
            .toList();
        if (chosen.isEmpty) continue;
        final frames = chosen.map((k) => (k['frame'] as num).toInt()).toSet();
        final last = frames.reduce(math.max);
        final keys = EditorSession.maps(row['keys'])
          ..sort((a, b) => (a['frame'] as num).compareTo(b['frame'] as num));
        for (var i = 0; i + 1 < keys.length; i++) {
          final frame = (keys[i]['frame'] as num).toInt();
          if (!frames.contains(frame) || (frames.length > 1 && frame == last))
            continue;
          result.add({
            'layer': layer['id'],
            'name': layer['name'],
            'property': row['id'],
            'frame': frame,
            'end': keys[i + 1]['frame'],
            'shape': keys[i]['interp'],
            'locked': layer['locked'] == true,
          });
        }
      }
    }
    return result;
  }

  bool get _canApply =>
      _segments.isNotEmpty &&
      !_segments.any((s) => s['locked'] == true) &&
      c.supports('ease');
  Map<String, dynamic> _payload(Map<String, dynamic> shape) => {
    for (final e in shape.entries)
      if (e.key == 'kind' || e.value is num) e.key: e.value,
  };
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    c.document.addListener(_read);
    c.frame.addListener(_redraw);
    _read();
  }

  void _redraw() {
    if (mounted) setState(() {});
  }

  void _read() {
    final segments = _segments;
    final source = segments.isEmpty
        ? ''
        : jsonEncode(segments.map((s) => s['shape']).toList());
    if (_target != _selection || (_pointer == null && source != _source)) {
      _epoch++;
      _pointer = null;
      _handle = null;
      _original = null;
      _target = _selection;
      _source = source;
      _shape = segments.isEmpty
          ? (EditorSession.map(c.deskWork.value['ease']).isEmpty
                ? {'kind': 'Linear'}
                : EditorSession.map(c.deskWork.value['ease']))
          : EditorSession.map(segments.first['shape']);
      _free = _shape['overshoots'] == true;
    }
    _redraw();
  }

  void _cancel() {
    _epoch++;
    if (_original != null) _shape = _original!;
    _original = null;
    _pointer = null;
    _handle = null;
    _redraw();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) _cancel();
  }

  @override
  void didChangeViewFocus(ViewFocusEvent event) {
    if (event.state == ViewFocusState.unfocused) _cancel();
  }

  @override
  void dispose() {
    _epoch++;
    WidgetsBinding.instance.removeObserver(this);
    c.document.removeListener(_read);
    c.frame.removeListener(_redraw);
    _focus.dispose();
    _presetFocus.dispose();
    super.dispose();
  }

  Future<void> _model(
    Map<String, dynamic> input, {
    int? handle,
    Offset? point,
  }) async {
    final epoch = ++_epoch;
    _valid = false;
    try {
      final result = EditorSession.map(
        await c.native('easeModel', {
          'shape': _payload(input),
          'overshoot': _free,
          if (handle != null) 'handle': handle,
          if (point != null) 'point': [point.dx, point.dy],
        }),
      );
      if (!mounted || epoch != _epoch) return;
      if (result['error'] != null) {
        c.error.value = '${result['error']}';
        return;
      }
      _valid = true;
      setState(() => _shape = result);
    } catch (e) {
      if (mounted && epoch == _epoch) c.error.value = '$e';
    }
  }

  Future<void> _commit() async {
    final target = _target;
    final epoch = _epoch;
    await _pending;
    if (!mounted || target != _selection || epoch != _epoch || !_valid) return;
    if (_original != null &&
        jsonEncode(_payload(_original!)) == jsonEncode(_payload(_shape))) {
      _original = null;
      return;
    }
    final shape = Map<String, dynamic>.of(_shape);
    _original = null;
    await c.storeDesk('ease', shape);
    if (mounted && epoch == _epoch && _canApply && target == _selection)
      await c.command('ease', {
        ..._payload(shape),
        'selection': jsonDecode(target),
      });
  }

  Future<void> _choose(Map<String, dynamic> shape) async {
    _cancel();
    _free = shape['overshoots'] == true || _free;
    _pending = _model(shape);
    await _commit();
  }

  List<Offset> _points(dynamic value) => (value as List? ?? [])
      .whereType<List>()
      .map((v) => Offset((v[0] as num).toDouble(), (v[1] as num).toDouble()))
      .toList();
  Widget _plot(
    Map<String, dynamic> shape, {
    bool handles = false,
    double? playhead,
  }) => CustomPaint(
    painter: EaseCurvePainter(
      shape: shape,
      handles: handles,
      free: _free,
      playhead: playhead,
    ),
    child: const SizedBox.expand(),
  );
  @override
  Widget build(BuildContext context) {
    final segments = _segments;
    final first = segments.firstOrNull;
    final mixed =
        segments
            .map((s) => jsonEncode(_payload(EditorSession.map(s['shape']))))
            .toSet()
            .length >
        1;
    final saved = EditorSession.maps(c.deskWork.value['easePresets']);
    final clip = EditorSession.map(c.deskWork.value['curveClip']);
    final presets = [
      ...EditorSession.maps(c.state['easeKinds']),
      if (clip.isNotEmpty) clip,
      ...saved,
    ];
    final target = first == null
        ? 'No interval · Workspace'
        : '${first['name']} · ${first['property']} · ${first['frame']}–${first['end']}${segments.length > 1 ? ' · ${segments.length} intervals' : ''}${mixed ? ' · Mixed' : ''}${!_canApply ? ' · Read only' : ''}';
    final u = first == null
        ? null
        : (c.frame.value - (first['frame'] as num)) /
              ((first['end'] as num) - (first['frame'] as num));
    return Focus(
      focusNode: _focus,
      onKeyEvent: (_, event) {
        if (event is KeyDownEvent &&
            event.logicalKey == LogicalKeyboardKey.escape &&
            _original != null) {
          _cancel();
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: ListView(
        padding: const EdgeInsets.all(8),
        children: [
          Text(
            target,
            style: const TextStyle(fontSize: 10, color: EditorTheme.muted),
          ),
          Wrap(
            spacing: 3,
            children: [
              panelButton('Apply', _canApply ? _commit : null),
              panelButton(
                'Copy curve',
                () => c
                    .storeDesk('curveClip', Map.of(_shape))
                    .then((_) => _redraw()),
              ),
              panelButton(
                'Overshoot',
                () => setState(() => _free = !_free),
                selected: _free,
              ),
              panelButton(
                'Save preset',
                () => c
                    .storeDesk('easePresets', [...saved, Map.of(_shape)])
                    .then((_) => _redraw()),
              ),
            ],
          ),
          AspectRatio(
            aspectRatio: 1,
            child: LayoutBuilder(
              builder: (context, box) {
                final painter = EaseCurvePainter(
                  shape: _shape,
                  handles: true,
                  free: _free,
                );
                final size = Size(box.maxWidth, box.maxHeight);
                return Listener(
                  key: const ValueKey('ease-plot'),
                  behavior: HitTestBehavior.opaque,
                  onPointerDown: (e) {
                    if (_pointer != null || e.buttons != 1) return;
                    final handles = _points(_shape['handles']);
                    final index = handles.indexWhere(
                      (p) =>
                          (painter.toPixel(p, size) - e.localPosition)
                              .distance <=
                          12,
                    );
                    if (index < 0) return;
                    _focus.requestFocus();
                    _pointer = e.pointer;
                    _handle = index;
                    _original = Map.of(_shape);
                  },
                  onPointerMove: (e) {
                    if (e.pointer != _pointer || _handle == null) return;
                    _pending = _model(
                      _shape,
                      handle: _handle,
                      point: painter.toCurve(e.localPosition, size),
                    );
                  },
                  onPointerUp: (e) {
                    if (e.pointer != _pointer) return;
                    _pointer = null;
                    _handle = null;
                    _commit();
                  },
                  onPointerCancel: (e) {
                    if (e.pointer == _pointer) _cancel();
                  },
                  child: _plot(
                    _shape,
                    handles: true,
                    playhead: u != null && u >= 0 && u <= 1 ? u : null,
                  ),
                );
              },
            ),
          ),
          LayoutBuilder(
            builder: (context, box) {
              final columns = math.max(1, ((box.maxWidth + 4) / 52).floor());
              return Focus(
                focusNode: _presetFocus,
                onFocusChange: (_) => _redraw(),
                onKeyEvent: (_, event) {
                  if (event is! KeyDownEvent || presets.isEmpty)
                    return KeyEventResult.ignored;
                  final key = event.logicalKey;
                  final delta = key == LogicalKeyboardKey.arrowRight
                      ? 1
                      : key == LogicalKeyboardKey.arrowLeft
                      ? -1
                      : key == LogicalKeyboardKey.arrowDown
                      ? columns
                      : key == LogicalKeyboardKey.arrowUp
                      ? -columns
                      : 0;
                  if (key == LogicalKeyboardKey.enter ||
                      key == LogicalKeyboardKey.space) {
                    _choose(presets[_focused.clamp(0, presets.length - 1)]);
                    return KeyEventResult.handled;
                  }
                  if (delta == 0 &&
                      key != LogicalKeyboardKey.home &&
                      key != LogicalKeyboardKey.end)
                    return KeyEventResult.ignored;
                  setState(
                    () => _focused =
                        (key == LogicalKeyboardKey.home
                                ? 0
                                : key == LogicalKeyboardKey.end
                                ? presets.length - 1
                                : _focused + delta)
                            .clamp(0, presets.length - 1),
                  );
                  return KeyEventResult.handled;
                },
                child: Semantics(
                  label: 'Easing presets',
                  child: Wrap(
                    spacing: 4,
                    runSpacing: 4,
                    children: [
                      for (var i = 0; i < presets.length; i++)
                        MouseRegion(
                          onEnter: (_) =>
                              setState(() => _hover = '${presets[i]['kind']}'),
                          onExit: (_) => setState(() => _hover = null),
                          child: Tooltip(
                            message: '${presets[i]['kind']}',
                            child: Semantics(
                              label: '${presets[i]['kind']} preset',
                              button: true,
                              child: InkWell(
                                canRequestFocus: false,
                                onTap: () {
                                  _focused = i;
                                  _choose(presets[i]);
                                },
                                child: Container(
                                  width: 48,
                                  height: 36,
                                  decoration: BoxDecoration(
                                    border: Border.all(
                                      color:
                                          (_presetFocus.hasFocus &&
                                                  _focused == i) ||
                                              jsonEncode(
                                                    _payload(presets[i]),
                                                  ) ==
                                                  jsonEncode(_payload(_shape))
                                          ? EditorTheme.accent
                                          : EditorTheme.line,
                                    ),
                                  ),
                                  child: _plot(presets[i]),
                                ),
                              ),
                            ),
                          ),
                        ),
                    ],
                  ),
                ),
              );
            },
          ),
          SizedBox(
            height: 20,
            child: Text(
              _hover ?? '${_shape['kind']}',
              style: const TextStyle(fontSize: 10, color: EditorTheme.muted),
            ),
          ),
          for (final param in _shape.entries.where((e) => e.value is num))
            Row(
              children: [
                Expanded(
                  child: Text(param.key, style: const TextStyle(fontSize: 11)),
                ),
                SizedBox(
                  width: 70,
                  child: EditorNumericField(
                    key: ValueKey('${_shape['kind']}:${param.key}'),
                    value: (param.value as num).toDouble(),
                    label: param.key,
                    enabled: true,
                    speed: .005,
                    onPreview: (v) async {
                      _original ??= Map.of(_shape);
                      _pending = _model({..._shape, param.key: v});
                      await _pending;
                    },
                    onCommit: (v) async {
                      _pending = _model({..._shape, param.key: v});
                      await _commit();
                    },
                    onFinish: () async {
                      if (_original != null) await _commit();
                    },
                    onCancel: () async => _cancel(),
                  ),
                ),
              ],
            ),
          if (saved.isNotEmpty)
            panelButton(
              'Clear saved presets',
              () => c.storeDesk('easePresets', []).then((_) => _redraw()),
            ),
        ],
      ),
    );
  }
}

class EaseCurvePainter extends CustomPainter {
  const EaseCurvePainter({
    required this.shape,
    this.handles = false,
    this.free = false,
    this.playhead,
  });
  final Map<String, dynamic> shape;
  final bool handles, free;
  final double? playhead;
  bool get expanded => free || shape['overshoots'] == true;
  double get lo => handles
      ? (expanded ? -.5 : -.35)
      : (shape['overshoots'] == true ? -.5 : 0);
  double get hi => handles
      ? (expanded ? 2.2 : 1.35)
      : (shape['overshoots'] == true ? 2.2 : 1);
  Rect plot(Size size) => (Offset.zero & size).deflate(handles ? 18 : 4);
  Offset toPixel(Offset p, Size size) {
    final r = plot(size);
    return Offset(
      r.left + p.dx * r.width,
      r.top + (hi - p.dy) / (hi - lo) * r.height,
    );
  }

  Offset toCurve(Offset p, Size size) {
    final r = plot(size);
    return Offset(
      ((p.dx - r.left) / r.width).clamp(-.2, 1.2),
      hi - (p.dy - r.top) / r.height * (hi - lo),
    );
  }

  @override
  void paint(Canvas canvas, Size size) {
    final grid = Paint()
      ..color = EditorTheme.line
      ..strokeWidth = .5;
    Offset p(double x, double y) => toPixel(Offset(x, y), size);
    if (handles)
      for (var i = 0; i <= 4; i++) {
        canvas.drawLine(p(i / 4, 0), p(i / 4, 1), grid);
        canvas.drawLine(p(0, i / 4), p(1, i / 4), grid);
      }
    final samples = (shape['samples'] as List? ?? [])
        .whereType<List>()
        .toList();
    final path = Path();
    if (shape['kind'] == 'Hold') {
      final a = p(0, 0), b = p(1, 0), c = p(1, 1);
      path.moveTo(a.dx, a.dy);
      path.lineTo(b.dx, b.dy);
      path.lineTo(c.dx, c.dy);
    } else if (samples.isNotEmpty) {
      for (var i = 0; i < samples.length; i++) {
        final q = p(
          (samples[i][0] as num).toDouble(),
          (samples[i][1] as num).toDouble(),
        );
        if (i == 0) {
          path.moveTo(q.dx, q.dy);
        } else {
          path.lineTo(q.dx, q.dy);
        }
      }
    } else {
      final a = p(0, 0), b = p(1, 1);
      path.moveTo(a.dx, a.dy);
      path.lineTo(b.dx, b.dy);
    }
    canvas.save();
    canvas.clipRect(Offset.zero & size);
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..color = EditorTheme.accent
        ..strokeWidth = 1.5,
    );
    if (handles) {
      final points = (shape['handles'] as List? ?? [])
          .whereType<List>()
          .map((v) => p((v[0] as num).toDouble(), (v[1] as num).toDouble()))
          .toList();
      if (shape['kind'] == 'Bezier' && points.length == 2) {
        canvas.drawLine(p(0, 0), points[0], grid);
        canvas.drawLine(p(1, 1), points[1], grid);
      }
      for (final q in points) {
        canvas.drawCircle(q, 4, Paint()..color = EditorTheme.ink);
      }
      if (playhead != null)
        canvas.drawLine(
          p(playhead!, lo),
          p(playhead!, hi),
          Paint()..color = EditorTheme.accent.withValues(alpha: .5),
        );
    }
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant EaseCurvePainter old) =>
      jsonEncode(old.shape) != jsonEncode(shape) ||
      old.free != free ||
      old.handles != handles ||
      old.playhead != playhead;
}
