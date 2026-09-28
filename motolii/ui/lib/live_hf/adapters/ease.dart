import 'package:flutter/widgets.dart';

import '../../hf/desk/ease.dart';
import '../../session/editor_session.dart';

/// The Ease desk's curves over the session: the host's ease kinds (`easeKinds`, each drawn from its own samples), the
/// intervals of the picked keys (`easeIntervals`), and `ease` for a changed curve — for one interval its first key is
/// picked first, since `ease` shapes the intervals that start at the picked keys.
class LiveEaseHost extends ChangeNotifier implements EaseHost {
  LiveEaseHost(this.c) {
    c.slice('liveEase', _watched).addListener(_read);
    c.deskWork.addListener(_read);
    c.frame.addListener(_frame);
    _read();
  }
  static const _watched = ['easeKinds', 'easeIntervals', 'selectedIds', 'selectedKeys', 'layers'];
  final EditorSession c;

  List<Preset> _kinds = const [];
  List<Map<String, dynamic>> _rows = const [];

  static Shape _sampled(List samples) {
    final pts = [for (final p in samples) if (p is List && p.length >= 2) ((p[0] as num).toDouble(), (p[1] as num).toDouble())];
    if (pts.length < 2) return (t) => t;
    return (t) {
      for (var i = 1; i < pts.length; i++) {
        final (u1, v1) = pts[i];
        if (t <= u1) {
          final (u0, v0) = pts[i - 1];
          return u1 == u0 ? v1 : v0 + (v1 - v0) * (t - u0) / (u1 - u0);
        }
      }
      return pts.last.$2;
    };
  }

  static Preset _preset(Map<String, dynamic> k) {
    final name = '${k['kind']}';
    final bez = name == 'Bezier' ? [for (final n in ['x1', 'y1', 'x2', 'y2']) (k[n] as num? ?? 0).toDouble()] : null;
    return Preset(name, bez, bez == null ? _sampled(k['samples'] as List? ?? const []) : null);
  }

  void _read() {
    final kinds = EditorSession.maps(c.state['easeKinds']);
    if (kinds.isNotEmpty) _kinds = [for (final k in kinds) _preset(k)];
    _rows = EditorSession.maps(c.state['easeIntervals']);
    _layers = _sequenceLayers();
    notifyListeners();
  }

  /// The ghost mode (2026-09-07 ruling, Classic's Ease desk): no keys picked and two or more picked layers that can
  /// carry a ghost — the curve spreads their delays, in the order they were picked.
  List<Map<String, dynamic>> _layers = const [];
  List<Map<String, dynamic>> _sequenceLayers() {
    if (EditorSession.maps(c.state['selectedKeys']).isNotEmpty) return const [];
    final byId = {for (final l in c.layers) l['id']: l};
    final picked = [
      for (final id in c.selectedIds)
        if (byId[id] case final l? when l['ghostable'] == true) l,
    ];
    return picked.length > 1 ? picked : const [];
  }

  @override
  int get sequence => _layers.length;

  // The playhead moves the graph's line and the interval shown (Classic DK-012/015): heard only when either changes.
  (int?, double?) _head = (null, null);
  void _frame() {
    final next = (current, playhead(current));
    if (next != _head) {
      _head = next;
      notifyListeners();
    }
  }

  /// The interval under the playhead, else the last one before it (Classic DK-012).
  @override
  int? get current {
    if (_layers.isNotEmpty || _rows.isEmpty) return null;
    final f = c.frame.value;
    final under = _rows.indexWhere((r) => (r['frame'] as num) <= f && f < (r['end'] as num));
    if (under >= 0) return under;
    final before = [for (final (i, r) in _rows.indexed) if ((r['frame'] as num) <= f) i];
    return before.isEmpty ? 0 : before.last;
  }

  @override
  double? playhead(int? interval) {
    if (interval == null || interval >= _rows.length) return null;
    final r = _rows[interval];
    final from = (r['frame'] as num).toDouble(), to = (r['end'] as num).toDouble();
    if (to <= from) return null;
    final t = (c.frame.value - from) / (to - from);
    return t < 0 || t > 1 ? null : t;
  }

  /// What the desk edits, in Classic's words (DK-010/027/032).
  @override
  String get target {
    if (_layers.isNotEmpty) return 'Sequence · ${_layers.length} layers · ghosts${_canSequence ? '' : ' · Read only'}';
    if (_rows.isEmpty) return 'No interval · Workspace';
    final r = _rows.first;
    final name = c.layers.where((l) => l['id'] == r['layer']).firstOrNull?['name'] ?? r['layer'];
    final kinds = {for (final row in _rows) EditorSession.map(row['shape'])['kind']};
    final locked = _rows.any((row) => c.layers.where((l) => l['id'] == row['layer']).firstOrNull?['locked'] == true);
    return '$name · ${r['property']} · ${r['frame']}–${r['end']}'
        '${_rows.length > 1 ? ' · ${_rows.length} intervals' : ''}${kinds.length > 1 ? ' · Mixed' : ''}${locked || !c.supports('ease') ? ' · Read only' : ''}';
  }

  @override
  String? get caption => _layers.isNotEmpty ? '${_layers.length} layers' : (_rows.isEmpty ? 'Workspace' : null);

  Seg _seg(Map<String, dynamic> shape) {
    final i = kinds.indexWhere((k) => k.name == shape['kind']);
    final seg = Seg(i < 0 ? 0 : i, 1, kinds);
    if (shape['kind'] == 'Bezier') seg.setValues([for (final n in ['x1', 'y1', 'x2', 'y2']) (shape[n] as num? ?? 0).toDouble()]);
    return seg;
  }

