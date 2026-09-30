import 'dart:convert';

import 'package:flutter/widgets.dart';

import '../../hf/desk/ease.dart';
import '../../session/editor_session.dart';
import 'desk_session.dart';

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
    return Preset(name, bez, bez == null ? _sampled(k['samples'] as List? ?? const []) : null, model: bez == null ? k : null);
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
    // Classic's rule: no key interval to shape (a single picked key has none) and several ghostable layers
    if (EditorSession.maps(c.state['easeIntervals']).isNotEmpty) return const [];
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
  // The playhead moves only the graph's line (frame, below), and the interval shown only when it changes: the desk is
  // not rebuilt every frame of playback.
  int? _now;
  void _frame() {
    final next = current;
    if (next != _now) {
      _now = next;
      notifyListeners();
    }
  }

  @override
  Listenable get frame => c.frame;

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

  Seg _seg(Map<String, dynamic> shape, {int frames = 1}) {
    final i = kinds.indexWhere((k) => k.name == shape['kind']);
    final seg = Seg(i < 0 ? 0 : i, frames, kinds);
    if (shape['kind'] == 'Bezier') {
      seg.setValues([for (final n in ['x1', 'y1', 'x2', 'y2']) (shape[n] as num? ?? 0).toDouble()]);
    } else if (shape['samples'] is List) {
      seg.model = shape; // the host's full description, with this curve's own parameters
    } else if (seg.model != null && shape.keys.any((k) => k != 'kind')) {
      // a kept curve carries its parameters, not its samples: those parameters now, and the host's description of
      // them once it answers — asked once per shape and kept, so reading the desk again does not ask again
      final key = jsonEncode(shape);
      seg.model = _described[key] ?? {...seg.model!, ...shape};
      if (!_described.containsKey(key) && _asking.add(key)) {
        model(seg, null, null).then((m) {
          _asking.remove(key);
          if (m == null) return;
          _described[key] = m;
          notifyListeners();
        });
      }
    }
    return seg;
  }

  final _described = <String, Map<String, dynamic>>{};
  final _asking = <String>{};

  @override
  bool get canClear => _presets.isNotEmpty;

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

  /// A curve as the host takes it: its kind and numeric parameters (a Bezier's four, another kind's own).
  Map<String, dynamic> _shape(Seg curve) => {
        'kind': kinds[curve.p].name,
        if (curve.bez) ...{'x1': curve.x1, 'y1': curve.y1, 'x2': curve.x2, 'y2': curve.y2}
        else if (curve.model != null)
          for (final e in curve.model!.entries)
            if (e.value is num && !const {'overshoots'}.contains(e.key)) e.key: e.value,
      };

  @override
  Future<Map<String, dynamic>?> model(Seg curve, int? handle, Offset? point) async {
    try {
      final reply = EditorSession.map(await c.native('easeModel', {
        'shape': _shape(curve),
        if (handle != null && point != null) ...{'handle': handle, 'point': [point.dx, point.dy]},
      }));
      if (reply['error'] != null) {
        c.error.value = '${reply['error']}';
        return null;
      }
      return reply['kind'] == null ? null : reply; // no description, no change
    } catch (e) {
      c.error.value = '$e';
      return null;
    }
  }

  Map<String, dynamic> _payload(Seg curve) => {
        'layers': [for (final l in _layers) l['id']],
        'shape': _shape(curve),
      };

  bool _previewed = false;

  @override
  void preview(int? interval, Seg curve) {
    if (_layers.isEmpty || !_canSequence || !c.supports('previewSequence')) return;
    _previewed = true;
    c.commandDirect('previewSequence', _payload(curve), 'ease-sequence');
  }

  @override
  void cancel() {
    if (_previewed) c.cancelPreview();
    _previewed = false;
  }

  @override
  List<Preset> get kinds => _kinds.isEmpty ? const [Preset('Linear', [0, 0, 1, 1], null)] : _kinds;

  @override
  List<Seg> get intervals => _layers.isNotEmpty || _rows.isEmpty
      ? [_seg(_stored)]
      : [
          for (final r in _rows)
            _seg(EditorSession.map(r['shape']), frames: ((r['end'] as num? ?? 1) - (r['frame'] as num? ?? 0)).round().clamp(1, 1 << 30)),
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
      _previewed = false;
      final payload = _payload(curve);
      await c.storeDesk('ease', payload['shape'] as Map<String, dynamic>);
      if (_canSequence) await c.command('sequence', payload);
      return;
    }
    // one interval is eased by naming its key (the selection is left as it is): one command, one undo step
    final narrowed = interval != null && interval < _rows.length;
    final r = narrowed ? _rows[interval] : null;
    await c.command('ease', {
      ..._shape(curve),
      if (r != null) 'keys': [{'layer': r['layer'], 'property': r['property'], 'frame': r['frame']}],
    });
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
  LiveEaseHost get host => DeskSession.of(widget.c).ease;
  @override
  void dispose() {
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => EaseDesk(host: host);
}
