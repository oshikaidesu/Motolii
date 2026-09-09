import 'dart:convert';
import 'dart:ui' show ViewFocusEvent, ViewFocusState;
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';

String _curveName(String kind) => kind.replaceAllMapped(
  RegExp(r'([a-z])([A-Z])'),
  (match) => '${match[1]} ${match[2]}',
);

const _easePaper = Color(0xffb7d8d1);
const _easeInk = Color(0xff203f39);
const _easeTime = Color(0xff854515);

String _curveMeaning(String kind) => switch (kind) {
  'Hold' => 'Wait, then change in one jump.',
  'Linear' => 'Move at a constant pace.',
  'Bezier' => 'Shape the start and finish.',
  'Bounce' => 'Reach the end, then rebound.',
  'Elastic' => 'Pass the end and spring back.',
  'Cyclic' => 'Repeat a wave as time advances.',
  'Random' => 'Vary the pace irregularly.',
  'Steps' => 'Move through distinct levels.',
  'ElasticSteps' => 'Spring into each new level.',
  _ => 'Preview the change from start to finish.',
};

double easeValueAt(Map<String, dynamic> shape, double x) {
  if (shape['kind'] == 'Hold') return x < 1 ? 0 : 1;
  final samples = (shape['samples'] as List? ?? []).whereType<List>().toList();
  if (samples.length < 2) return x;
  for (var i = 0; i + 1 < samples.length; i++) {
    final a = samples[i], b = samples[i + 1];
    final ax = (a[0] as num).toDouble(), bx = (b[0] as num).toDouble();
    if (x <= bx) {
      final t = bx == ax ? 0.0 : ((x - ax) / (bx - ax)).clamp(0.0, 1.0);
      final ay = (a[1] as num).toDouble(), by = (b[1] as num).toDouble();
      return ay + (by - ay) * t;
    }
  }
  return (samples.last[1] as num).toDouble();
}

class EaseDesk extends StatefulWidget {
  const EaseDesk({super.key, required this.controller, this.leading});
  final EditorSession controller;
  final Widget? leading;
  @override
  State<EaseDesk> createState() => _EaseDeskState();
}