  // Classic's Ease desk keeps these in the desk settings: the copied curve (`curveClip`), the saved presets
  // (`easePresets`) and the shape new keys take (`newKeyShape`).
  List<Map<String, dynamic>> get _presets => EditorSession.maps(c.deskWork.value['easePresets']);

  @override
  List<Seg> get saved => [
        if (EditorSession.map(c.deskWork.value['curveClip']).isNotEmpty) _seg(EditorSession.map(c.deskWork.value['curveClip'])),
        for (final p in _presets) _seg(p),
      ];

  @override
  void copyCurve(Seg curve) => c.storeDesk('curveClip', _shape(curve));

  @override
  void savePreset(Seg curve) => c.storeDesk('easePresets', [..._presets, _shape(curve)]);

  @override
  void clearSaved() => c.storeDesk('easePresets', <Map<String, dynamic>>[]); // Classic's bin: the saved presets

  @override
  void useForNewKeys(Seg curve) {
    c.storeDesk('newKeyShape', _shape(curve));
    if (c.animating) c.setAnimate(true); // re-arm with the new shape
  }

  bool get _canSequence => c.supports('sequence') && !_layers.any((l) => l['locked'] == true);

  /// The curve the ghost mode keeps between uses (the desk's own `ease` setting, as Classic keeps it).
  Map<String, dynamic> get _stored => EditorSession.map(c.deskWork.value['ease']);

  Map<String, dynamic> _shape(Seg curve) => {
        'kind': kinds[curve.p].name,
        if (curve.bez) ...{'x1': curve.x1, 'y1': curve.y1, 'x2': curve.x2, 'y2': curve.y2},
      };

  Map<String, dynamic> _payload(Seg curve) => {
        'layers': [for (final l in _layers) l['id']],
        'shape': _shape(curve),
      };

  Map<String, dynamic>? _pending;
  Future<void>? _flight;
  bool _previewed = false;

  @override
  void preview(int? interval, Seg curve) {
    if (_layers.isEmpty || !_canSequence || !c.supports('previewSequence')) return;
    _pending = _payload(curve);
    _previewed = true;
    _flight ??= () async {
      try {
        while (_pending != null) {
          final next = _pending!;
          _pending = null;
          await c.command('previewSequence', next);
        }
      } finally {
        _flight = null;
      }
    }();
  }

  @override
  void cancel() {
    _pending = null;
    if (_previewed) c.cancelPreview();
    _previewed = false;
  }

  @override
  List<Preset> get kinds => _kinds.isEmpty ? const [Preset('Linear', [0, 0, 1, 1], null)] : _kinds;

  @override
  List<Seg> get intervals => _layers.isNotEmpty || _rows.isEmpty
      ? [
          () {
            final shape = _stored;
            final i = kinds.indexWhere((k) => k.name == shape['kind']);
            final seg = Seg(i < 0 ? 0 : i, 1, kinds);
            if (shape['kind'] == 'Bezier') seg.setValues([for (final n in ['x1', 'y1', 'x2', 'y2']) (shape[n] as num? ?? 0).toDouble()]);
            return seg;
          }(),
        ]
      : [
        for (final r in _rows)
          () {
            final shape = EditorSession.map(r['shape']);
            final i = kinds.indexWhere((k) => k.name == shape['kind']);
            final seg = Seg(i < 0 ? 0 : i, ((r['end'] as num? ?? 1) - (r['frame'] as num? ?? 0)).round().clamp(1, 1 << 30), kinds);
            if (shape['kind'] == 'Bezier') seg.setValues([for (final n in ['x1', 'y1', 'x2', 'y2']) (shape[n] as num? ?? 0).toDouble()]);
            return seg;
          }(),
      ];

  @override
  Future<void> apply(int? interval, Seg curve) async {
    if (_layers.isEmpty && _rows.isEmpty) {
      // no interval and no sequence: the workspace curve, kept, not applied (Classic DK-030)
      await c.storeDesk('ease', _shape(curve));
      return;
    }
    if (_layers.isNotEmpty) {
      // One step on release (one undo); the preview in flight lands first.
      _pending = null;
      await _flight;
      _previewed = false;
      final payload = _payload(curve);
      await c.storeDesk('ease', payload['shape'] as Map<String, dynamic>);
      if (_canSequence) await c.command('sequence', payload);
      return;
    }
    final picked = {'ids': [...c.selectedIds], 'keys': [...EditorSession.maps(c.state['selectedKeys'])]};
    final narrowed = interval != null && interval < _rows.length;
    if (narrowed) {
      final r = _rows[interval];
      await c.command('select', {
        'ids': [r['layer']],
        'keys': [
          {'layer': r['layer'], 'property': r['property'], 'frame': r['frame']},
        ],
      });
    }
    final kind = kinds[curve.p].name;
    await c.command('ease', {
      'kind': kind,
      if (curve.bez) ...{'x1': curve.x1, 'y1': curve.y1, 'x2': curve.x2, 'y2': curve.y2},
    });
    if (narrowed) await c.command('select', picked);
  }

  @override
  void dispose() {
    c.slice('liveEase', _watched).removeListener(_read);
    c.deskWork.removeListener(_read);
    c.frame.removeListener(_frame);
    super.dispose();
  }
}

/// The Ease desk on the session.
class LiveEase extends StatefulWidget {
  const LiveEase({super.key, required this.c});
  final EditorSession c;
  @override
  State<LiveEase> createState() => _LiveEaseState();
}

class _LiveEaseState extends State<LiveEase> {
  late final host = LiveEaseHost(widget.c);
  @override
  void dispose() {
    host.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => EaseDesk(host: host);
}
