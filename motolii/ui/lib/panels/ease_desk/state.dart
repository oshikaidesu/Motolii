part of '../ease_desk.dart';

/// 机の中身: 選択から区間を導き、模型に形を問い、下書きと確定を出す。
/// 描くのは `_EaseDeskState.build` の側。
mixin _EaseDeskLogic
    on
        State<EaseDesk>,
        WidgetsBindingObserver,
        SingleTickerProviderStateMixin<EaseDesk> {
  EditorSession get c => widget.controller;
  DocumentSlice get _slice => c.slice('ease', const [
    'layers',
    'selectedId',
    'selectedIds',
    'selectedKeys',
    'easeKinds',
    'capabilities',
  ]);
  Map<String, dynamic> _shape = {'kind': 'Linear'};
  Map<String, dynamic>? _original;
  int? _pointer, _handle;
  int _epoch = 0, _focused = 0, _commits = 0;
  Map<String, dynamic>? _interval;
  String _target = '';

  /// What the desk showed last: the segments by content (a moved layer
  /// re-derives the same empty list), the interval and whether it can apply.
  (List<Map<String, dynamic>>, Object?, Object?, Object?, bool)? _source;
  bool _moved(
    (List<Map<String, dynamic>>, Object?, Object?, Object?, bool) now,
  ) {
    final was = _source;
    return was == null ||
        !sameValue(was.$1, now.$1) ||
        was.$2 != now.$2 ||
        was.$3 != now.$3 ||
        was.$4 != now.$4 ||
        was.$5 != now.$5;
  }

  int? _hover;

  /// The tallest preset label, measured once per width and preset set.
  Object? _labelKey;
  double _labelHeight = 0;
  Map<String, dynamic>? _audition;
  String? _notice;
  late final AnimationController _motion;
  bool _free = false, _valid = true;
  Future<void> _pending = Future.value();

  /// Dragging a handle asks the model for a shape. Latest wish wins, so a
  /// sweep leaves one query with the port, not one per pointer move.
  late final _handles = EditorPreviewQueue<(int, Offset)>(
    (wish) => _model(_shape, handle: wish.$1, point: wish.$2),
  );
  final _focus = FocusNode();
  final _presetFocus = FocusNode();
  String get _selection => jsonEncode(c.state['selectedKeys'] ?? []);

  /// Once per (selection, layers): the session keeps an unchanged key's
  /// object, so a frame tick or a hover reads the same list.
  (Object?, Object?)? _segmentsFrom;
  List<Map<String, dynamic>> _segmentsCache = const [];
  bool _mixedCache = false;
  List<Map<String, dynamic>> get _segments {
    final from = (c.state['selectedKeys'], c.state['layers']);
    if (!identical(from.$1, _segmentsFrom?.$1) ||
        !identical(from.$2, _segmentsFrom?.$2)) {
      _segmentsFrom = from;
      _segmentsCache = _deriveSegments();
      _mixedCache =
          _segmentsCache
              .map((s) => _keyOf(EditorSession.map(s['shape'])))
              .toSet()
              .length >
          1;
    }
    return _segmentsCache;
  }

  /// Whether the selected intervals hold more than one shape; derived with
  /// the segments, not per build.
  bool get _mixed {
    _segments;
    return _mixedCache;
  }

  /// One encoding per shape object; the status and the desk hand back the
  /// same maps until they change, and `_shape` is replaced, never mutated.
  final _presetKeys = Expando<String>('presetKeys');
  String _keyOf(Map<String, dynamic> shape) =>
      _presetKeys[shape] ??= jsonEncode(_payload(shape));

  List<Map<String, dynamic>> _deriveSegments() {
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
        final keys = EditorSession.maps(row['keys']).toList()
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
    _slice.addListener(_read);
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

  /// 机が見せている物すべての指紋。ここが動かないなら、書類やヘッドが動いても
  /// 描き直す物はない。
  void _read() {
    final segments = _segments;
    final interval = _intervalAt(segments);
    final source = (
      segments,
      interval?['layer'],
      interval?['property'],
      interval?['frame'],
      _canApply,
    );
    if (_target != _identity ||
        (_pointer == null &&
            _original == null &&
            _commits == 0 &&
            _moved(source))) {
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
      _redraw();
    }
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
    _slice.removeListener(_read);
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
      colors: EditorTheme.of(context),
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
}