class _EaseDeskState extends State<EaseDesk>
    with WidgetsBindingObserver, SingleTickerProviderStateMixin {
  EditorSession get c => widget.controller;
  Map<String, dynamic> _shape = {'kind': 'Linear'};
  Map<String, dynamic>? _original;
  int? _pointer, _handle;
  int _epoch = 0, _focused = 0, _commits = 0;
  Map<String, dynamic>? _interval;
  String _target = '', _source = '';
  int? _hover;
  Map<String, dynamic>? _audition;
  String? _notice;
  late final AnimationController _motion;
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

  /// ゴーストモード(Sequence / Stagger): 複数の層を選んだ時、選んだ順に各層のゴースト 1 枚の遅れを
  /// 曲線で配る。AE の Sequence Layers・Motion Tools Pro の Sequence を、in 点を動かさず(壊さず)やる。
  List<Map<String, dynamic>> get _sequence {
    if (_segments.isNotEmpty || c.selectedIds.length < 2) return const [];
    final byId = {for (final l in c.layers) l['id']: l};
    // ゴーストを持てない層(HDR・音声・カメラ)は並びに入れない。
    return [
      for (final id in c.selectedIds)
        if (byId[id] case final l? when l['ghostable'] == true) l,
    ];
  }

  bool get _ghostMode => _sequence.length > 1;
  String get _identity => jsonEncode({
    'keys': c.state['selectedKeys'] ?? [],
    'sequence': _ghostMode ? _sequence.map((l) => l['id']).toList() : null,
  });

  bool get _canApply => _ghostMode
      ? (c.supports('sequence') && !_sequence.any((l) => l['locked'] == true))
      : _segments.isNotEmpty &&
            !_segments.any((s) => s['locked'] == true) &&
            c.supports('ease');

  /// 選んだ順に i / N を曲線に通し、最後の層の遅れ(既定 6 f × N、既に在ればその最大)を全体として配る。
  /// 最初の層は遅れ 0 = ゴースト無し。
  Map<String, dynamic>? _sequencePayload(Map<String, dynamic> shape) {
    final layers = _sequence;
    if (layers.length < 2) return null;
    final n = layers.length - 1;
    final existing = [
      for (final l in layers)
        if (l['ghost'] case final num d) d.toInt(),
    ];
    final total = existing.isEmpty ? 6 * n : existing.reduce(math.max);
    return {
      'layers': [for (final l in layers) l['id']],
      'ghosts': [
        for (var i = 0; i < layers.length; i++)
          (total * easeValueAt(shape, i / n)).round().clamp(
            -(1 << 20),
            1 << 20,
          ),
      ],
    };
  }

  /// 放した時に 1 手(Undo 1 つ)で配る。
  Future<void> _applySequence(Map<String, dynamic> shape) async {
    final payload = _sequencePayload(shape);
    if (payload != null) await c.command('sequence', payload);
  }

  /// 掴んでいる間は下書きで配る。Stage のゴーストがその場で動く。
  /// 送るのは最新の 1 つだけ(飛行中の分が返るまで途中は捨てる)。待たせない。
  Map<String, dynamic>? _previewPending;
  Future<void>? _previewFlight;
  void _previewSequence(Map<String, dynamic> shape) {
    if (!_ghostMode || !c.supports('previewSequence')) return;
    final payload = _sequencePayload(shape);
    if (payload == null) return;
    _previewPending = payload;
    _previewFlight ??= _pumpPreview();
  }

  Future<void> _pumpPreview() async {
    try {
      while (_previewPending != null && mounted) {
        final payload = _previewPending!;
        _previewPending = null;
        await c.command('previewSequence', payload);
      }
    } finally {
      _previewFlight = null;
    }
  }

  Map<String, dynamic> _payload(Map<String, dynamic> shape) => {
    for (final e in shape.entries)
      if (e.key == 'kind' || e.value is num) e.key: e.value,
  };
  @override
  void initState() {
    super.initState();
    _motion = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1200),
    );
    WidgetsBinding.instance.addObserver(this);
    c.document.addListener(_read);
    c.frame.addListener(_read);
    _read();
  }

  void _redraw() {
    if (mounted) setState(() {});
  }

  Map<String, dynamic>? _intervalAt(List<Map<String, dynamic>> segments) {
    final frame = c.frame.value;
    final active = segments.where(
      (s) => frame >= (s['frame'] as num) && frame < (s['end'] as num),
    );
    if (active.isNotEmpty) return active.first;
    final previous =
        segments.where((s) => frame >= (s['frame'] as num)).toList()
          ..sort((a, b) => (a['frame'] as num).compareTo(b['frame'] as num));
    return previous.lastOrNull ?? segments.firstOrNull;
  }

  void _read() {
    final segments = _segments;
    final interval = _intervalAt(segments);
    final source = segments.isEmpty
        ? ''
        : jsonEncode([
            segments.map((s) => s['shape']).toList(),
            if (interval != null)
              [interval['layer'], interval['property'], interval['frame']],
          ]);
    if (_target != _identity ||
        (_pointer == null &&
            _original == null &&
            _commits == 0 &&
            source != _source)) {
      _epoch++;
      _motion.reset();
      _hover = null;
      _audition = null;
      _notice = null;
      _pointer = null;
      _handle = null;
      _original = null;
      _target = _identity;
      _source = source;
      _interval = interval;
      _shape = interval == null
          ? (EditorSession.map(c.deskWork.value['ease']).isEmpty
                ? {'kind': 'Linear'}
                : EditorSession.map(c.deskWork.value['ease']))
          : EditorSession.map(interval['shape']);
      _free = _shape['overshoots'] == true;
    }
    _redraw();
  }

  void _cancel() {
    _epoch++;
    _motion.reset();
    _hover = null;
    _audition = null;
    if (_original != null && _ghostMode) c.cancelPreview();
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
    c.frame.removeListener(_read);
    _focus.dispose();
    _presetFocus.dispose();
    _motion.dispose();
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
      if (_ghostMode && _original != null) _previewSequence(result);
    } catch (e) {
      if (mounted && epoch == _epoch) c.error.value = '$e';
    }
  }

  Future<void> _commit() async {
    _commits++;
    try {
      final target = _target;
      final epoch = _epoch;
      await _pending;
      _previewPending = null;
      await _previewFlight;
      if (!mounted || target != _identity || epoch != _epoch || !_valid) return;
      if (_original != null &&
          jsonEncode(_payload(_original!)) == jsonEncode(_payload(_shape))) {
        _original = null;
        return;
      }
      final shape = Map<String, dynamic>.of(_shape);
      _original = null;
      await c.storeDesk('ease', shape);
      if (!mounted || epoch != _epoch || !_canApply || target != _identity)
        return;
      if (_ghostMode) {
        await _applySequence(shape);
      } else {
        await c.command('ease', {
          ..._payload(shape),
          'selection': jsonDecode(_selection),
        });
      }
    } finally {
      _commits--;
      if (mounted && _commits == 0) _read();
    }
  }

  Future<void> _choose(Map<String, dynamic> shape) async {
    _cancel();
    _free = shape['overshoots'] == true || _free;
    _pending = _model(shape);
    await _commit();
    if (mounted) _runMotion();
  }

  void _runMotion() {
    if (MediaQuery.disableAnimationsOf(context)) {
      _motion.value = 1;
    } else {
      _motion.forward(from: 0);
    }
  }

  void _peek(Map<String, dynamic> shape, int index) {
    if (_original != null) return;
    setState(() {
      _hover = index;
      _audition = shape;
    });
    _runMotion();
  }

  void _endPeek() {
    if (_audition == null) return;
    _motion.reset();
    setState(() {
      _hover = null;
      _audition = null;
    });
  }

  List<Offset> _points(dynamic value) => (value as List? ?? [])
      .whereType<List>()
      .map((v) => Offset((v[0] as num).toDouble(), (v[1] as num).toDouble()))
      .toList();
  Widget _plot(
    Map<String, dynamic> shape, {
    bool handles = false,
    bool selected = false,
    double? playhead,
  }) => CustomPaint(
    painter: EaseCurvePainter(
      shape: shape,
      handles: handles,
      selected: selected,
      free: _free,
      playhead: playhead,
      ghost: handles && _ghostMode,
      marks: handles && _ghostMode
          ? [
              for (var i = 0; i < _sequence.length; i++)
                i / (_sequence.length - 1),
            ]
          : const [],
    ),
    child: const SizedBox.expand(),
  );
  @override
  Widget build(BuildContext context) {
    final segments = _segments;
    final first = _interval;
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
    final sequence = _ghostMode ? _sequence : const <Map<String, dynamic>>[];
    final target = sequence.isNotEmpty
        ? 'Sequence · ${sequence.length} layers · ghosts${!_canApply ? ' · Read only' : ''}'
        : first == null
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
      child: LayoutBuilder(
        builder: (context, viewport) {
          final narrow = viewport.maxWidth < EditorMetrics.s280;
          final compact = viewport.maxHeight < EditorMetrics.sheetWide;
          final shown = _audition ?? _shape;
          final kind = '${_shape['kind']}';
          final previewKind = '${shown['kind']}';
          final contentWidth = viewport.maxWidth - EditorMetrics.s16;
          final side = math.min(
            EditorMetrics.cell - EditorMetrics.s16,
            math.min(
              contentWidth * .5,
              math.max(
                EditorMetrics.s85,
                viewport.maxHeight -
                    EditorMetrics.row -
                    EditorMetrics.bar -
                    EditorMetrics.s16 -
                    EditorMetrics.s8 -
                    EditorMetrics.s70,
              ),
            ),
          );
          final infoWidth = contentWidth - side - EditorMetrics.s8;
          final params = _shape.entries.where((e) => e.value is num).toList()
            ..sort((a, b) {
              if (kind == 'Bezier') {
                const order = ['x1', 'y1', 'x2', 'y2'];
                return order.indexOf(a.key).compareTo(order.indexOf(b.key));
              }
              return a.key.compareTo(b.key);
            });
          Widget action(String label, IconData icon, VoidCallback onPressed) =>
              EditorIconButton(
                tooltip: label,
                padding: EdgeInsets.zero,
                constraints: EditorTheme.iconConstraints,
                iconSize: EditorMetrics.s16,
                color: EditorTheme.muted,
                onPressed: onPressed,
                icon: Icon(icon),
              );
          final graph = Container(
            width: side,
            height: side,
            decoration: BoxDecoration(
              color: _easePaper,
              borderRadius: BorderRadius.circular(EditorMetrics.s4),
            ),
            child: LayoutBuilder(
              builder: (context, box) {
                final painter = EaseCurvePainter(
                  shape: _shape,
                  handles: true,
                  free: _free,
                  ghost: sequence.isNotEmpty,
                  marks: [
                    for (var i = 0; i < sequence.length; i++)
                      i / (sequence.length - 1),
                  ],
                );
                final size = Size(box.maxWidth, box.maxHeight);
                return Listener(
                  key: const ValueKey('ease-plot'),
                  behavior: HitTestBehavior.opaque,
                  onPointerDown: (e) {
                    if (_pointer != null || e.buttons != 1) return;
                    _endPeek();
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
                  child: _plot(_shape, handles: true, playhead: u),
                );
              },
            ),
          );
          final choices = LayoutBuilder(
            builder: (context, box) {
              final columns = math.max(
                1,
                math.min(
                  3,
                  ((box.maxWidth + EditorMetrics.s4) / EditorMetrics.s96)
                      .floor(),
                ),
              );
              final tileWidth =
                  (box.maxWidth - (columns - 1) * EditorMetrics.s4) / columns;
              final labelStyle = DefaultTextStyle.of(context).style
                  .copyWith(fontSize: EditorMetrics.font);
              final labelHeight = presets.fold<double>(0, (height, preset) {
                final label = TextPainter(
                  text: TextSpan(
                    text: _curveName('${preset['kind']}'),
                    style: labelStyle,
                  ),
                  maxLines: 2,
                  textDirection: Directionality.of(context),
                  textScaler: MediaQuery.textScalerOf(context),
                )..layout(maxWidth: tileWidth - EditorMetrics.s12);
                return math.max(height, label.height);
              });
              final tileHeight = math.max(
                EditorMetrics.s70,
                EditorMetrics.s44 + EditorMetrics.s12 + labelHeight,
              );
              return Focus(
                focusNode: _presetFocus,
                onFocusChange: (focused) {
                  if (!focused) _endPeek();
                  _redraw();
                },
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
                  _peek(presets[_focused], _focused);
                  return KeyEventResult.handled;
                },
                child: Semantics(
                  label: 'Easing presets',
                  child: Wrap(
                    spacing: EditorMetrics.s4,
                    runSpacing: EditorMetrics.s4,
                    children: [
                      for (var i = 0; i < presets.length; i++)
                        MouseRegion(
                          onEnter: (_) => _peek(presets[i], i),
                          onExit: (_) => _endPeek(),
                          child: Tooltip(
                            key: ValueKey('ease-preset:$i'),
                            message: '${presets[i]['kind']}',
                            child: Semantics(
                              label: '${presets[i]['kind']} preset',
                              button: true,
                              selected:
                                  jsonEncode(_payload(presets[i])) ==
                                  jsonEncode(_payload(_shape)),
                              child: InkWell(
                                canRequestFocus: false,
                                onTap: () {
                                  _focused = i;
                                  _presetFocus.requestFocus();
                                  _choose(presets[i]);
                                },
                                borderRadius: BorderRadius.circular(
                                  EditorMetrics.s4,
                                ),
                                child: Container(
                                  width: tileWidth,
                                  height: tileHeight,
                                  decoration: BoxDecoration(
                                    color:
                                        jsonEncode(_payload(presets[i])) ==
                                            jsonEncode(_payload(_shape))
                                        ? _easePaper
                                        : _hover == i
                                        ? EditorTheme.hover
                                        : EditorTheme.panel,
                                    borderRadius: BorderRadius.circular(
                                      EditorMetrics.s4,
                                    ),
                                    border: Border.all(
                                      color:
                                          _presetFocus.hasFocus && _focused == i
                                          ? _easePaper
                                          : Colors.transparent,
                                    ),
                                  ),
                                  child: Padding(
                                    padding: const EdgeInsets.all(
                                      EditorMetrics.s4,
                                    ),
                                    child: Column(
                                      children: [
                                        Expanded(
                                          child: Center(
                                            child: SizedBox.square(
                                              dimension: EditorMetrics.s44,
                                              child: _plot(
                                                presets[i],
                                                selected:
                                                    jsonEncode(
                                                      _payload(presets[i]),
                                                    ) ==
                                                    jsonEncode(
                                                      _payload(_shape),
                                                    ),
                                              ),
                                            ),
                                          ),
                                        ),
                                        Text(
                                          _curveName('${presets[i]['kind']}'),
                                          maxLines: 2,
                                          textAlign: TextAlign.center,
                                          overflow: TextOverflow.ellipsis,
                                          style: TextStyle(
                                            fontSize: EditorMetrics.font,
                                            color:
                                                jsonEncode(
                                                      _payload(presets[i]),
                                                    ) ==
                                                    jsonEncode(_payload(_shape))
                                                ? _easeInk
                                                : EditorTheme.ink,
                                          ),
                                        ),
                                      ],
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
              );
            },
          );

          final fields = <Widget>[
            for (final param in params)
              SizedBox(
                width:
                    ((compact ? contentWidth : infoWidth) - EditorMetrics.s8) /
                    2,
                height: EditorMetrics.control,
                child: Row(
                  children: [
                    Expanded(
                      child: Tooltip(
                        message: param.key.replaceAll('_', ' '),
                        child: Text(
                          param.key.startsWith('x') || param.key.startsWith('y')
                              ? param.key.toUpperCase()
                              : param.key.replaceAll('_', ' '),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(
                            fontSize: EditorMetrics.dense,
                            color: EditorTheme.muted,
                          ),
                        ),
                      ),
                    ),
                    SizedBox(
                      width: EditorMetrics.s44,
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
              ),
          ];
          final info = SizedBox(
            width: infoWidth,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Row(
                  children: [
                    if (widget.leading != null) ...[
                      widget.leading!,
                      const SizedBox(width: EditorMetrics.s4),
                    ],
                    Expanded(
                      child: Text(
                        _curveName(kind),
                        key: const ValueKey('ease-name'),
                        maxLines: 2,
                        style: const TextStyle(
                          fontSize: EditorMetrics.title,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: EditorMetrics.s2),
                Text(
                  _audition == null
                      ? _curveMeaning(kind)
                      : '${_curveName(previewKind)} · ${_curveMeaning(previewKind)}',
                  key: const ValueKey('ease-meaning'),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: EditorMetrics.font,
                    color: EditorTheme.muted,
                  ),
                ),
                const SizedBox(height: EditorMetrics.s6),
                SizedBox(
                  height: EditorMetrics.bar,
                  child: Row(
                    children: [
                      EditorIconButton(
                        tooltip: 'Preview motion',
                        padding: EdgeInsets.zero,
                        constraints: EditorTheme.iconConstraints,
                        iconSize: EditorMetrics.s16,
                        onPressed: _runMotion,
                        icon: const Icon(Icons.play_arrow_outlined),
                      ),
                      const SizedBox(width: EditorMetrics.s4),
                      Expanded(
                        child: AnimatedBuilder(
                          animation: _motion,
                          builder: (_, __) => Semantics(
                            label: 'Motion from start to finish',
                            child: CustomPaint(
                              key: const ValueKey('ease-motion'),
                              painter: EaseMotionPainter(
                                shape: shown,
                                time: _motion.value,
                                free: _free,
                              ),
                              child: const SizedBox.expand(),
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                if (!compact) ...[
                  const SizedBox(height: EditorMetrics.s4),
                  Wrap(spacing: EditorMetrics.s8, children: fields),
                ],
              ],
            ),
          );
          final savedActions = <Widget>[
            action('Copy curve', Icons.copy_outlined, () async {
              await c.storeDesk('curveClip', Map.of(_shape));
              if (mounted) setState(() => _notice = 'Curve copied');
            }),
            action('Save preset', Icons.bookmark_add_outlined, () async {
              await c.storeDesk('easePresets', [...saved, Map.of(_shape)]);
              if (mounted) setState(() => _notice = 'Preset saved');
            }),
            if (saved.isNotEmpty)
              action(
                'Clear saved presets',
                Icons.delete_sweep_outlined,
                () async {
                  await c.storeDesk('easePresets', []);
                  if (mounted)
                    setState(() => _notice = 'Saved presets cleared');
                },
              ),
            Expanded(
              child: Tooltip(
                message: target,
                child: Text(
                  _notice ??
                      (sequence.isNotEmpty
                          ? '${sequence.length} layers'
                          : first == null
                          ? 'Workspace'
                          : '${EditorSession.maps(c.state['selectedKeys']).length} keys selected'),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: EditorMetrics.dense,
                    color: EditorTheme.muted,
                  ),
                ),
              ),
            ),
          ];
          final applyActions = <Widget>[
            Tooltip(
              message: 'Overshoot',
              child: TextButton(
                style: TextButton.styleFrom(
                  foregroundColor: _free ? _easePaper : EditorTheme.muted,
                  padding: const EdgeInsets.symmetric(
                    horizontal: EditorMetrics.s4,
                  ),
                  minimumSize: Size.zero,
                  tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                ),
                onPressed: () => setState(() => _free = !_free),
                child: Semantics(
                  toggled: _free,
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        _free
                            ? Icons.check_box_outlined
                            : Icons.check_box_outline_blank,
                        size: EditorMetrics.s14,
                      ),
                      if (!narrow) ...[
                        const SizedBox(width: EditorMetrics.s3),
                        const Text(
                          'Overshoot',
                          style: TextStyle(fontSize: EditorMetrics.dense),
                        ),
                      ],
                    ],
                  ),
                ),
              ),
            ),
            const SizedBox(width: EditorMetrics.s4),
            Tooltip(
              message: _canApply ? 'Apply to selected intervals' : target,
              child: FilledButton(
                style: FilledButton.styleFrom(
                  backgroundColor: _easePaper,
                  foregroundColor: _easeInk,
                  padding: const EdgeInsets.symmetric(
                    horizontal: EditorMetrics.s8,
                  ),
                  minimumSize: const Size(
                    EditorMetrics.field,
                    EditorMetrics.control,
                  ),
                  visualDensity: VisualDensity.compact,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(EditorMetrics.s4),
                  ),
                  textStyle: const TextStyle(
                    fontSize: EditorMetrics.font,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                onPressed: _canApply ? _commit : null,
                child: const Text('Apply'),
              ),
            ),
          ];
          final footer = SizedBox(
            height: EditorMetrics.bar,
            child: Row(children: [...savedActions, ...applyActions]),
          );
          return ColoredBox(
            color: EditorTheme.app,
            child: Padding(
              padding: const EdgeInsets.all(EditorMetrics.s8),
              child: Column(
                children: [
                  SizedBox(
                    height: EditorMetrics.row,
                    child: Tooltip(
                      message: target,
                      child: Row(
                        children: [
                          Expanded(
                            child: Text(
                              first == null
                                  ? (sequence.isNotEmpty
                                        ? 'Sequence · ${sequence.length} layers'
                                        : 'Workspace · no key interval')
                                  : '${first['name']} · ${first['property']}  ${first['frame']}–${first['end']} f',
                              key: const ValueKey('ease-interval-target'),
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: const TextStyle(
                                fontSize: EditorMetrics.dense,
                                color: EditorTheme.ink,
                              ),
                            ),
                          ),
                          const SizedBox(width: EditorMetrics.s4),
                          Text(
                            '${c.frame.value} f${u == null
                                ? ''
                                : u < 0
                                ? ' · before'
                                : u > 1
                                ? ' · after'
                                : ''}',
                            key: const ValueKey('ease-current-frame'),
                            style: TextStyle(
                              fontSize: EditorMetrics.dense,
                              color: u != null && (u < 0 || u > 1)
                                  ? EditorTheme.muted
                                  : EditorTheme.accent,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      graph,
                      const SizedBox(width: EditorMetrics.s8),
                      Expanded(child: info),
                    ],
                  ),
                  const SizedBox(height: EditorMetrics.s4),
                  Expanded(
                    child: ListView(
                      key: const ValueKey('ease-choices-scroll'),
                      padding: EdgeInsets.zero,
                      children: [
                        choices,
                        if (compact && fields.isNotEmpty) ...[
                          const SizedBox(height: EditorMetrics.s8),
                          Wrap(spacing: EditorMetrics.s8, children: fields),
                        ],
                      ],
                    ),
                  ),
                  const SizedBox(height: EditorMetrics.s4),
                  footer,
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}

class EaseCurvePainter extends CustomPainter {
  const EaseCurvePainter({
    required this.shape,
    this.handles = false,
    this.selected = false,
    this.free = false,
    this.playhead,
    this.ghost = false,
    this.marks = const [],
  });
  final Map<String, dynamic> shape;
  final bool handles, free, selected;
  final double? playhead;

  /// ゴーストモード: 差し色を反転し、鎖の各ゴーストの位置を曲線上に打つ。
  final bool ghost;
  final List<double> marks;
  bool get expanded => free || shape['overshoots'] == true;
  double get lo =>
      handles ? (expanded ? -.5 : 0) : (shape['overshoots'] == true ? -.5 : 0);
  double get hi =>
      handles ? (expanded ? 2.2 : 1) : (shape['overshoots'] == true ? 2.2 : 1);
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
      ..color = _easeInk.withValues(alpha: .15)
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
        ..color = handles || selected ? _easeInk : EditorTheme.ink
        ..strokeWidth = handles ? 2.5 : 1.5
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round,
    );
    if (ghost) {
      for (final x in marks) {
        canvas.drawCircle(
          p(x, easeValueAt(shape, x)),
          3.5,
          Paint()..color = EditorTheme.accent,
        );
      }
    }
    if (handles && playhead != null) {
      final inside = playhead! >= 0 && playhead! <= 1;
      final x = playhead!.clamp(0.0, 1.0);
      canvas.drawLine(
        p(x, lo),
        p(x, hi),
        Paint()
          ..color = inside ? _easeTime : _easeTime.withValues(alpha: .4)
          ..strokeWidth = EditorMetrics.s2,
      );
      if (inside) {
        final current = p(x, easeValueAt(shape, x));
        canvas.drawCircle(
          current,
          EditorMetrics.s6,
          Paint()..color = _easePaper,
        );
        canvas.drawCircle(
          current,
          EditorMetrics.s4,
          Paint()..color = _easeTime,
        );
      } else {
        final tip = p(x, (lo + hi) / 2);
        final direction = playhead! < 0 ? -1.0 : 1.0;
        final arrow = Path()
          ..moveTo(tip.dx + direction * EditorMetrics.s8, tip.dy)
          ..lineTo(tip.dx, tip.dy - EditorMetrics.s4)
          ..lineTo(tip.dx, tip.dy + EditorMetrics.s4)
          ..close();
        canvas.drawPath(arrow, Paint()..color = _easeTime);
      }
    }
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
        canvas.drawCircle(
          q,
          8,
          Paint()
            ..color = handles || selected
                ? _easeInk
                : EditorTheme.ink.withValues(alpha: .25),
        );
        canvas.drawCircle(q, 5, Paint()..color = _easeInk);
        canvas.drawCircle(q, 2, Paint()..color = _easePaper);
      }
    }
    canvas.restore();
    if (handles) {
      void label(String value, Offset position) {
        final text = TextPainter(
          text: TextSpan(
            text: value,
            style: const TextStyle(
              fontSize: EditorMetrics.micro,
              color: _easeInk,
            ),
          ),
          textDirection: TextDirection.ltr,
        )..layout();
        text.paint(canvas, position);
      }

      label('Value', const Offset(EditorMetrics.s4, EditorMetrics.s2));
      label(
        'Time',
        Offset(
          size.width / 2 - EditorMetrics.s12,
          size.height - EditorMetrics.s12,
        ),
      );
    }
  }

  @override
  bool shouldRepaint(covariant EaseCurvePainter old) =>
      jsonEncode(old.shape) != jsonEncode(shape) ||
      old.free != free ||
      old.handles != handles ||
      old.selected != selected ||
      old.playhead != playhead ||
      old.ghost != ghost ||
      jsonEncode(old.marks) != jsonEncode(marks);
}

class EaseMotionPainter extends CustomPainter {
  const EaseMotionPainter({
    required this.shape,
    required this.time,
    this.free = false,
  });
  final Map<String, dynamic> shape;
  final double time;
  final bool free;

  @override
  void paint(Canvas canvas, Size size) {
    final range = EaseCurvePainter(shape: shape, handles: true, free: free);
    final half = EditorMetrics.s6;
    double x(double value) =>
        half +
        (value - range.lo) /
            (range.hi - range.lo) *
            math.max(0, size.width - half * 2);
    final y = size.height / 2;
    canvas.drawLine(
      Offset(x(0), y),
      Offset(x(1), y),
      Paint()..color = EditorTheme.border,
    );
    canvas.drawLine(
      Offset(x(1), y - EditorMetrics.s4),
      Offset(x(1), y + EditorMetrics.s4),
      Paint()..color = EditorTheme.muted,
    );
    canvas.drawCircle(
      Offset(x(easeValueAt(shape, time)), y),
      half,
      Paint()..color = _easePaper,
    );
  }

  @override
  bool shouldRepaint(covariant EaseMotionPainter old) =>
      time != old.time ||
      free != old.free ||
      jsonEncode(shape) != jsonEncode(old.shape);
}
